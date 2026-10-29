//! Existing evaluator methods moved without changing their bodies.
use num_traits::ToPrimitive;
use super::{
    BigInt,
    BigRational,
    Body,
    CallableResultPlaceFrame,
    EvalContext,
    EvalError,
    Expr,
    ExprKind,
    Fixed64,
    FlowControl,
    HashMap,
    HashSet,
    IndexedSelectionExpr,
    Integer,
    Literal,
    LocalBodyScope,
    MessageTransfer,
    ParamPin,
    Rc,
    ResultPlace,
    SeedDef,
    SeedKind,
    Stmt,
    ThunkResult,
    TypeRef,
    Value,
    check_value_type,
    expression_preserves_local_bindings,
    func,
    inferred_binding_type,
    internal_hash_text,
    is_truthy,
    literal_to_value,
    make_big_int_pack_from_bigint,
    make_rational_pack_from_big_rational,
    pow,
    restore_local_binding,
    root_binding_type_key,
    statement_cannot_abort,
    stmt_registry_key,
    tuple_position_type_equal,
    type_mismatch_error,
};

// F4: statement-specific scratch exists only for the selected tail-match arm.
#[inline(never)]
fn in_statement_frame<F>(arm: F) -> Result<FlowControl, EvalError>
where
    F: FnOnce() -> Result<FlowControl, EvalError>,
{
    arm()
}

impl<'a> EvalContext<'a> {
    pub(super) fn eval_seed(&mut self, seed: &SeedDef, args: Vec<Value>) -> Result<Value, EvalError> {
        self.work_cost.borrow_mut().charge_dispatch()?;
        self.work_cost.borrow_mut().enter_call()?;
        let result = self.eval_seed_after_dispatch(seed, args);
        self.work_cost.borrow_mut().exit_call();
        result
    }

    pub(super) fn eval_seed_after_dispatch(
        &mut self,
        seed: &SeedDef,
        args: Vec<Value>,
    ) -> Result<Value, EvalError> {
        if self.program.identity_calculations.is_erased_seed(seed) {
            if args.len()!=1 {return Err("E_PROOF_STRUCTURE_CONSUMPTION: 실제 값 입력 하나가 필요합니다".to_string().into());}
            self.check_param_type(&seed.params[0],&args[0])?;
        }
        if !seed.type_params.is_empty() {
            return Err("E_GENERIC_RESOLUTION_UNSUPPORTED: 호출별 정적 씨자리 결속이 아직 공급되지 않았습니다"
                .to_string().into());
        }
        let prev_seed = self.current_seed_name.clone();
        let prev_callable = self.active_callable_name.clone();
        let prev_owner = self.current_owner_identity.clone();
        let frame_depth = self.result_place_frames.len();
        let resultful =
            seed.result_type.is_some() || matches!(seed.seed_kind, SeedKind::ValueFunc);
        if resultful {
            self.invocation_ordinal = self
                .invocation_ordinal
                .checked_add(1)
                .ok_or_else(|| EvalError::Message("E_INVOCATION_IDENTITY_OVERFLOW".to_string()))?;
            let mut hasher = blake3::Hasher::new();
            hasher.update(b"ddonirang.resultplace.invocation.v1");
            internal_hash_text(&mut hasher, &seed.canonical_name);
            hasher.update(&seed.id.to_le_bytes());
            hasher.update(&self.invocation_ordinal.to_le_bytes());
            self.result_place_frames.push(CallableResultPlaceFrame {
                canonical_name: seed.canonical_name.clone(),
                _invocation_identity: hasher.finalize().to_hex().to_string(),
                result_type: seed.result_type.clone(),
                place: ResultPlace::new(),
            });
        }
        self.current_seed_name = Some(seed.canonical_name.clone());
        self.active_callable_name = Some(seed.canonical_name.clone());
        if matches!(&seed.seed_kind, SeedKind::Named(kind) if kind == "임자") {
            self.current_owner_identity = Some(seed.canonical_name.clone());
        }
        self.enter_const_scope();
        let caller_pattern_bindings = std::mem::take(&mut self.sum_pattern_bindings);
        let result = (|| {
            if self.aborted {
                return if resultful {
                    Err("E_RESULTPLACE_ABORTED: 중단된 결과 호출은 성공 결과를 게시할 수 없습니다"
                        .to_string()
                        .into())
                } else {
                    Ok(Value::None)
                };
            }
            let mut locals = HashMap::new();
            if seed.params.len() != args.len() {
                return Err(format!(
                    "함수 '{}' 인자 수 불일치: 기대 {}, 실제 {}",
                    seed.canonical_name,
                    seed.params.len(),
                    args.len()
                )
                .into());
            }
            for (param, arg) in seed.params.iter().zip(args) {
                self.check_param_type(param, &arg)?;
                self.binding_type_scopes.last_mut().unwrap().insert(
                    param.pin_name.clone(), (param.type_ref.clone(), param.optional),
                );
                locals.insert(param.pin_name.clone(), arg);
            }
            let piecewise=if self.program.program.origin.piecewise_definitions.contains_key(&seed.id) {
                let input=seed.params.first().and_then(|pin|locals.get(&pin.pin_name))
                    .ok_or_else(||EvalError::Message("조건별 계산: 실제 입력이 없습니다".into()))?;
                self.program.piecewise_calculations.selected_expression(&self.program.source_identity,seed,input)?
            }else{None};
            let out = if let Some(expression)=piecewise {
                // 같은 원래 식을 공통 정확 연산/작업비용 evaluator가 선택 후에만 실행.
                // 미선택 행이나 분할 검사 결과를 runtime Proof 값으로 평가하지 않는다.
                FlowControl::Return(self.eval_expr(&mut locals,&expression)?)
            } else if let Some(body)=&seed.body {
                self.with_local_body_scope(&mut locals, true, |context, locals| {
                    context.eval_body_unscoped(locals, body)
                })?
            } else {
                return if resultful {
                    Err(format!(
                        "E_RESULTPLACE_UNASSIGNED: 결과 호출 '{}'의 본문이 없습니다",
                        seed.canonical_name
                    )
                    .into())
                } else {
                    Ok(Value::None)
                };
            };
            let out_value = match out {
                FlowControl::Continue => None,
                FlowControl::Return(value) => Some(value),
                FlowControl::ContinueLoop(_) => {
                    return Err("건너뛰기는 반복 안에서만 사용할 수 있습니다"
                        .to_string()
                        .into())
                }
                FlowControl::Break(_) => {
                    return Err("멈추기는 반복 안에서만 사용할 수 있습니다"
                        .to_string()
                        .into())
                }
            };
            if resultful {
                if self.aborted || self.guard_rejected {
                    return Err("E_RESULTPLACE_ABORTED: 중단된 호출은 성공 결과를 게시할 수 없습니다"
                        .to_string()
                        .into());
                }
                if let Some(value) = out_value {
                    self.assign_current_result_place(value)?;
                }
                self.result_place_frames
                    .last()
                    .and_then(|frame| frame.place.value())
                    .cloned()
                    .ok_or_else(|| {
                        format!(
                            "E_RESULTPLACE_UNASSIGNED: 결과 호출 '{}'의 결과 자리가 배정되지 않았습니다",
                            seed.canonical_name
                        )
                        .into()
                    })
            } else if matches!(seed.seed_kind, SeedKind::Umjikssi) {
                Ok(Value::None)
            } else {
                let value = out_value.unwrap_or(Value::None);
                if let Some(result_type) = &seed.result_type {
                    if let Err(detail) = self.check_value_type(&value, result_type) {
                        return Err(type_mismatch_error(
                            &format!("{} 결과", seed.canonical_name),
                            &detail.expected,
                            &detail.actual,
                        ));
                    }
                }
                Ok(value)
            }
        })();
        self.result_place_frames.truncate(frame_depth);
        self.exit_const_scope();
        self.sum_pattern_bindings = caller_pattern_bindings;
        self.current_seed_name = prev_seed;
        self.active_callable_name = prev_callable;
        self.current_owner_identity = prev_owner;
        result
    }

    pub(super) fn owns_pure_local_bindings(&self) -> bool {
        if self.program.program.source_epoch != ddonirang_lang::SourceEpoch::V25 {
            return false;
        }
        self.active_callable_name.as_ref()
            .filter(|name| self.current_seed_name.as_ref() == Some(*name))
            .and_then(|name| self.program.functions.get(name))
            .is_some_and(|seed| matches!(seed.seed_kind, SeedKind::Semssi | SeedKind::ValueFunc))
    }

