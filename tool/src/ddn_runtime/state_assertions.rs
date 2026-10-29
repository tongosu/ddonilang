//! State-machine transitions and assertion evaluation; unchanged implementations.
use super::*;

pub(super) fn ensure_declared_state(machine: &StateMachine, state: &str) -> Result<(), EvalError> {
    if machine.states.iter().any(|name| name == state) {
        Ok(())
    } else {
        Err(format!("E_STATE_UNDECLARED: {}", state).into())
    }
}

pub(super) fn build_state_machine_transition_bindings(from: &str, to: &str) -> HashMap<String, Value> {
    let from_value = Value::String(from.to_string());
    let to_value = Value::String(to.to_string());
    HashMap::from([
        ("현재".to_string(), from_value.clone()),
        ("다음".to_string(), to_value.clone()),
        ("현재상태".to_string(), from_value.clone()),
        ("다음상태".to_string(), to_value.clone()),
        ("출발상태".to_string(), from_value),
        ("도착상태".to_string(), to_value),
    ])
}

pub(super) fn state_machine_transition_action_args(
    seed: &SeedDef,
    bindings: &HashMap<String, Value>,
) -> Result<Vec<Value>, EvalError> {
    let mut args = Vec::with_capacity(seed.params.len());
    for param in &seed.params {
        let value = bindings.get(&param.pin_name).cloned().ok_or_else(|| {
            EvalError::from(format!(
                "E_STATE_TRANSITION_ACTION_ARG_UNRESOLVED: {}:{}",
                seed.canonical_name, param.pin_name
            ))
        })?;
        args.push(value);
    }
    Ok(args)
}

pub(super) fn assertion_bindings_from_value(value: &Value) -> Result<HashMap<String, Value>, EvalError> {
    match value {
        Value::Pack(pack) => Ok(pack
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()),
        Value::Map(entries) => {
            let mut out = HashMap::new();
            for entry in entries.values() {
                let Value::String(key) = &entry.key else {
                    return Err("살피기 값들은 글 키를 가진 묶음/짝맞춤이어야 합니다"
                        .to_string()
                        .into());
                };
                out.insert(key.clone(), entry.value.clone());
            }
            Ok(out)
        }
        _ => Err("살피기 값들은 묶음 또는 짝맞춤이어야 합니다"
            .to_string()
            .into()),
    }
}

pub(super) fn parse_assertion_body(assertion: &Assertion) -> Result<Body, EvalError> {
    let wrapped = format!("검사:셈씨 = {{{}}}", assertion.body_source);
    let program = parse_with_mode(&wrapped, "#assertion", ParseMode::Strict).map_err(|err| {
        EvalError::Message(format!(
            "세움 본문 파싱 실패: {}",
            format_parse_error(&wrapped, &err)
        ))
    })?;
    let Some(TopLevelItem::SeedDef(seed)) = program.items.first() else {
        return Err("세움 본문을 찾을 수 없습니다".to_string().into());
    };
    let Some(body) = &seed.body else {
        return Err("세움 본문이 비어 있습니다".to_string().into());
    };
    Ok(body.clone())
}

pub(super) fn extract_assertion_body_source(canon: &str) -> Option<String> {
    let start = canon.find('{')?;
    let end = canon.rfind('}')?;
    if end <= start {
        return None;
    }
    Some(canon[start + 1..end].to_string())
}

pub(super) enum AssertionEvaluation {
    Passed,
    Rejected(String),
    Fault(String),
}

impl<'a> EvalContext<'a> {
    pub(super) fn resolve_named_assertion(
        &self,
        locals: &HashMap<String, Value>,
        name: &str,
    ) -> Option<Assertion> {
        if let Some(Value::Assertion(assertion)) = locals.get(name) {
            return Some(assertion.clone());
        }
        if let Some(Value::Assertion(assertion)) = self.resources.get(name) {
            return Some(assertion.clone());
        }
        if let Some(Value::Assertion(assertion)) = self.defaults.get(name) {
            return Some(assertion.clone());
        }
        None
    }

