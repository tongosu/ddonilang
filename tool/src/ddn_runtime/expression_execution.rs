//! Existing evaluator methods moved without changing their bodies.
// F5-GA[선택·인자]: 범위는 checked_selection_range가 주소 변환과 시작≤끝≤길이를 확인한다.
// 같은 immutable items에서 clone까지 길이가 바뀌지 않으며 범위 교체는 같은 길이를 요구한다.
// 단일 Tuple 위치는 items.get 성공 뒤 복제하고, Map은 같은 key의 존재 확인 뒤 복제한다.
// 조작단계·가리키기·논리 이항 호출은 인자 개수 검사 실패 시 즉시 반환한 뒤에만 인덱싱한다.
// 돋의 기본 인자 0·1은 길이2~3 검사 뒤, 시간2는 길이3 분기 안에서만 읽는다.
// 보기법 params[0]은 동일 seed에 대한 bogae_recipe::seal의 단일 필수 매개변수 검증 뒤 읽는다.
// F5-GA[역할재배열]: resolve_source_builtin_roles는 enumerate 순번을 빈 slot에 한 번씩만 넣는다.
// 중복·누락을 거부한 canonical_order는 values 길이 안의 서로 다른 순번이며 각 take는 한 번뿐이다.
// F5-GA[보기슬롯]: 이 분기의 다섯 이름은 고정 view_function_sigs에 모두 있고 params는 비어 있지 않다.
// slot과 params는 같은 길이이며 position으로 얻은 index만 쓴다. required는 그 길이에서1을 뺀 값이다.
// 위 경계와 아래 literal 공급 증명은 해당 expect·인덱스에만 적용한다. 메모리·산술 상한은 별도 감사한다.
use unicode_segmentation::UnicodeSegmentation;
use super::{
    Arc,
    Assertion,
    AtSuffix,
    BTreeMap,
    BigInt,
    BigRational,
    EvalContext,
    EvalError,
    EvaluatedIndexedSelection,
    Expr,
    ExprKind,
    Fixed64,
    Formula,
    HashMap,
    IndexedSelectionExpr,
    LambdaValue,
    Literal,
    NUMERIC_KIND_RATIONAL,
    Rc,
    ResourceHandle,
    StateMachine,
    TypeRef,
    Value,
    WorkCostMeter,
    apply_suffix_value,
    builtin_work_cost_rule,
    check_value_type,
    checked_selection_range,
    exact_numeric_from_value,
    expr_registry_key,
    func,
    inferred_binding_type,
    is_truthy,
    literal_registry_key,
    literal_to_value,
    make_rational_pack_from_big_rational,
    manipulation_phase_text,
    map_get,
    map_key_canon,
    next_lambda_id,
    numeric_pack_kind,
    resource_value_from_typed_value,
    root_binding_type_key,
    tuple_position_type_equal,
    type_mismatch_error,
    value_from_resource_value,
    value_to_i64,
    value_to_tensor_index,
};

// F4: only the selected expression arm owns its scratch frame.
// The authored match is a tail result: early return and ? still return the same result.
#[inline(never)]
fn in_expression_frame<F>(arm: F) -> Result<Value, EvalError>
where
    F: FnOnce() -> Result<Value, EvalError>,
{
    arm()
}

impl<'a> EvalContext<'a> {
    pub(super) fn eval_expr_expected(
        &mut self, locals: &mut HashMap<String, Value>, expr: &Expr, expected: &TypeRef,
    ) -> Result<Value, EvalError> {
        if self.program.numeric_constraint_plan.is_some() {
            // The admitted compilation already issued literal goals. A
            // consumer must not build a competing plan from runtime values.
            return self.eval_expr(locals, expr);
        }
        let rational = matches!(expected, TypeRef::Named(name)
            if ddonirang_lang::stdlib::canonicalize_type_alias(name) == "나눔수");
        if !rational || !Self::source_numeric_literal_tree(expr) { return self.eval_expr(locals, expr); }
        // F5-GA[LITERAL-JOBS]: 닫힌 literal 트리만 들어오며, 자식 완료 뒤 부모 Job가 실행된다.
        // F5-GA[LITERAL-JOBS]: leaf 1값, 이항 2값→1값, 음수 1값→1값이므로 pop은 충분하다.
        // F5-GA[LITERAL-KIND]: leaf와 음수의 성공 결과는 make_rational_pack_from_big_rational의 Rational이다.
        // F5-GA[LITERAL-KIND]: 허용 이항 +,-,*,/는 Rational을 exact 경로에서 처리하고 Rational로 반환한다.
        // F5-GA[LITERAL-KIND]: 이 성공 경로의 exact_numeric_from_value는 항상 Some이다.
        // F5-GA[LITERAL-JOBS]: ? 오류는 즉시 반환한다. 내부 생성자·할당·산술 실패까지 안전하다는 뜻은 아니다.
        enum Job<'a> { Expr(&'a Expr), Op(&'a str), Negate }
        let mut jobs = vec![Job::Expr(expr)];
        let mut values = Vec::new();
        while let Some(job) = jobs.pop() {
            match job {
                Job::Expr(node) => {
                    self.work_cost.borrow_mut().charge_dispatch()?;
                    if let Some(operand) = node.source_negated_operand() {
                        jobs.push(Job::Negate); jobs.push(Job::Expr(operand));
                    } else if let ExprKind::Infix { left, op, right } = &node.kind {
                        jobs.push(Job::Op(op)); jobs.push(Job::Expr(right)); jobs.push(Job::Expr(left));
                    } else {
                        values.push(make_rational_pack_from_big_rational(&self.source_literal_exact(node)?));
                    }
                }
                Job::Op(op) => {
                    let right = values.pop().expect("right operand");
                    let left = values.pop().expect("left operand");
                    values.push(self.eval_infix(op, left, right)?);
                }
                Job::Negate => {
                    let operand = values.pop().expect("negative operand");
                    let exact = exact_numeric_from_value(&operand)?.expect("exact negative operand");
                    values.push(make_rational_pack_from_big_rational(&-exact.value));
                }
            }
        }
        let value = values.pop().expect("exact expression root");
        let exact = exact_numeric_from_value(&value)?.expect("exact expression result");
        // Denominator one still has the expected rational owner (RD-076).
        Ok(make_rational_pack_from_big_rational(&exact.value))
    }

    // Resolve authored type owners, never guess a numeric kind from an
    // integral Fixed64 payload. Literal-only division remains undecided.
    pub(super) fn comparison_operand_type(&self, locals: &HashMap<String, Value>, expr: &Expr) -> Option<ddonirang_core::platform::RecordFieldType> {
        if let Some(plan) = &self.program.numeric_constraint_plan {
            return plan.position_types.get(&expr.id).cloned();
        }
        use ddonirang_lang::runtime::record_type_identity;
        let mut bare = expr;
        while let Some(operand) = bare.source_negated_operand() { bare = operand; }
        if matches!(bare.kind, ExprKind::Literal(_)) { return None; }
        let mut bindings = std::collections::BTreeMap::new();
        let mut calls = std::collections::BTreeMap::new();
        let mut pending = vec![expr];
        while let Some(node) = pending.pop() {
            match &node.kind {
                ExprKind::Var(name) => {
                    let local = self.binding_type_scopes.last().and_then(|scope| scope.get(name));
                    let owner = local.or_else(|| (!locals.contains_key(name)).then(||
                        self.binding_type_scopes.first().and_then(|scope| scope.get(name))).flatten());
                    let kind = owner.and_then(|(kind, _)| record_type_identity(kind, &self.program.record_schemas).ok())
                        .or_else(|| (self.binding_type_scopes.len() == 1 || !locals.contains_key(name))
                            .then(|| self.tuple_type_plan.expected(name).cloned()).flatten());
                    if let Some(kind) = kind { bindings.insert(name.clone(), kind); }
                }
                ExprKind::Call { func, .. } if !locals.contains_key(func) && !self.resources.contains_key(func) => {
                    let canonical = ddonirang_lang::stdlib::canonicalize_stdlib_alias(func);
                    if crate::work_cost::builtin_work_cost_rule(canonical).is_ok_and(|rule| rule.is_none()) {
                        let kind = self.program.functions.get(func).and_then(|seed| seed.result_type.as_ref())
                            .and_then(|kind| record_type_identity(kind, &self.program.record_schemas).ok());
                        calls.insert(func.clone(), kind);
                    }
                }
                ExprKind::Infix { left, right, .. } => { pending.push(left); pending.push(right); }
                ExprKind::SelectionPlace { target, .. } => pending.push(target),
                ExprKind::MultiAxisSelection(selection) => pending.push(&selection.target),
                ExprKind::TupleConstructor { elements } => pending.extend(elements),
                _ => {}
            }
        }
        crate::tuple_saved_type_plan::expression_type_from_owners(expr, bindings, &calls)
    }