    pub(super) fn assign_current_result_place(&mut self, value: Value) -> Result<(), EvalError> {
        let current_name = self
            .current_seed_name
            .as_deref()
            .ok_or_else(|| EvalError::Message("E_RESULTPLACE_ABSENT".to_string()))?;
        let frame = self
            .result_place_frames
            .last_mut()
            .filter(|frame| frame.canonical_name == current_name)
            .ok_or_else(|| {
                EvalError::Message(
                    "E_RESULTPLACE_CAPTURE_FORBIDDEN: 바깥 호출의 결과 자리를 쓸 수 없습니다"
                        .to_string(),
                )
            })?;
        if let Some(result_type) = &frame.result_type {
            if let Err(detail) = check_value_type(&value, result_type) {
                return Err(type_mismatch_error(
                    &format!("{} 결과", frame.canonical_name),
                    &detail.expected,
                    &detail.actual,
                ));
            }
        }
        frame.place.assign_sequential(value);
        Ok(())
    }

    pub(super) fn check_value_type(&self, value: &Value, type_ref: &TypeRef) -> Result<(), ddonirang_lang::runtime::TypeMismatchDetail> {
        self.program.check_value_type(value,type_ref)
    }

    pub(super) fn check_param_type(&self, param: &ParamPin, value: &Value) -> Result<(), EvalError> {
        if matches!(param.type_ref, TypeRef::Infer) {
            return Ok(());
        }
        if param.optional && matches!(value, Value::None) {
            return Ok(());
        }
        match self.check_value_type(value, &param.type_ref) {
            Ok(()) => Ok(()),
            Err(detail) => {
                let expected = if param.optional {
                    format!("{}?", detail.expected)
                } else {
                    detail.expected
                };
                Err(type_mismatch_error(
                    &param.pin_name,
                    &expected,
                    &detail.actual,
                ))
            }
        }
    }

    /// The runner evaluates a root program once per committed madi.  A periodic
    /// hook therefore selects whether its body participates in this madi; it is
    /// not a declaration to be silently discarded by the generic statement
    /// evaluator.  Start is the initial committed madi, while End remains
    /// unavailable here because this entry point has no finish lifecycle.
    pub(super) fn periodic_hook_is_due(&self, kind: ddonirang_lang::HookKind) -> Result<bool, EvalError> {
        match kind {
            ddonirang_lang::HookKind::Start => Ok(self.tick_id == 0),
            ddonirang_lang::HookKind::EveryMadi => Ok(true),
            ddonirang_lang::HookKind::EveryNMadi(interval) if interval > 0 => {
                Ok(self.tick_id % interval == 0)
            }
            ddonirang_lang::HookKind::EveryNMadi(_) => Err(
                "E_RUNTIME_HOOK_INTERVAL: 마디 간격은 0보다 커야 합니다"
                    .to_string()
                    .into(),
            ),
            ddonirang_lang::HookKind::End => Err(
                "E_RUNTIME_UNSUPPORTED_STMT: 현재 ddonirang-tool 실행기에서는 끝 훅을 지원하지 않습니다"
                    .to_string()
                    .into(),
            ),
        }
    }

    fn with_local_body_scope<T>(
        &mut self, locals: &mut HashMap<String, Value>, callable: bool,
        execute: impl FnOnce(&mut Self, &mut HashMap<String, Value>) -> Result<T, EvalError>,
    ) -> Result<T, EvalError> {
        // Legacy frontdoors keep their established callable/setup lifetime.
        // Only the sealed declared-v25 source path supplies lexical declarations.
        if self.program.sealed_source_capture().is_none() && self.program.sealed_project_sources().is_none() {
            return execute(self,locals);
        }
        let type_floor = if callable { self.binding_type_scopes.len() - 1 }
            else { self.local_body_scopes.last().map(|scope|scope.type_floor)
                .unwrap_or(self.binding_type_scopes.len() - 1) };
        if !callable { self.enter_const_scope(); }
        let previous = if callable { locals.keys().map(|name|(name.clone(),None)).collect() }
            else { HashMap::new() };
        self.local_body_scopes.push(LocalBodyScope {previous,type_floor});
        let outcome = execute(self,locals);
        let scope = self.local_body_scopes.pop().expect("active local body scope");
        for (name,previous) in scope.previous { restore_local_binding(locals,&name,previous); }
        if !callable { self.exit_const_scope(); }
        outcome
    }

    pub(super) fn eval_body(
        &mut self,
        locals: &mut HashMap<String, Value>,
        body: &Body,
    ) -> Result<FlowControl, EvalError> {
        self.with_local_body_scope(locals,false,|context,locals|context.eval_body_unscoped(locals,body))
    }

    pub(super) fn eval_body_unscoped(
        &mut self, locals: &mut HashMap<String, Value>, body: &Body,
    ) -> Result<FlowControl, EvalError> {
        if self.aborted {
            return Ok(FlowControl::Continue);
        }
        for stmt in &body.stmts {
            if self.aborted {
                return Ok(FlowControl::Continue);
            }
            let _registry_key = stmt_registry_key(stmt);
            self.work_cost.borrow_mut().charge_dispatch()?;
            let snapshot = (!statement_cannot_abort(stmt)).then(|| self.capture_frame_snapshot(locals));
            match self.eval_stmt(locals, stmt)? {
                FlowControl::Continue => {}
                other => return Ok(other),
            }
            if self.aborted {
                let Some(snapshot)=snapshot else {
                    return Err("E_RUNTIME_ABORT_PROOF: 중단 복원 경로를 증명할 수 없습니다".to_string().into());
                };
                self.restore_frame_snapshot_preserving_abort_contract_diag(locals, &snapshot);
                return Ok(FlowControl::Continue);
            }
        }
        Ok(FlowControl::Continue)
    }

    pub(super) fn eval_body_for_value(
        &mut self,
        locals: &mut HashMap<String, Value>,
        body: &Body,
    ) -> Result<Value, EvalError> {
        match self.eval_body_for_value_inner(locals, body)? {
            ThunkResult::Value(v) | ThunkResult::Return(v) => Ok(v),
            ThunkResult::ContinueLoop(_) => Err("건너뛰기는 반복 안에서만 사용할 수 있습니다"
                .to_string()
                .into()),
            ThunkResult::Break(_) => Err("멈추기는 반복 안에서만 사용할 수 있습니다"
                .to_string()
                .into()),
        }
    }

    pub(super) fn eval_body_for_value_inner(
        &mut self,
        locals: &mut HashMap<String, Value>,
        body: &Body,
    ) -> Result<ThunkResult, EvalError> {
        self.with_local_body_scope(locals,false,|context,locals|context.eval_body_for_value_unscoped(locals,body))
    }

    fn eval_body_for_value_unscoped(
        &mut self, locals: &mut HashMap<String, Value>, body: &Body,
    ) -> Result<ThunkResult, EvalError> {
        let mut last = Value::None;
        if self.aborted {
            return Ok(ThunkResult::Value(Value::None));
        }
        for stmt in &body.stmts {
            if self.aborted {
                return Ok(ThunkResult::Value(Value::None));
            }
            let _registry_key = stmt_registry_key(stmt);
            self.work_cost.borrow_mut().charge_dispatch()?;
            let snapshot = (!statement_cannot_abort(stmt)).then(|| self.capture_frame_snapshot(locals));
            match self.eval_stmt_for_value(locals, stmt)? {
                ThunkResult::Return(v) => return Ok(ThunkResult::Return(v)),
                ThunkResult::ContinueLoop(span) => return Ok(ThunkResult::ContinueLoop(span)),
                ThunkResult::Break(span) => return Ok(ThunkResult::Break(span)),
                ThunkResult::Value(v) => last = v,
            }
            if self.aborted {
                let Some(snapshot)=snapshot else {
                    return Err("E_RUNTIME_ABORT_PROOF: 중단 복원 경로를 증명할 수 없습니다".to_string().into());
                };
                self.restore_frame_snapshot_preserving_abort_contract_diag(locals, &snapshot);
                return Ok(ThunkResult::Value(Value::None));
            }
        }
        Ok(ThunkResult::Value(last))
    }