    pub(super) fn eval_state_machine_transition_checks(
        &mut self,
        locals: &HashMap<String, Value>,
        machine: &StateMachine,
        from: &str,
        to: &str,
    ) -> Result<(), EvalError> {
        if machine.on_transition_checks.is_empty() {
            return Ok(());
        }
        let bindings = build_state_machine_transition_bindings(from, to);
        for check_name in &machine.on_transition_checks {
            let assertion = self
                .resolve_named_assertion(locals, check_name)
                .ok_or_else(|| {
                    EvalError::from(format!(
                        "E_STATE_TRANSITION_CHECK_UNRESOLVED: {}",
                        check_name
                    ))
                })?;
            let passed = self.eval_assertion(&assertion, bindings.clone())?;
            match passed {
                Value::Bool(true) => {}
                _ => {
                    return Err(format!("E_STATE_TRANSITION_CHECK_FAILED: {}", check_name).into());
                }
            }
        }
        Ok(())
    }

    pub(super) fn eval_state_machine_guard(
        &mut self,
        locals: &HashMap<String, Value>,
        guard_name: &str,
        bindings: &HashMap<String, Value>,
    ) -> Result<bool, EvalError> {
        let assertion = self
            .resolve_named_assertion(locals, guard_name)
            .ok_or_else(|| {
                EvalError::from(format!(
                    "E_STATE_TRANSITION_GUARD_UNRESOLVED: {}",
                    guard_name
                ))
            })?;
        match self.eval_assertion_outcome(&assertion, bindings.clone()) {
            AssertionEvaluation::Passed => Ok(true),
            AssertionEvaluation::Rejected(_) => Ok(false),
            // A malformed or unresolved guard is not a normal `false` guard
            // result.  Keep the established fail-closed diagnostic family
            // rather than introducing a new public diagnostic spelling here.
            AssertionEvaluation::Fault(_) => Err(EvalError::from(format!(
                "E_STATE_TRANSITION_GUARD_REJECTED: {}",
                guard_name
            ))),
        }
    }

    pub(super) fn eval_state_machine_transition_action(
        &mut self,
        action_name: &str,
        bindings: &HashMap<String, Value>,
    ) -> Result<(), EvalError> {
        let seed = self
            .program
            .functions
            .get(action_name)
            .ok_or_else(|| {
                EvalError::from(format!(
                    "E_STATE_TRANSITION_ACTION_UNRESOLVED: {}",
                    action_name
                ))
            })?
            .clone();
        let args = state_machine_transition_action_args(&seed, bindings)?;
        let mut child = EvalContext {
            explicit_nfc_provenance: self.explicit_nfc_provenance.clone(),
            result_emissions: self.result_emissions.clone(),
            program: self.program,
            source_span_cache: Rc::clone(&self.source_span_cache),
            subject_stage:self.subject_stage.as_ref().map(Rc::clone),
            tuple_type_plan: Rc::clone(&self.tuple_type_plan),
            world: self.world,
            defaults: self.defaults,
            param_overrides: self.param_overrides,
            resources: self.resources.clone(),
            input: self.input,
            logical_input: self.logical_input.clone(),
            control_pressed: self.control_pressed,
            input_reads: Rc::clone(&self.input_reads),
            patch_ops: self.patch_ops.clone(),
            guard_rejected: self.guard_rejected,
            aborted: false,
            current_seed_name: self.current_seed_name.clone(),
            active_callable_name: self.active_callable_name.clone(),
            invocation_ordinal: self.invocation_ordinal,
            result_place_frames: self.result_place_frames.clone(),
            current_owner_identity: self.current_owner_identity.clone(),
            pending_async_signals: self.pending_async_signals.clone(),
            async_outbox: self.async_outbox.clone(),
            async_wait_receipts: self.async_wait_receipts.clone(),
            rng_state: self.rng_state,
            flow_stack: self.flow_stack.clone(),
            tick_id: self.tick_id,
            const_scopes: self.const_scopes.clone(),
            binding_type_scopes: self.binding_type_scopes.clone(),
            local_body_scopes: Vec::new(),
            pending_top_level_decl_names: self.pending_top_level_decl_names.clone(),
            factor_route_counts: self.factor_route_counts.clone(),
            factor_bits_total: self.factor_bits_total,
            factor_bits_min: self.factor_bits_min,
            factor_bits_max: self.factor_bits_max,
            public_observations: self.public_observations.clone(),
            root_program_active: self.root_program_active,
            derived_initialized: self.derived_initialized,
            display_outputs: self.display_outputs.clone(),
            root_binding_types: self.root_binding_types.clone(),
            sum_definitions: self.sum_definitions.clone(),
            sum_pattern_bindings: self.sum_pattern_bindings.clone(),
            project_asset_registry: self.project_asset_registry,
            work_cost: Rc::clone(&self.work_cost),
            internal_dispatch: self.internal_dispatch.clone(),
        };
        child.eval_seed(&seed, args)?;
        if child.aborted {
            return Err(EvalError::from(format!(
                "E_STATE_TRANSITION_ACTION_ABORTED: {}",
                action_name
            )));
        }
        self.resources = child.resources;
        self.patch_ops = child.patch_ops;
        self.guard_rejected = child.guard_rejected;
        self.rng_state = child.rng_state;
        self.const_scopes = child.const_scopes;
        self.binding_type_scopes = child.binding_type_scopes;
        self.factor_route_counts = child.factor_route_counts;
        self.factor_bits_total = child.factor_bits_total;
        self.factor_bits_min = child.factor_bits_min;
        self.factor_bits_max = child.factor_bits_max;
        self.public_observations = child.public_observations;
        self.explicit_nfc_provenance = child.explicit_nfc_provenance;
        self.display_outputs = child.display_outputs;
        self.root_binding_types = child.root_binding_types;
        self.result_emissions = child.result_emissions;
        self.pending_async_signals = child.pending_async_signals;
        self.async_outbox = child.async_outbox;
        self.async_wait_receipts = child.async_wait_receipts;
        self.internal_dispatch = child.internal_dispatch;
        Ok(())
    }

