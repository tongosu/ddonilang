//! Evaluation context, metering, diagnostics and rollback; implementations moved unchanged.
use super::*;

impl<'a> EvalContext<'a> {
    pub(super) fn new(
        program: &'a DdnProgram,
        world: &'a NuriWorld,
        defaults: &'a HashMap<String, Value>,
        param_overrides: Option<&'a BTreeMap<String, Value>>,
        input: InputState,
        rng_seed: u64,
        tick_id: u64,
        project_asset_registry: Option<&'a gate0_registry::ProjectAssetRegistry>,
    ) -> Self {
        Self::new_with_work_cost(
            program,
            world,
            defaults,
            param_overrides,
            input,
            rng_seed,
            tick_id,
            project_asset_registry,
            Rc::new(RefCell::new(WorkCostMeter::default())),
            ddonirang_lang::DEFAULT_ASYNC_QUEUE_CAPACITY,
            InternalDdnRunnerDispatchState::default(),
        )
    }

    pub(super) fn new_with_work_cost(
        program: &'a DdnProgram,
        world: &'a NuriWorld,
        defaults: &'a HashMap<String, Value>,
        param_overrides: Option<&'a BTreeMap<String, Value>>,
        input: InputState,
        rng_seed: u64,
        tick_id: u64,
        project_asset_registry: Option<&'a gate0_registry::ProjectAssetRegistry>,
        work_cost: Rc<RefCell<WorkCostMeter>>,
        async_queue_capacity: usize,
        internal_dispatch: InternalDdnRunnerDispatchState,
    ) -> Self {
        Self {
            program,
            subject_stage:None,
            result_emissions: Default::default(),
            source_span_cache: Rc::new(RefCell::new(HashMap::new())),
            tuple_type_plan: Rc::new(crate::tuple_saved_type_plan::TupleSavedTypePlan::default()),
            world,
            defaults,
            param_overrides,
            resources: ResourceBindings::default(),
            input,
            logical_input: None,
            input_reads: Rc::new(RefCell::new(BTreeSet::new())),
            control_pressed: None,
            patch_ops: PatchLog::default(),
            guard_rejected: false,
            aborted: false,
            current_seed_name: None,
            active_callable_name: None,
            invocation_ordinal: 0,
            result_place_frames: Vec::new(),
            current_owner_identity: None,
            pending_async_signals: LogicalAsyncQueue::with_capacity(async_queue_capacity),
            async_outbox: StagedDeliveryOutbox::default(),
            async_wait_receipts: Vec::new(),
            rng_state: rng_seed,
            flow_stack: Vec::new(),
            tick_id,
            const_scopes: Vec::new(),
            binding_type_scopes: vec![HashMap::new()],
            pending_top_level_decl_names: program.top_level_decl_names.clone(),
            factor_route_counts: BTreeMap::new(),
            factor_bits_total: 0,
            factor_bits_min: None,
            factor_bits_max: 0,
            public_observations: Vec::new(),
            explicit_nfc_provenance: Vec::new(),
            display_outputs: Vec::new(),
            root_program_active: false,
            derived_initialized: false,
            root_binding_types: HashMap::new(),
            sum_definitions: program.program.root_stmts.iter().filter_map(|stmt|match stmt {
                Stmt::SumDefinition {name,branches,..} => Some((name.clone(),branches.clone())),
                _ => None,
            }).collect(),
            sum_pattern_bindings: Vec::new(),
            project_asset_registry,
            work_cost,
            internal_dispatch,
        }
    }

    pub(super) fn resolve_asset_handle(&self, path: &str) -> Result<ResourceHandle, EvalError> {
        match self.project_asset_registry {
            Some(registry) => registry.resolve_handle(path).map_err(EvalError::from),
            None => gate0_registry::resolve_asset_handle(path).map_err(EvalError::from),
        }
    }

    pub(super) fn enter_const_scope(&mut self) {
        self.const_scopes.push(HashSet::new());
        self.binding_type_scopes.push(HashMap::new());
    }

    pub(super) fn exit_const_scope(&mut self) {
        self.const_scopes.pop();
        self.binding_type_scopes.pop();
    }

    pub(super) fn declare_binding_type(&mut self, name: &str, type_ref: &TypeRef) {
        self.binding_type_scopes
            .last_mut()
            .expect("root binding scope")
            .insert(name.to_string(), (type_ref.clone(), false));
    }

    pub(super) fn check_binding_type(&self, name: &str, value: &Value) -> Result<(), EvalError> {
        // A call may see its own locals and persistent root resources, never a
        // caller's local type just because the spelling is the same.
        let binding_type = self.binding_type_scopes.last().and_then(|scope| scope.get(name))
            .or_else(|| self.binding_type_scopes.first().and_then(|scope| scope.get(name)));
        if let Some((type_ref, optional)) = binding_type {
            if *optional && matches!(value, Value::None) {
                return Ok(());
            }
            if let Err(detail) = self.check_value_type(value, type_ref) {
                return Err(type_mismatch_error(name, &detail.expected, &detail.actual));
            }
        }
        Ok(())
    }

    pub(super) fn declare_const(&mut self, name: &str) {
        if let Some(scope) = self.const_scopes.last_mut() {
            scope.insert(name.to_string());
        }
    }

    pub(super) fn is_const(&self, name: &str) -> bool {
        self.const_scopes
            .iter()
            .rev()
            .any(|scope| scope.contains(name))
    }

    /// Only an explicitly unique rational expectation may issue exact decimal
    /// literals. Unsupported trees fall through to established evaluation;
    /// no evaluated Fixed64 is reverse-converted and no mixed-literal ban is
    /// introduced while that admission policy remains with SSOT/Owner.

    pub(super) fn eval_callable(
        &mut self,
        locals: &HashMap<String, Value>,
        callable: &Value,
        args: Vec<Value>,
        source_span: Option<SourceSpan>,
    ) -> Result<Value, EvalError> {
        match callable {
            Value::Lambda(lambda) => self.eval_lambda(lambda, &args),
            Value::String(name) => {
                let func = name.trim_start_matches('#');
                self.eval_call(locals, func, args, source_span)
            }
            _ => Err("함수/씨앗 인자가 필요합니다".to_string().into()),
        }
    }

    pub(super) fn eval_lambda(&mut self, lambda: &LambdaValue, args: &[Value]) -> Result<Value, EvalError> {
        self.work_cost.borrow_mut().charge_dispatch()?;
        self.work_cost.borrow_mut().enter_call()?;
        let result = self.eval_lambda_after_dispatch(lambda, args);
        self.work_cost.borrow_mut().exit_call();
        result
    }

    pub(super) fn eval_lambda_after_dispatch(
        &mut self,
        lambda: &LambdaValue,
        args: &[Value],
    ) -> Result<Value, EvalError> {
        let params = lambda_param_names(&lambda.param);
        if args.len() != params.len() {
            return Err(format!("씨앗 인자는 {}개여야 합니다", params.len()).into());
        }
        let mut locals = lambda.captured.clone();
        for (param, arg) in params.iter().zip(args.iter()) {
            locals.insert(param.clone(), arg.clone());
        }
        self.eval_expr(&mut locals, &lambda.body)
    }

    pub(super) fn eval_inject_fields(
        &mut self,
        locals: &mut HashMap<String, Value>,
        inject: &[(String, Expr)],
    ) -> Result<Value, EvalError> {
        let mut map = std::collections::BTreeMap::new();
        for (name, value) in inject {
            if map.contains_key(name) {
                return Err(format!("주입 키 '{}'가 중복되었습니다", name).into());
            }
            let v = self.eval_expr(locals, value)?;
            map.insert(name.clone(), v);
        }
        Ok(Value::Pack(map))
    }