    pub(super) fn eval_stmt_for_value(
        &mut self,
        locals: &mut HashMap<String, Value>,
        stmt: &Stmt,
    ) -> Result<ThunkResult, EvalError> {
        if self.aborted {
            return Ok(ThunkResult::Value(Value::None));
        }
        match stmt {
            Stmt::DeclBlock { .. } | Stmt::Definition { .. } | Stmt::DerivedBinding {..} | Stmt::SumDefinition {..} => match self.eval_stmt(locals, stmt)? {
                FlowControl::Continue => Ok(ThunkResult::Value(Value::None)),
                FlowControl::Return(value) => Ok(ThunkResult::Return(value)),
                FlowControl::ContinueLoop(span) => Ok(ThunkResult::ContinueLoop(span)),
                FlowControl::Break(span) => Ok(ThunkResult::Break(span)),
            },
            Stmt::Mutate { .. } | Stmt::ResultFlow { .. } | Stmt::SumMatch {..} => match self.eval_stmt(locals, stmt)? {
                FlowControl::Continue => Ok(ThunkResult::Value(Value::None)),
                FlowControl::Return(value) => Ok(ThunkResult::Return(value)),
                FlowControl::ContinueLoop(span) => Ok(ThunkResult::ContinueLoop(span)),
                FlowControl::Break(span) => Ok(ThunkResult::Break(span)),
            },
            Stmt::Receive { .. } => Ok(ThunkResult::Value(Value::None)),
            Stmt::Send { .. } => match self.eval_stmt(locals, stmt)? {
                FlowControl::Continue => Ok(ThunkResult::Value(Value::None)),
                FlowControl::Return(value) => Ok(ThunkResult::Return(value)),
                FlowControl::ContinueLoop(span) => Ok(ThunkResult::ContinueLoop(span)),
                FlowControl::Break(span) => Ok(ThunkResult::Break(span)),
            },
            Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
                Ok(ThunkResult::Value(self.eval_expr(locals, expr)?))
            }
            Stmt::MetaBlock { .. } => match self.eval_stmt(locals, stmt)? {
                FlowControl::Continue => Ok(ThunkResult::Value(Value::None)),
                FlowControl::Return(value) => Ok(ThunkResult::Return(value)),
                FlowControl::ContinueLoop(span) => Ok(ThunkResult::ContinueLoop(span)),
                FlowControl::Break(span) => Ok(ThunkResult::Break(span)),
            },
            Stmt::PublicObservation { .. } => match self.eval_stmt(locals, stmt)? {
                FlowControl::Continue => Ok(ThunkResult::Value(Value::None)),
                FlowControl::Return(value) => Ok(ThunkResult::Return(value)),
                FlowControl::ContinueLoop(span) => Ok(ThunkResult::ContinueLoop(span)),
                FlowControl::Break(span) => Ok(ThunkResult::Break(span)),
            },
            Stmt::Pragma { .. } => Ok(ThunkResult::Value(Value::None)),
            Stmt::Return { value, .. } => Ok(ThunkResult::Return(self.eval_expr(locals, value)?)),
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                let cond = self.eval_expr(locals, condition)?;
                if is_truthy(&cond)? {
                    self.eval_body_for_value_inner(locals, then_body)
                } else if let Some(body) = else_body {
                    self.eval_body_for_value_inner(locals, body)
                } else {
                    Ok(ThunkResult::Value(Value::None))
                }
            }
            Stmt::Try { action, body, .. } => {
                let value = self.eval_expr(locals, action)?;
                let prev = locals.insert("그것".to_string(), value);
                let out = self.eval_body_for_value_inner(locals, body);
                if let Some(prev) = prev {
                    locals.insert("그것".to_string(), prev);
                } else {
                    locals.remove("그것");
                }
                out
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                for branch in branches {
                    let cond = self.eval_expr(locals, &branch.condition)?;
                    if is_truthy(&cond)? {
                        return self.eval_body_for_value_inner(locals, &branch.body);
                    }
                }
                self.eval_body_for_value_inner(locals, else_body)
            }
            Stmt::Repeat { body, .. } => {
                loop {
                    if self.aborted {
                        break;
                    }
                    match self.eval_body_for_value_inner(locals, body)? {
                        ThunkResult::Value(_) => {}
                        ThunkResult::Return(value) => return Ok(ThunkResult::Return(value)),
                        ThunkResult::ContinueLoop(_) => continue,
                        ThunkResult::Break(_) => break,
                    }
                }
                Ok(ThunkResult::Value(Value::None))
            }
            Stmt::While {
                condition, body, ..
            } => {
                loop {
                    if self.aborted {
                        break;
                    }
                    let cond = self.eval_expr(locals, condition)?;
                    if !is_truthy(&cond)? {
                        break;
                    }
                    match self.eval_body_for_value_inner(locals, body)? {
                        ThunkResult::Value(_) => {}
                        ThunkResult::Return(value) => return Ok(ThunkResult::Return(value)),
                        ThunkResult::ContinueLoop(_) => continue,
                        ThunkResult::Break(_) => break,
                    }
                }
                Ok(ThunkResult::Value(Value::None))
            }
            Stmt::ForEach {
                item,
                iterable,
                body,
                ..
            } => {
                let mut iter_value = self.eval_expr(locals, iterable)?;
                let items = match &mut iter_value {
                    Value::List(items) => std::mem::take(items),
                    Value::Set(items) => std::mem::take(items).into_values().collect(),
                    Value::Map(entries) => std::mem::take(entries)
                        .into_values()
                        .map(|entry| Value::List(vec![entry.key, entry.value]))
                        .collect(),
                    _ => {
                        return Err("순회 대상은 차림/모음/짝맞춤이어야 합니다"
                            .to_string()
                            .into())
                    }
                };
                let previous = locals.get(item).cloned();
                let previous_types = self.binding_type_scopes.last().unwrap().clone();
                let previous_root_type = self.root_binding_types.remove(item);
                self.declare_binding_type(item, &TypeRef::Infer);
                let outcome = (|| -> Result<ThunkResult, EvalError> {
                    for value in items {
                        if self.aborted {
                            break;
                        }
                        self.work_cost.borrow_mut().charge_element_visits(1)?;
                        self.root_binding_types.remove(item);
                        self.declare_binding_type(item, &inferred_binding_type(&value));
                        locals.insert(item.clone(), value);
                        let iteration_locals = locals.keys().cloned().collect::<HashSet<_>>();
                        let step = self.eval_body_for_value_inner(locals, body);
                        locals.retain(|name, _| iteration_locals.contains(name));
                        self.binding_type_scopes.last_mut().unwrap().retain(|name, _| iteration_locals.contains(name));
                        match step? {
                            ThunkResult::Value(_) => {}
                            ThunkResult::Return(value) => {
                                return Ok(ThunkResult::Return(value));
                            }
                            ThunkResult::ContinueLoop(_) => continue,
                            ThunkResult::Break(_) => break,
                        }
                        if self.aborted {
                            break;
                        }
                    }
                    Ok(ThunkResult::Value(Value::None))
                })();
                restore_local_binding(locals, item, previous);
                *self.binding_type_scopes.last_mut().unwrap() = previous_types;
                self.root_binding_types.remove(item);
                if let Some(previous) = previous_root_type {
                    self.root_binding_types.insert(item.clone(), previous);
                }
                outcome
            }
            Stmt::Hook { kind, body, .. } => {
                if self.periodic_hook_is_due(*kind)? {
                    self.eval_body_for_value_inner(locals, body)
                } else {
                    Ok(ThunkResult::Value(Value::None))
                }
            }
            Stmt::BeatBlock { .. }
            | Stmt::HookWhenBecomes { .. }
            | Stmt::HookWhile { .. } => Err(
                "E_RUNTIME_UNSUPPORTED_STMT: 현재 ddonirang-tool 실행기에서는 박자/훅 문장을 지원하지 않습니다"
                    .to_string()
                    .into(),
            ),
            Stmt::Quantifier { .. } => Ok(ThunkResult::Value(Value::None)),
            Stmt::Break { span, .. } => Ok(ThunkResult::Break(*span)),
            Stmt::ContinueLoop { span, .. } => Ok(ThunkResult::ContinueLoop(*span)),
            Stmt::Contract {
                kind,
                mode,
                condition,
                then_body,
                else_body,
                ..
            } => {
                let snapshot = self.capture_frame_snapshot(locals);
                let cond_value = self.eval_expr(locals, condition)?;
                let ok = is_truthy(&cond_value)?;
                match kind {
                    ddonirang_lang::ContractKind::Pre => {
                        if ok {
                            if let Some(body) = then_body {
                                let out = self.eval_body_for_value_inner(locals, body)?;
                                if self.aborted {
                                    self.restore_frame_snapshot_preserving_abort_contract_diag(
                                        locals, &snapshot,
                                    );
                                    return Ok(ThunkResult::Value(Value::None));
                                }
                                Ok(out)
                            } else {
                                Ok(ThunkResult::Value(Value::None))
                            }
                        } else {
                            match self.eval_body(locals, else_body)? {
                                FlowControl::Continue => {}
                                FlowControl::Return(value) => {
                                    return Ok(ThunkResult::Return(value))
                                }
                                FlowControl::ContinueLoop(span) => {
                                    return Ok(ThunkResult::ContinueLoop(span))
                                }
                                FlowControl::Break(span) => return Ok(ThunkResult::Break(span)),
                            }
                            if self.aborted {
                                self.restore_frame_snapshot_preserving_abort_contract_diag(
                                    locals, &snapshot,
                                );
                                return Ok(ThunkResult::Value(Value::None));
                            }
                            if matches!(mode, ddonirang_lang::ContractMode::Abort) {
                                self.restore_frame_snapshot(locals, &snapshot);
                            }
                            self.emit_contract_violation(
                                ddonirang_lang::ContractKind::Pre,
                                *mode,
                                condition,
                                "전제하에 조건이 실패했습니다".to_string(),
                            );
                            Ok(ThunkResult::Value(Value::None))
                        }
                    }
                    ddonirang_lang::ContractKind::Post => {
                        if !ok {
                            match self.eval_body(locals, else_body)? {
                                FlowControl::Continue => {}
                                FlowControl::Return(value) => {
                                    return Ok(ThunkResult::Return(value))
                                }
                                FlowControl::ContinueLoop(span) => {
                                    return Ok(ThunkResult::ContinueLoop(span))
                                }
                                FlowControl::Break(span) => return Ok(ThunkResult::Break(span)),
                            }
                            if self.aborted {
                                self.restore_frame_snapshot_preserving_abort_contract_diag(
                                    locals, &snapshot,
                                );
                                return Ok(ThunkResult::Value(Value::None));
                            }
                            let cond_value = self.eval_expr(locals, condition)?;
                            if !is_truthy(&cond_value)? {
                                if matches!(mode, ddonirang_lang::ContractMode::Abort) {
                                    self.restore_frame_snapshot(locals, &snapshot);
                                }
                                self.emit_contract_violation(
                                    ddonirang_lang::ContractKind::Post,
                                    *mode,
                                    condition,
                                    "보장하고 조건이 실패했습니다".to_string(),
                                );
                                return Ok(ThunkResult::Value(Value::None));
                            }
                        }
                        if let Some(body) = then_body {
                            let out = self.eval_body_for_value_inner(locals, body)?;
                            if self.aborted {
                                self.restore_frame_snapshot_preserving_abort_contract_diag(
                                    locals, &snapshot,
                                );
                                return Ok(ThunkResult::Value(Value::None));
                            }
                            Ok(out)
                        } else {
                            Ok(ThunkResult::Value(Value::None))
                        }
                    }
                }
            }
            Stmt::Guard { .. } => match self.eval_stmt(locals, stmt)? {
                FlowControl::Continue => Ok(ThunkResult::Value(Value::None)),
                FlowControl::Return(value) => Ok(ThunkResult::Return(value)),
                FlowControl::ContinueLoop(span) => Ok(ThunkResult::ContinueLoop(span)),
                FlowControl::Break(span) => Ok(ThunkResult::Break(span)),
            },
        }
    }

    pub(super) fn eval_stmt(
        &mut self,
        locals: &mut HashMap<String, Value>,
        stmt: &Stmt,
    ) -> Result<FlowControl, EvalError> {
        if self.aborted {
            return Ok(FlowControl::Continue);
        }
        match stmt {
            Stmt::DerivedBinding {..} => in_statement_frame(|| {
                if !self.root_program_active{return Err("E_DERIVED_SCOPE: 파생 관계는 맨 바깥에서만 활성화합니다".to_string().into());}
                if !self.derived_initialized{self.settle_derived(locals)?;}
                Ok(FlowControl::Continue)
            }),
            Stmt::SumDefinition {name,branches,..} => in_statement_frame(|| {
                self.sum_definitions.insert(name.clone(),branches.clone());
                Ok(FlowControl::Continue)
            }),
            Stmt::SumMatch {subject,arms,default_body,..} => in_statement_frame(|| {
                let subject=self.eval_expr(locals,subject)?;
                let Value::SumBranch {owner,branch,payload,..}=&subject else {
                    return Err("E_SUM_MATCH_SUBJECT_TYPE: `에 따라`의 값은 고름씨 갈래여야 합니다".to_string().into());
                };
                if arms.first().map(|arm|arm.owner.as_str())!=Some(owner.as_str()) {
                    return Err("E_SUM_MATCH_OWNER_MISMATCH: `에 따라`의 갈래 소유자가 값의 고름씨와 다릅니다".to_string().into());
                }
                if let Some(arm)=arms.iter().find(|arm|arm.owner==*owner&&arm.branch==*branch) {
                    // Resolve every selection before entering the arm. A missing
                    // pin must not publish a partial group of local bindings.
                    let mut bindings=Vec::new();
                    for binder in &arm.binder {
                        let name=&binder.pin;
                        let value=payload.get(name).ok_or_else(||EvalError::from(
                            "E_SUM_MATCH_PAYLOAD_MISSING: 값 갈래 payload가 없습니다".to_string()))?.clone();
                        let schema=if let Some(kind)=&arm.applied_owner {
                            ddonirang_lang::sum_owner::resolve_applied_sum_owner(kind,arm.span)
                                .map_err(EvalError::from)?.branches.into_iter().find(|declaration|declaration.name==arm.branch)
                                .map(|declaration|declaration.payload)
                        } else {
                            self.sum_definitions.get(&arm.owner)
                                .and_then(|branches|branches.iter().find(|declaration|declaration.name==arm.branch))
                                .map(|declaration|declaration.payload.clone())
                        };
                        let binder_type=schema.as_ref().and_then(|payload|payload.iter().find(|schema|schema.binder==*name))
                            .map(|schema|schema.type_ref.clone())
                            .ok_or_else(||EvalError::from("E_SUM_MATCH_BINDER_UNKNOWN: 받는 이름을 해소할 수 없습니다".to_string()))?;
                        bindings.push((name.clone(),value,binder_type,locals.get(name).cloned()));
                    }
                    let pattern_depth=self.sum_pattern_bindings.len();
                    self.enter_const_scope();
                    for (name,value,binder_type,_) in &bindings {
                        self.declare_binding_type(name,binder_type);
                        self.sum_pattern_bindings.push(name.clone());
                        locals.insert(name.clone(),value.clone());
                    }
                    let outcome=self.eval_body(locals,&arm.body);
                    for (name,_,_,previous) in bindings {restore_local_binding(locals,&name,previous);}
                    self.sum_pattern_bindings.truncate(pattern_depth);
                    self.exit_const_scope();
                    outcome
                } else if let Some(body)=default_body {self.eval_body(locals,body)}
                else {Err("E_SUM_MATCH_BRANCH_UNCOVERED: 선언된 갈래에 대응하는 문장이 없습니다".to_string().into())}
            }),
            Stmt::DeclBlock { items, .. } => in_statement_frame(|| {
                for item in items {
                    if self.program.derived_plan.declaration_ids.contains(&item.id) {
                        self.pending_top_level_decl_names.remove(&item.name);
                        self.declare_binding_type(&item.name,&item.type_ref);
                        continue;
                    }
                    if self.local_body_scopes.last().is_some_and(|scope|scope.previous.contains_key(&item.name)) {
                        return Err(format!("E_V25_DEFINITION_DUPLICATE: 이미 정의된 이름을 다시 정의할 수 없습니다: {}",item.name).into());
                    }
                    let is_top_level_decl = !self.owns_pure_local_bindings()
                        && self.pending_top_level_decl_names.remove(&item.name);
                    let body_local = !is_top_level_decl && !self.local_body_scopes.is_empty();
                    let resource_exists = if body_local { false } else if is_top_level_decl {
                        self.committed_resource_exists(&item.name)
                    } else {
                        self.resource_exists(&item.name)
                    };
                    // A3-P3 provisional: top-level setup initializes a retained
                    // world slot, not a per-madi expression. Reopening/restoring
                    // that slot must not run its initializer again. A reset has
                    // no committed slot and follows the normal first-run path.
                    let value = if is_top_level_decl && resource_exists {
                        self.get_resource(&item.name).ok_or_else(|| EvalError::Message(
                            format!("E_CHAEBI_RESOURCE_UNRESOLVED: 채비의 저장 값을 읽을 수 없습니다: {}", item.name)))?
                    } else if let Some(expr) = &item.value {
                        match self.eval_expr_expected(locals, expr, &item.type_ref) {
                            Ok(val) => val,
                            Err(err) => return Err(err),
                        }
                    } else if matches!(item.kind, ddonirang_lang::DeclKind::Butbak) {
                        return Err(format!(
                            "채비에서 '=' 항목은 초기값이 필요합니다: {}",
                            item.name
                        )
                        .into());
                    } else {
                        Value::None
                    };
                    let value = if is_top_level_decl && !resource_exists {
                        self.param_overrides
                            .and_then(|overrides| overrides.get(&item.name))
                            .cloned()
                            .unwrap_or(value)
                    } else {
                        value
                    };
                    if item.value.is_some() {
                        if let Err(detail) = self.check_value_type(&value, &item.type_ref) {
                            return Err(type_mismatch_error(&item.name, &detail.expected, &detail.actual));
                        }
                    }
                    let value = if !is_top_level_decl && item.value.is_some() && resource_exists {
                        self.get_resource(&item.name).unwrap_or(value)
                    } else {
                        value
                    };
                    let binding_type = if matches!(item.type_ref, TypeRef::Infer) {
                        item.value.as_ref().map(|expr| self.inferred_expression_binding_type(locals, expr, &value))
                            .unwrap_or_else(|| inferred_binding_type(&value))
                    } else { item.type_ref.clone() };
                    self.declare_binding_type(&item.name, &binding_type);
                    if is_top_level_decl {
                        self.binding_type_scopes[0].insert(item.name.clone(), (binding_type, false));
                    }
                    if let Some(scope)=self.local_body_scopes.last_mut() {
                        scope.previous.insert(item.name.clone(),locals.get(&item.name).cloned());
                    }
                    locals.insert(item.name.clone(), value);
                    if !body_local && item.value.is_some() && !resource_exists {
                        let initial = locals
                            .get(&item.name)
                            .cloned()
                            .unwrap_or(Value::None);
                        if !matches!(initial, Value::None) {
                            self.set_source_resource(&item.name, initial, item.id)?;
                        }
                    }
                    if matches!(item.kind, ddonirang_lang::DeclKind::Butbak) {
                        self.declare_const(&item.name);
                    }
                }
                Ok(FlowControl::Continue)
            }),
            Stmt::Definition {
                id,
                name,
                type_ref,
                value,
                ..
            } => in_statement_frame(|| {
                if self.guard_rejected {
                    return Ok(FlowControl::Continue);
                }
                if self.local_body_scopes.last().map(|scope|scope.previous.contains_key(name))
                    .unwrap_or_else(||locals.contains_key(name)) {
                    return Err(format!(
                        "E_V25_DEFINITION_DUPLICATE: 이미 정의된 이름을 다시 정의할 수 없습니다: {}",
                        name
                    )
                    .into());
                }
                // The V25 public frontdoor admits `:=` for first bindings in
                // a top-level `채비` block.  That surface reaches this AST
                // variant (rather than `DeclBlock`), but it has the same
                // persistent-state obligation: a later hook must see the
                // initialized resource on this and following madis.
                let is_top_level_decl = !self.owns_pure_local_bindings()
                    && self.pending_top_level_decl_names.remove(name);
                let resource_exists = if is_top_level_decl {
                    self.committed_resource_exists(name)
                } else {
                    self.resource_exists(name)
                };
                let evaluated = self.eval_expr_expected(locals, value, type_ref)?;
                if self.aborted {
                    return Ok(FlowControl::Continue);
                }
                if let Err(detail) = self.check_value_type(&evaluated, type_ref) {
                    return Err(type_mismatch_error(name, &detail.expected, &detail.actual));
                }
                let evaluated = if is_top_level_decl && !resource_exists {
                    self.param_overrides
                        .and_then(|overrides| overrides.get(name))
                        .cloned()
                        .unwrap_or(evaluated)
                } else if is_top_level_decl && resource_exists {
                    self.get_resource(name).unwrap_or(evaluated)
                } else {
                    evaluated
                };
                if let Err(detail) = self.check_value_type(&evaluated, type_ref) {
                    return Err(type_mismatch_error(name, &detail.expected, &detail.actual));
                }
                let binding_type = if matches!(type_ref, TypeRef::Infer) {
                    self.inferred_expression_binding_type(locals, value, &evaluated)
                } else { type_ref.clone() };
                self.declare_binding_type(name, &binding_type);
                if is_top_level_decl {
                    self.binding_type_scopes[0].insert(name.clone(), (binding_type, false));
                }
                if let Some(scope)=self.local_body_scopes.last_mut() {
                    scope.previous.insert(name.clone(),locals.get(name).cloned());
                }
                locals.insert(name.clone(), evaluated);
                if is_top_level_decl && !resource_exists {
                    let initial = locals.get(name).cloned().unwrap_or(Value::None);
                    if !matches!(initial, Value::None) {
                        self.set_source_resource(name, initial, *id)?;
                    }
                }
                Ok(FlowControl::Continue)
            }),
            Stmt::Mutate { id, target, value, .. } => in_statement_frame(|| {
                if self.guard_rejected {
                    return Ok(FlowControl::Continue);
                }
                let mut selection_indices = Vec::new();
                let mut root = target;
                let target_name = loop {
                    match &root.kind {
                        ExprKind::Var(name) => break name,
                        ExprKind::SelectionPlace { target, index } => {
                            selection_indices.push(IndexedSelectionExpr::Single(index.as_ref()));
                            root = target;
                        }
                        ExprKind::Call { func, args } if func == "차림.값" && args.len() == 2 => {
                            selection_indices.push(IndexedSelectionExpr::Single(&args[1].expr));
                            root = &args[0].expr;
                        }
                        ExprKind::MultiAxisSelection(selection) => {
                            selection_indices.push(IndexedSelectionExpr::Axes(&selection.selectors));
                            root = &selection.target;
                        }
                        _ => return Err("대입 대상은 이름에서 시작하는 선택 위치여야 합니다".to_string().into()),
                    }
                };
                selection_indices.reverse();
                if self.is_const(target_name) {
                    return Err(format!("붙박이는 재대입할 수 없습니다: {}", target_name).into());
                }
                // A simple-name assignment in a pure callable publishes its local
                // only after all fallible checks. These RHS kinds cannot write
                // locals, so rollback needs metadata but not a deep local copy.
                let keep_locals = !(self.owns_pure_local_bindings()
                    && selection_indices.is_empty()
                    && expression_preserves_local_bindings(value));
                let assignment_snapshot = self.capture_frame_snapshot_with_locals(locals, keep_locals);
                let evaluated_value = if !selection_indices.is_empty() {
                    // Keep all selection evaluation inside the assignment result so every
                    // error flows through the snapshot restore below.
                    (|| {
                        let mut collection = self.eval_expr(locals, &Expr::new(
                            target.id,
                            target.span,
                            ExprKind::Var(target_name.clone()),
                        ))?;
                        let mut parents = Vec::new();
                        let last = selection_indices.len() - 1;
                        for index_expr in &selection_indices[..last] {
                            let index = self.eval_indexed_selection(locals, index_expr)?;
                            let child = self.read_indexed_selection(locals, collection.clone(), &index, target)?;
                            parents.push((collection, index));
                            collection = child;
                        }
                        let index = self.eval_indexed_selection(locals, &selection_indices[last])?;
                        let replacement = self.eval_expr(locals, value)?;
                        let mut updated = self.replace_indexed_selection(locals, collection, &index, replacement, target)?;
                        // Rebuild immutable parents; publish only the final root.
                        for (parent, index) in parents.into_iter().rev() {
                            updated = self.replace_indexed_selection(locals, parent, &index, updated, target)?;
                        }
                        Ok(updated)
                    })()
                } else {
                    let expected = self.result_place_frames.last()
                        .filter(|frame| frame.canonical_name == *target_name)
                        .and_then(|frame| frame.result_type.clone())
                        .or_else(|| self.binding_type_scopes.iter().rev()
                            .find_map(|scope| scope.get(target_name).map(|(kind, _)| kind.clone())));
                    self.eval_expr_expected(locals, value, expected.as_ref().unwrap_or(&TypeRef::Infer))
                };
                if self.aborted {
                    return Ok(FlowControl::Continue);
                }
                let val = match evaluated_value {
                    Ok(val) => val,
                    Err(err) => {
                        self.restore_frame_snapshot(locals, &assignment_snapshot);
                        return Err(err);
                    }
                };
                if selection_indices.is_empty()
                    && self
                        .result_place_frames
                        .iter()
                        .any(|frame| frame.canonical_name == *target_name)
                {
                    if self.current_seed_name.as_deref() != Some(target_name.as_str()) {
                        self.restore_frame_snapshot(locals, &assignment_snapshot);
                        return Err(
                            "E_RESULTPLACE_CAPTURE_FORBIDDEN: 바깥 호출의 결과 자리를 쓸 수 없습니다"
                                .to_string()
                                .into(),
                        );
                    }
                    if let Err(error) = self.assign_current_result_place(val) {
                        self.restore_frame_snapshot(locals, &assignment_snapshot);
                        return Err(error);
                    }
                    return Ok(FlowControl::Continue);
                }
                if selection_indices.is_empty()
                    && self.current_seed_name.as_deref() == Some(target_name.as_str())
                {
                    self.restore_frame_snapshot(locals, &assignment_snapshot);
                    return Err(
                        "E_RESULTPLACE_ABSENT: 결과씨 없는 호출에는 결과 자리가 없습니다"
                            .to_string()
                            .into(),
                    );
                }
                // Pure named callables own their ordinary bindings. A local
                // pin/temp with the same spelling as a world resource must not
                // acquire a world write merely because that resource exists.
                let pattern_local = self.sum_pattern_bindings.iter().any(|name|name==target_name);
                let pure_local = self.owns_pure_local_bindings() || pattern_local
                    || self.local_body_scopes.iter().rev().any(|scope|scope.previous.contains_key(target_name));
                if pure_local && self.visible_binding_type(target_name).is_none() {
                    self.declare_binding_type(target_name, &self.inferred_expression_binding_type(locals, value, &val));
                }
                if let Some(old @ Value::Tuple(_))=locals.get(target_name) {
                    if !tuple_position_type_equal(old,&val) {
                        self.restore_frame_snapshot(locals,&assignment_snapshot);
                        return Err("E_TUPLE_REPLACEMENT: 위치형 재결속의 위치씨가 다릅니다".to_string().into());
                    }
                }
                if let Err(error) = self.check_binding_type(target_name, &val) {
                    self.restore_frame_snapshot(locals, &assignment_snapshot);
                    if self.root_program_active && self.current_seed_name.is_none() && !pure_local {
                        if let Some(expected) = self.root_binding_types.get(target_name) {
                            return Err(format!(
                                "E_ROOT_BINDING_TYPE_MISMATCH: '{}'의 재결속 값 형식이 다릅니다 (기대: {}, 실제: {})",
                                target_name, expected, root_binding_type_key(&val)
                            ).into());
                        }
                    }
                    return Err(error);
                }
                if self.root_program_active && self.current_seed_name.is_none() && !pure_local {
                    let actual = root_binding_type_key(&val);
                    if let Some(expected) = self.root_binding_types.get(target_name).cloned() {
                        if expected != actual {
                            self.restore_frame_snapshot(locals, &assignment_snapshot);
                            return Err(format!(
                                "E_ROOT_BINDING_TYPE_MISMATCH: '{}'의 재결속 값 형식이 다릅니다 (기대: {}, 실제: {})",
                                target_name, expected, actual
                            )
                            .into());
                        }
                    } else {
                        self.root_binding_types.insert(target_name.clone(), actual);
                    }
                }
                if self.visible_binding_type(target_name).is_none()
                {
                    self.declare_binding_type(target_name, &self.inferred_expression_binding_type(locals, value, &val));
                }
                if pure_local {
                    locals.insert(target_name.clone(), val);
                } else if locals.contains_key(target_name) {
                    locals.insert(target_name.clone(), val.clone());
                    if self.resource_exists(target_name) {
                        if let Err(err) = self.set_source_resource(target_name, val, *id) {
                            self.restore_frame_snapshot(locals, &assignment_snapshot);
                            return Err(err);
                        }
                    }
                } else if matches!(val,Value::SumBranch {..}) {
                    // Existing VC08 contract: branch values are lexical only;
                    // do not create a resource carrier or a JSON fallback.
                    locals.insert(target_name.clone(),val);
                } else {
                    match self.set_source_resource(target_name, val, *id) {
                        Ok(()) => {}
                        Err(err) => {
                            self.restore_frame_snapshot(locals, &assignment_snapshot);
                            return Err(err);
                        }
                    }
                }
                Ok(FlowControl::Continue)
            }),
            Stmt::ResultFlow {
                producer,
                destination,
                ..
            } => in_statement_frame(|| {
                if self.guard_rejected {
                    return Ok(FlowControl::Continue);
                }
                let (target, index) = match &destination.kind {
                    ExprKind::SelectionPlace {target,index} => (target.as_ref(),IndexedSelectionExpr::Single(index)),
                    ExprKind::MultiAxisSelection(selection) if selection.writable => (selection.target.as_ref(),IndexedSelectionExpr::Axes(&selection.selectors)),
                    _ => return Err("E_RESULT_FLOW_DESTINATION_PLACE_REQUIRED".to_string().into()),
                };
                let ExprKind::Var(root_name) = &target.kind else {
                    return Err("E_RESULT_FLOW_ROOT_REQUIRED".to_string().into());
                };
                if self.is_const(root_name) {
                    return Err(format!("붙박이는 재대입할 수 없습니다: {}", root_name).into());
                }

                // A result-flow preserves the producer/result role.  Evaluate the
                // producer before reading the writable place, build an immutable
                // replacement candidate, and commit the root only after every
                // step succeeds.
                let assignment_snapshot = self.capture_frame_snapshot(locals);
                let replacement = (|| {
                    let produced = self.eval_expr(locals, producer)?;
                    let collection = self.eval_expr(locals, target)?;
                    let selection_index = self.eval_indexed_selection(locals, &index)?;
                    self.replace_indexed_selection(locals, collection, &selection_index, produced, destination)
                })();
                let replacement = match replacement {
                    Ok(value) => value,
                    Err(err) => {
                        self.restore_frame_snapshot(locals, &assignment_snapshot);
                        return Err(err);
                    }
                };

                let pattern_local = self.sum_pattern_bindings.iter().any(|name|name==root_name);
                let pure_local = self.owns_pure_local_bindings() || pattern_local
                    || self.local_body_scopes.iter().rev().any(|scope|scope.previous.contains_key(root_name));
                if let Err(error) = self.check_binding_type(root_name, &replacement) {
                    self.restore_frame_snapshot(locals, &assignment_snapshot);
                    return Err(error);
                }
                if self.root_program_active && self.current_seed_name.is_none() && !pure_local {
                    let actual = root_binding_type_key(&replacement);
                    if let Some(expected) = self.root_binding_types.get(root_name).cloned() {
                        if expected != actual {
                            self.restore_frame_snapshot(locals, &assignment_snapshot);
                            return Err(format!(
                                "E_ROOT_BINDING_TYPE_MISMATCH: '{}'의 결과흐름 값 형식이 다릅니다 (기대: {}, 실제: {})",
                                root_name, expected, actual
                            )
                            .into());
                        }
                    } else {
                        self.root_binding_types.insert(root_name.clone(), actual);
                    }
                }
                if pure_local && locals.contains_key(root_name) {
                    locals.insert(root_name.clone(), replacement);
                } else if locals.contains_key(root_name) {
                    locals.insert(root_name.clone(), replacement.clone());
                    if self.resource_exists(root_name) {
                        if let Err(err) = self.set_resource(root_name, replacement) {
                            self.restore_frame_snapshot(locals, &assignment_snapshot);
                            return Err(err);
                        }
                    }
                } else if self.resource_exists(root_name) {
                    if let Err(err) = self.set_resource(root_name, replacement) {
                        self.restore_frame_snapshot(locals, &assignment_snapshot);
                        return Err(err);
                    }
                } else {
                    self.restore_frame_snapshot(locals, &assignment_snapshot);
                    return Err(format!("E_RESULT_FLOW_ROOT_UNBOUND: {}", root_name).into());
                }
                Ok(FlowControl::Continue)
            }),
            Stmt::Receive { .. } => in_statement_frame(|| Ok(FlowControl::Continue)),
            Stmt::Send {
                transfer,
                sender,
                payload,
                receiver,
                span,
                ..
            } => in_statement_frame(|| {
                match transfer {
                    MessageTransfer::HistoricalAsync(_) => {
                        self.dispatch_signal_send(locals, sender.as_ref(), payload, receiver)?;
                    }
                    MessageTransfer::Async(_) => {
                        self.dispatch_async_send(
                            locals,
                            sender.as_ref(),
                            payload,
                            receiver,
                            *span,
                        )?;
                    }
                    MessageTransfer::Rendezvous(_) => {
                        return Err("E_RENDEZVOUS_PRODUCT_OPEN: synchronized hand-off runtime is not implemented".to_string().into());
                    }
                }
                Ok(FlowControl::Continue)
            }),
            Stmt::Expr { expr, .. } => in_statement_frame(|| {
                if self.program.program.origin.retirement_commands.contains_key(&expr.id) {
                    let stage=self.subject_stage.as_ref().map(Rc::clone).ok_or_else(||EvalError::from(
                        "E_VC04_RETIRE_CONTEXT_REQUIRED: 봉인된 대상·가리킴과 폐기 권한의 실행 공급이 필요합니다".to_string()))?;
                    let ExprKind::Call{args,..}=&expr.kind else{return Err("E_VC04_RETIRE_PIN".to_string().into())};
                    let target=args.first().ok_or_else(||EvalError::from("E_VC04_RETIRE_PIN".to_string()))?;
                    let target_value=self.eval_expr(locals,&target.expr)?;
                    let Value::IdentityReference(reference)=&target_value else {
                        return Err("E_VC04_RETIRE_TARGET: 임자를 가리키는 가리킴이 필요합니다".to_string().into());
                    };
                    stage.borrow_mut().retire(&reference).map_err(EvalError::from)?;
                    return Ok(FlowControl::Continue);
                }
                match self.eval_expr(locals, expr) {
                    Ok(_) => {}
                    Err(err) => return Err(err),
                };
                Ok(FlowControl::Continue)
            }),
            Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => in_statement_frame(|| {
                match self.eval_expr(locals, expr) {
                    Ok(value) => {
                        self.result_emissions.stage(self.tick_id,
                            self.program.result_output_type_plan.as_deref().and_then(|plan|plan.kind(expr.id))
                                .or_else(||self.program.numeric_constraint_plan.as_deref().and_then(|plan|plan.position_types.get(&expr.id))),
                            &self.program.nominal_type_ids(),&value,
                            self.program.result_output_type_plan.as_deref().map(|plan|plan.source()),&self.program.registered_atom_contracts);
                        self.append_output_log(value)?;
                    },
                    Err(err) => return Err(err),
                };
                Ok(FlowControl::Continue)
            }),
            Stmt::PublicObservation { fields, .. } => in_statement_frame(|| {
                self.emit_public_observation(locals, fields)?;
                Ok(FlowControl::Continue)
            }),
            Stmt::MetaBlock { kind, entries, .. } => in_statement_frame(|| {
                if matches!(kind, ddonirang_lang::MetaBlockKind::Bogae) {
                    self.apply_bogae_meta_block(locals, entries)?;
                }
                Ok(FlowControl::Continue)
            }),
            Stmt::Pragma { .. } => in_statement_frame(|| Ok(FlowControl::Continue)),
            Stmt::Return { value, .. } => in_statement_frame(|| {
                let expected = self.result_place_frames.last().and_then(|frame| frame.result_type.clone());
                let val = self.eval_expr_expected(locals, value, expected.as_ref().unwrap_or(&TypeRef::Infer))?;
                Ok(FlowControl::Return(val))
            }),
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => in_statement_frame(|| {
                let cond = self.eval_expr(locals, condition)?;
                if is_truthy(&cond)? {
                    return Ok(self.eval_body(locals, then_body)?);
                } else if let Some(body) = else_body {
                    return Ok(self.eval_body(locals, body)?);
                }
                Ok(FlowControl::Continue)
            }),
            Stmt::Try { action, body, .. } => in_statement_frame(|| {
                let value = self.eval_expr(locals, action)?;
                let prev = locals.insert("그것".to_string(), value);
                let out = self.eval_body(locals, body)?;
                if let Some(prev) = prev {
                    locals.insert("그것".to_string(), prev);
                } else {
                    locals.remove("그것");
                }
                Ok(out)
            }),
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => in_statement_frame(|| {
                for branch in branches {
                    let cond = self.eval_expr(locals, &branch.condition)?;
                    if is_truthy(&cond)? {
                        return Ok(self.eval_body(locals, &branch.body)?);
                    }
                }
                Ok(self.eval_body(locals, else_body)?)
            }),
            Stmt::Repeat { body, .. } => in_statement_frame(|| {
                loop {
                    if self.aborted {
                        break;
                    }
                    match self.eval_body(locals, body)? {
                        FlowControl::Continue => {}
                        FlowControl::Return(value) => return Ok(FlowControl::Return(value)),
                        FlowControl::ContinueLoop(_) => continue,
                        FlowControl::Break(_) => break,
                    }
                }
                Ok(FlowControl::Continue)
            }),
            Stmt::While {
                condition, body, ..
            } => in_statement_frame(|| {
                loop {
                    if self.aborted {
                        break;
                    }
                    let cond = self.eval_expr(locals, condition)?;
                    if !is_truthy(&cond)? {
                        break;
                    }
                    match self.eval_body(locals, body)? {
                        FlowControl::Continue => {}
                        FlowControl::Return(value) => return Ok(FlowControl::Return(value)),
                        FlowControl::ContinueLoop(_) => continue,
                        FlowControl::Break(_) => break,
                    }
                }
                Ok(FlowControl::Continue)
            }),
            Stmt::ForEach {
                item,
                iterable,
                body,
                ..
            } => in_statement_frame(|| {
                let mut iter_value = self.eval_expr(locals, iterable)?;
                let items = match &mut iter_value {
                    Value::List(items) => std::mem::take(items),
                    Value::Set(items) => std::mem::take(items).into_values().collect(),
                    Value::Map(entries) => std::mem::take(entries)
                        .into_values()
                        .map(|entry| Value::List(vec![entry.key, entry.value]))
                        .collect(),
                    _ => {
                        return Err("순회 대상은 차림/모음/짝맞춤이어야 합니다"
                            .to_string()
                            .into())
                    }
                };
                let previous = locals.get(item).cloned();
                let previous_types = self.binding_type_scopes.last().unwrap().clone();
                let previous_root_type = self.root_binding_types.remove(item);
                self.declare_binding_type(item, &TypeRef::Infer);
                let outcome = (|| -> Result<FlowControl, EvalError> {
                    for value in items {
                        if self.aborted {
                            break;
                        }
                        self.work_cost.borrow_mut().charge_element_visits(1)?;
                        self.root_binding_types.remove(item);
                        self.declare_binding_type(item, &inferred_binding_type(&value));
                        locals.insert(item.clone(), value);
                        let iteration_locals = locals.keys().cloned().collect::<HashSet<_>>();
                        let step = self.eval_body(locals, body);
                        locals.retain(|name, _| iteration_locals.contains(name));
                        self.binding_type_scopes.last_mut().unwrap().retain(|name, _| iteration_locals.contains(name));
                        match step? {
                            FlowControl::Continue => {}
                            FlowControl::Return(value) => {
                                return Ok(FlowControl::Return(value));
                            }
                            FlowControl::ContinueLoop(_) => continue,
                            FlowControl::Break(_) => break,
                        }
                        if self.aborted {
                            break;
                        }
                    }
                    Ok(FlowControl::Continue)
                })();
                restore_local_binding(locals, item, previous);
                *self.binding_type_scopes.last_mut().unwrap() = previous_types;
                self.root_binding_types.remove(item);
                if let Some(previous) = previous_root_type {
                    self.root_binding_types.insert(item.clone(), previous);
                }
                outcome
            }),
            Stmt::Hook { kind, body, .. } => in_statement_frame(|| {
                if self.periodic_hook_is_due(*kind)? {
                    self.eval_body(locals, body)
                } else {
                    Ok(FlowControl::Continue)
                }
            }),
            Stmt::BeatBlock { .. }
            | Stmt::HookWhenBecomes { .. }
            | Stmt::HookWhile { .. } => in_statement_frame(|| Err(
                "E_RUNTIME_UNSUPPORTED_STMT: 현재 ddonirang-tool 실행기에서는 박자/훅 문장을 지원하지 않습니다"
                    .to_string()
                    .into(),
            )),
            Stmt::Quantifier { .. } => in_statement_frame(|| Ok(FlowControl::Continue)),
            Stmt::Break { span, .. } => in_statement_frame(|| Ok(FlowControl::Break(*span))),
            Stmt::ContinueLoop { span, .. } => in_statement_frame(|| Ok(FlowControl::ContinueLoop(*span))),
            Stmt::Contract {
                kind,
                mode,
                condition,
                then_body,
                else_body,
                ..
            } => in_statement_frame(|| {
                let snapshot = self.capture_frame_snapshot(locals);
                let cond_value = self.eval_expr(locals, condition)?;
                let ok = is_truthy(&cond_value)?;
                match kind {
                    ddonirang_lang::ContractKind::Pre => {
                        if ok {
                            if let Some(body) = then_body {
                                let out = self.eval_body(locals, body)?;
                                if self.aborted {
                                    self.restore_frame_snapshot_preserving_abort_contract_diag(
                                        locals, &snapshot,
                                    );
                                    return Ok(FlowControl::Continue);
                                }
                                return Ok(out);
                            }
                            Ok(FlowControl::Continue)
                        } else {
                            match self.eval_body(locals, else_body)? {
                                FlowControl::Continue => {}
                                other => return Ok(other),
                            }
                            if self.aborted {
                                self.restore_frame_snapshot_preserving_abort_contract_diag(
                                    locals, &snapshot,
                                );
                                return Ok(FlowControl::Continue);
                            }
                            if matches!(mode, ddonirang_lang::ContractMode::Abort) {
                                self.restore_frame_snapshot(locals, &snapshot);
                            }
                            self.emit_contract_violation(
                                ddonirang_lang::ContractKind::Pre,
                                *mode,
                                condition,
                                "전제하에 조건이 실패했습니다".to_string(),
                            );
                            Ok(FlowControl::Continue)
                        }
                    }
                    ddonirang_lang::ContractKind::Post => {
                        if !ok {
                            match self.eval_body(locals, else_body)? {
                                FlowControl::Continue => {}
                                other => return Ok(other),
                            }
                            if self.aborted {
                                self.restore_frame_snapshot_preserving_abort_contract_diag(
                                    locals, &snapshot,
                                );
                                return Ok(FlowControl::Continue);
                            }
                            let cond_value = self.eval_expr(locals, condition)?;
                            if !is_truthy(&cond_value)? {
                                if matches!(mode, ddonirang_lang::ContractMode::Abort) {
                                    self.restore_frame_snapshot(locals, &snapshot);
                                }
                                self.emit_contract_violation(
                                    ddonirang_lang::ContractKind::Post,
                                    *mode,
                                    condition,
                                    "보장하고 조건이 실패했습니다".to_string(),
                                );
                                return Ok(FlowControl::Continue);
                            }
                        }
                        if let Some(body) = then_body {
                            let out = self.eval_body(locals, body)?;
                            if self.aborted {
                                self.restore_frame_snapshot_preserving_abort_contract_diag(
                                    locals, &snapshot,
                                );
                                return Ok(FlowControl::Continue);
                            }
                            return Ok(out);
                        }
                        Ok(FlowControl::Continue)
                    }
                }
            }),
            Stmt::Guard {
                condition, body, ..
            } => in_statement_frame(|| {
                let cond_value = self.eval_expr(locals, condition)?;
                if is_truthy(&cond_value)? {
                    self.guard_rejected = true;
                    self.patch_ops.clear();
                    self.emit_guard_violation(condition.id);
                    return Ok(self.eval_body(locals, body)?);
                }
                Ok(FlowControl::Continue)
            }),
        }
    }

    pub(super) fn source_numeric_literal_tree(expr: &Expr) -> bool {
        crate::numeric_constraint_plan::literal_arithmetic_tree(expr)
    }

    pub(super) fn source_literal_exact(&mut self, expr: &Expr) -> Result<BigRational, EvalError> {
        let source = expr.numeric_literal_source.as_ref().ok_or_else(|| EvalError::from(
            "E_TYPE_MISMATCH: 수 리터럴의 원 token 근거가 없습니다".to_string()))?;
        self.work_cost.borrow_mut().charge_canonical_utf8_bytes(source.raw())?;
        let (whole, fraction) = source.raw().split_once('.').unwrap_or((source.raw(), ""));
        let coefficient = BigInt::parse_bytes(format!("{whole}{fraction}").as_bytes(), 10)
            .ok_or_else(|| EvalError::from("E_TYPE_MISMATCH: 정확한 수 리터럴이 아닙니다".to_string()))?;
        let scale = u32::try_from(fraction.len())
            .map_err(|_| EvalError::from("E_TYPE_MISMATCH: 수 리터럴 자릿수 범위 초과".to_string()))?;
        Ok(BigRational::new(coefficient, BigInt::from(10).pow(scale)))
    }

    pub(super) fn issue_planned_numeric_literal(&mut self, expr: &Expr, kind: crate::numeric_constraint_plan::NumericKind) -> Result<Value, EvalError> {
        use crate::numeric_constraint_plan::NumericKind as K;
        let ExprKind::Literal(literal) = &expr.kind else {
            return Err("E_TYPE_MISMATCH: 정적 수 발급 위치가 리터럴이 아닙니다".to_string().into());
        };
        match kind {
            K::Approximate => Ok(literal_to_value(literal)),
            K::Rational => Ok(make_rational_pack_from_big_rational(&self.source_literal_exact(expr)?)),
            K::BigInteger => {
                let exact = self.source_literal_exact(expr)?;
                if !exact.is_integer() { return Err("E_TYPE_MISMATCH: 큰바른수 리터럴은 정수여야 합니다".to_string().into()); }
                Ok(make_big_int_pack_from_bigint(&exact.to_integer()))
            }
            K::Integer => {
                let exact = self.source_literal_exact(expr)?;
                if !exact.is_integer() {
                    return Err("E_TYPE_MISMATCH: 바른수 리터럴의 정확한 정수 범위를 벗어났습니다".to_string().into());
                }
                // Check the exact Q32 carrier before construction. from_i64
                // saturates and therefore cannot be an admission operation.
                let raw=(exact.to_integer()<<32usize).to_i64().ok_or_else(||EvalError::from(
                    "E_TYPE_MISMATCH: 바른수 리터럴의 정확한 물리 표현 범위를 벗어났습니다".to_string()))?;
                Ok(Value::Fixed64(Fixed64::from_raw_i64(raw)))
            }
            K::Factor => Err("E_TYPE_MISMATCH: 곱수 리터럴의 정적 발급 계약이 공급되지 않았습니다".to_string().into()),
        }
    }




}