    pub(super) fn apply_state_machine_transition(
        &mut self,
        locals: &HashMap<String, Value>,
        machine: &StateMachine,
        transition: &StateTransition,
    ) -> Result<(), EvalError> {
        let bindings = build_state_machine_transition_bindings(&transition.from, &transition.to);
        let mut child = EvalContext {
            explicit_nfc_provenance: self.explicit_nfc_provenance.clone(),
            result_emissions: self.result_emissions.clone(),
            program: self.program,
            source_span_cache: Rc::clone(&self.source_span_cache),
            subject_stage:self.subject_stage.as_ref().map(Rc::clone),
            tuple_type_plan: Rc::clone(&self.tuple_type_plan),
            world: self.world,
            defaults: self.defaults,
            param_overrides: self.param_overrides,
            resources: self.resources.clone(),
            input: self.input,
            logical_input: self.logical_input.clone(),
            control_pressed: self.control_pressed,
            patch_ops: self.patch_ops.clone(),
            input_reads: Rc::clone(&self.input_reads),
            guard_rejected: self.guard_rejected,
            aborted: false,
            current_seed_name: self.current_seed_name.clone(),
            active_callable_name: self.active_callable_name.clone(),
            invocation_ordinal: self.invocation_ordinal,
            result_place_frames: self.result_place_frames.clone(),
            current_owner_identity: self.current_owner_identity.clone(),
            pending_async_signals: self.pending_async_signals.clone(),
            async_outbox: self.async_outbox.clone(),
            async_wait_receipts: self.async_wait_receipts.clone(),
            rng_state: self.rng_state,
            flow_stack: self.flow_stack.clone(),
            tick_id: self.tick_id,
            const_scopes: self.const_scopes.clone(),
            binding_type_scopes: self.binding_type_scopes.clone(),
            local_body_scopes: Vec::new(),
            pending_top_level_decl_names: self.pending_top_level_decl_names.clone(),
            factor_route_counts: self.factor_route_counts.clone(),
            factor_bits_total: self.factor_bits_total,
            factor_bits_min: self.factor_bits_min,
            factor_bits_max: self.factor_bits_max,
            public_observations: self.public_observations.clone(),
            root_program_active: self.root_program_active,
            derived_initialized: self.derived_initialized,
            display_outputs: self.display_outputs.clone(),
            root_binding_types: self.root_binding_types.clone(),
            sum_definitions: self.sum_definitions.clone(),
            sum_pattern_bindings: self.sum_pattern_bindings.clone(),
            project_asset_registry: self.project_asset_registry,
            work_cost: Rc::clone(&self.work_cost),
            internal_dispatch: self.internal_dispatch.clone(),
        };
        if let Some(action_name) = &transition.action_name {
            child.eval_state_machine_transition_action(action_name, &bindings)?;
        }
        child.eval_state_machine_transition_checks(
            locals,
            machine,
            &transition.from,
            &transition.to,
        )?;
        self.resources = child.resources;
        self.patch_ops = child.patch_ops;
        self.guard_rejected = child.guard_rejected;
        self.rng_state = child.rng_state;
        self.const_scopes = child.const_scopes;
        self.binding_type_scopes = child.binding_type_scopes;
        self.factor_route_counts = child.factor_route_counts;
        self.factor_bits_total = child.factor_bits_total;
        self.factor_bits_min = child.factor_bits_min;
        self.factor_bits_max = child.factor_bits_max;
        self.public_observations = child.public_observations;
        self.explicit_nfc_provenance = child.explicit_nfc_provenance;
        self.display_outputs = child.display_outputs;
        self.pending_async_signals = child.pending_async_signals;
        self.result_emissions = child.result_emissions;
        self.async_outbox = child.async_outbox;
        self.async_wait_receipts = child.async_wait_receipts;
        self.internal_dispatch = child.internal_dispatch;
        Ok(())
    }