    pub(super) fn inferred_expression_binding_type(&self, locals: &HashMap<String, Value>, expr: &Expr, value: &Value) -> TypeRef {
        // Absence initializes no inferred payload type. Keep the existing
        // inferred_binding_type rule before considering a source projection.
        if matches!(value,Value::None) {return TypeRef::Infer;}
        // Inferred 차림 keeps its existing coarse runtime category. Private
        // homogeneous source evidence authorizes numeric consumers, not a
        // permanent List<T> declaration that bans ordinary heterogeneous use.
        // Explicitly declared List<T> still uses its separate declaration gate.
        if matches!(value,Value::List(_)) {return inferred_binding_type(value);}
        if let Some(kind) = self.program.numeric_constraint_plan.as_ref().and_then(|plan| plan.expressions.get(&expr.id)) {
            return kind.clone();
        }
        // Copy an already-issued owner through a binding, not a default for a
        // literal nor a guess from Fixed64 bits. R2 monomorphic inference is not
        // implemented here. A caller's same-spelled local is never consulted.
        if let Some(ddonirang_core::platform::RecordFieldType::Named { ref name }) = self.comparison_operand_type(locals, expr) {
            let canonical = ddonirang_lang::stdlib::canonicalize_type_alias(name);
            if matches!(canonical, "바른수" | "셈수" | "나눔수" | "큰바른수" | "곱수") {
                let kind = TypeRef::Named(canonical.into());
                if check_value_type(value, &kind).is_ok() { return kind; }
            }
        }
        inferred_binding_type(value)
    }

    pub(super) fn check_numeric_comparison_owners(&self, locals: &HashMap<String, Value>, left: &Expr, right: &Expr, op: &str, values: Option<(&Value, &Value)>) -> Result<(), EvalError> {
        use ddonirang_core::platform::RecordFieldType as R;
        let known_value = |value: Option<&Value>| value.and_then(numeric_pack_kind)
            .map(|name| R::Named { name: name.into() });
        let mut left_kind = self.comparison_operand_type(locals, left).or_else(|| known_value(values.map(|v| v.0)));
        let mut right_kind = self.comparison_operand_type(locals, right).or_else(|| known_value(values.map(|v| v.1)));
        let rational = |kind: &Option<R>| matches!(kind,Some(R::Named {name})
            if ddonirang_lang::stdlib::canonicalize_type_alias(name)=="나눔수");
        // Only the authored, statically issued owner supplies expectation.
        // Runtime pack discovery alone does not authorize literal reissuance.
        if rational(&self.comparison_operand_type(locals,left)) && Self::source_numeric_literal_tree(right) {
            right_kind=Some(R::Named {name:"나눔수".into()});
        }
        if rational(&self.comparison_operand_type(locals,right)) && Self::source_numeric_literal_tree(left) {
            left_kind=Some(R::Named {name:"나눔수".into()});
        }
        let mut pending = match (&left_kind, &right_kind) {
            (Some(left), Some(right)) => vec![(left, right)],
            _ => Vec::new(),
        };
        while let Some((left, right)) = pending.pop() {
            match (left, right) {
                (R::Named { name: left }, R::Named { name: right }) => {
                    let left = ddonirang_lang::stdlib::canonicalize_type_alias(&left);
                    let right = ddonirang_lang::stdlib::canonicalize_type_alias(&right);
                    let numeric = |name: &str| matches!(name, "바른수" | "셈수" | "나눔수" | "큰바른수" | "곱수");
                    if numeric(left) && numeric(right) && left != right {
                        return Err(format!("E_NUMERIC_COMPARISON_TYPE_MISMATCH: {op} 비교의 왼쪽은 {left}, 오른쪽은 {right}입니다. 형식이 정해진 서로 다른 수 종류는 자동 변환하여 비교하지 않습니다. 같은 수 종류로 명시적으로 맞춰 주세요").into());
                    }
                }
                (R::Tuple { items: left }, R::Tuple { items: right }) => pending.extend(left.iter().zip(right)),
                _ => {}
            }
        }
        Ok(())
    }

    pub(super) fn eval_expr(
        &mut self,
        locals: &mut HashMap<String, Value>,
        expr: &Expr,
    ) -> Result<Value, EvalError> {
        let result = self.eval_expr_inner(locals, expr);
        // These typed constructors validate at runtime. Preserve the error's
        // existing code and the innermost failing call, but do not lose its
        // authored source location at the shared CLI/WASM boundary.
        if let Err(EvalError::Message(message)) = &result {
            if matches!(&expr.kind, ExprKind::Tensor(_) | ExprKind::MultiAxisSelection(_))
                || message.starts_with("E_NUMERIC_COMPARISON_TYPE_MISMATCH:")
                || message.starts_with("E_NUMERIC_COMPARISON_TYPE_UNRESOLVED:") {
                if let Some(location) = self.source_span_for_expr(expr) {
                    return result.map_err(|error| EvalError::SourceLocated {
                        error: Box::new(error), span: (expr.span.start,expr.span.end),
                        line: location.start_line, column: location.start_col.unwrap_or(1),
                    });
                }
            }
            if matches!(&expr.kind, ExprKind::Call { func, .. }
                if func.starts_with("표준보개.") || func.starts_with("표준보기.")
                    || func.starts_with("표준장면.") || func.starts_with("표준시간."))
                && (message.starts_with("E_BOGAE_") || message.starts_with("E_SCENE_"))
            {
                if let Some(location) = self.source_span_for_expr(expr) {
                    return result.map_err(|error| EvalError::SourceLocated {
                        error: Box::new(error),
                        span: (expr.span.start, expr.span.end),
                        line: location.start_line,
                        column: location.start_col.unwrap_or(1),
                    });
                }
            }
        }
        result
    }

    pub(super) fn eval_indexed_selection(&mut self, locals: &mut HashMap<String, Value>, selection: &IndexedSelectionExpr<'_>) -> Result<EvaluatedIndexedSelection, EvalError> {
        let single = match selection {
            IndexedSelectionExpr::Single(expr) => Some(*expr),
            IndexedSelectionExpr::Axes(selectors) => match *selectors {
                [ddonirang_lang::ast::TensorAxisSelector::Index(expr)] => Some(expr.as_ref()),
                [ddonirang_lang::ast::TensorAxisSelector::Range {start,end}] =>
                    return Ok(EvaluatedIndexedSelection::Range {start:*start,end:*end,dynamic:false}),
                _ => None,
            },
        };
        if let Some(Expr {kind:ExprKind::Call {func,args},..}) = single {
            if func == "표준.범위" && args.len() == 3 {
                let start_value=self.eval_expr(locals,&args[0].expr)?;
                let end_value=self.eval_expr(locals,&args[1].expr)?;
                let start=value_to_tensor_index(&start_value)? as u64;
                let end=value_to_tensor_index(&end_value)? as u64;
                return Ok(EvaluatedIndexedSelection::Range {start,end,dynamic:true});
            }
        }
        match selection {
            IndexedSelectionExpr::Single(expr) => self.eval_expr(locals, expr).map(|value| EvaluatedIndexedSelection::Single{value,static_index:crate::source_analysis::static_tuple_index(expr)}),
            IndexedSelectionExpr::Axes(selectors) => {
                if let [ddonirang_lang::ast::TensorAxisSelector::Index(expr)] = *selectors {
                    self.eval_expr(locals, expr).map(|value| EvaluatedIndexedSelection::Single{value,static_index:crate::source_analysis::static_tuple_index(expr)})
                } else {
                    self.eval_tensor_selectors(locals, selectors).map(EvaluatedIndexedSelection::Axes)
                }
            }
        }
    }