#[cfg(test)]
mod lexical_body_scope_tests {
    use super::*;
    use crate::ddn_runtime::{DdnProgram,DdnRunner,NuriWorld};
    use crate::runtime_surface::DEFAULT_UPDATE_NAME;

    fn output(source:&str)->Result<Vec<String>,String> {
        let program=DdnProgram::from_v25_source(source,"lexical-body.ddn")?;
        let summary=DdnRunner::new(program,DEFAULT_UPDATE_NAME)
            .run_update(&NuriWorld::new(),&ddonirang_core::InputSnapshot { tick_id:0,dt:Fixed64::from_i64(1),keys_pressed:0,last_key_name:String::new(),pointer_x_i32:0,pointer_y_i32:0,ai_injections:Vec::new(),net_events:Vec::new(),frame_source:ddonirang_core::InputSource::Person,rng_seed:0 },&HashMap::new())?;
        Ok(summary.display_outputs.into_iter().map(|out|out.text).collect())
    }

    #[test]
    fn inner_definition_reads_outer_and_restores_only_its_declaration() {
        for (body,expected) in [
            ("{ 참 }이면 { 지역:수 := 지역 + 1. }.","1"),
            ("{ 참 }이면 { 지역 <- 지역 + 1. 지역:수 := 지역 + 10. }.","2"),
            ("{ 참 }이면 { 임시:수 := 9. 지역 <- 지역 + 1. }.","2"),
            ("{ 참 }이면 { 지역:수 := 지역 + 1. { 참 }이면 { 지역:수 := 지역 + 10. }. }.","1"),
        ] {
            let source=format!("(입력값:수~을) 범위값:수:셈씨 := {{ 지역:수 := 입력값. {body} 범위값 <- 지역. }}.\n((입력값=1) 범위값) 보여주기.");
            assert_eq!(output(&source).unwrap(),vec![expected.to_string()],"{source}");
        }
    }