    pub(super) fn emit_state_machine_transition(
        &mut self,
        from: &str,
        to: &str,
        guard_name: Option<&str>,
        action_name: Option<&str>,
    ) {
        let origin = self
            .current_seed_name
            .as_ref()
            .map(|name| format!("seed:{}", name))
            .unwrap_or_else(|| "#system:ddn".to_string());
        let event = DiagEvent {
            madi: self.tick_id,
            seq: 0,
            fault_id: "STATE_TRANSITION".to_string(),
            rule_id: "L1-STATE-01".to_string(),
            reason: "STATE_TRANSITION".to_string(),
            sub_reason: Some("NEXT".to_string()),
            mode: None,
            contract_kind: None,
            origin: origin.clone(),
            targets: vec![origin],
            sam_hash: None,
            source_span: None,
            expr: Some(ExprTrace {
                tag: format!("state_transition:{}->{}", from, to),
                text: Some(format!(
                    "guard={};action={}",
                    guard_name.unwrap_or("-"),
                    action_name.unwrap_or("-")
                )),
            }),
            message: Some(format!("상태 전이: {} -> {}", from, to)),
        };
        self.patch_ops.push(PatchOp::EmitSignal {
            signal: Signal::Diag { event },
            targets: Vec::new(),
        });
    }