    pub(super) fn read_indexed_selection(&mut self, locals: &mut HashMap<String, Value>, collection: Value, selection: &EvaluatedIndexedSelection, location: &Expr) -> Result<Value, EvalError> {
        match selection {
            EvaluatedIndexedSelection::Range {start,end,dynamic} => {
                if let Value::Tuple(items)=&collection {
                    if *dynamic {return Err("E_TUPLE_STATIC_SELECTION: 위치형 범위는 정적이어야 합니다".to_string().into());}
                    let (start,end)=checked_selection_range(*start,*end,items.len())?;
                    self.work_cost.borrow_mut().charge_element_visits(end-start)?;
                    return Ok(Value::Tuple(items[start..end].to_vec()));
                }
                if let Value::List(items)=&collection {
                    let (start,end)=checked_selection_range(*start,*end,items.len())?;
                    self.work_cost.borrow_mut().charge_element_visits(end-start)?;
                    return Ok(Value::List(items[start..end].to_vec()));
                }
                if let Value::Tensor(tensor)=&collection {
                    if *dynamic {return Err("E_TENSOR_STATIC_RANGE: 텐서 축 범위는 정적이어야 합니다".to_string().into());}
                    self.work_cost.borrow_mut().charge_element_visits(tensor.elements().len())?;
                    return crate::unit_wire::tensor_select(tensor,&[crate::unit_wire::TensorSelector::Range {start:*start,end:*end}]).map_err(Into::into);
                }
                Err("E_SELECTION_RANGE_TARGET: 범위 선택 대상은 차림 또는 텐서여야 합니다".to_string().into())
            }
            EvaluatedIndexedSelection::Single{value:index,static_index} => {
                if let Value::Tuple(items)=&collection {
                    let index=static_index.ok_or_else(||"E_TUPLE_STATIC_SELECTION: 위치형 위치는 정적이어야 합니다".to_string())?;
                    self.work_cost.borrow_mut().charge_element_visits(1)?;
                    return items.get(index).cloned().ok_or_else(||"E_SELECTION_BOUNDS: 위치형 위치가 범위를 벗어났습니다".to_string().into());
                }
                if let Value::Map(entries) = &collection {
                    self.work_cost.borrow_mut().charge_element_visits(1)?;
                    return entries.get(&map_key_canon(index)).map(|entry| entry.value.clone())
                        .ok_or_else(|| "E_SELECTION_KEY_MISSING: 짝맞춤에 선택한 열쇠가 없습니다".to_string().into());
                }
                if let Value::String(text) = &collection {
                    let position = value_to_tensor_index(index).map_err(|_| EvalError::from(
                        "E_SELECTION_TEXT_INDEX: 글 위치는 0 이상의 정수여야 합니다".to_string()))?;
                    // RD354: 기존 글 bytes를 보존하고 고정 Unicode 확장 글자 경계만 읽는다.
                    // 선택 때문에 NFC를 다시 적용하거나 code point로 대체하지 않는다.
                    for (ordinal, grapheme) in text.graphemes(true).enumerate() {
                        self.work_cost.borrow_mut().charge_element_visits(1)?;
                        if ordinal == position { return Ok(Value::String(grapheme.to_string())); }
                    }
                    return Err("E_SELECTION_TEXT_BOUNDS: 글 위치가 글자 수 범위를 벗어났습니다".to_string().into());
                }
                self.eval_call(locals,"차림.값",vec![collection,index.clone()],self.source_span_for_expr(location))
            }
            EvaluatedIndexedSelection::Axes(selectors) => {
                let Value::Tensor(tensor) = &collection else { return Err("E_TENSOR_SELECTION_TARGET: 다축 선택 대상은 텐서여야 합니다".to_string().into()); };
                self.work_cost.borrow_mut().charge_element_visits(tensor.elements().len())?;
                crate::unit_wire::tensor_select(&tensor,selectors).map_err(Into::into)
            }
        }
    }

    pub(super) fn replace_indexed_selection(&mut self, locals: &mut HashMap<String, Value>, collection: Value, selection: &EvaluatedIndexedSelection, replacement: Value, location: &Expr) -> Result<Value, EvalError> {
        match selection {
            EvaluatedIndexedSelection::Range {start,end,dynamic} => {
                if let Value::Tuple(items)=&collection {
                    if *dynamic {return Err("E_TUPLE_STATIC_SELECTION: 위치형 범위는 정적이어야 합니다".to_string().into());}
                    let (start,end)=checked_selection_range(*start,*end,items.len())?;
                    let Value::Tuple(values)=&replacement else {return Err("E_TUPLE_REPLACEMENT: 위치형 범위는 같은 위치씨의 위치형 값으로 교체해야 합니다".to_string().into());};
                    if values.len()!=end-start || items[start..end].iter().zip(values).any(|(a,b)|!tuple_position_type_equal(a,b)) {
                        return Err("E_TUPLE_REPLACEMENT: 위치형 범위의 길이 또는 위치씨가 다릅니다".to_string().into());
                    }
                    self.work_cost.borrow_mut().charge_element_visits(items.len()+values.len())?;
                    let mut updated=items.clone();updated[start..end].clone_from_slice(values);
                    return Ok(Value::Tuple(updated));
                }
                if let Value::List(items)=&collection {
                    let (start,end)=checked_selection_range(*start,*end,items.len())?;
                    let Value::List(values)=&replacement else {return Err("E_SELECTION_RANGE_REPLACEMENT: 차림 범위는 같은 길이의 차림으로 교체해야 합니다".to_string().into());};
                    if values.len()!=end-start {return Err("E_SELECTION_RANGE_REPLACEMENT: 범위 교체로 차림 길이를 바꿀 수 없습니다".to_string().into());}
                    self.work_cost.borrow_mut().charge_element_visits(items.len()+values.len())?;
                    let mut updated=items.clone();
                    updated[start..end].clone_from_slice(values);
                    return Ok(Value::List(updated));
                }
                if let Value::Tensor(tensor)=&collection {
                    if *dynamic {return Err("E_TENSOR_STATIC_RANGE: 텐서 축 범위는 정적이어야 합니다".to_string().into());}
                    self.work_cost.borrow_mut().charge_element_visits(tensor.elements().len())?;
                    return crate::unit_wire::tensor_replace(tensor,&[crate::unit_wire::TensorSelector::Range {start:*start,end:*end}],&replacement).map(Value::Tensor).map_err(Into::into);
                }
                Err("E_SELECTION_RANGE_TARGET: 범위 교체 대상은 차림 또는 텐서여야 합니다".to_string().into())
            }
            EvaluatedIndexedSelection::Single{value:index,static_index} => {
                if let Value::Tuple(items)=&collection {
                    let index=static_index.ok_or_else(||"E_TUPLE_STATIC_SELECTION: 위치형 위치는 정적이어야 합니다".to_string())?;
                    let old=items.get(index).ok_or_else(||EvalError::from("E_SELECTION_BOUNDS: 위치형 위치가 범위를 벗어났습니다".to_string()))?;
                    if !tuple_position_type_equal(old,&replacement){return Err("E_TUPLE_REPLACEMENT: 위치씨가 다릅니다".to_string().into());}
                    self.work_cost.borrow_mut().charge_element_visits(items.len())?;
                    let mut updated=items.clone();updated[index]=replacement;
                    return Ok(Value::Tuple(updated));
                }
                if let Value::Map(entries) = &collection {
                    let key = map_key_canon(index);
                    if !entries.contains_key(&key) {
                        return Err("E_SELECTION_KEY_MISSING: 짝맞춤에 선택한 열쇠가 없습니다".to_string().into());
                    }
                    self.work_cost.borrow_mut().charge_element_visits(entries.len())?;
                    let mut updated = entries.clone();
                    updated.get_mut(&key).expect("existing map entry").value = replacement;
                    return Ok(Value::Map(updated));
                }
                if let Value::String(text)=&collection {
                    let position=value_to_tensor_index(index).map_err(|_| EvalError::from(
                        "E_SELECTION_TEXT_INDEX: 글 위치는 0 이상의 정수여야 합니다".to_string()))?;
                    let Value::String(value)=&replacement else {
                        return Err("E_SELECTION_TEXT_REPLACEMENT: 글 위치는 글자 하나를 담은 글로 교체해야 합니다".to_string().into());
                    };
                    let mut replacement_letters=value.graphemes(true);
                    self.work_cost.borrow_mut().charge_element_visits(2)?;
                    if replacement_letters.next().is_none() || replacement_letters.next().is_some() {
                        return Err("E_SELECTION_TEXT_REPLACEMENT: 글 위치 교체로 글자 수를 바꿀 수 없습니다".to_string().into());
                    }
                    let mut selected=None;
                    for (ordinal,(start,letter)) in text.grapheme_indices(true).enumerate() {
                        self.work_cost.borrow_mut().charge_element_visits(1)?;
                        if ordinal==position { selected=Some((start,start+letter.len()));break; }
                    }
                    let Some((start,end))=selected else {
                        return Err("E_SELECTION_TEXT_BOUNDS: 글 위치가 글자 수 범위를 벗어났습니다".to_string().into());
                    };
                    let length=text.len().checked_sub(end-start).and_then(|n|n.checked_add(value.len()))
                        .ok_or_else(|| EvalError::from("E_SELECTION_TEXT_REPLACEMENT: 글 크기 범위를 벗어났습니다".to_string()))?;
                    let mut updated=String::new();
                    updated.try_reserve_exact(length).map_err(|_| EvalError::from(
                        "E_SELECTION_TEXT_REPLACEMENT: 글 교체 메모리를 확보할 수 없습니다".to_string()))?;
                    updated.push_str(&text[..start]);updated.push_str(value);updated.push_str(&text[end..]);
                    // RD355 replaces one existing item, never merges neighbours
                    // or inserts new items. Preserve the exact nonselected bytes.
                    let mut actual=updated.graphemes(true);
                    for (ordinal,letter) in text.graphemes(true).enumerate() {
                        self.work_cost.borrow_mut().charge_element_visits(2)?;
                        let expected=if ordinal==position {value.as_str()} else {letter};
                        if actual.next()!=Some(expected) {
                            return Err("E_SELECTION_TEXT_REPLACEMENT: 교체로 주변 글자의 경계를 바꿀 수 없습니다".to_string().into());
                        }
                    }
                    if actual.next().is_some() {
                        return Err("E_SELECTION_TEXT_REPLACEMENT: 글 위치 교체로 글자 수를 바꿀 수 없습니다".to_string().into());
                    }
                    return Ok(Value::String(updated));
                }
                self.eval_call(locals,"차림.바꾼값",vec![collection,index.clone(),replacement],self.source_span_for_expr(location))
            }
            EvaluatedIndexedSelection::Axes(selectors) => {
                let Value::Tensor(tensor) = &collection else { return Err("E_TENSOR_SELECTION_TARGET: 다축 선택 대상은 텐서여야 합니다".to_string().into()); };
                self.work_cost.borrow_mut().charge_element_visits(tensor.elements().len())?;
                crate::unit_wire::tensor_replace(&tensor,selectors,&replacement).map(Value::Tensor).map_err(Into::into)
            }
        }
    }