    pub(super) fn eval_pipe(
        &mut self,
        locals: &mut HashMap<String, Value>,
        stages: &[Expr],
    ) -> Result<Value, EvalError> {
        self.flow_stack.push(None);
        let idx = self.flow_stack.len() - 1;
        for stage in stages {
            let value = self.eval_expr(locals, stage)?;
            if !matches!(value, Value::None) {
                self.flow_stack[idx] = Some(value);
            }
        }
        let out = self.flow_stack.pop().unwrap_or(None);
        Ok(out.unwrap_or(Value::None))
    }

    pub(super) fn next_rng_u64(&mut self) -> u64 {
        let (state, value) = splitmix64_next(self.rng_state);
        self.rng_state = state;
        value
    }

    pub(super) fn next_rng_fixed64(&mut self) -> Fixed64 {
        let value = self.next_rng_u64();
        Fixed64::from_raw_i64((value & 0xFFFF_FFFF) as i64)
    }

    pub(super) fn canonicalize_stdlib_alias(name: &str) -> &str {
        ddonirang_lang::stdlib::canonicalize_stdlib_alias(name)
    }

    pub(super) fn charge_builtin_value_visits(
        &mut self,
        value: &Value,
        rule: BuiltinWorkCostRule,
    ) -> Result<(), EvalError> {
        match value {
            Value::String(text) if rule.visit_canonical_utf8_arguments => {
                self.work_cost
                    .borrow_mut()
                    .charge_canonical_utf8_bytes(text)?;
            }
            Value::Regex(regex) if rule.visit_canonical_utf8_arguments => {
                let mut meter = self.work_cost.borrow_mut();
                meter.charge_canonical_utf8_bytes(&regex.pattern)?;
                meter.charge_canonical_utf8_bytes(&regex.flags)?;
            }
            Value::Template(template) if rule.visit_canonical_utf8_arguments => {
                self.work_cost
                    .borrow_mut()
                    .charge_canonical_utf8_bytes(&template.raw)?;
            }
            Value::List(items) if rule.visit_collection_arguments => {
                self.work_cost
                    .borrow_mut()
                    .charge_element_visits(items.len())?;
                for item in items {
                    self.charge_builtin_value_visits(item, rule)?;
                }
            }
            Value::Set(items) if rule.visit_collection_arguments => {
                self.work_cost
                    .borrow_mut()
                    .charge_element_visits(items.len())?;
                for item in items.values() {
                    self.charge_builtin_value_visits(item, rule)?;
                }
            }
            Value::Map(entries) if rule.visit_collection_arguments => {
                self.work_cost
                    .borrow_mut()
                    .charge_element_visits(entries.len())?;
                for entry in entries.values() {
                    self.charge_builtin_value_visits(&entry.key, rule)?;
                    self.charge_builtin_value_visits(&entry.value, rule)?;
                }
            }
            Value::Pack(fields) | Value::Record { fields, .. } => {
                for value in fields.values() {
                    self.charge_builtin_value_visits(value, rule)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub(super) fn charge_big_number_parts(&self, values: &[&Value]) -> Result<(), EvalError> {
        let mut parts = Vec::<BigInt>::new();
        for value in values {
            if let Some(exact) = exact_numeric_from_value(value)? {
                parts.push(exact.value.numer().clone());
                if !exact.value.is_integer() {
                    parts.push(exact.value.denom().clone());
                }
            } else if let Value::String(text) = value {
                if let Ok(integer) = parse_bigint_text(text, "WorkCostV1 operand") {
                    parts.push(integer);
                }
            }
        }
        let refs = parts.iter().collect::<Vec<_>>();
        self.work_cost
            .borrow_mut()
            .charge_bigint_rational_primitive(&refs, &[])?;
        Ok(())
    }

    pub(super) fn eval_exact_numeric_infix_metered(
        &self,
        op: &str,
        left: &Value,
        right: &Value,
    ) -> Result<Option<Value>, EvalError> {
        if !is_exact_numeric_family_value(left) && !is_exact_numeric_family_value(right) {
            return Ok(None);
        }
        self.charge_big_number_parts(&[left, right])?;
        let result = eval_exact_numeric_infix(op, left, right)?;
        if let Some(value) = &result {
            self.charge_big_number_parts(&[value])?;
        }
        Ok(result)
    }

    pub(super) fn eval_exact_numeric_compare_infix_metered(
        &self,
        op: &str,
        left: &Value,
        right: &Value,
    ) -> Result<Option<bool>, EvalError> {
        if !is_exact_numeric_family_value(left) && !is_exact_numeric_family_value(right) {
            return Ok(None);
        }
        self.charge_big_number_parts(&[left, right])?;
        eval_exact_numeric_compare_infix(op, left, right)
    }

    pub(super) fn resolve_ordinary_callable_overload(
        &self,
        func: &str,
        args: &[Value],
    ) -> Result<Option<SeedDef>, EvalError> {
        let Some(candidates) = self.program.ordinary_overloads.get(func) else {
            return Ok(None);
        };
        if candidates.len() <= 1 {
            return Ok(None);
        }

        let mut matching = candidates
            .iter()
            .filter(|seed| {
                seed.params.len() == args.len()
                    && seed
                        .params
                        .iter()
                        .zip(args)
                        .all(|(param, value)| check_value_type(value, &param.type_ref).is_ok())
            })
            .cloned()
            .collect::<Vec<_>>();
        matching.sort_by_key(ddonirang_lang::ordinary_callable_input_signature);
        match matching.len() {
            1 => Ok(matching.pop()),
            0 => {
                let mut expected = candidates
                    .iter()
                    .map(ddonirang_lang::ordinary_callable_input_signature)
                    .collect::<Vec<_>>();
                expected.sort();
                Err(format!(
                    "E_ORDINARY_CALLABLE_NO_MATCH: effective_set={} candidates={}",
                    func,
                    expected.join("|")
                )
                .into())
            }
            _ => Err(format!(
                "E_ORDINARY_CALLABLE_AMBIGUOUS: effective_set={} candidates={}",
                func,
                matching
                    .iter()
                    .map(ddonirang_lang::ordinary_callable_input_signature)
                    .collect::<Vec<_>>()
                    .join("|")
            )
            .into()),
        }
    }

    pub(super) fn resource_backed_caller_names(&self, locals: &HashMap<String, Value>) -> Vec<String> {
        let caller_params = self
            .current_seed_name
            .as_ref()
            .and_then(|name| self.program.functions.get(name))
            .map(|seed| {
                seed.params
                    .iter()
                    .map(|param| param.pin_name.as_str())
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default();
        locals
            .iter()
            .filter(|(name, value)| {
                !caller_params.contains(name.as_str()) && self.resources.get(*name) == Some(*value)
            })
            .map(|(name, _)| name.clone())
            .collect()
    }

    pub(super) fn refresh_caller_resource_bindings(
        &self,
        locals: &mut HashMap<String, Value>,
        names: &[String],
    ) {
        for name in names {
            if let Some(value) = self.resources.get(name) {
                locals.insert(name.clone(), value.clone());
            }
        }
    }

    pub(super) fn eval_collection_search_equality(&self, left: &Value, right: &Value) -> Result<bool, EvalError> {
        // The caller already charges each candidate visit. Preserve existing
        // scalar search semantics/cost; composite values use RD-071 equality
        // and meter their additional traversal through the same Rust owner.
        let scalar = |value: &Value| matches!(value,
            Value::None | Value::Bool(_) | Value::Fixed64(_) | Value::Unit(_) | Value::String(_));
        if scalar(left) && scalar(right) {
            return Ok(values_equal(left, right));
        }
        self.eval_exact_value_equality(left, right)
    }

    pub(super) fn validate_exact_comparison_value(&self, value: &Value, operand: &str) -> Result<(), EvalError> {
        // RD-071: even an unequal earlier item cannot hide an incomparable
        // nested value. Paths are diagnostic-only, never semantic identities.
        enum Segment<'a> {
            Index(usize),
            Set(usize),
            MapKey(usize),
            MapValue(usize),
            Field(&'a str),
        }
        let mut pending = vec![(value, None)];
        // Keep structural paths without formatting every successful comparison.
        // Work charging and traversal order are unchanged; text is needed only
        // when an unsupported value actually produces a diagnostic.
        let mut paths: Vec<(Option<usize>, Segment<'_>)> = Vec::new();
        while let Some((value, parent)) = pending.pop() {
            self.work_cost.borrow_mut().charge_element_visits(1)?;
            let mut push = |child, segment| {
                let next = paths.len();
                paths.push((parent, segment));
                pending.push((child, Some(next)));
            };
            match value {
                Value::List(items) | Value::Tuple(items) => {
                    for (index, item) in items.iter().enumerate().rev() {
                        push(item, Segment::Index(index));
                    }
                }
                Value::Set(items) => {
                    for (index, item) in items.values().enumerate().rev() {
                        push(item, Segment::Set(index));
                    }
                }
                Value::Map(entries) => {
                    for (index, entry) in entries.values().enumerate().rev() {
                        push(&entry.value, Segment::MapValue(index));
                        push(&entry.key, Segment::MapKey(index));
                    }
                }
                Value::Pack(fields) => {
                    for (name, field) in fields.iter().rev() {
                        self.work_cost.borrow_mut().charge_canonical_utf8_bytes(name)?;
                        push(field, Segment::Field(name));
                    }
                }
                Value::BogaeRelation(_) | Value::BogaeComposition(_) => {
                    return Err("보개관계 정체 해소 전에는 관계를 비교할 수 없습니다".to_string().into());
                }
                Value::Scene(_) | Value::SceneElement(_) | Value::View(ddonirang_lang::runtime::ViewValue::Scene(_)) => {
                    return Err("E_SCENE_OPERATION_UNSUPPORTED: 장면 값의 비교·열쇠 사용은 아직 지원하지 않습니다".to_string().into());
                }
                Value::View(view) if matches!(view.without_time(), ddonirang_lang::runtime::ViewValue::Scene(_)) => {
                    return Err("E_SCENE_OPERATION_UNSUPPORTED: 장면 값의 비교·열쇠 사용은 아직 지원하지 않습니다".to_string().into());
                }
                Value::Lambda(_) | Value::ResourceHandle(_) | Value::StateMachine(_) | Value::Formula(_) => {
                    let mut segments = Vec::new();
                    let mut cursor = parent;
                    while let Some(index) = cursor {
                        segments.push(match &paths[index].1 {
                            Segment::Index(index) => format!("[{index}]"),
                            Segment::Set(index) => format!("[모음:{index}]"),
                            Segment::MapKey(index) => format!("[짝:{index}].열쇠"),
                            Segment::MapValue(index) => format!("[짝:{index}].값"),
                            // F5-GA[FIELD-JSON-STRING]: name은 UTF-8 &str이며 사용자 Serialize나 파일 writer가 아니다.
                            Segment::Field(name) => format!("[{}]", serde_json::to_string(name).unwrap()),
                        });
                        cursor = paths[index].0;
                    }
                    segments.reverse();
                    return Err(format!("E_VALUE_EQUALITY_UNSUPPORTED: {operand}${}의 값은 기본 동등성 비교를 지원하지 않습니다", segments.concat()).into());
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub(super) fn eval_exact_value_equality(&self, left: &Value, right: &Value) -> Result<bool, EvalError> {
        self.validate_exact_comparison_value(left, "왼쪽")?;
        self.validate_exact_comparison_value(right, "오른쪽")?;
        let mut pending = vec![(left, right)];
        while let Some((left, right)) = pending.pop() {
            self.work_cost.borrow_mut().charge_element_visits(1)?;
            if let Some(equal) = self.eval_exact_numeric_compare_infix_metered("==", left, right)? {
                if !equal { return Ok(false); }
                continue;
            }
            if let (Some(left), Some(right)) = (numeric_value(left), numeric_value(right)) {
                if left.dim != right.dim {
                    return Err(unit_error(UnitError::DimensionMismatch { left: left.dim, right: right.dim }));
                }
                if left.value.raw_i64() != right.value.raw_i64() { return Ok(false); }
                continue;
            }
            match (left, right) {
                (Value::List(left), Value::List(right)) | (Value::Tuple(left), Value::Tuple(right)) => {
                    if left.len() != right.len() { return Ok(false); }
                    pending.extend(left.iter().zip(right).rev());
                }
                (Value::Pack(left), Value::Pack(right)) | (Value::Set(left), Value::Set(right)) => {
                    if left.len() != right.len() { return Ok(false); }
                    for ((left_key, left_value), (right_key, right_value)) in left.iter().zip(right).rev() {
                        self.work_cost.borrow_mut().charge_canonical_utf8_bytes(left_key)?;
                        self.work_cost.borrow_mut().charge_canonical_utf8_bytes(right_key)?;
                        if left_key != right_key { return Ok(false); }
                        pending.push((left_value, right_value));
                    }
                }
                (Value::Map(left), Value::Map(right)) => {
                    if left.len() != right.len() { return Ok(false); }
                    for ((left_key, left_entry), (right_key, right_entry)) in left.iter().zip(right).rev() {
                        self.work_cost.borrow_mut().charge_canonical_utf8_bytes(left_key)?;
                        self.work_cost.borrow_mut().charge_canonical_utf8_bytes(right_key)?;
                        if left_key != right_key { return Ok(false); }
                        pending.push((&left_entry.value, &right_entry.value));
                        pending.push((&left_entry.key, &right_entry.key));
                    }
                }
                (Value::String(left), Value::String(right)) => {
                    self.work_cost.borrow_mut().charge_canonical_utf8_bytes(left)?;
                    self.work_cost.borrow_mut().charge_canonical_utf8_bytes(right)?;
                    if left != right { return Ok(false); }
                }
                _ if values_equal(left, right) => {}
                _ => return Ok(false),
            }
        }
        Ok(true)
    }

    pub(super) fn eval_infix(&self, op: &str, left: Value, right: Value) -> Result<Value, EvalError> {
        match op {
            "relation_eq" | "=:=" => {
                let Value::Formula(left_formula) = &left else {
                    return Err("방정식 관계의 왼쪽은 수식값이어야 합니다"
                        .to_string()
                        .into());
                };
                let Value::Formula(right_formula) = &right else {
                    return Err("방정식 관계의 오른쪽은 수식값이어야 합니다"
                        .to_string()
                        .into());
                };
                Ok(make_relation_pack(left_formula.clone(), right_formula.clone()))
            }
            "+" => {
                if matches!((&left, &right), (Value::String(_), Value::String(_))) {
                    return string_concat(&left, &right).map_err(runtime_error);
                }
                if let Some(exact) = self.eval_exact_numeric_infix_metered(op, &left, &right)? {
                    return Ok(exact);
                }
                let l = unit_value_from_value(&left).map_err(|_| {
                    type_mismatch_error(
                        "더하기 왼쪽 피연산자",
                        "수",
                        &root_binding_type_key(&left),
                    )
                })?;
                let r = unit_value_from_value(&right).map_err(|_| {
                    type_mismatch_error(
                        "더하기 오른쪽 피연산자",
                        "수",
                        &root_binding_type_key(&right),
                    )
                })?;
                let sum = checked_unit_add(l, r)?;
                Ok(unit_value_to_value(sum))
            }
            "-" => {
                if let Some(exact) = self.eval_exact_numeric_infix_metered(op, &left, &right)? {
                    return Ok(exact);
                }
                let l = unit_value_from_value(&left)?;
                let r = unit_value_from_value(&right)?;
                let diff = checked_unit_sub(l, r)?;
                Ok(unit_value_to_value(diff))
            }
            "*" => {
                if let Some(exact) = self.eval_exact_numeric_infix_metered(op, &left, &right)? {
                    return Ok(exact);
                }
                let l = unit_value_from_value(&left)?;
                let r = unit_value_from_value(&right)?;
                Ok(unit_value_to_value(checked_unit_mul(l, r)?))
            }
            "/" => {
                if let Some(exact) = self.eval_exact_numeric_infix_metered(op, &left, &right)? {
                    return Ok(exact);
                }
                let l = unit_value_from_value(&left)?;
                let r = unit_value_from_value(&right)?;
                let div = l.div(r).map_err(unit_error)?;
                Ok(unit_value_to_value(div))
            }
            "%" => {
                if let Some(exact) = self.eval_exact_numeric_infix_metered(op, &left, &right)? {
                    return Ok(exact);
                }
                let l = unit_value_from_value(&left)?;
                let r = unit_value_from_value(&right)?;
                if l.dim != r.dim {
                    return Err(unit_error(UnitError::DimensionMismatch {
                        left: l.dim,
                        right: r.dim,
                    }));
                }
                if r.value.raw_i64() == 0 {
                    return Err(unit_error(UnitError::DivisionByZero));
                }
                // MIN % -1도 기존 수식 경로와 같은 산술 넘침으로 거부한다.
                // 0 나눗셈의 기존 원인은 위 검사에서 그대로 유지한다.
                let raw = l.value.raw_i64().checked_rem(r.value.raw_i64())
                    .ok_or(ddonirang_core::ArithmeticFaultKind::Overflow)?;
                Ok(unit_value_to_value(UnitValue {
                    value: Fixed64::from_raw_i64(raw),
                    dim: l.dim,
                }))
            }
            "==" | "!=" => {
                let eq = self.eval_exact_value_equality(&left, &right)?;
                Ok(Value::Bool(if op == "==" { eq } else { !eq }))
            }
            "<" | "<=" | ">" | ">=" => {
                if let Some(cmp) =
                    self.eval_exact_numeric_compare_infix_metered(op, &left, &right)?
                {
                    return Ok(Value::Bool(cmp));
                }
                let l = unit_value_from_value(&left)?;
                let r = unit_value_from_value(&right)?;
                if l.dim != r.dim {
                    return Err(unit_error(UnitError::DimensionMismatch {
                        left: l.dim,
                        right: r.dim,
                    }));
                }
                let cmp = match op {
                    "<" => l.value < r.value,
                    "<=" => l.value <= r.value,
                    ">" => l.value > r.value,
                    ">=" => l.value >= r.value,
                    _ => false,
                };
                Ok(Value::Bool(cmp))
            }
            "&&" | "그리고" => {
                let l = is_truthy(&left)?;
                let r = is_truthy(&right)?;
                Ok(Value::Bool(l && r))
            }
            "||" | "또는" => {
                let l = is_truthy(&left)?;
                let r = is_truthy(&right)?;
                Ok(Value::Bool(l || r))
            }
            _ => Err(format!("지원하지 않는 연산자: {}", op).into()),
        }
    }

    pub(super) fn emit_unit_mismatch(
        &mut self,
        left: UnitDim,
        right: UnitDim,
        target: Option<String>,
        source_span: Option<SourceSpan>,
        expr: Option<ExprTrace>,
    ) {
        self.emit_arith_fault(
            ArithmeticFaultKind::DimensionMismatch { left, right },
            target,
            source_span,
            expr,
        );
    }

    pub(super) fn emit_arith_fault(
        &mut self,
        kind: ArithmeticFaultKind,
        target: Option<String>,
        source_span: Option<SourceSpan>,
        expr: Option<ExprTrace>,
    ) {
        let location = match kind {
            ArithmeticFaultKind::DimensionMismatch { .. } => "ddn:unit_mismatch",
            ArithmeticFaultKind::DivByZero => "ddn:div0",
            ArithmeticFaultKind::Overflow => "ddn:overflow",
        };
        let ctx = FaultContext {
            tick_id: self.tick_id,
            location,
            source_span,
            expr,
        };
        self.patch_ops.push(PatchOp::EmitSignal {
            signal: Signal::ArithmeticFault { ctx, kind },
            targets: target.map(|label| vec![label]).unwrap_or_default(),
        });
    }

    pub(super) fn emit_contract_violation(
        &mut self,
        kind: ddonirang_lang::ContractKind,
        mode: ddonirang_lang::ContractMode,
        condition: &Expr,
        message: String,
    ) {
        let (reason, sub_reason, contract_kind, tag) = match kind {
            ddonirang_lang::ContractKind::Pre => {
                ("CONTRACT_PRE", "PRE_VIOLATION", "pre", "contract:pre")
            }
            ddonirang_lang::ContractKind::Post => {
                ("CONTRACT_POST", "POST_VIOLATION", "post", "contract:post")
            }
        };
        let origin = self
            .current_seed_name
            .as_ref()
            .map(|name| format!("seed:{}", name))
            .unwrap_or_else(|| "#system:ddn".to_string());
        let event = DiagEvent {
            madi: self.tick_id,
            seq: 0,
            fault_id: reason.to_string(),
            rule_id: "L0-CONTRACT-01".to_string(),
            reason: reason.to_string(),
            sub_reason: Some(sub_reason.to_string()),
            mode: Some(match mode {
                ddonirang_lang::ContractMode::Alert => "알림".to_string(),
                ddonirang_lang::ContractMode::Abort => "물림".to_string(),
            }),
            contract_kind: Some(contract_kind.to_string()),
            origin: origin.clone(),
            targets: vec![origin],
            sam_hash: None,
            source_span: self.source_span_for_expr(condition),
            expr: Some(ExprTrace {
                tag: tag.to_string(),
                text: None,
            }),
            message: Some(message),
        };
        self.patch_ops.push(PatchOp::EmitSignal {
            signal: Signal::Diag { event },
            targets: Vec::new(),
        });
        if matches!(mode, ddonirang_lang::ContractMode::Abort) {
            self.aborted = true;
        }
    }

    pub(super) fn emit_factor_decomposition_deferred_diag(
        &mut self,
        value: &Value,
        source_span: Option<SourceSpan>,
    ) {
        let (deferred_value, deferred_reason, deferred_route, deferred_bits) = match value {
            Value::Pack(fields) => {
                let raw = fields.get("값").and_then(|raw| match raw {
                    Value::String(text) => Some(text.clone()),
                    _ => None,
                });
                let reason =
                    fields
                        .get(FACTOR_DECOMP_DEFERRED_REASON_KEY)
                        .and_then(|raw| match raw {
                            Value::String(text) => Some(text.clone()),
                            _ => None,
                        });
                let route = fields
                    .get(FACTOR_DECOMP_ROUTE_KEY)
                    .and_then(|raw| match raw {
                        Value::String(text) => Some(text.clone()),
                        _ => None,
                    });
                let bits = fields
                    .get(FACTOR_DECOMP_BITS_KEY)
                    .and_then(|raw| match raw {
                        Value::Fixed64(value) => Some(value.int_part()),
                        _ => None,
                    });
                (raw, reason, route, bits)
            }
            _ => (None, None, None, None),
        };
        let origin = self
            .current_seed_name
            .as_ref()
            .map(|name| format!("seed:{}", name))
            .unwrap_or_else(|| "#system:ddn".to_string());
        let message = match (deferred_value, deferred_reason.clone()) {
            (Some(raw), Some(reason)) => Some(format!("곱수 분해를 지연했습니다({reason}): {raw}")),
            (Some(raw), None) => Some(format!("곱수 분해를 지연했습니다: {raw}")),
            _ => None,
        };
        let agg_suffix = self
            .factor_route_summary_components()
            .map(|(routes, total, _, _, _)| format!(";agg_routes={routes};agg_total={total}"))
            .unwrap_or_default();
        let trace_text = match (deferred_route, deferred_bits) {
            (Some(route), Some(bits)) => Some(format!("route={route};bits={bits}{agg_suffix}")),
            (Some(route), None) => Some(format!("route={route}{agg_suffix}")),
            (None, Some(bits)) => Some(format!("bits={bits}{agg_suffix}")),
            (None, None) => None,
        };
        let event = DiagEvent {
            madi: self.tick_id,
            seq: 0,
            fault_id: NUMERIC_DIAG_REASON_FACTOR_DECOMP_DEFERRED.to_string(),
            rule_id: NUMERIC_DIAG_RULE_ID_FACTOR_DECOMP_DEFERRED.to_string(),
            reason: NUMERIC_DIAG_REASON_FACTOR_DECOMP_DEFERRED.to_string(),
            sub_reason: Some(
                deferred_reason.unwrap_or_else(|| FACTOR_DECOMP_STATUS_DEFERRED.to_string()),
            ),
            mode: Some("알림".to_string()),
            contract_kind: None,
            origin: origin.clone(),
            targets: vec![origin],
            sam_hash: None,
            source_span,
            expr: Some(ExprTrace {
                tag: NUMERIC_DIAG_TAG_FACTOR_DECOMP_DEFERRED.to_string(),
                text: trace_text,
            }),
            message,
        };
        self.patch_ops.push(PatchOp::EmitSignal {
            signal: Signal::Diag { event },
            targets: Vec::new(),
        });
    }

    pub(super) fn emit_guard_violation(&mut self, expr_id: u64) {
        self.patch_ops.push(PatchOp::GuardViolation {
            entity: EntityId(0),
            rule_id: format!("RULE_GUARD#{}", expr_id),
        });
    }

    pub(super) fn arith_trace(&self, _expr: &Expr, tag: &str) -> ExprTrace {
        ExprTrace {
            tag: tag.to_string(),
            text: None,
        }
    }

    pub(super) fn note_factor_route_from_value(&mut self, value: &Value) {
        let (route, bits) = match value {
            Value::Pack(fields) => {
                let route = fields
                    .get(FACTOR_DECOMP_ROUTE_KEY)
                    .and_then(|raw| match raw {
                        Value::String(route) => Some(route.clone()),
                        _ => None,
                    });
                let bits = fields
                    .get(FACTOR_DECOMP_BITS_KEY)
                    .and_then(|raw| match raw {
                        Value::Fixed64(value) if value.int_part() >= 0 => {
                            Some(value.int_part() as u64)
                        }
                        _ => None,
                    });
                (route, bits)
            }
            _ => (None, None),
        };
        if let Some(route) = route {
            *self.factor_route_counts.entry(route).or_insert(0) += 1;
        }
        if let Some(bits) = bits {
            self.factor_bits_total = self.factor_bits_total.saturating_add(bits as u128);
            self.factor_bits_min = Some(match self.factor_bits_min {
                Some(prev) => prev.min(bits),
                None => bits,
            });
            self.factor_bits_max = self.factor_bits_max.max(bits);
        }
    }

    pub(super) fn flush_factor_route_metrics_resource(&mut self) {
        let Some((summary, total, bit_min, bit_max, bit_sum)) =
            self.factor_route_summary_components()
        else {
            return;
        };
        let policy_summary = factor_policy_summary_text();
        let total_i64 = if total > i64::MAX as u64 {
            i64::MAX
        } else {
            total as i64
        };
        let bit_min_i64 = if bit_min > i64::MAX as u64 {
            i64::MAX
        } else {
            bit_min as i64
        };
        let bit_max_i64 = if bit_max > i64::MAX as u64 {
            i64::MAX
        } else {
            bit_max as i64
        };
        let bit_sum_i64 = if bit_sum > i64::MAX as u128 {
            i64::MAX
        } else {
            bit_sum as i64
        };
        self.resources.insert(
            NUMERIC_FACTOR_ROUTE_SUMMARY_RESOURCE_KEY.to_string(),
            Value::String(summary),
        );
        self.resources.insert(
            NUMERIC_FACTOR_ROUTE_TOTAL_RESOURCE_KEY.to_string(),
            Value::Fixed64(Fixed64::from_i64(total_i64)),
        );
        self.resources.insert(
            NUMERIC_FACTOR_BITS_MIN_RESOURCE_KEY.to_string(),
            Value::Fixed64(Fixed64::from_i64(bit_min_i64)),
        );
        self.resources.insert(
            NUMERIC_FACTOR_BITS_MAX_RESOURCE_KEY.to_string(),
            Value::Fixed64(Fixed64::from_i64(bit_max_i64)),
        );
        self.resources.insert(
            NUMERIC_FACTOR_BITS_SUM_RESOURCE_KEY.to_string(),
            Value::Fixed64(Fixed64::from_i64(bit_sum_i64)),
        );
        self.resources.insert(
            NUMERIC_FACTOR_POLICY_RESOURCE_KEY.to_string(),
            Value::String(policy_summary),
        );
    }

    pub(super) fn emit_factor_route_summary_diag(&mut self) {
        let Some((summary, total, bit_min, bit_max, bit_sum)) =
            self.factor_route_summary_components()
        else {
            return;
        };
        let policy_summary = factor_policy_summary_text();
        let origin = self
            .current_seed_name
            .as_ref()
            .map(|name| format!("seed:{}", name))
            .unwrap_or_else(|| "#system:ddn".to_string());
        let trace_text = format!(
            "routes={summary};total={total};bit_min={bit_min};bit_max={bit_max};bit_sum={bit_sum};policy={policy_summary}"
        );
        let message = format!(
            "곱수 분해경로 집계: {summary} (총 {total}, 비트 최소={bit_min}, 최대={bit_max}, 합={bit_sum}, 정책={policy_summary})"
        );
        let event = DiagEvent {
            madi: self.tick_id,
            seq: 0,
            fault_id: NUMERIC_DIAG_REASON_FACTOR_ROUTE_SUMMARY.to_string(),
            rule_id: NUMERIC_DIAG_RULE_ID_FACTOR_ROUTE_SUMMARY.to_string(),
            reason: NUMERIC_DIAG_REASON_FACTOR_ROUTE_SUMMARY.to_string(),
            sub_reason: None,
            mode: Some("알림".to_string()),
            contract_kind: None,
            origin: origin.clone(),
            targets: vec![origin],
            sam_hash: None,
            source_span: None,
            expr: Some(ExprTrace {
                tag: NUMERIC_DIAG_TAG_FACTOR_ROUTE_SUMMARY.to_string(),
                text: Some(trace_text),
            }),
            message: Some(message),
        };
        self.patch_ops.push(PatchOp::EmitSignal {
            signal: Signal::Diag { event },
            targets: Vec::new(),
        });
    }

    pub(super) fn factor_route_summary_components(&self) -> Option<(String, u64, u64, u64, u128)> {
        if self.factor_route_counts.is_empty() {
            return None;
        }
        let summary = self
            .factor_route_counts
            .iter()
            .map(|(route, count)| format!("{route}={count}"))
            .collect::<Vec<_>>()
            .join(",");
        let total = self
            .factor_route_counts
            .values()
            .fold(0u64, |acc, v| acc.saturating_add(*v));
        let bit_min = self.factor_bits_min.unwrap_or(0);
        let bit_max = self.factor_bits_max;
        let bit_sum = self.factor_bits_total;
        Some((summary, total, bit_min, bit_max, bit_sum))
    }

    pub(super) fn capture_frame_snapshot(&self, locals: &HashMap<String, Value>) -> EvalFrameSnapshot {
        self.capture_frame_snapshot_with_locals(locals, true)
    }

    pub(super) fn capture_frame_snapshot_with_locals(
        &self,
        locals: &HashMap<String, Value>,
        keep_locals: bool,
    ) -> EvalFrameSnapshot {
        EvalFrameSnapshot {
            result_emissions: self.result_emissions.clone(),
            derived_initialized:self.derived_initialized,
            subject_stage:self.subject_stage.as_ref().map(|stage|stage.borrow().clone()),
            explicit_nfc_provenance: self.explicit_nfc_provenance.clone(),
            locals: keep_locals.then(|| locals.clone()),
            resources: self.resources.clone(),
            patch_ops: self.patch_ops.clone(),
            guard_rejected: self.guard_rejected,
            const_scopes: self.const_scopes.clone(),
            binding_type_scopes: self.binding_type_scopes.clone(),
            factor_route_counts: self.factor_route_counts.clone(),
            factor_bits_total: self.factor_bits_total,
            factor_bits_min: self.factor_bits_min,
            factor_bits_max: self.factor_bits_max,
            public_observations: self.public_observations.clone(),
            display_outputs: self.display_outputs.clone(),
            root_binding_types: self.root_binding_types.clone(),
            sum_definitions: self.sum_definitions.clone(),
            sum_pattern_bindings: self.sum_pattern_bindings.clone(),
        }
    }

    pub(super) fn restore_frame_snapshot(
        &mut self,
        locals: &mut HashMap<String, Value>,
        snapshot: &EvalFrameSnapshot,
    ) {
        if let Some(saved_locals) = &snapshot.locals {
            *locals = saved_locals.clone();
        }
        self.resources = snapshot.resources.clone();
        self.result_emissions = snapshot.result_emissions.clone();
        self.derived_initialized=snapshot.derived_initialized;
        if let (Some(stage),Some(saved))=(&self.subject_stage,&snapshot.subject_stage) {*stage.borrow_mut()=saved.clone();}
        self.patch_ops = snapshot.patch_ops.clone();
        self.guard_rejected = snapshot.guard_rejected;
        self.const_scopes = snapshot.const_scopes.clone();
        self.binding_type_scopes = snapshot.binding_type_scopes.clone();
        self.factor_route_counts = snapshot.factor_route_counts.clone();
        self.factor_bits_total = snapshot.factor_bits_total;
        self.factor_bits_min = snapshot.factor_bits_min;
        self.factor_bits_max = snapshot.factor_bits_max;
        self.public_observations = snapshot.public_observations.clone();
        self.explicit_nfc_provenance = snapshot.explicit_nfc_provenance.clone();
        self.display_outputs = snapshot.display_outputs.clone();
        self.root_binding_types = snapshot.root_binding_types.clone();
        self.sum_definitions = snapshot.sum_definitions.clone();
        self.sum_pattern_bindings = snapshot.sum_pattern_bindings.clone();
    }

    pub(super) fn restore_frame_snapshot_preserving_abort_contract_diag(
        &mut self,
        locals: &mut HashMap<String, Value>,
        snapshot: &EvalFrameSnapshot,
    ) {
        let preserved = self.abort_contract_diag_ops_since(snapshot.patch_ops.len());
        self.restore_frame_snapshot(locals, snapshot);
        self.patch_ops.extend(preserved);
    }

    pub(super) fn abort_contract_diag_ops_since(&self, base_len: usize) -> Vec<PatchOp> {
        self.patch_ops
            .iter()
            .skip(base_len)
            .filter_map(|op| match op {
                PatchOp::EmitSignal {
                    signal: Signal::Diag { event },
                    targets,
                } if event.rule_id == "L0-CONTRACT-01" && event.mode.as_deref() == Some("물림") => {
                    Some(PatchOp::EmitSignal {
                        signal: Signal::Diag {
                            event: event.clone(),
                        },
                        targets: targets.clone(),
                    })
                }
                _ => None,
            })
            .collect()
    }

    pub(super) fn source_span_for_expr(&self, expr: &Expr) -> Option<SourceSpan> {
        self.source_span_from_span(&expr.span)
    }

    pub(super) fn source_span_from_span(&self, span: &ddonirang_lang::Span) -> Option<SourceSpan> {
        let origin = &self.program.program.origin;
        if origin.source.is_empty() {
            return None;
        }
        if span.start > origin.source.len() || span.end > origin.source.len() {
            return None;
        }
        let key = (span.start, span.end);
        if let Some(cached) = self.source_span_cache.borrow().get(&key) {
            return Some(cached.clone());
        }
        let (start_line, start_col) = position_to_line_col(&origin.source, span.start);
        let (end_line, end_col) = position_to_line_col(&origin.source, span.end);
        let result = SourceSpan {
            file: origin.file_path.clone(),
            start_line,
            start_col: Some(start_col),
            end_line,
            end_col: Some(end_col),
        };
        self.source_span_cache.borrow_mut().insert(key, result.clone());
        Some(result)
    }

    #[allow(dead_code)]
    pub(super) fn resource_exists(&self, name: &str) -> bool {
        if self.resources.contains_key(name) || self.defaults.contains_key(name) {
            return true;
        }
        self.committed_resource_exists(name)
    }

    pub(super) fn committed_resource_exists(&self, name: &str) -> bool {
        self.world.get_resource_value(name).is_some()
            || self.world.get_resource_fixed64(name).is_some()
            || self.world.get_resource_handle(name).is_some()
    }

    pub(super) fn get_resource(&mut self, name: &str) -> Option<Value> {
        if self.logical_input.is_some() && (name == "입력키" || name.starts_with("샘.키보드.")) {
            self.input_reads.borrow_mut().insert("logical.forbidden_physical".into());
            return None;
        }
        // 원문, 내장 `자원`, 보개 값 접근이 이 실제 읽기 지점을 공유한다.
        // 선언/문자열 검색으로 추정하지 않으므로 단락 평가로 건너뛴 식은 기록되지 않는다.
        if name == "입력키" && self.resources.contains_key(name) {
            self.input_reads.borrow_mut().insert("last_key_name".into());
        }
        if let Some(value) = self.resources.get(name) {
            return Some(value.clone());
        }
        if let Some(value) = self.world.get_resource_value(name) {
            let v = value_from_resource_value(&value);
            self.resources.insert(name.to_string(), v.clone());
            return Some(v);
        }
        if let Some(value) = self.world.get_resource_fixed64(name) {
            if let Some(spec) = parse_resource_unit_tag(name) {
                let unit_value = UnitValue::from_spec(value, spec);
                let v = Value::Unit(unit_value);
                self.resources.insert(name.to_string(), v.clone());
                return Some(v);
            }
            let v = Value::Fixed64(value);
            self.resources.insert(name.to_string(), v.clone());
            return Some(v);
        }
        if let Some(handle) = self.world.get_resource_handle(name) {
            let v = Value::ResourceHandle(handle);
            self.resources.insert(name.to_string(), v.clone());
            return Some(v);
        }
        if let Some(value) = self.defaults.get(name) {
            let v = value.clone();
            self.resources.insert(name.to_string(), v.clone());
            return Some(v);
        }
        None
    }

    pub(super) fn set_resource(&mut self, name: &str, value: Value) -> Result<(), EvalError> {
        self.set_resource_with_type(name, value, None)
    }

    pub(super) fn set_source_resource(&mut self, name: &str, value: Value, site: NodeId) -> Result<(), EvalError> {
        if value.contains_transient_subject_view(){return Err("E_VC04_READ_SNAPSHOT_ESCAPE: 임자 읽기값은 현재 실행 안에서만 사용할 수 있습니다".to_string().into());}
        let kind = self.tuple_type_plan.for_write(site, name).cloned();
        self.set_resource_with_type(name, value, kind.as_ref())
    }

    pub(super) fn set_resource_with_type(&mut self, name: &str, value: Value, kind: Option<&ddonirang_core::platform::RecordFieldType>) -> Result<(), EvalError> {
        if self.program.derived_plan.contains(name){return Err(format!("E_DERIVED_DIRECT_WRITE: {name}: 파생 대상에 직접 값을 넣을 수 없습니다").into());}
        self.set_resource_with_type_owner(name,value,kind)
    }
    pub(super) fn set_resource_with_type_owner(&mut self, name: &str, value: Value, kind: Option<&ddonirang_core::platform::RecordFieldType>) -> Result<(), EvalError> {
        if value.contains_transient_subject_view(){return Err("E_VC04_READ_SNAPSHOT_ESCAPE: 임자 읽기값은 지속 상태에 넣을 수 없습니다".to_string().into());}
        // A value may replace its own lane, never shadow another committed
        // lane or an earlier write in this same candidate patch.
        let lane=match &value {
            Value::Fixed64(_) => 0,
            Value::Unit(unit) if parse_resource_unit_tag(name).is_some() || unit.is_dimensionless() => 0,
            Value::ResourceHandle(_) => 1,
            _ => 2,
        };
        let wrong_committed=(lane!=0 && self.world.get_resource_fixed64(name).is_some())
            || (lane!=1 && self.world.get_resource_handle(name).is_some())
            || (lane!=2 && self.world.get_resource_value(name).is_some());
        let wrong_staged=self.patch_ops.iter().any(|op|match op {
            PatchOp::SetResourceFixed64 {tag,..} => tag==name && lane!=0,
            PatchOp::SetResourceHandle {tag,..} => tag==name && lane!=1,
            PatchOp::SetResourceValue {tag,..} => tag==name && lane!=2,
            PatchOp::SetResourceJson {tag,..} => tag==name,
            _=>false,
        });
        if wrong_committed || wrong_staged {
            return Err("E_RESOURCE_SNAPSHOT_INVALID: 같은 자원 이름이 여러 저장 통로에 있습니다".to_string().into());
        }
        if let Some(Value::Tensor(old)) = self.resources.get(name) {
            let Value::Tensor(next) = &value else {
                return Err("Tensor 교체는 같은 원소씨와 모양이어야 합니다".to_string().into());
            };
            if old.element_identity() != next.element_identity() || old.shape() != next.shape() {
                return Err("Tensor 교체 모양 또는 원소씨 불일치".to_string().into());
            }
        }
        match &value {
            Value::SumBranch {..} => return Err("고름씨 갈래값은 자원에 대입할 수 없습니다".to_string().into()),
            Value::FrozenSubjectViewV1(_) => return Err("E_VC04_READ_SNAPSHOT_ESCAPE".to_string().into()),
            Value::Tuple(_) | Value::Rational(_) => {
                let kind = kind.ok_or_else(|| EvalError::from(format!("E_TUPLE_PERSISTENCE_UNBOUND: 원문에 결속된 위치별 씨를 확인할 수 없습니다: {name}")))?;
                let resource = resource_value_from_typed_value(&value, kind)?;
                self.patch_ops.push(PatchOp::SetResourceValue { tag: name.to_owned(), value: resource });
                self.resources.insert(name.to_owned(), value);
            }
            Value::Tensor(tensor) => {
                let saved = crate::unit_wire::tensor_to_resource(&tensor).map_err(EvalError::from)?;
                self.patch_ops.push(PatchOp::SetResourceValue { tag: name.to_string(), value: saved });
                self.resources.insert(name.to_string(), value);
            }
            Value::IdentityReference(reference) => {
                self.patch_ops.push(PatchOp::SetResourceValue { tag:name.to_string(),value:ResourceValue::IdentityReference(reference.clone()) });
                self.resources.insert(name.to_string(),value);
            },
            Value::NominalValueV1(nominal) => {
                self.patch_ops.push(PatchOp::SetResourceValue {tag:name.to_owned(),value:ResourceValue::NominalValueV1(nominal.clone())});
                self.resources.insert(name.to_owned(),value);
            }
            Value::SubjectHandleV1(handle) => {
                self.patch_ops.push(PatchOp::SetResourceValue {tag:name.to_owned(),value:ResourceValue::SubjectHandleV1(handle.clone())});
                self.resources.insert(name.to_owned(),value);
            }
            Value::RegisteredAtom(atom) => {
                self.patch_ops.push(PatchOp::SetResourceValue { tag: name.to_string(), value: ResourceValue::RegisteredAtom(atom.clone()) });
                self.resources.insert(name.to_string(), value);
            }
            Value::Bytes(bytes) => {
                self.patch_ops.push(PatchOp::SetResourceValue {
                    tag: name.to_string(), value: ResourceValue::Bytes(bytes.clone()),
                });
                self.resources.insert(name.to_string(), value);
            }
            Value::Fixed64(v) => {
                let v = *v;
                if let Some(spec) = parse_resource_unit_tag(name) {
                    let unit_value = UnitValue::try_from_spec(v, spec).map_err(unit_error)?;
                    self.patch_ops.push(PatchOp::SetResourceFixed64 {
                        tag: name.to_string(),
                        value: v,
                    });
                    self.resources
                        .insert(name.to_string(), Value::Unit(unit_value));
                } else {
                    self.patch_ops.push(PatchOp::SetResourceFixed64 {
                        tag: name.to_string(),
                        value: v,
                    });
                    self.resources.insert(name.to_string(), Value::Fixed64(v));
                }
            }
            Value::Unit(unit) => {
                let unit = *unit;
                if let Some(spec) = parse_resource_unit_tag(name) {
                    if unit.dim != spec.dim {
                        return Err(unit_error(UnitError::DimensionMismatch {
                            left: unit.dim,
                            right: spec.dim,
                        }));
                    }
                    let stored = unit.to_unit(spec).map_err(unit_error)?;
                    self.patch_ops.push(PatchOp::SetResourceFixed64 {
                        tag: name.to_string(),
                        value: stored,
                    });
                    self.resources.insert(name.to_string(), Value::Unit(unit));
                } else if unit.is_dimensionless() {
                    self.patch_ops.push(PatchOp::SetResourceFixed64 {
                        tag: name.to_string(),
                        value: unit.value,
                    });
                    self.resources
                        .insert(name.to_string(), Value::Fixed64(unit.value));
                } else {
                    // A3-P2: a normal binding keeps its typed UnitValue. The
                    // existing ResourceValue wire already owns its dimension;
                    // a unit suffix in the variable spelling is not required.
                    self.patch_ops.push(PatchOp::SetResourceValue {
                        tag: name.to_string(),
                        value: ResourceValue::Unit(unit),
                    });
                    self.resources.insert(name.to_string(), Value::Unit(unit));
                }
            }
            Value::String(s) => {
                self.patch_ops.push(PatchOp::SetResourceValue {
                    tag: name.to_string(),
                    value: ResourceValue::String(s.clone()),
                });
                self.resources.insert(name.to_string(), value);
            }
            Value::Bool(b) => {
                self.patch_ops.push(PatchOp::SetResourceValue {
                    tag: name.to_string(),
                    value: ResourceValue::Bool(*b),
                });
                self.resources.insert(name.to_string(), value);
            }
            Value::ResourceHandle(handle) => {
                self.patch_ops.push(PatchOp::SetResourceHandle {
                    tag: name.to_string(),
                    handle: *handle,
                });
                self.resources
                    .insert(name.to_string(), value);
            }
            Value::None => return Err("없음은 자원에 대입할 수 없습니다".to_string().into()),
            Value::List(_)
            | Value::Set(_)
            | Value::Map(_)
            | Value::Pack(_)
            | Value::Record { .. }
            | Value::Assertion(_)
            | Value::StateMachine(_)
            | Value::Formula(_) => {
                if parse_resource_unit_tag(name).is_some() {
                    return Err(
                        "단위 태그 자원에는 차림/모음/짝맞춤/묶음/세움값/상태머신값/수식값을 저장할 수 없습니다"
                            .to_string()
                            .into(),
                    );
                }
                let resource_value = if runtime_value_contains_tuple(&value) {
                    let kind = kind.ok_or_else(|| EvalError::from("E_TUPLE_PERSISTENCE_UNBOUND: 중첩 위치형의 원문 씨가 필요합니다".to_owned()))?;
                    resource_value_from_typed_value(&value, kind)?
                } else { resource_value_from_value(&value)? };
                self.patch_ops.push(PatchOp::SetResourceValue {
                    tag: name.to_string(),
                    value: resource_value,
                });
                self.resources.insert(name.to_string(), value);
            }
            Value::Template(_) => {
                return Err("글무늬는 자원에 대입할 수 없습니다".to_string().into())
            }
            Value::Regex(_) => return Err("정규식은 자원에 대입할 수 없습니다".to_string().into()),
            Value::Lambda(_) => {
                self.resources
                    .insert(name.to_string(), value);
            }
            Value::TimePolicy(_) => { self.resources.insert(name.to_string(),value); }
            Value::View(_) => {
                // Authored 보기값은 실행기 결속만 유지한다. world Patch/hash에 넣지 않는다.
                self.resources.insert(name.to_string(), value);
            }
            Value::SceneElement(_) => { self.resources.insert(name.to_string(), value); }
            Value::Scene(_) => { self.resources.insert(name.to_string(), value); }
            Value::BogaeRelation(_) => {
                self.resources.insert(name.to_string(), value);
            }
            Value::BogaeComposition(_) => {
                self.resources.insert(name.to_string(), value);
            }
        }
        Ok(())
    }

    pub(super) fn append_output_log(&mut self, value: Value) -> Result<(), EvalError> {
        let mut entries = self.take_list_resource("output_log");
        let line_no = entries.len().saturating_add(1).min(i64::MAX as usize) as i64;
        let mut row = BTreeMap::new();
        let insert = |row: &mut BTreeMap<String, MapEntry>, key: &str, value: Value| {
            let key_value = Value::String(key.to_string());
            row.insert(
                map_key_canon(&key_value),
                MapEntry {
                    key: key_value,
                    value,
                },
            );
        };
        insert(
            &mut row,
            "tick",
            Value::Fixed64(Fixed64::from_i64(self.tick_id.min(i64::MAX as u64) as i64)),
        );
        insert(
            &mut row,
            "line_no",
            Value::Fixed64(Fixed64::from_i64(line_no)),
        );
        insert(&mut row, "kind", Value::String("output".to_string()));
        insert(&mut row, "text", Value::String(value_to_string(&value)));
        self.display_outputs.push(crate::output_display::OutputDisplay {
            madi: self.tick_id, line_no, text: crate::output_display::text(&value),
        });
        entries.push(Value::Map(row));
        self.set_resource("output_log", Value::List(entries))
    }

    pub(super) fn take_list_resource(&mut self, name: &str) -> Vec<Value> {
        if let Some(mut value) = self.resources.remove(name) {
            if let Value::List(items) = &mut value { return std::mem::take(items); }
        }
        match &mut self.get_resource(name) {
            Some(Value::List(items)) => {
                self.resources.remove(name);
                std::mem::take(items)
            }
            _ => {
                self.resources.remove(name);
                Vec::new()
            }
        }
    }
}

#[cfg(test)]
mod ordinary_remainder_panic_regression {
    use super::*;

    fn evaluate(source: &str) -> Result<DdnRunOutput, EvalError> {
        let program = DdnProgram::from_v25_source(source, "remainder-probe.ddn").unwrap();
        let input = InputSnapshot {
            tick_id: 0, dt: Fixed64::from_i64(1), keys_pressed: 0,
            last_key_name: String::new(), pointer_x_i32: 0, pointer_y_i32: 0,
            ai_injections: Vec::new(), net_events: Vec::new(),
            frame_source: ddonirang_core::InputSource::Person, rng_seed: 0,
        };
        DdnRunner::new(program, crate::runtime_surface::DEFAULT_UPDATE_NAME)
            .run_named_seed(&NuriWorld::new(), &input, &HashMap::new(), None,
                crate::runtime_surface::DEFAULT_UPDATE_NAME, false)
    }

    #[test]
    fn minimum_raw_remainder_raw_source_returns_existing_overflow() {
        let error = evaluate("(-2147483648.0 % -0.0000000002) 보여주기.\n")
            .err().expect("overflow must reject the update");
        assert!(matches!(error, EvalError::ArithmeticFault(
            ddonirang_core::ArithmeticFaultKind::Overflow)));
    }

    #[test]
    fn ordinary_raw_source_remainders_and_zero_error_are_preserved() {
        for (source, expected) in [
            ("(5.0 % 2.0) 보여주기.\n", "1"),
            ("(-5.0 % 2.0) 보여주기.\n", "-1"),
            ("(5.0 % -2.0) 보여주기.\n", "1"),
        ] {
            let output = evaluate(source).unwrap();
            assert_eq!(output.display_outputs.iter().map(|row| row.text.as_str())
                .collect::<Vec<_>>(), vec![expected]);
        }
        let error = evaluate("(5.0 % 0.0) 보여주기.\n")
            .err().expect("zero divisor must reject the update");
        assert!(error.message().contains("0으로 나눌 수 없습니다"));
    }
}