    pub(super) fn eval_assertion_outcome(
        &mut self,
        assertion: &Assertion,
        bindings: HashMap<String, Value>,
    ) -> AssertionEvaluation {
        if let Some((left, right)) = parse_symbolic_equivalence_assertion(&assertion.body_source) {
            return match verify_symbolic_assertion(&left, &right) {
                Ok(true) => AssertionEvaluation::Passed,
                Ok(false) => AssertionEvaluation::Rejected(
                    "세움 symbolic equivalence proof failed".to_string(),
                ),
                Err(message) => AssertionEvaluation::Fault(message),
            };
        }
        if let Some((left, right)) = parse_symbolic_relation_assertion(&assertion.body_source) {
            let solve_bindings = match solve_bindings_from_hashmap(&bindings) {
                Ok(bindings) => bindings,
                Err(error) => return AssertionEvaluation::Fault(error.message()),
            };
            let left_text = match relation_formula_text(&left) {
                Ok(text) => text,
                Err(error) => return AssertionEvaluation::Fault(error.message()),
            };
            let right_text = match relation_formula_text(&right) {
                Ok(text) => text,
                Err(error) => return AssertionEvaluation::Fault(error.message()),
            };
            return match ddonirang_symbolic::relation_holds(
                &left_text,
                &right_text,
                &solve_bindings,
            ) {
                Ok(true) => AssertionEvaluation::Passed,
                Ok(false) => {
                    AssertionEvaluation::Rejected("세움 relation assertion failed".to_string())
                }
                Err(error) => {
                    AssertionEvaluation::Fault(symbolic_formula_error("equiv", error).message())
                }
            };
        }
        let body = match parse_assertion_body(assertion) {
            Ok(body) => body,
            Err(error) => return AssertionEvaluation::Fault(error.message()),
        };
        let mut child = EvalContext {
            program: self.program,
            result_emissions: Default::default(),
            source_span_cache: Rc::clone(&self.source_span_cache),
            subject_stage:self.subject_stage.as_ref().map(Rc::clone),
            tuple_type_plan: Rc::clone(&self.tuple_type_plan),
            world: self.world,
            defaults: self.defaults,
            param_overrides: self.param_overrides,
            resources: self.resources.clone(),
            input: self.input,
            logical_input: self.logical_input.clone(),
            control_pressed: self.control_pressed,
            patch_ops: PatchLog::default(),
            input_reads: Rc::clone(&self.input_reads),
            guard_rejected: false,
            aborted: false,
            current_seed_name: self.current_seed_name.clone(),
            active_callable_name: self.active_callable_name.clone(),
            invocation_ordinal: self.invocation_ordinal,
            result_place_frames: self.result_place_frames.clone(),
            current_owner_identity: self.current_owner_identity.clone(),
            pending_async_signals: self.pending_async_signals.clone(),
            async_outbox: self.async_outbox.clone(),
            async_wait_receipts: self.async_wait_receipts.clone(),
            rng_state: self.rng_state,
            flow_stack: self.flow_stack.clone(),
            tick_id: self.tick_id,
            const_scopes: self.const_scopes.clone(),
            binding_type_scopes: self.binding_type_scopes.clone(),
            local_body_scopes: Vec::new(),
            pending_top_level_decl_names: self.pending_top_level_decl_names.clone(),
            factor_route_counts: self.factor_route_counts.clone(),
            factor_bits_total: self.factor_bits_total,
            factor_bits_min: self.factor_bits_min,
            factor_bits_max: self.factor_bits_max,
            public_observations: Vec::new(),
            root_program_active: self.root_program_active,
            derived_initialized: self.derived_initialized,
            explicit_nfc_provenance: Vec::new(),
            display_outputs: Vec::new(),
            root_binding_types: self.root_binding_types.clone(),
            sum_definitions: self.sum_definitions.clone(),
            sum_pattern_bindings: self.sum_pattern_bindings.clone(),
            project_asset_registry: self.project_asset_registry,
            work_cost: Rc::clone(&self.work_cost),
            internal_dispatch: self.internal_dispatch.clone(),
        };
        let mut locals = bindings;
        match child.eval_body(&mut locals, &body) {
            Ok(_) if !child.aborted => AssertionEvaluation::Passed,
            Ok(_) => AssertionEvaluation::Rejected("세움 살피기가 실패했습니다".to_string()),
            Err(error) => AssertionEvaluation::Fault(error.message()),
        }
    }

    pub(super) fn eval_assertion_result(
        &mut self,
        assertion: &Assertion,
        bindings: HashMap<String, Value>,
    ) -> Result<(), String> {
        match self.eval_assertion_outcome(assertion, bindings) {
            AssertionEvaluation::Passed => Ok(()),
            AssertionEvaluation::Rejected(message) | AssertionEvaluation::Fault(message) => {
                Err(message)
            }
        }
    }

    pub(super) fn eval_assertion(
        &mut self,
        assertion: &Assertion,
        bindings: HashMap<String, Value>,
    ) -> Result<Value, EvalError> {
        match self.eval_assertion_result(assertion, bindings) {
            Ok(()) => Ok(Value::Bool(true)),
            Err(message) => {
                self.emit_assertion_failure(assertion, message);
                Ok(Value::Bool(false))
            }
        }
    }

    pub(super) fn emit_assertion_failure(&mut self, assertion: &Assertion, message: String) {
        let origin = self
            .current_seed_name
            .as_ref()
            .map(|name| format!("seed:{}", name))
            .unwrap_or_else(|| "#system:ddn".to_string());
        let event = DiagEvent {
            madi: self.tick_id,
            seq: 0,
            fault_id: "ASSERTION_FAILED".to_string(),
            rule_id: "L1-ASSERT-01".to_string(),
            reason: "ASSERTION_FAILED".to_string(),
            sub_reason: Some("CHECK_FAILED".to_string()),
            mode: None,
            contract_kind: None,
            origin: origin.clone(),
            targets: vec![origin],
            sam_hash: None,
            source_span: None,
            expr: Some(ExprTrace {
                tag: "assertion:check".to_string(),
                text: Some(assertion.canon.clone()),
            }),
            message: Some(message),
        };
        self.patch_ops.push(PatchOp::EmitSignal {
            signal: Signal::Diag { event },
            targets: Vec::new(),
        });
    }
}