    pub(super) fn eval_tensor_selectors(&mut self, locals: &mut HashMap<String, Value>, selectors: &[ddonirang_lang::ast::TensorAxisSelector]) -> Result<Vec<crate::unit_wire::TensorSelector>, EvalError> {
        use ddonirang_lang::ast::TensorAxisSelector;
        use crate::unit_wire::TensorSelector;
        selectors.iter().map(|s| match s {
            TensorAxisSelector::Index(expr) => {
                let value = self.eval_expr(locals, expr)?;
                Ok(TensorSelector::Index(value_to_tensor_index(&value)? as u64))
            }
            TensorAxisSelector::Range {start,end} => Ok(TensorSelector::Range {start:*start,end:*end}),
            TensorAxisSelector::Full => Ok(TensorSelector::Full),
        }).collect()
    }

    // F4: 호출 전용 임시 값이 모든 재귀 식 프레임을 차지하지 않게 한다.
    #[inline(never)]
    fn eval_call_expression(
        &mut self,
        locals: &mut HashMap<String, Value>,
        expr: &Expr,
        args: &Vec<ddonirang_lang::ast::ArgBinding>,
        func: &String,
    ) -> Result<Value, EvalError> {
        if let Some(binding)=self.program.program.origin.manipulation_phase_calls.get(&expr.id){
            if binding.definition!="B1.DEF.INPUT_PHASE.R1"||binding.ordinal_pin!="B1.PIN.INPUT_PHASE.ORDINAL.R1"
                ||args.len()!=1||args[0].resolved_pin.as_deref()!=Some("순서")||locals.contains_key(func){
                return Err("E_INPUT_PHASE_BINDING: 조작단계의 정의·순서 받는자리 결속이 다릅니다".to_string().into());
            }
            let value=self.eval_expr(locals,&args[0].expr)?;
            let ordinal=value_to_i64(&value)?;
            if !(0..4096).contains(&ordinal){return Err("E_LOGICAL_INPUT_ORDER: 사건 순서는 0~4095의 원순번이어야 합니다".to_string().into());}
            let input=self.logical_input.as_ref().ok_or_else(||EvalError::from("E_LOGICAL_INPUT_PROFILE_REQUIRED: 조작단계에는 결속된 입력 마디가 필요합니다".to_string()))?;
            self.work_cost.borrow_mut().charge_element_visits(input.events().len())?;
            let event=input.events().iter().find(|event|event.receive_ordinal==ordinal as u64)
                .ok_or_else(||EvalError::from("E_LOGICAL_INPUT_ORDER: 현재 마디에 그 원순번 사건이 없습니다".to_string()))?;
            let phase=match event.kind {
                ddonirang_core::input::LogicalInputEventKind::Number(_,phase)
                |ddonirang_core::input::LogicalInputEventKind::Coordinate(_,phase)=>phase,
                _=>return Err("E_LOGICAL_INPUT_CHANNEL: 조작단계는 수·좌표 사건만 조회합니다".to_string().into()),
            };
            self.input_reads.borrow_mut().insert(format!("logical.phase:{ordinal}"));
            self.work_cost.borrow_mut().charge_dispatch()?;
            return Ok(Value::String(manipulation_phase_text(phase).into()));
        }
        if let Some(binding)=self.program.program.origin.subject_reference_calls.get(&expr.id){
            let read=binding.definition=="VC04.DEF.ILGI.R1"&&binding.receiver_pin=="VC04.PIN.ILGI.REFERENCE.R1";
            if (!read&&(binding.definition!="VC04.DEF.GARIKIGI.R1"||binding.receiver_pin!="VC04.PIN.GARIKIGI.TARGET.R1"))||args.len()!=1 {
                return Err("E_VC04_REFERENCE_BINDING: 가리키기의 정의·받는자리 정체가 다릅니다".to_string().into());
            }
            let definition=self.program.numeric_constraint_plan.as_ref().and_then(|plan|plan.subject_reference_targets.get(&expr.id)).copied()
                .ok_or_else(||EvalError::from("E_VC04_REFERENCE_STATIC_TARGET: 원 임자 선언이 결속되지 않았습니다".to_string()))?;
            let schema=self.program.nominal_constructors.get(&definition).filter(|schema|schema.category==1)
                .ok_or_else(||EvalError::from("E_VC04_REGISTERED_CONTEXT: 원 임자 등록 정체가 없습니다".to_string()))?;
            let expected=schema.name_type;
            let stage=self.subject_stage.as_ref().map(Rc::clone).ok_or_else(||EvalError::from("E_VC04_SUBJECT_CONTEXT_REQUIRED".to_string()))?;
            let value=self.eval_expr(locals,&args[0].expr)?;
            if read {
                let Value::IdentityReference(reference)=&value else {return Err("E_VC04_READ_RECEIVER: 발급된 가리킴이 필요합니다".to_string().into())};
                let seed=self.program.products.values().find(|seed|seed.id==definition).ok_or_else(||EvalError::from("E_VC04_READ_STATIC_TARGET".to_string()))?;
                let kind=TypeRef::Applied{name:"맺음".into(),args:vec![TypeRef::Named(seed.canonical_name.clone()),TypeRef::Named("가리킴안됨".into())]};
                let owner=ddonirang_lang::sum_owner::resolve_applied_sum_owner(&kind,expr.span).map_err(EvalError::from)?;
                let outcome=stage.borrow().read(reference,&expected).map_err(EvalError::from)?;
                let (branch,pin,payload)=match outcome {
                    crate::unit_wire::subject_execution::SubjectReadOutcome::Snapshot(view)=>("잘됨","값",Value::FrozenSubjectViewV1(view)),
                    crate::unit_wire::subject_execution::SubjectReadOutcome::Broken=>("안됨","이유",Value::SumBranch{owner:"가리킴안됨".into(),branch:"끊김".into(),payload:BTreeMap::new()}),
                };
                return Ok(Value::SumBranch{owner:owner.key,branch:branch.into(),payload:BTreeMap::from([(pin.into(),payload)])});
            }
            let Value::SubjectHandleV1(handle)=&value else {return Err("E_VC04_REFERENCE_RECEIVER: 실제 임자 값이 필요합니다".to_string().into())};
            if handle.name_type_identity!=expected{return Err("E_VC04_REFERENCE_TARGET_TYPE: 정적 원 임자와 실제 대상 씨가 다릅니다".to_string().into())}
            let reference=stage.borrow().acquire(&handle).map_err(EvalError::from)?;
            return Ok(Value::IdentityReference(Box::new(reference)));
        }
        if func == "표준보개.돋" {
            use ddonirang_lang::runtime::BogaeRelationValue;
            if !(2..=3).contains(&args.len()) {return Err("E_BOGAE_RECIPE_TYPE: 돋은 대상·보기법과 선택 시간 인자를 받습니다".to_string().into());}
            let target_arg=args.iter().find(|arg|arg.resolved_pin.as_deref()==Some("대상")).unwrap_or(&args[0]);
            let recipe_arg=args.iter().find(|arg|arg.resolved_pin.as_deref()==Some("보기법")).unwrap_or(&args[1]);
            let ExprKind::Var(name)=&recipe_arg.expr.kind else {
                return Err("E_BOGAE_RECIPE_UNSEALED: 보기법은 봉인 가능한 이름 있는 셈씨여야 합니다".to_string().into());
            };
            // Local values/closures cannot impersonate a source definition.
            if locals.contains_key(name) || self.get_resource(name).is_some() {
                return Err("E_BOGAE_RECIPE_UNSEALED: 지역 값이나 세계 자원은 보기법 정체가 아닙니다".to_string().into());
            }
            let seed=self.program.functions.get(name).cloned().ok_or_else(||EvalError::from(
                "E_BOGAE_RECIPE_UNSEALED: 보기법 정의를 해소할 수 없습니다".to_string()))?;
            // Never validate one overload and execute a different one
            // through a nested call. Ambiguous definitions stay unsupported.
            let overloaded=self.program.ordinary_overloads.iter().filter(|(_,rows)|rows.len()!=1)
                .map(|(name,_)|name.clone()).collect();
            let recipe=crate::bogae_recipe::seal(&seed,&self.program.source_identity,&self.program.functions,&overloaded)
                .map_err(EvalError::from)?;
            self.work_cost.borrow_mut().charge_dispatch()?;
            self.work_cost.borrow_mut().charge_element_visits(recipe.validation_visits)?;
            let target=self.eval_expr(locals,&target_arg.expr)?;
            let (_,visits)=crate::bogae_recipe::project_target_with_visits(&target).map_err(EvalError::from)?;
            self.work_cost.borrow_mut().charge_element_visits(visits.checked_mul(2).ok_or_else(||
                EvalError::from("E_BOGAE_TARGET_UNSUPPORTED: 대상 검사 비용이 범위를 넘습니다".to_string()))?)?;
            self.check_param_type(&seed.params[0],&target)?;
            let mut result=self.eval_seed(&seed,vec![target.clone()])?;
            let Value::View(view)=&mut result else {return Err("E_BOGAE_RECIPE_TYPE: 보기법 결과는 보기값이어야 합니다".to_string().into());};
            let view=std::mem::replace(view,ddonirang_lang::runtime::ViewValue::ResultTable);
            let source_location=self.source_span_for_expr(expr).map(|span|
                [span.start_line,span.start_col.unwrap_or(0),span.end_line,span.end_col.unwrap_or(0)]);
            let time_override=if args.len()==3 {
                let time_arg=args.iter().find(|arg|arg.resolved_pin.as_deref()==Some("시간")).unwrap_or(&args[2]);
                match self.eval_expr(locals,&time_arg.expr)? {
                    Value::TimePolicy(policy)=>Some(policy),
                    _=>return Err("E_BOGAE_TIME_POLICY_INVALID: 시간은 보개시간값이어야 합니다".to_string().into()),
                }
            }else{None};
            return Ok(Value::BogaeRelation(BogaeRelationValue::Magnifier {
                target:Box::new(target),recipe,view,source_location,time_override,
            }));
        }
        if matches!(func.as_str(), "그리고" | "또는") {
            if args.len() != 2 {
                return Err(format!("{func}는 인자 2개를 받습니다").into());
            }
            let left = self.eval_expr(locals, &args[0].expr)?;
            let truth = is_truthy(&left)?;
            if (func == "그리고" && !truth) || (func == "또는" && truth) {
                self.work_cost.borrow_mut().charge_dispatch()?;
                return Ok(Value::Bool(truth));
            }
            let right = self.eval_expr(locals, &args[1].expr)?;
            return self.eval_call(locals, func, vec![left, right], self.source_span_for_expr(expr));
        }
        let mut values = Vec::with_capacity(args.len());
        let math_is_declared=matches!(ddonirang_lang::ordinary_callable::resolve_scalar_math_callable(&self.program.program,func),
            Some(ddonirang_lang::ordinary_callable::ScalarMathCallableOwner::Declared(_)));
        let declared_pins = if !locals.contains_key(func)
            && (math_is_declared || crate::work_cost::builtin_work_cost_rule(
                ddonirang_lang::stdlib::canonicalize_stdlib_alias(func)).is_ok_and(|rule| rule.is_none())) {
            self.program.functions.get(func).map(|seed| seed.params.clone())
        } else { None };
        for (index, arg) in args.iter().enumerate() {
            // Reuse the already resolved ordinary callable and pin.
            // Sealed builtin/lambda dispatch cannot borrow its name.
            let expected = declared_pins.as_ref().and_then(|pins| {
                if let Some(pin) = arg.resolved_pin.as_deref() {
                    pins.iter().find(|p| p.pin_name == pin)
                } else if args.len() == pins.len() { pins.get(index) } else { None }
            }).map(|pin| &pin.type_ref).unwrap_or(&TypeRef::Infer);
            values.push(self.eval_expr_expected(locals, &arg.expr, expected)?);
        }
        if let Some(operation) = self.program.program.origin.bytes_calls.get(&expr.id).copied() {
            use ddonirang_lang::bytes::BytesConversion;
            self.work_cost.borrow_mut().charge_dispatch()?;
            return match (operation, values.as_slice()) {
                (BytesConversion::EncodeUtf8Exact, [Value::String(text)]) => {
                    self.work_cost.borrow_mut().charge_canonical_utf8_bytes(text)?;
                    Ok(Value::Bytes(ddonirang_core::bytes::ByteSequence::encode_utf8_exact(text)))
                }
                (BytesConversion::DecodeUtf8Nfc, [Value::Bytes(bytes)]) => {
                    self.work_cost.borrow_mut().charge_element_visits(bytes.len())?;
                    let provenance = bytes.decode_utf8_nfc_with_provenance()
                        .map_err(|error| {
                            use ddonirang_core::bytes::BytesError;
                            let code = match error {
                                BytesError::Utf8 => "E_BYTES_UTF8_INVALID",
                                BytesError::UnicodeVersion => "E_UNICODE_DATA_VERSION_MISMATCH",
                                _ => return EvalError::Message(error.to_string()),
                            };
                            EvalError::FieldAccess {
                                code,
                                message: error.to_string(),
                                span: (expr.span.start, expr.span.end),
                            }
                        })?;
                    let text = provenance.decoded_text.clone();
                    self.explicit_nfc_provenance.push((expr.id, provenance));
                    Ok(Value::String(text))
                }
                _ => Err(EvalError::Message("E_TYPE_MISMATCH: UTF-8 변환의 인자 타입이 일치하지 않습니다".into())),
            };
        }
        // A3-TIME-P1: evaluate in authored order, then bind optional
        // named time arguments using the single public signature table.
        if let Some(owner)=crate::numeric_constraint_plan::resolve_source_builtin_roles(&self.program.program,func,args,locals.contains_key(func)).map_err(EvalError::from)? {
            let mut source_values=values.into_iter().map(Some).collect::<Vec<_>>();
            values=owner.canonical_order.into_iter().map(|index|source_values[index].take().expect("unique admitted source role")).collect();
        }
        if matches!(func.as_str(),"표준보기.결과표"|"표준보기.장면"|"표준보개.으뜸"|"표준보개.곁"|"표준보개.겹")
            && args.iter().any(|arg|arg.resolved_pin.is_some()) {
            let signature=ddonirang_lang::stdlib::view_function_sigs().into_iter().find(|sig|sig.name==func).expect("view signature");
            let mut slots:Vec<Option<Value>>=vec![None;signature.params.len()];
            for (arg,value) in args.iter().zip(values) {
                let index=if let Some(pin)=arg.resolved_pin.as_deref() {
                    signature.params.iter().position(|name|*name==pin)
                        .ok_or_else(||EvalError::from(format!("E_CALL_PIN_UNKNOWN: {func}에 `{pin}` 받는자리가 없습니다")))?
                }else{slots.iter().position(Option::is_none)
                    .ok_or_else(||EvalError::from(format!("E_CALL_ARGUMENT_COUNT: {func} 인자가 너무 많습니다")))?};
                if slots[index].is_some() {return Err(format!("E_CALL_PIN_DUPLICATE: {} 받는자리가 겹칩니다",signature.params[index]).into());}
                slots[index]=Some(value);
            }
            let required=signature.params.len()-1;
            if slots[..required].iter().any(Option::is_none) {
                return Err(format!("E_CALL_REQUIRED_PIN_MISSING: {func}의 필수 받는자리가 없습니다").into());
            }
            values=slots.into_iter().flatten().collect();
        }
        let resource_backed_names = self.resource_backed_caller_names(locals);
        let value =
            self.eval_call(locals, func, values, self.source_span_for_expr(expr))?;
        self.refresh_caller_resource_bindings(locals, &resource_backed_names);
        Ok(value)
    }