    #[test]
    fn inner_definition_does_not_escape_and_same_scope_duplicate_stays_rejected() {
        let missing="(입력값:수~을) 범위값:수:셈씨 := { { 참 }이면 { 임시:수 := 9. }. 범위값 <- 임시. }. ((입력값=1) 범위값) 보여주기.";
        assert!(output(missing).is_err());
        let duplicate="(입력값:수~을) 범위값:수:셈씨 := { 입력값:수 := 9. 범위값 <- 입력값. }. ((입력값=1) 범위값) 보여주기.";
        assert!(output(duplicate).is_err());
    }
    #[test]
    fn lexical_types_restore_and_outer_assignment_keeps_its_type() {
        let good="(입력값:수~을) 범위값:수:셈씨 := { 지역:수 := 입력값. { 참 }이면 { 지역:글 := \"안\". 지역 <- \"녕\". }. 범위값 <- 지역. }. ((입력값=1) 범위값) 보여주기.";
        assert_eq!(output(good).unwrap(),vec!["1".to_string()]);
        let bad="(입력값:수~을) 범위값:수:셈씨 := { 지역:수 := 입력값. { 참 }이면 { 지역 <- \"글\". }. 범위값 <- 지역. }. ((입력값=1) 범위값) 보여주기.";
        assert!(output(bad).is_err());
    }

    #[test]
    fn runtime_error_unwinds_only_successfully_shadowed_bindings() {
        let source="정리:수:셈씨 := { 지역:수 := 1. { 참 }이면 { 지역 <- 지역 + 1. 지역:수 := 지역 + 10. 없는값 보여주기. }. 정리 <- 지역. }.";
        let program=DdnProgram::from_v25_source(source,"scope-error.ddn").unwrap();
        let body=program.functions.get("정리").unwrap().body.as_ref().unwrap();
        let Stmt::If {then_body,..}=&body.stmts[1] else {panic!("if body")};
        let world=NuriWorld::new();let defaults=HashMap::new();
        let mut context=crate::ddn_runtime::EvalContext::new(&program,&world,&defaults,None,
            ddonirang_lang::InputState::new(0,0),0,0,None);
        let mut locals=HashMap::from([("지역".to_string(),Value::Fixed64(Fixed64::from_i64(1)))]);
        assert!(context.eval_body(&mut locals,then_body).is_err());
        assert_eq!(locals.get("지역"),Some(&Value::Fixed64(Fixed64::from_i64(2))));
        assert_eq!(locals.len(),1);assert!(context.local_body_scopes.is_empty());
        assert_eq!(context.binding_type_scopes.len(),1);
    }

}