    // F4: 이항식 전용 임시 값은 해당 경로에서만 스택에 둔다.
    #[inline(never)]
    fn eval_infix_expression(
        &mut self,
        locals: &mut HashMap<String, Value>,
        expr: &Expr,
        left: &Box<Expr>,
        op: &String,
        right: &Box<Expr>,
    ) -> Result<Value, EvalError> {
        let comparison = matches!(op.as_str(), "==" | "!=" | "<" | "<=" | ">" | ">=");
        if comparison {
            self.check_numeric_comparison_owners(locals, left, right, op, None)?;
        }
        // A statically issued rational operand supplies the unique
        // operator expectation to a source literal tree. This reuses
        // the typed declaration/pin path before rounding, not a cast
        // of an evaluated Fixed64 or a default for standalone (3/10).
        let (left_expected,right_expected) = if comparison {
            let rational = |expr: &Expr| match self.comparison_operand_type(locals,expr) {
                Some(ddonirang_core::platform::RecordFieldType::Named { ref name })
                    if ddonirang_lang::stdlib::canonicalize_type_alias(&name)=="나눔수" => Some(TypeRef::Named("나눔수".into())),
                _ => None,
            };
            (rational(right),rational(left))
        } else { (None,None) };
        let mut l = if let Some(expected)=left_expected { self.eval_expr_expected(locals,left,&expected)? }
            else { self.eval_expr(locals, left)? };
        if matches!(op.as_str(), "그리고" | "또는" | "&&" | "||") {
            let truth = is_truthy(&l)?;
            if (matches!(op.as_str(), "그리고" | "&&") && !truth)
                || (matches!(op.as_str(), "또는" | "||") && truth)
            {
                return Ok(Value::Bool(truth));
            }
        }
        let mut r = if let Some(expected)=right_expected { self.eval_expr_expected(locals,right,&expected)? }
            else { self.eval_expr(locals, right)? };
        if l.contains_transient_subject_view()||r.contains_transient_subject_view(){return Err("E_VC04_READ_SNAPSHOT_OPERATION_UNSUPPORTED: 읽기값 자체의 연산 대신 붙이를 조회하세요".to_string().into());}
        if comparison {
            self.check_numeric_comparison_owners(locals, left, right, op, Some((&l, &r)))?;
        }
        if matches!(op.as_str(), "+"|"-"|"*"|"/") && self.program.numeric_constraint_plan.as_ref()
            .and_then(|plan| plan.expressions.get(&expr.id)).is_some_and(|kind| *kind == TypeRef::Named("나눔수".into())) {
            // RD083: only statically issued integer operands may be
            // widened here. An approximate Fixed64 is never reversed.
            for (operand, value) in [(left.as_ref(), &mut l), (right.as_ref(), &mut r)] {
                let integer = self.program.numeric_constraint_plan.as_ref().and_then(|plan| plan.expressions.get(&operand.id))
                    .is_some_and(|kind| *kind == TypeRef::Named("바른수".into()));
                if integer {
                    let Value::Fixed64(fixed) = value else { return Err("E_TYPE_MISMATCH: 바른수 연산 carrier가 맞지 않습니다".to_string().into()); };
                    if fixed.frac_part() != 0 { return Err("E_TYPE_MISMATCH: 바른수 연산에 소수값이 들어왔습니다".to_string().into()); }
                    *value = make_rational_pack_from_big_rational(&BigRational::from_integer(BigInt::from(fixed.int_part())));
                }
            }
        }
        let negated_rational = expr.source_negated_operand().is_some()
            && numeric_pack_kind(&r) == Some(NUMERIC_KIND_RATIONAL);
        let result = self.eval_infix(op, l, r)?;
        if negated_rational {
            // The old generic 0-rhs path can choose an integer carrier
            // for n/1. Negating an issued rational must not erase its
            // owner. No approximate value is admitted as exact here.
            let exact = exact_numeric_from_value(&result)?
                .ok_or_else(|| EvalError::from("E_NUMERIC_NEGATION_OWNER: 나눔수 부호 결과를 보존하지 못했습니다".to_string()))?;
            return Ok(make_rational_pack_from_big_rational(&exact.value));
        }
        Ok(result)
    }

    pub(super) fn eval_expr_inner(
        &mut self,
        locals: &mut HashMap<String, Value>,
        expr: &Expr,
    ) -> Result<Value, EvalError> {
        let _registry_key = expr_registry_key(&expr.kind);
        self.work_cost.borrow_mut().charge_dispatch()?;
        match &expr.kind {
            ExprKind::SumBranchValue {owner,applied_owner,branch,payload} => in_expression_frame(|| {
                let schema=if let Some(kind)=applied_owner {
                    let resolved=ddonirang_lang::sum_owner::resolve_applied_sum_owner(kind,expr.span).map_err(EvalError::from)?;
                    if resolved.key!=*owner {return Err("E_SUM_MATCH_OWNER_MISMATCH: 적용 고름씨 정체가 일치하지 않습니다".to_string().into());}
                    resolved.branches.into_iter().find(|candidate|candidate.name==*branch).map(|declaration|declaration.payload)
                } else {self.sum_definitions.get(owner)
                    .and_then(|branches|branches.iter().find(|candidate|candidate.name==*branch)).map(|declaration|declaration.payload.clone())}
                    .ok_or_else(||EvalError::from(format!("E_SUM_BRANCH_UNKNOWN: `{owner}.${branch}` 갈래가 선언되지 않았습니다")))?;
                if schema.len()!=payload.len() {
                    return Err(format!("E_SUM_BRANCH_PAYLOAD_REQUIRED: `{owner}.${branch}`의 값 개수가 일치하지 않습니다").into());
                }
                let mut evaluated_payload=BTreeMap::new();
                for argument in payload {
                        let expected=schema.iter().find(|schema|schema.binder==argument.pin)
                            .ok_or_else(||EvalError::from(format!("E_SUM_BRANCH_PAYLOAD_PIN: `{owner}.${branch}` 값 이름이 일치하지 않습니다")))?;
                        if evaluated_payload.contains_key(&argument.pin) {
                            return Err("E_SUM_BRANCH_PAYLOAD_DUPLICATE: 값 이름이 중복되었습니다".to_string().into());
                        }
                        let value=self.eval_expr(locals,&argument.value)?;
                        if let Err(detail)=self.check_value_type(&value,&expected.type_ref) {
                            return Err(type_mismatch_error(&expected.binder,&detail.expected,&detail.actual));
                        }
                        evaluated_payload.insert(argument.pin.clone(),value);
                }
                Ok(Value::SumBranch {owner:owner.clone(),branch:branch.clone(),payload:evaluated_payload})
            }),
            ExprKind::ProductConstruction { owner, definition, fields, category } => in_expression_frame(|| {
                let registered=if category.is_some() {
                    let resolved=self.program.nominal_constructors.get(definition).cloned()
                        .ok_or_else(||EvalError::from("E_SCHEMA_REGISTERED_CONTEXT_REQUIRED: 봉인된 명목 정체와 생성 권한의 실행 공급이 필요합니다".to_string()))?;
                    if resolved.category==1 && self.subject_stage.is_none() {return Err("E_SCHEMA_REGISTERED_CONTEXT_REQUIRED: 임자 생성에는 실제 Subject 수명 owner가 필요합니다".to_string().into());}
                    Some(resolved)
                }else{None};
                let schema = self.program.products.get(owner).filter(|schema| schema.id == *definition)
                    .cloned().ok_or_else(|| EvalError::from(
                        format!("E_PRODUCT_OWNER_MISMATCH: `{owner}`의 구성 schema가 일치하지 않습니다")))?;
                if schema.params.len() != fields.len() {
                    return Err("E_PRODUCT_FIELD_MISSING: 구성 필드 수가 일치하지 않습니다".to_string().into());
                }
                let mut values = BTreeMap::new();
                for field in fields {
                    let name = field.resolved_pin.as_deref().ok_or_else(|| EvalError::from(
                        "E_PRODUCT_NAMED_FIELD_REQUIRED: 구성 필드 이름이 없습니다".to_string()))?;
                    let expected = schema.params.iter().find(|candidate| candidate.pin_name == name)
                        .ok_or_else(|| EvalError::from("E_PRODUCT_FIELD_UNKNOWN: 알려지지 않은 구성 필드입니다".to_string()))?;
                    if values.contains_key(name) {
                        return Err("E_PRODUCT_FIELD_DUPLICATE: 구성 필드가 중복되었습니다".to_string().into());
                    }
                    let value = self.eval_expr(locals, &field.expr)?;
                    if let Some(registered)=&registered {
                        let slot=registered.slots.iter().find(|slot|slot.label==name)
                            .ok_or_else(||EvalError::from("E_SCHEMA_REGISTERED_CONTEXT_REQUIRED: 붙이 정체가 없습니다".to_string()))?;
                        resource_value_from_typed_value(&value,&slot.kind)?;
                    }else if let Err(detail) = self.check_value_type(&value, &expected.type_ref) {
                        return Err(type_mismatch_error(name, &detail.expected, &detail.actual));
                    }
                    values.insert(name.to_string(), value);
                }
                if let Some(registered)=registered {
                    let slots=registered.slots.iter().map(|slot| {
                        let value=values.get(&slot.label).ok_or_else(||EvalError::from("E_PRODUCT_FIELD_MISSING: 붙이가 없습니다".to_string()))?;
                        Ok((slot.identity,slot.kind.clone(),resource_value_from_typed_value(value,&slot.kind)?))
                    }).collect::<Result<Vec<_>,EvalError>>()?;
                    let saved=ddonirang_core::nominal::NominalSlotTableV1::from_slots(registered.name_type,&slots,2_000_000,1_000_000)
                        .map_err(|_|EvalError::from("E_SCHEMA_VALUE_LIMIT: 명목값의 크기 또는 붙이 결속이 잘못되었습니다".to_string()))?;
                    let saved=Arc::new(saved);
                    return if registered.category==1 {
                        let handle=self.subject_stage.as_ref().ok_or_else(||EvalError::from("E_VC04_SUBJECT_CONTEXT_REQUIRED".to_string()))?
                            .borrow_mut().create(saved).map_err(EvalError::from)?;
                        Ok(Value::SubjectHandleV1(Box::new(handle)))
                    } else {Ok(Value::NominalValueV1(saved))};
                }
                let mut identity = ddonirang_lang::runtime::record_type_identity(
                    &TypeRef::Named(owner.clone()), &self.program.record_schemas)?;
                let ddonirang_core::platform::RecordFieldType::Record { fields: field_types } = &mut identity else {
                    return Err("E_PRODUCT_TYPE_IDENTITY: 묶음 타입이 필요합니다".to_string().into());
                };
                Ok(Value::Record { field_types: std::mem::take(field_types), fields: values })
            }),
            ExprKind::MultiAxisSelection(selection) => in_expression_frame(|| {
                if selection.writable {
                    return Err("E_TENSOR_SELECTION_PLACE: 쓰기 위치는 값으로 읽을 수 없습니다".to_string().into());
                }
                let value = self.eval_expr(locals, &selection.target)?;
                let selectors = self.eval_indexed_selection(locals,
                    &IndexedSelectionExpr::Axes(&selection.selectors))?;
                self.read_indexed_selection(locals, value, &selectors, expr)
            }),
            ExprKind::Tensor(provider) => in_expression_frame(|| {
                use ddonirang_lang::ast::TensorProviderExpr;
                match provider {
                    TensorProviderExpr::FromCollection { input, element_type, shape } => {
                        crate::basic_tensor_element_type_identity::issue(element_type)
                            .map_err(|error| EvalError::from(format!("E_TENSOR_TYPE: 원소씨 검증 실패: {error:?}")))?;
                        let count = if shape.contains(&0) { Some(0) } else {
                            shape.iter().try_fold(1u64, |count, axis| count.checked_mul(*axis))
                        }.ok_or_else(|| EvalError::from("E_TENSOR_SHAPE: 원소 수 overflow".to_string()))?;
                        let count = usize::try_from(count).map_err(|_| EvalError::from("E_TENSOR_SHAPE: 원소 수 범위 초과".to_string()))?;
                        self.work_cost.borrow_mut().charge_element_visits(count)?;
                        let input = self.eval_expr(locals, input)?;
                        crate::unit_wire::tensor_from_collection(element_type, shape, &input)
                            .map(Value::Tensor).map_err(EvalError::from)
                    }
                    TensorProviderExpr::ToCollection { input } => {
                        let evaluated = self.eval_expr(locals, input)?;
                        let Value::Tensor(value) = &evaluated else {
                            return Err("E_TENSOR_TYPE: 차림으로 입력은 Tensor여야 합니다".to_string().into());
                        };
                        crate::unit_wire::verify_runtime_tensor(&value).map_err(EvalError::from)?;
                        crate::unit_wire::preflight_tensor_collection(value.shape()).map_err(EvalError::from)?;
                        fn rebuild(shape: &[u64], values: &mut std::slice::Iter<'_, Fixed64>, meter: &mut WorkCostMeter) -> Result<Value, EvalError> {
                            match shape.split_first() {
                                None => values.next().copied().map(Value::Fixed64)
                                    .ok_or_else(|| EvalError::from("E_TENSOR_SHAPE: 원소 누락".to_string())),
                                Some((axis, rest)) => {
                                    let count = usize::try_from(*axis).map_err(|_| EvalError::from("E_TENSOR_SHAPE: 축 범위 초과".to_string()))?;
                                    meter.charge_element_visits(count)?;
                                    let mut out = Vec::new();
                                    out.try_reserve_exact(count).map_err(|_| EvalError::from("E_TENSOR_SHAPE: 차림 용량 초과".to_string()))?;
                                    for _ in 0..count { out.push(rebuild(rest, values, meter)?); }
                                    Ok(Value::List(out))
                                }
                            }
                        }
                        rebuild(value.shape(), &mut value.elements().iter(), &mut self.work_cost.borrow_mut())
                    }
                }
            }),
            ExprKind::Literal(Literal::Resource(path)) => in_expression_frame(|| {
                let handle = self.resolve_asset_handle(path)?;
                Ok(Value::ResourceHandle(handle))
            }),
            ExprKind::Literal(lit) => in_expression_frame(|| {
                let _registry_key = literal_registry_key(lit);
                if let Some(occurrence)=self.program.program.origin.atom_value_occurrences.get(&expr.id)
                    .filter(|_|self.program.program.source_epoch==ddonirang_lang::ast::SourceEpoch::V25){
                    if let ddonirang_lang::ast::AtomValueRole::StandardBogaeSide{selector}=&occurrence.role{
                        let side=ddonirang_lang::runtime::SidePlacement::from_authored(selector)
                            .ok_or_else(||EvalError::from("E_ATOM_POLICY_OWNER_REQUIRED: 곁 위치 policy 결속이 다릅니다".to_owned()))?;
                        self.work_cost.borrow_mut().charge_canonical_utf8_bytes(side.authored_name())?;
                        return Ok(Value::String(side.authored_name().into()));
                    }
                    let atom=self.program.registered_atom_literals.get(&expr.id)
                        .ok_or_else(||EvalError::from("E_VC04_ATOM_SOURCE_CONTEXT_REQUIRED: 원자 값의 기록된 원문 등록이 필요합니다".to_owned()))?;
                    self.work_cost.borrow_mut().charge_canonical_utf8_bytes(&occurrence.label)?;
                    return Ok(Value::RegisteredAtom(Box::new(atom.clone())));
                }
                if let Some(plan) = &self.program.numeric_constraint_plan {
                    if expr.numeric_literal_source.is_some() {
                        let kind = plan.literals.get(&expr.id).copied().ok_or_else(||EvalError::from(
                            "E_TYPE_MISMATCH: 원문 수 리터럴의 정적 발급 계획이 없습니다".to_string()))?;
                        return self.issue_planned_numeric_literal(expr, kind);
                    }
                }
                match lit {
                    Literal::Bytes(bytes) => {
                        self.work_cost.borrow_mut().charge_element_visits(bytes.len())?;
                    }
                    Literal::String(text) | Literal::Atom(text) | Literal::Resource(text) => {
                        self.work_cost
                            .borrow_mut()
                            .charge_canonical_utf8_bytes(text)?;
                    }
                    Literal::Regex(regex) => {
                        let mut meter = self.work_cost.borrow_mut();
                        meter.charge_canonical_utf8_bytes(&regex.pattern)?;
                        meter.charge_canonical_utf8_bytes(&regex.flags)?;
                    }
                    Literal::Int(_) | Literal::Fixed64(_) | Literal::Bool(_) | Literal::None => {}
                }
                Ok(literal_to_value(lit))
            }),
            ExprKind::Var(name) => in_expression_frame(|| {
                if name=="프레임수" && self.program.host_numeric_supply.is_some()
                    && !locals.contains_key(name) && !self.resources.contains_key(name) && !self.committed_resource_exists(name) {
                    let expected=self.program.numeric_constraint_plan.as_ref().and_then(|plan|plan.expressions.get(&expr.id));
                    if expected!=Some(&TypeRef::Named("바른수".into()))
                        || !matches!(self.defaults.get(name),Some(Value::Fixed64(value)) if *value==Fixed64::ZERO) {
                        return Err("E_TYPE_MISMATCH: 프레임수 host 원천은 바른수 초기 0이며 source 씨/초기화와 충돌합니다".to_string().into());
                    }
                }
                if self
                    .result_place_frames
                    .iter()
                    .any(|frame| frame.canonical_name == *name)
                {
                    Err(format!(
                        "E_RESULTPLACE_READ_FORBIDDEN: '{}' 결과 자리는 쓰기 전용입니다",
                        name
                    )
                    .into())
                } else if let Some(value) = locals.get(name) {
                    Ok(value.clone())
                } else if name == "참" {
                    Ok(Value::Bool(true))
                } else if name == "거짓" {
                    Ok(Value::Bool(false))
                } else if name == "없음" {
                    Ok(Value::None)
                } else if self.logical_input.is_some() && (name == "입력키" || name.starts_with("샘.키보드.")) {
                    Err("E_LOGICAL_INPUT_PHYSICAL_UNSUPPORTED: 논리 입력 원문은 물리 입력 자원을 읽지 않습니다".to_string().into())
                } else if let Some(value) = self.get_resource(name) {
                    Ok(value)
                } else {
                    Err(format!("정의되지 않은 변수: {}", name).into())
                }
            }),
            ExprKind::FieldAccess {
                target,
                field,
                field_span,
            } => in_expression_frame(|| {
                let base = self.eval_expr(locals, target)?;
                match &base {
                    Value::NominalValueV1(value)=>{
                        let catalog=self.program.registered_nominal_catalog.as_ref().ok_or_else(||EvalError::from("E_SCHEMA_REGISTERED_CONTEXT_REQUIRED: 이름씨 붙이의 등록 정보가 없습니다".to_string()))?;
                        let schema=catalog.schema(value.name_type_identity()).filter(|s|s.category==2)
                            .ok_or_else(||EvalError::from("E_SCHEMA_REGISTERED_CONTEXT_REQUIRED: 다른 원문의 이름씨입니다".to_string()))?;
                        let slot=schema.slots.iter().find(|s|s.label==*field).ok_or_else(||EvalError::FieldAccess{code:"E_FIELD_MISSING",message:format!("이름씨에 `{field}` 붙이가 없습니다"),span:(field_span.start,field_span.end)})?;
                        let stage=self.subject_stage.as_ref().map(|s|s.borrow());
                        let refs=stage.as_ref().map(|s|s.reference_context()).transpose().map_err(EvalError::from)?;
                        let values=crate::unit_wire::read_registered_nominal_slots(value,catalog,refs.as_ref(),&self.program.registered_atom_contracts)
                            .map_err(|cause|EvalError::from(format!("E_SCHEMA_VALUE_BINDING: 이름씨 붙이 결속이 다릅니다 ({cause:?})")))?;
                        Ok(value_from_resource_value(values.get(&slot.identity).ok_or_else(||EvalError::from("E_FIELD_MISSING: 붙이 값이 없습니다".to_string()))?))
                    }
                    Value::FrozenSubjectViewV1(view)=>{
                        let schema=self.program.nominal_constructors.values().find(|schema|schema.category==1&&&schema.name_type==view.name_type_identity())
                            .ok_or_else(||EvalError::from("E_VC04_READ_CATALOG: 읽기값의 원 등록 씨가 없습니다".to_string()))?;
                        let slot=schema.slots.iter().find(|slot|slot.label==*field)
                            .ok_or_else(||EvalError::from("E_FIELD_MISSING: 읽기값의 붙이가 없습니다".to_string()))?;
                        let value=view.slot(&slot.identity).ok_or_else(||EvalError::from("E_VC04_READ_SLOT: 고정 붙이 값이 없습니다".to_string()))?;
                        Ok(value_from_resource_value(value))
                    }
                    Value::Pack(pack) | Value::Record { fields: pack, .. } => {
                        let Some(value) = pack.get(field) else {
                            return Err(EvalError::FieldAccess {
                                code: "E_FIELD_MISSING",
                                message: format!("묶음에 `{field}` 필드가 없습니다"),
                                span: (field_span.start, field_span.end),
                            });
                        };
                        if matches!(value, Value::None) {
                            return Err(EvalError::FieldAccess {
                                code: "E_FIELD_VALUE_NONE",
                                message: format!("`{field}` 필드에 값이 없습니다"),
                                span: (field_span.start, field_span.end),
                            });
                        }
                        Ok(value.clone())
                    }
                    Value::Map(entries) => {
                        let key = Value::String(field.clone());
                        let value = map_get(&entries, &key);
                        if matches!(value, Value::None) {
                            return Err(EvalError::FieldAccess {
                                code: "E_FIELD_MISSING",
                                message: format!("짝맞춤에 `{field}` 열쇠가 없습니다"),
                                span: (field_span.start, field_span.end),
                            });
                        }
                        Ok(value)
                    }
                    _ => Err(EvalError::FieldAccess {
                        code: "E_FIELD_TARGET_TYPE",
                        message: format!("`{field}` 앞의 값은 이름형 값꾸러미가 아닙니다"),
                        span: (field_span.start, field_span.end),
                    }),
                }
            }),
            ExprKind::Call { args, func } => self.eval_call_expression(locals, expr, args, func),
            ExprKind::CollectionConstructor { elements } => in_expression_frame(|| {
                let mut values = Vec::with_capacity(elements.len());
                let mut element_type: Option<String> = None;
                for element in elements {
                    self.work_cost.borrow_mut().charge_element_visits(1)?;
                    let value = self.eval_expr(locals, element)?;
                    // RD-130: authored relation roles share 보개관계값 in a
                    // flat relation list. Keep their specific binding types.
                    let current_type = if matches!(value, Value::BogaeRelation(_)) {
                        "보개관계값".to_string()
                    } else { root_binding_type_key(&value) };
                    if let Some(expected) = &element_type {
                        if expected != &current_type {
                            return Err(EvalError::TypeMismatch {
                                pin: "차림 원소".to_string(),
                                expected: expected.clone(),
                                actual: current_type,
                            });
                        }
                    } else {
                        element_type = Some(current_type);
                    }
                    values.push(value);
                }
                Ok(Value::List(values))
            }),
            ExprKind::SelectionPlace { .. } => in_expression_frame(|| Err("E_SELECTION_PLACE_VALUE_CONTEXT: 선택 위치는 `<-` 대입 대상에서만 사용할 수 있습니다".to_string().into())),
            ExprKind::SeedLiteral { param, body } => in_expression_frame(|| {
                if locals.values().any(Value::contains_transient_subject_view){return Err("E_VC04_READ_SNAPSHOT_ESCAPE: 임자 읽기값을 함수 캡처에 보존할 수 없습니다".to_string().into());}
                self.work_cost.borrow_mut().note_created_identity()?;
                Ok(Value::Lambda(LambdaValue {
                    id: next_lambda_id(),
                    param: param.clone(),
                    body: (*body.clone()),
                    captured: locals.clone(),
                }))
            }),
            ExprKind::Infix { left, op, right } => self.eval_infix_expression(locals, expr, left, op, right),
            ExprKind::Suffix { value, at } => in_expression_frame(|| {
                let base = self.eval_expr(locals, value)?;
                match at {
                    AtSuffix::Asset(path) => {
                        let handle = self.resolve_asset_handle(path)?;
                        Ok(Value::ResourceHandle(handle))
                    }
                    AtSuffix::Unit(_) => apply_suffix_value(base, at),
                }
            }),
            ExprKind::Thunk(_) => in_expression_frame(|| Err("Thunk는 즉시 평가 표지가 필요합니다".to_string().into())),
            ExprKind::Eval { thunk, mode } => in_expression_frame(|| {
                let ExprKind::Thunk(body) = &thunk.kind else {
                    return Err("평가 표지는 안은문장에만 붙일 수 있습니다"
                        .to_string()
                        .into());
                };
                match mode {
                    ddonirang_lang::ThunkEvalMode::Value => {
                        let value = self.eval_body_for_value(locals, body)?;
                        Ok(value)
                    }
                    ddonirang_lang::ThunkEvalMode::Bool => {
                        let value = self.eval_body_for_value(locals, body)?;
                        Ok(Value::Bool(is_truthy(&value)?))
                    }
                    ddonirang_lang::ThunkEvalMode::Not => {
                        let value = self.eval_body_for_value(locals, body)?;
                        Ok(Value::Bool(!is_truthy(&value)?))
                    }
                    ddonirang_lang::ThunkEvalMode::Do => {
                        let _ = self.eval_body(locals, body)?;
                        Ok(Value::None)
                    }
                    ddonirang_lang::ThunkEvalMode::Pipe => {
                        let value = self.eval_body_for_value(locals, body)?;
                        Ok(value)
                    }
                }
            }),
            ExprKind::Pipe { stages } => in_expression_frame(|| self.eval_pipe(locals, stages)),
            ExprKind::FlowValue => in_expression_frame(|| {
                if let Some(Some(value)) = self.flow_stack.last() {
                    Ok(value.clone())
                } else {
                    Ok(Value::None)
                }
            }),
            ExprKind::TupleConstructor { elements } => in_expression_frame(|| {
                let mut values=Vec::with_capacity(elements.len());
                for element in elements {
                    self.work_cost.borrow_mut().charge_element_visits(1)?;
                    values.push(self.eval_expr(locals,element)?);
                }
                Ok(Value::Tuple(values))
            }),
            ExprKind::Pack { fields } => in_expression_frame(|| {
                let mut map = std::collections::BTreeMap::new();
                for (name, value) in fields {
                    self.work_cost.borrow_mut().charge_element_visits(1)?;
                    if map.contains_key(name) {
                        return Err(format!("묶음 키 '{}'가 중복되었습니다", name).into());
                    }
                    let v = self.eval_expr(locals, value)?;
                    map.insert(name.clone(), v);
                }
                Ok(Value::Pack(map))
            }),
            ExprKind::Assertion(assertion) => in_expression_frame(|| Ok(Value::Assertion(assertion.clone()))),
            ExprKind::StateMachine(machine) => in_expression_frame(|| Ok(Value::StateMachine(machine.clone()))),
            ExprKind::Formula(formula) => in_expression_frame(|| Ok(Value::Formula(formula.clone()))),
            ExprKind::Template(template) => in_expression_frame(|| {
                self.work_cost
                    .borrow_mut()
                    .charge_canonical_utf8_bytes(&template.raw)?;
                Ok(Value::Template(template.clone()))
            }),
            ExprKind::TemplateRender { template, inject } => in_expression_frame(|| {
                let pack = self.eval_inject_fields(locals, inject)?;
                self.eval_call(
                    locals,
                    "채우기",
                    vec![Value::Template(template.clone()), pack],
                    self.source_span_for_expr(expr),
                )
            }),
            ExprKind::FormulaEval { formula, inject } => in_expression_frame(|| {
                let pack = self.eval_inject_fields(locals, inject)?;
                self.eval_call(
                    locals,
                    "풀기",
                    vec![Value::Formula(formula.clone()), pack],
                    self.source_span_for_expr(expr),
                )
            }),
            ExprKind::Nuance { expr, .. } => in_expression_frame(|| self.eval_expr(locals, expr)),
        }
    }

}
