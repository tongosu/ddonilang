//! Standard-call dispatch; arm bodies, early exits and charges moved unchanged.
use super::*;

impl<'a> EvalContext<'a> {
    pub(super) fn eval_call(
        &mut self,
        locals: &HashMap<String, Value>,
        func: &str,
        args: Vec<Value>,
        source_span: Option<SourceSpan>,
    ) -> Result<Value, EvalError> {
        use ddonirang_lang::ordinary_callable::{resolve_scalar_math_callable,ScalarMathCallableOwner};
        let math_owner=resolve_scalar_math_callable(&self.program.program,func);
        if !locals.contains_key(func) && matches!(math_owner,Some(ScalarMathCallableOwner::Declared(_))) {
            let seed=self.resolve_ordinary_callable_overload(func,&args)?
                .or_else(||self.program.functions.get(func).cloned())
                .ok_or_else(||EvalError::from("E_ORDINARY_CALLABLE_NO_MATCH: 수학 함수와 동명인 원문 정의를 해소할 수 없습니다".to_string()))?;
            if let Some(call_span)=source_span.as_ref() {
                let origin=&self.program.program.origin;
                for (call_id,admitted) in &origin.registered_call_candidates {
                    if origin.node_spans.get(call_id).and_then(|span|self.source_span_from_span(span))
                        .is_some_and(|span|&span==call_span) && !admitted.contains(&seed.id) {
                        return Err("E_ORDINARY_CALLABLE_NO_MATCH: 등록 표면의 정의와 인자 타입이 일치하지 않습니다".to_string().into());
                    }
                }
            }
            return self.eval_seed(&seed,args);
        }
        let func = match math_owner {
            Some(ScalarMathCallableOwner::Builtin(builtin))=>builtin.canonical_name(),
            _=>Self::canonicalize_stdlib_alias(func),
        };
        if matches!(func,"표준시간.공통"|"표준시간.고정마디"|"표준시간.상대마디") {
            self.work_cost.borrow_mut().charge_dispatch()?;
            builtin_work_cost_rule(func)?;
            use ddonirang_lang::runtime::BogaeTimePolicy;
            if func=="표준시간.공통" && args.is_empty() { return Ok(Value::TimePolicy(BogaeTimePolicy::Common)); }
            let [Value::Fixed64(number)]=args.as_slice() else {
                return Err("E_BOGAE_TIME_POLICY_INVALID: 고정·상대 마디는 정수 마디 하나를 받습니다".to_string().into());
            };
            let raw=number.raw_i64();
            if raw%(1i64<<32)!=0 {return Err("E_BOGAE_TIME_POLICY_INVALID: 마디는 정수여야 합니다".to_string().into());}
            let madi=raw/(1i64<<32);
            let policy=match func {
                "표준시간.고정마디" if madi>=0=>BogaeTimePolicy::Fixed(madi as u64),
                "표준시간.상대마디"=>BogaeTimePolicy::Relative(madi),
                _=>return Err("E_BOGAE_TIME_POLICY_INVALID: 고정 마디는 음수가 될 수 없습니다".to_string().into()),
            };
            return Ok(Value::TimePolicy(policy));
        }
        // A3-TIME-P1 provisional optional `시간` pin on existing constructors.
        let time_base_count=match func {"표준보기.결과표"=>Some(0),
            "표준보기.장면"|"표준보개.으뜸"=>Some(1),"표준보개.곁"=>Some(2),
            "표준보개.겹"=>Some(3),_=>None};
        let mut args=args;
        // F5-GA[OPTIONAL-TIME-POP]: count는 고정 0..3이고 len==count+1일 때만 pop한다.
        // F5-GA[OPTIONAL-TIME-POP]: 같은 소유 Vec를 검사한 뒤 pop 전에는 변경·외부 호출이 없다.
        let time_override=if time_base_count.is_some_and(|count|args.len()==count+1) {
            match args.pop().expect("one optional time argument") {
                Value::TimePolicy(policy)=>Some(policy),
                _=>return Err("E_BOGAE_TIME_POLICY_INVALID: 시간은 보개시간값이어야 합니다".to_string().into()),
            }
        }else{None};
        if self.logical_input.is_some() && matches!(func,
            "눌렸나" | "막눌렸나" | "입력키" | "입력키?" | "입력키!") {
            return Err("E_LOGICAL_INPUT_PHYSICAL_UNSUPPORTED: 논리 입력 원문은 물리 키를 읽지 않습니다".to_string().into());
        }
        self.work_cost.borrow_mut().charge_dispatch()?;
        let result = (|| {
            let rule = builtin_work_cost_rule(func)?;
            // 아직 정체를 결속하지 않은 작성 관계는 순서 있는 차림으로만 운반한다.
            // 그 외 builtin의 열쇠·집합·검색은 내용/표시를 identity로 사용할 수 없다.
            if rule.is_some() && !matches!(func, "차림" | "목록" | "표준보개.짜임" | "표준보개.겹"
                | "표준보개.으뜸" | "표준보개.곁" | "표준장면.원" | "표준장면.다각형" | "표준장면.글" | "표준장면.평면" | "표준보기.장면") {
                for arg in &args {
                    if contains_unresolved_bogae_relation_for_builtin(arg, &mut self.work_cost.borrow_mut(), func == "추가")? {
                        return Err("E_BOGAE_RELATION_UNRESOLVED: 보개관계 정체 해소 전에 이 내장 함수를 사용할 수 없습니다".to_string().into());
                    }
                }
            }
            if rule.is_none() && !self.program.functions.contains_key(func) {
                return Err(self
                    .work_cost
                    .borrow()
                    .registry_missing(format!("runtime_builtin:{func}"))
                    .into());
            }
            let ordinary_overload = if rule.is_none() {
                self.resolve_ordinary_callable_overload(func, &args)?
            } else {
                None
            };
            if let Some(call_span) = source_span.as_ref() {
                let origin = &self.program.program.origin;
                for (call_id, admitted) in &origin.registered_call_candidates {
                    let matches_call = origin
                        .node_spans
                        .get(call_id)
                        .and_then(|span| self.source_span_from_span(span))
                        .is_some_and(|span| &span == call_span);
                    if matches_call {
                        let selected = ordinary_overload
                            .as_ref()
                            .or_else(|| self.program.functions.get(func));
                        if !selected.is_some_and(|seed| admitted.contains(&seed.id)) {
                            return Err("E_ORDINARY_CALLABLE_NO_MATCH: 등록 표면의 정의와 인자 타입이 일치하지 않습니다"
                                .to_string()
                                .into());
                        }
                    }
                }
            }
            if let Some(rule) = rule {
                if rule.bigint_rational_operand_and_result_bytes {
                    self.charge_big_number_parts(&args.iter().collect::<Vec<_>>())?;
                }
            }
            // F5-GA[BUILTIN-ARITY]: 감사한 분기는 첫 len!=N 조기 Err 뒤에만 N 미만의 상수 자리를 읽고, 소유 args의 길이를 바꾸거나 다시 가리지 않는다.
            // F5-GA[BUILTIN-ARITY]: 이 근거는 해당 args 첨자 경계에만 적용하며 값 형식·수 연산·동적 첨자의 안전성을 대신하지 않는다.
            let value = match func {
                "표준보기.결과표" => in_call_arm_frame(|| {
                    if !args.is_empty() {
                        return Err("표준보기.결과표는 인자를 받지 않습니다".to_string().into());
                    }
                    let view=ddonirang_lang::runtime::ViewValue::ResultTable;
                    Ok(Value::View(match time_override {Some(policy)=>ddonirang_lang::runtime::ViewValue::Timed {view:Box::new(view),policy},None=>view}))
                }),
                "표준보개.으뜸" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("표준보개.으뜸은 보기 인자 1개를 받습니다".to_string().into());
                    }
                    let Value::View(view) = &args[0] else {
                        return Err("표준보개.으뜸의 보기는 보기값이어야 합니다".to_string().into());
                    };
                    self.work_cost.borrow_mut().charge_element_visits(crate::scene_value::clone_visits(&args[0]))?;
                    let source_location = source_span.as_ref().map(|span|
                        [span.start_line, span.start_col.unwrap_or(0), span.end_line, span.end_col.unwrap_or(0)]);
                    Ok(Value::BogaeRelation(ddonirang_lang::runtime::BogaeRelationValue::Primary { view: view.clone(), source_location,time_override }))
                }),
                "표준보개.짜임" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("표준보개.짜임은 관계들 인자 1개를 받습니다".to_string().into());
                    }
                    let Value::List(items) = &args[0] else {
                        return Err("E_BOGAE_UNSUPPORTED: 관계들은 평평한 보개관계값 차림이어야 합니다".to_string().into());
                    };
                    let mut relations = Vec::new();
                    for item in items {
                        self.work_cost.borrow_mut().charge_element_visits(1)?;
                        let Value::BogaeRelation(relation) = item else {
                            return Err("E_BOGAE_UNSUPPORTED: 관계들은 평평한 보개관계값 차림이어야 합니다".to_string().into());
                        };
                        self.work_cost.borrow_mut().charge_element_visits(crate::scene_value::view_clone_visits(relation.view()))?;
                        if let ddonirang_lang::runtime::BogaeRelationValue::Magnifier {target,..}=relation {
                            let (_,visits)=crate::bogae_recipe::project_target_with_visits(target).map_err(EvalError::from)?;
                            self.work_cost.borrow_mut().charge_element_visits(visits)?;
                        }
                        relations.push(relation.clone());
                    }
                    let composition = ddonirang_lang::runtime::BogaeCompositionValue::from_relations(relations)
                        .map_err(|error| EvalError::from(error.to_string()))?;
                    Ok(Value::BogaeComposition(composition))
                }),
                "표준보개.겹" => in_call_arm_frame(|| {
                    use ddonirang_lang::runtime::BogaeRelationValue;
                    if args.len() != 3 { return Err("표준보개.겹은 보기·기준·순서 인자 3개를 받습니다".to_string().into()); }
                    let Value::View(view) = &args[0] else { return Err("겹의 보기는 보기값이어야 합니다".to_string().into()); };
                    self.work_cost.borrow_mut().charge_element_visits(crate::scene_value::clone_visits(&args[0]))?;
                    let Value::BogaeRelation(BogaeRelationValue::Primary {source_location: basis_location,..}) = &args[1] else {
                        return Err("E_BOGAE_OVERLAY_BASIS_INVALID: 겹 기준은 으뜸 관계여야 합니다".to_string().into());
                    };
                    let order = expect_int_strict(&args[2])?;
                    let source_location = source_span.as_ref().map(|span|
                        [span.start_line, span.start_col.unwrap_or(0), span.end_line, span.end_col.unwrap_or(0)]);
                    Ok(Value::BogaeRelation(BogaeRelationValue::Overlay {view:view.clone(), basis_location:*basis_location, order, source_location,time_override}))
                }),
                "표준보개.곁" => in_call_arm_frame(|| {
                    use ddonirang_lang::runtime::{BogaeRelationValue, SidePlacement};
                    if args.len() != 2 { return Err("표준보개.곁은 보기·자리 인자 2개를 받습니다".to_string().into()); }
                    let Value::View(view) = &args[0] else { return Err("곁의 보기는 보기값이어야 합니다".to_string().into()); };
                    self.work_cost.borrow_mut().charge_element_visits(crate::scene_value::clone_visits(&args[0]))?;
                    let Value::String(place) = &args[1] else { return Err("E_BOGAE_UNSUPPORTED: 곁 자리는 왼쪽·오른쪽·위·아래·자동입니다".to_string().into()); };
                    let placement = SidePlacement::from_authored(place).ok_or_else(|| EvalError::from(
                        "E_BOGAE_UNSUPPORTED: 곁 자리는 왼쪽·오른쪽·위·아래·자동입니다".to_string()))?;
                    let source_location = source_span.as_ref().map(|span|
                        [span.start_line, span.start_col.unwrap_or(0), span.end_line, span.end_col.unwrap_or(0)]);
                    Ok(Value::BogaeRelation(BogaeRelationValue::Side {view:view.clone(), placement, source_location,time_override}))
                }),
                "표준보개.돋" => in_call_arm_frame(|| {
                    Err("E_BOGAE_RECIPE_UNSEALED: 돋 보기법은 작성 정의 정체로 해소해야 합니다".to_string().into())
                }),
                "표준장면.원" | "표준장면.다각형" | "표준장면.글" | "표준장면.평면" | "표준보기.장면" => in_call_arm_frame(|| {
                    for arg in &args {
                        self.work_cost.borrow_mut().charge_element_visits(crate::scene_value::clone_visits(arg))?;
                    }
                    if let Some(Value::List(elements)) = args.first() {
                        self.work_cost.borrow_mut().charge_element_visits(elements.len())?;
                        if func == "표준장면.다각형" {
                            let visits=elements.len().checked_mul(elements.len()).ok_or_else(||EvalError::from("E_SCENE_GEOMETRY: 꼭짓점 검사 비용이 범위를 넘습니다".to_string()))?;
                            self.work_cost.borrow_mut().charge_element_visits(visits)?;
                        } else {
                            for element in elements {self.work_cost.borrow_mut().charge_element_visits(crate::scene_value::clone_visits(element))?;}
                        }
                    }
                    let mut value=crate::scene_value::construct_with_meter(func, &args,Some(&mut self.work_cost.borrow_mut())).map_err(EvalError::from)?;
                    if let (Value::View(view),Some(policy))=(&mut value,time_override) {
                        let view=std::mem::replace(view,ddonirang_lang::runtime::ViewValue::ResultTable);
                        Ok(Value::View(ddonirang_lang::runtime::ViewValue::Timed {view:Box::new(view),policy}))
                    } else { Ok(value) }
                }),
                "아님" | "아니다" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("아님은 인자 1개를 받습니다".to_string().into());
                    }
                    let value = is_truthy(&args[0])?;
                    Ok(Value::Bool(!value))
                }),
                "그리고" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("그리고는 인자 2개를 받습니다".to_string().into());
                    }
                    let left = is_truthy(&args[0])?;
                    let right = is_truthy(&args[1])?;
                    Ok(Value::Bool(left && right))
                }),
                "또는" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("또는은 인자 2개를 받습니다".to_string().into());
                    }
                    let left = is_truthy(&args[0])?;
                    let right = is_truthy(&args[1])?;
                    Ok(Value::Bool(left || right))
                }),
                "수" | "셈수" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("셈수는 인자 1개를 받습니다".to_string().into());
                    }
                    let value = value_to_fixed64_coerce(&args[0])?;
                    Ok(Value::Fixed64(value))
                }),
                "바른수" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("바른수는 인자 1개를 받습니다".to_string().into());
                    }
                    let value = value_to_i64_strict(&args[0])?;
                    Ok(Value::Fixed64(Fixed64::from_i64(value)))
                }),
                "큰바른수" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("큰바른수는 인자 1개를 받습니다".to_string().into());
                    }
                    Ok(make_big_int_value(&args[0])?)
                }),
                "나눔수" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("나눔수는 인자 2개(분자, 분모)를 받습니다"
                            .to_string()
                            .into());
                    }
                    Ok(make_rational_value(&args[0], &args[1])?)
                }),
                "곱수" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("곱수는 인자 1개를 받습니다".to_string().into());
                    }
                    let (value, status) = make_factor_value(&args[0])?;
                    self.note_factor_route_from_value(&value);
                    if status == FACTOR_DECOMP_STATUS_DEFERRED {
                        self.emit_factor_decomposition_deferred_diag(&value, source_span);
                    }
                    Ok(value)
                }),
                "차림" | "목록" => in_call_arm_frame(|| {
                    self.work_cost
                        .borrow_mut()
                        .charge_element_visits(args.len())?;
                    Ok(list_new(args.clone()))
                }),
                "모음" => in_call_arm_frame(|| {
                    let mut items = BTreeMap::new();
                    for item in &args {
                        self.work_cost.borrow_mut().charge_element_visits(1)?;
                        let key = value_canon(&item);
                        items.entry(key).or_insert_with(|| item.clone());
                    }
                    Ok(Value::Set(items))
                }),
                "짝맞춤" => in_call_arm_frame(|| {
                    if args.len() % 2 != 0 {
                        return Err("짝맞춤은 열쇠/값 쌍 인자를 받습니다".to_string().into());
                    }
                    let mut entries = BTreeMap::new();
                    for chunk in args.chunks(2) {
                        self.work_cost.borrow_mut().charge_element_visits(2)?;
                        let key_value = chunk[0].clone();
                        let entry = MapEntry {
                            key: key_value.clone(),
                            value: chunk[1].clone(),
                        };
                        entries.insert(map_key_canon(&key_value), entry);
                    }
                    Ok(Value::Map(entries))
                }),
                "짝맞춤.값" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("짝맞춤.값은 인자 2개를 받습니다".to_string().into());
                    }
                    let Value::Map(map) = &args[0] else {
                        return Err("짝맞춤.값은 짝맞춤 인자가 필요합니다".to_string().into());
                    };
                    checked_optional_map_get(map, &args[1])
                }),
                "짝맞춤.바꾼값" => in_call_arm_frame(|| {
                    if args.len() != 3 {
                        return Err("짝맞춤.바꾼값은 인자 3개를 받습니다".to_string().into());
                    }
                    let Value::Map(map) = &args[0] else {
                        return Err("짝맞춤.바꾼값은 짝맞춤 인자가 필요합니다"
                            .to_string()
                            .into());
                    };
                    let mut entries = map.clone();
                    entries.insert(
                        map_key_canon(&args[1]),
                        MapEntry {
                            key: args[1].clone(),
                            value: args[2].clone(),
                        },
                    );
                    Ok(Value::Map(entries))
                }),
                "흐름.만들기" => in_call_arm_frame(|| {
                    if args.len() != 1 && args.len() != 2 {
                        return Err(
                            "흐름.만들기는 인자 1개(용량) 또는 2개(용량, 초기값차림)를 받습니다"
                                .to_string()
                                .into(),
                        );
                    }
                    let capacity = value_to_i64(&args[0])?;
                    if capacity <= 0 {
                        return Err("흐름 용량은 1 이상이어야 합니다".to_string().into());
                    }
                    let mut stream = stream_new(capacity as usize)?;
                    if args.len() == 2 {
                        let Value::List(initial) = &args[1] else {
                            return Err("흐름.만들기의 두 번째 인자는 차림이어야 합니다"
                                .to_string()
                                .into());
                        };
                        for item in initial {
                            stream = stream_push(stream, item.clone());
                        }
                    }
                    Ok(stream_to_value(&stream))
                }),
                "흐름.밀어넣기" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("흐름.밀어넣기는 인자 2개(흐름, 값)를 받습니다"
                            .to_string()
                            .into());
                    }
                    let stream = stream_from_value(&args[0])?;
                    let stream = stream_push(stream, args[1].clone());
                    Ok(stream_to_value(&stream))
                }),
                "흐름.차림" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("흐름.차림은 인자 1개(흐름)를 받습니다".to_string().into());
                    }
                    let stream = stream_from_value(&args[0])?;
                    Ok(Value::List(stream_oldest_to_newest(&stream)))
                }),
                "흐름.최근값" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("흐름.최근값은 인자 1개(흐름)를 받습니다".to_string().into());
                    }
                    let stream = stream_from_value(&args[0])?;
                    if stream.len == 0 || stream.capacity == 0 {
                        Ok(Value::None)
                    } else {
                        Ok(stream.buffer[stream.head].clone())
                    }
                }),
                "흐름.길이" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("흐름.길이는 인자 1개(흐름)를 받습니다".to_string().into());
                    }
                    let stream = stream_from_value(&args[0])?;
                    Ok(Value::Fixed64(Fixed64::from_i64(stream.len as i64)))
                }),
                "흐름.용량" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("흐름.용량은 인자 1개(흐름)를 받습니다".to_string().into());
                    }
                    let stream = stream_from_value(&args[0])?;
                    Ok(Value::Fixed64(Fixed64::from_i64(stream.capacity as i64)))
                }),
                "흐름.비우기" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("흐름.비우기는 인자 1개(흐름)를 받습니다".to_string().into());
                    }
                    let stream = stream_from_value(&args[0])?;
                    Ok(stream_to_value(&stream_clear(stream)))
                }),
                "흐름.잘라보기" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("흐름.잘라보기는 인자 2개(흐름, 개수)를 받습니다"
                            .to_string()
                            .into());
                    }
                    let stream = stream_from_value(&args[0])?;
                    let count = value_to_i64(&args[1])?;
                    if count < 0 {
                        return Err("흐름.잘라보기 개수는 0 이상이어야 합니다"
                            .to_string()
                            .into());
                    }
                    let ordered = stream_oldest_to_newest(&stream);
                    let take = (count as usize).min(ordered.len());
                    let start = ordered.len().saturating_sub(take);
                    Ok(Value::List(ordered[start..].to_vec()))
                }),
                "키목록" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("키목록은 인자 1개를 받습니다".to_string().into());
                    }
                    let (Value::Pack(pack) | Value::Record { fields: pack, .. }) = &args[0] else {
                        return Err("키목록은 묶음 인자를 받습니다".to_string().into());
                    };
                    let keys = pack.keys().cloned().map(Value::String).collect::<Vec<_>>();
                    Ok(Value::List(keys))
                }),
                "값목록" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("값목록은 인자 1개를 받습니다".to_string().into());
                    }
                    let (Value::Pack(pack) | Value::Record { fields: pack, .. }) = &args[0] else {
                        return Err("값목록은 묶음 인자를 받습니다".to_string().into());
                    };
                    let values = pack.values().cloned().collect::<Vec<_>>();
                    Ok(Value::List(values))
                }),
                "쌍목록" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("쌍목록은 인자 1개를 받습니다".to_string().into());
                    }
                    let (Value::Pack(pack) | Value::Record { fields: pack, .. }) = &args[0] else {
                        return Err("쌍목록은 묶음 인자를 받습니다".to_string().into());
                    };
                    let pairs = pack
                        .iter()
                        .map(|(key, value)| {
                            Value::List(vec![Value::String(key.clone()), value.clone()])
                        })
                        .collect::<Vec<_>>();
                    Ok(Value::List(pairs))
                }),
                "격자.만들기" => in_call_arm_frame(|| {
                    if args.len() != 3 {
                        return Err("격자.만들기는 인자 3개를 받습니다".to_string().into());
                    }
                    let width = value_to_i64(&args[0])?;
                    let height = value_to_i64(&args[1])?;
                    make_std_grid(width, height, args[2].clone())
                }),
                "격자.너비" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("격자.너비는 인자 1개를 받습니다".to_string().into());
                    }
                    let fields = std_grid_fields(&args[0])?;
                    let (width, _) = std_grid_dims(fields)?;
                    Ok(fixed_value(width))
                }),
                "격자.높이" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("격자.높이는 인자 1개를 받습니다".to_string().into());
                    }
                    let fields = std_grid_fields(&args[0])?;
                    let (_, height) = std_grid_dims(fields)?;
                    Ok(fixed_value(height))
                }),
                "격자.값" => in_call_arm_frame(|| {
                    if args.len() != 3 {
                        return Err("격자.값은 인자 3개를 받습니다".to_string().into());
                    }
                    let fields = std_grid_fields(&args[0])?;
                    let idx =
                        std_grid_index(fields, value_to_i64(&args[1])?, value_to_i64(&args[2])?)?;
                    let cells = std_grid_cells(fields)?;
                    Ok(cells.get(idx).cloned().unwrap_or(Value::None))
                }),
                "격자.바꾼값" => in_call_arm_frame(|| {
                    if args.len() != 4 {
                        return Err("격자.바꾼값은 인자 4개를 받습니다".to_string().into());
                    }
                    let fields = std_grid_fields(&args[0])?;
                    let idx =
                        std_grid_index(fields, value_to_i64(&args[1])?, value_to_i64(&args[2])?)?;
                    let mut next = fields.clone();
                    let mut cells = std_grid_cells(fields)?.clone();
                    if idx >= cells.len() {
                        return Err("격자 칸 목록이 크기와 맞지 않습니다".to_string().into());
                    }
                    cells[idx] = args[3].clone();
                    next.insert("칸들".to_string(), Value::List(cells));
                    Ok(Value::Pack(next))
                }),
                "격자.안인가" => in_call_arm_frame(|| {
                    if args.len() != 3 {
                        return Err("격자.안인가는 인자 3개를 받습니다".to_string().into());
                    }
                    Ok(Value::Bool(std_grid_inside(
                        &args[0],
                        value_to_i64(&args[1])?,
                        value_to_i64(&args[2])?,
                    )?))
                }),
                "격자.막혔나" => {
                    if args.len() != 4 {
                        return Err("격자.막혔나는 인자 4개를 받습니다".to_string().into());
                    }
                    if !std_grid_inside(&args[0], value_to_i64(&args[1])?, value_to_i64(&args[2])?)?
                    {
                        return Ok(Value::Bool(true));
                    }
                    let fields = std_grid_fields(&args[0])?;
                    let idx =
                        std_grid_index(fields, value_to_i64(&args[1])?, value_to_i64(&args[2])?)?;
                    let cell = std_grid_cells(fields)?
                        .get(idx)
                        .cloned()
                        .unwrap_or(Value::None);
                    Ok(Value::Bool(value_matches_any(&cell, &args[3])))
                }
                "격자.길찾기" => in_call_arm_frame(|| {
                    if args.len() != 6 {
                        return Err("격자.길찾기는 인자 6개를 받습니다".to_string().into());
                    }
                    std_grid_pathfind(
                        &args[0],
                        value_to_i64(&args[1])?,
                        value_to_i64(&args[2])?,
                        value_to_i64(&args[3])?,
                        value_to_i64(&args[4])?,
                        &args[5],
                    )
                }),
                "블록조각.만들기" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("블록조각.만들기는 인자 1개를 받습니다".to_string().into());
                    }
                    let cells = std_block_piece_cells_from_value(&args[0])?;
                    Ok(make_std_block_piece(cells))
                }),
                "블록조각.칸목록" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("블록조각.칸목록은 인자 1개를 받습니다".to_string().into());
                    }
                    let cells = std_block_piece_cells(&args[0])?;
                    Ok(std_block_piece_cells_value(&cells))
                }),
                "블록조각.이동" => in_call_arm_frame(|| {
                    if args.len() != 3 {
                        return Err("블록조각.이동은 인자 3개를 받습니다".to_string().into());
                    }
                    let cells = std_block_piece_cells(&args[0])?;
                    let dx = value_to_i64(&args[1])?;
                    let dy = value_to_i64(&args[2])?;
                    Ok(make_std_block_piece(
                        cells.into_iter().map(|(x, y)| (x + dx, y + dy)).collect(),
                    ))
                }),
                "블록조각.회전" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("블록조각.회전은 인자 2개를 받습니다".to_string().into());
                    }
                    let cells = std_block_piece_cells(&args[0])?;
                    let Value::String(direction) = &args[1] else {
                        return Err("블록조각.회전은 글 방향을 받습니다".to_string().into());
                    };
                    let rotated = cells
                        .into_iter()
                        .map(|(x, y)| match direction.as_str() {
                            "오른쪽" => Ok((-y, x)),
                            "왼쪽" => Ok((y, -x)),
                            "뒤집기" => Ok((-x, -y)),
                            _ => Err(format!(
                                "블록조각 회전 방향을 지원하지 않습니다: {direction}"
                            )
                            .into()),
                        })
                        .collect::<Result<Vec<_>, EvalError>>()?;
                    Ok(make_std_block_piece(rotated))
                }),
                "블록조각.충돌?" => in_call_arm_frame(|| {
                    if args.len() != 3 {
                        return Err("블록조각.충돌?은 인자 3개를 받습니다".to_string().into());
                    }
                    Ok(Value::Bool(std_block_piece_collides(
                        &args[0], &args[1], &args[2],
                    )?))
                }),
                "블록조각.고정" => in_call_arm_frame(|| {
                    if args.len() != 3 {
                        return Err("블록조각.고정은 인자 3개를 받습니다".to_string().into());
                    }
                    std_block_piece_lock(&args[0], &args[1], args[2].clone())
                }),
                "물리1d.위치갱신" => in_call_arm_frame(|| {
                    if args.len() != 3 {
                        return Err("물리1d.위치갱신은 인자 3개를 받습니다".to_string().into());
                    }
                    let position = expect_unit_value(&args[0])?;
                    let velocity = expect_unit_value(&args[1])?;
                    let dt = expect_unit_value(&args[2])?;
                    ensure_dimensionless_unit(&dt)?;
                    ensure_same_unit_dim(&position, &velocity)?;
                    let displacement = velocity.value.checked_mul(dt.value)?;
                    let next_position = position.value.checked_add(displacement)?;
                    Ok(unit_value_to_runtime_value(UnitValue {
                        value: next_position,
                        dim: position.dim,
                    }))
                }),
                "물리1d.속도갱신" => in_call_arm_frame(|| {
                    if args.len() != 3 {
                        return Err("물리1d.속도갱신은 인자 3개를 받습니다".to_string().into());
                    }
                    let velocity = expect_unit_value(&args[0])?;
                    let acceleration = expect_unit_value(&args[1])?;
                    let dt = expect_unit_value(&args[2])?;
                    ensure_dimensionless_unit(&dt)?;
                    ensure_same_unit_dim(&velocity, &acceleration)?;
                    let delta_velocity = acceleration.value.checked_mul(dt.value)?;
                    let next_velocity = velocity.value.checked_add(delta_velocity)?;
                    Ok(unit_value_to_runtime_value(UnitValue {
                        value: next_velocity,
                        dim: velocity.dim,
                    }))
                }),
                "물리1d.탄성충돌1d" => in_call_arm_frame(|| {
                    if args.len() != 4 {
                        return Err("물리1d.탄성충돌1d는 인자 4개를 받습니다".to_string().into());
                    }
                    let m1 = expect_unit_value(&args[0])?;
                    let v1 = expect_unit_value(&args[1])?;
                    let m2 = expect_unit_value(&args[2])?;
                    let v2 = expect_unit_value(&args[3])?;
                    ensure_dimensionless_unit(&m1)?;
                    ensure_dimensionless_unit(&m2)?;
                    ensure_same_unit_dim(&v1, &v2)?;
                    let denom = m1.value.checked_add(m2.value)?;
                    if denom.raw_i64() == 0 {
                        return Err("물리1d.탄성충돌1d 질량 합은 0이 될 수 없습니다"
                            .to_string()
                            .into());
                    }
                    let two = Fixed64::from_i64(2);
                    let next_v1_num = m1
                        .value
                        .checked_sub(m2.value)?
                        .checked_mul(v1.value)?
                        .checked_add(two.checked_mul(m2.value)?.checked_mul(v2.value)?)?;
                    let next_v2_num = two
                        .checked_mul(m1.value)?
                        .checked_mul(v1.value)?
                        .checked_add(m2.value.checked_sub(m1.value)?.checked_mul(v2.value)?)?;
                    let next_v1 = next_v1_num.try_div(denom)?;
                    let next_v2 = next_v2_num.try_div(denom)?;
                    Ok(Value::List(vec![
                        unit_value_to_runtime_value(UnitValue {
                            value: next_v1,
                            dim: v1.dim,
                        }),
                        unit_value_to_runtime_value(UnitValue {
                            value: next_v2,
                            dim: v1.dim,
                        }),
                    ]))
                }),
                "채우기" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("채우기는 인자 2개를 받습니다".to_string().into());
                    }
                    let Value::Template(template) = &args[0] else {
                        return Err("채우기는 글무늬 인자가 필요합니다".to_string().into());
                    };
                    let Value::Pack(pack) = &args[1] else {
                        return Err("채우기는 묶음 주입 인자가 필요합니다".to_string().into());
                    };
                    let rendered = render_template(template, pack)?;
                    Ok(Value::String(rendered))
                }),
                "풀기" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("풀기는 인자 2개를 받습니다".to_string().into());
                    }
                    let Value::Formula(formula) = &args[0] else {
                        return Err("풀기는 수식값 인자가 필요합니다".to_string().into());
                    };
                    let Value::Pack(pack) = &args[1] else {
                        return Err("풀기는 묶음 주입 인자가 필요합니다".to_string().into());
                    };
                    if matches!(
                        formula.dialect,
                        FormulaDialect::Latex | FormulaDialect::Other(_)
                    ) {
                        return Err("FATAL:FORMULA_DIALECT_UNSUPPORTED".to_string().into());
                    }
                    let parsed = parse_formula_value(formula)?;
                    let required = &parsed.vars;
                    let provided: BTreeSet<String> = pack.keys().cloned().collect();
                    let missing: Vec<String> = required.difference(&provided).cloned().collect();
                    if !missing.is_empty() {
                        return Err(format!(
                            "풀기: 주입 키가 누락되었습니다: {}",
                            missing.join(", ")
                        )
                        .into());
                    }
                    let extra: Vec<String> = provided.difference(&required).cloned().collect();
                    if !extra.is_empty() {
                        return Err(
                            format!("풀기: 주입 키가 여분입니다: {}", extra.join(", ")).into()
                        );
                    }
                    for (key, value) in pack {
                        if matches!(value, Value::None) {
                            return Err(format!("풀기: 키 '{}' 값이 없습니다", key).into());
                        }
                    }
                    self.work_cost.borrow_mut().charge_solver_evaluations(1)?;
                    let value = eval_formula_expr(&parsed.expr, pack)?;
                    Ok(unit_value_to_value(value))
                }),
                "살피기" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("살피기는 인자 2개를 받습니다".to_string().into());
                    }
                    let Value::Assertion(assertion) = &args[0] else {
                        return Err("살피기는 세움값 인자가 필요합니다".to_string().into());
                    };
                    let bindings = assertion_bindings_from_value(&args[1])?;
                    self.eval_assertion(assertion, bindings)
                }),
                "처음으로" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("처음으로는 인자 1개를 받습니다".to_string().into());
                    }
                    let Value::StateMachine(machine) = &args[0] else {
                        return Err("처음으로는 상태머신값 인자가 필요합니다".to_string().into());
                    };
                    Ok(Value::String(machine.initial.clone()))
                }),
                "지금상태" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("지금상태는 인자 2개를 받습니다".to_string().into());
                    }
                    let Value::StateMachine(machine) = &args[0] else {
                        return Err("지금상태는 상태머신값 인자가 필요합니다".to_string().into());
                    };
                    let Value::String(state) = &args[1] else {
                        return Err("지금상태는 글 상태 이름 인자가 필요합니다"
                            .to_string()
                            .into());
                    };
                    ensure_declared_state(machine, state)?;
                    Ok(Value::String(state.clone()))
                }),
                "다음으로" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("다음으로는 인자 2개를 받습니다".to_string().into());
                    }
                    let Value::StateMachine(machine) = &args[0] else {
                        return Err("다음으로는 상태머신값 인자가 필요합니다".to_string().into());
                    };
                    let Value::String(state) = &args[1] else {
                        return Err("다음으로는 글 상태 이름 인자가 필요합니다"
                            .to_string()
                            .into());
                    };
                    ensure_declared_state(machine, state)?;
                    let mut candidates: Vec<&StateTransition> = machine
                        .transitions
                        .iter()
                        .filter(|t| t.from == *state)
                        .collect();
                    if candidates.is_empty() {
                        return Err(format!("E_STATE_TRANSITION_MISSING: {}", state).into());
                    }
                    candidates.sort_by_key(|transition| {
                        (
                            transition.from.as_str(),
                            transition.to.as_str(),
                            transition.guard_name.as_deref().unwrap_or(""),
                            transition.action_name.as_deref().unwrap_or(""),
                        )
                    });
                    let mut passing = Vec::new();
                    let mut guard_rejected = false;
                    for transition in candidates {
                        let bindings = build_state_machine_transition_bindings(
                            &transition.from,
                            &transition.to,
                        );
                        if let Some(guard_name) = &transition.guard_name {
                            if !self.eval_state_machine_guard(locals, guard_name, &bindings)? {
                                guard_rejected = true;
                                continue;
                            }
                        }
                        passing.push(transition);
                    }
                    let [next] = passing.as_slice() else {
                        if passing.len() > 1 {
                            return Err(
                                format!("E_STATE_TRANSITION_GUARD_REJECTED: {}", state).into()
                            );
                        }
                        if guard_rejected {
                            return Err(
                                format!("E_STATE_TRANSITION_GUARD_REJECTED: {}", state).into()
                            );
                        }
                        return Err(format!("E_STATE_TRANSITION_MISSING: {}", state).into());
                    };
                    self.apply_state_machine_transition(locals, machine, next)?;
                    self.emit_state_machine_transition(
                        state,
                        &next.to,
                        next.guard_name.as_deref(),
                        next.action_name.as_deref(),
                    );
                    Ok(Value::String(next.to.clone()))
                }),
                "정리하기" => in_call_arm_frame(|| {
                    let formula = expect_single_formula(&args, "정리하기")?;
                    let transformed = transform_formula_value(
                        formula,
                        FormulaTransformOptions::default(),
                        "simplify",
                        "정리하기",
                    )?;
                    Ok(Value::Formula(transformed))
                }),
                "전개하기" => in_call_arm_frame(|| {
                    let formula = expect_single_formula(&args, "전개하기")?;
                    let transformed = transform_formula_value(
                        formula,
                        FormulaTransformOptions::default(),
                        "expand",
                        "전개하기",
                    )?;
                    Ok(Value::Formula(transformed))
                }),
                "인수분해하기" => in_call_arm_frame(|| {
                    let formula = expect_single_formula(&args, "인수분해하기")?;
                    let transformed = transform_formula_value(
                        formula,
                        FormulaTransformOptions::default(),
                        "factor",
                        "인수분해하기",
                    )?;
                    Ok(Value::Formula(transformed))
                }),
                "미분하기" => in_call_arm_frame(|| {
                    let (formula, options) = expect_formula_transform(&args, "미분하기")?;
                    let transformed =
                        transform_formula_value(formula, options, "diff", "미분하기")?;
                    Ok(Value::Formula(transformed))
                }),
                "적분하기" => in_call_arm_frame(|| {
                    let (formula, options) = expect_formula_transform(&args, "적분하기")?;
                    let transformed = transform_formula_value(formula, options, "int", "적분하기")?;
                    Ok(Value::Formula(transformed))
                }),
                "동치인가" => in_call_arm_frame(|| {
                    let (left, right) = expect_two_formulas(&args, "동치인가")?;
                    Ok(Value::Bool(symbolic_formulas_equivalent(&left, &right)?))
                }),
                "잇기" => in_call_arm_frame(|| {
                    let (left, right) = expect_two_formulas(&args, "잇기")?;
                    Ok(make_relation_pack(left, right))
                }),
                "이음관계.관계목록" => in_call_arm_frame(|| eval_endpoint_relation_list(&args)),
                "이음관계.정규화" => in_call_arm_frame(|| eval_endpoint_relation_normalize(&args)),
                "이음관계.방정식목록" => in_call_arm_frame(|| eval_endpoint_formula_relation_list(&args)),
                "이음관계.방정식화" => in_call_arm_frame(|| eval_endpoint_formula_relation_set(&args)),
                "이음관계.값관계목록" => in_call_arm_frame(|| eval_endpoint_boundary_value_relation_list(&args)),
                "이음관계.값주입" => in_call_arm_frame(|| eval_endpoint_boundary_value_injection(&args)),
                "이음관계.풀기" => in_call_arm_frame(|| {
                    eval_endpoint_explicit_solve_with_meter(&args, Some(&self.work_cost))
                }),
                "이음관계.풀이값목록" => in_call_arm_frame(|| eval_endpoint_solve_result_value_list(&args)),
                "이음관계.풀이원복" => in_call_arm_frame(|| eval_endpoint_solve_result_remap(&args)),
                "이음관계.범위위반목록" => in_call_arm_frame(|| {
                    eval_endpoint_boundary_range_violation_list(&args)
                }),
                "이음관계.범위검사" => in_call_arm_frame(|| eval_endpoint_boundary_range_check(&args)),
                "이음관계.풀고범위위반목록" => in_call_arm_frame(|| {
                    eval_endpoint_explicit_solve_range_violation_list_with_meter(
                        &args,
                        Some(&self.work_cost),
                    )
                }),
                "이음관계.풀고범위검사" => in_call_arm_frame(|| {
                    eval_endpoint_explicit_solve_range_check_with_meter(
                        &args,
                        Some(&self.work_cost),
                    )
                }),
                "이음관계.풀고범위행목록" => in_call_arm_frame(|| {
                    eval_endpoint_solve_range_report_rows_with_meter(&args, Some(&self.work_cost))
                }),
                "이음관계.풀고범위보고서" => in_call_arm_frame(|| {
                    eval_endpoint_solve_range_report_with_meter(&args, Some(&self.work_cost))
                }),
                "이음관계.보고서문자표" => in_call_arm_frame(|| eval_endpoint_solve_range_text_report(&args)),
                "이음관계.풀고범위문자표" => in_call_arm_frame(|| {
                    eval_endpoint_explicit_solve_range_text_report_with_meter(
                        &args,
                        Some(&self.work_cost),
                    )
                }),
                "이음관계.풀고범위케이스" => in_call_arm_frame(|| {
                    eval_endpoint_solve_range_case_with_meter(&args, Some(&self.work_cost))
                }),
                "이음관계.풀고범위스위트" => in_call_arm_frame(|| {
                    eval_endpoint_solve_range_case_suite_with_meter(&args, Some(&self.work_cost))
                }),
                "이음관계.풀고범위스위트문자표" => in_call_arm_frame(|| {
                    eval_endpoint_solve_range_case_suite_text(&args)
                }),
                "이음관계.풀고범위스위트상세문자표" => in_call_arm_frame(|| {
                    eval_endpoint_solve_range_case_suite_detail_text(&args)
                }),
                "이음관계.풀고범위실행상세문자표" => in_call_arm_frame(|| {
                    eval_endpoint_solve_range_case_suite_run_detail_text_with_meter(
                        &args,
                        Some(&self.work_cost),
                    )
                }),
                "이음관계.풀고범위스위트요약" => in_call_arm_frame(|| {
                    eval_endpoint_solve_range_case_suite_summary(&args)
                }),
                "이음관계.풀고범위실행요약" => in_call_arm_frame(|| {
                    eval_endpoint_solve_range_case_suite_run_summary_with_meter(
                        &args,
                        Some(&self.work_cost),
                    )
                }),
                "이음관계.풀고범위스위트판정" => in_call_arm_frame(|| {
                    eval_endpoint_solve_range_case_suite_check(&args)
                }),
                "이음관계.풀고범위실행판정" => in_call_arm_frame(|| {
                    eval_endpoint_solve_range_case_suite_run_check_with_meter(
                        &args,
                        Some(&self.work_cost),
                    )
                }),
                "방정식풀기" => in_call_arm_frame(|| {
                    let relations = expect_equation_relations(&args)?;
                    eval_relation_solve_result_with_meter(&relations, Some(&self.work_cost))
                }),
                "다항식.풀기" => in_call_arm_frame(|| {
                    eval_polynomial_solve_result_with_meter(&args, Some(&self.work_cost))
                }),
                "증명하기" => in_call_arm_frame(|| {
                    let proof =
                        eval_symbolic_proof_tactic_with_meter(&args, Some(&self.work_cost))?;
                    Ok(Value::Pack(proof))
                }),
                "번째" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("번째는 인자 2개를 받습니다".to_string().into());
                    }
                    let value = list_nth(&args[0], &args[1]).map_err(runtime_error)?;
                    self.work_cost.borrow_mut().charge_element_visits(1)?;
                    Ok(value)
                }),
                "차림.값" => {
                    if args.len() != 2 {
                        return Err("차림.값은 인자 2개를 받습니다".to_string().into());
                    }
                    if let Value::Tensor(tensor) = &args[0] {
                        let index = value_to_tensor_index(&args[1])?;
                        self.work_cost.borrow_mut().charge_element_visits(tensor.elements().len())?;
                        return crate::unit_wire::tensor_select(tensor, &[crate::unit_wire::TensorSelector::Index(index as u64)])
                            .map_err(Into::into);
                    }
                    let value = list_nth(&args[0], &args[1]).map_err(runtime_error)?;
                    self.work_cost.borrow_mut().charge_element_visits(1)?;
                    Ok(value)
                }
                "차림.바꾼값" => {
                    if args.len() != 3 {
                        return Err("차림.바꾼값은 인자 3개를 받습니다".to_string().into());
                    }
                    if let Value::Tensor(tensor) = &args[0] {
                        let index = value_to_tensor_index(&args[1])?;
                        self.work_cost.borrow_mut().charge_element_visits(tensor.elements().len())?;
                        return crate::unit_wire::tensor_replace(tensor, &[crate::unit_wire::TensorSelector::Index(index as u64)], &args[2])
                            .map(Value::Tensor).map_err(Into::into);
                    }
                    let index = value_to_index(&args[1])?;
                    let Value::List(items) = &args[0] else {
                        return Err("차림.바꾼값은 차림 인자를 받습니다".to_string().into());
                    };
                    if index >= items.len() {
                        return Err(runtime_error(RuntimeError::IndexOutOfRange));
                    }
                    self.work_cost
                        .borrow_mut()
                        .charge_element_visits(items.len())?;
                    let mut out = items.clone();
                    out[index] = args[2].clone();
                    Ok(Value::List(out))
                }
                "길이" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("길이는 인자 1개를 받습니다".to_string().into());
                    }
                    match args[0] {
                        Value::List(_) => list_len(&args[0]).map_err(runtime_error),
                        Value::String(_) => string_len(&args[0]).map_err(runtime_error),
                        _ => Err("길이는 차림/글에만 적용됩니다".to_string().into()),
                    }
                }),
                "범위" => in_call_arm_frame(|| {
                    let (start, end, step) = match args.len() {
                        2 => {
                            let start = unit_value_from_value(&args[0])?;
                            let end = unit_value_from_value(&args[1])?;
                            if start.dim != end.dim {
                                return Err(unit_error(UnitError::DimensionMismatch {
                                    left: start.dim,
                                    right: end.dim,
                                }));
                            }
                            let step = UnitValue {
                                value: Fixed64::from_i64(1),
                                dim: start.dim,
                            };
                            (start, end, step)
                        }
                        3 => {
                            let start = unit_value_from_value(&args[0])?;
                            let end = unit_value_from_value(&args[1])?;
                            let step = unit_value_from_value(&args[2])?;
                            if start.dim != end.dim {
                                return Err(unit_error(UnitError::DimensionMismatch {
                                    left: start.dim,
                                    right: end.dim,
                                }));
                            }
                            if start.dim != step.dim {
                                return Err(unit_error(UnitError::DimensionMismatch {
                                    left: start.dim,
                                    right: step.dim,
                                }));
                            }
                            (start, end, step)
                        }
                        _ => {
                            return Err("범위는 인자 2개 또는 3개를 받습니다".to_string().into());
                        }
                    };
                    if step.value.raw_i64() == 0 {
                        return Err("범위 간격은 0이 될 수 없습니다".to_string().into());
                    }
                    let mut items = Vec::new();
                    let mut current = start.value;
                    let step_value = step.value;
                    if step_value.raw_i64() > 0 {
                        while current.raw_i64() <= end.value.raw_i64() {
                            items.push(unit_value_to_value(UnitValue {
                                value: current,
                                dim: start.dim,
                            }));
                            if current == end.value {
                                break;
                            }
                            current = current.checked_add(step_value)?;
                        }
                    } else {
                        while current.raw_i64() >= end.value.raw_i64() {
                            items.push(unit_value_to_value(UnitValue {
                                value: current,
                                dim: start.dim,
                            }));
                            if current == end.value {
                                break;
                            }
                            current = current.checked_add(step_value)?;
                        }
                    }
                    Ok(Value::List(items))
                }),
                "표준.범위" => in_call_arm_frame(|| {
                    if args.len() != 3 {
                        return Err("표준.범위는 인자 3개를 받습니다".to_string().into());
                    }
                    let start = unit_value_from_value(&args[0])?;
                    let end = unit_value_from_value(&args[1])?;
                    if start.dim != end.dim {
                        return Err(unit_error(UnitError::DimensionMismatch {
                            left: start.dim,
                            right: end.dim,
                        }));
                    }
                    let include = unit_value_from_value(&args[2])?;
                    if include.dim != UnitDim::NONE {
                        return Err(unit_error(UnitError::DimensionMismatch {
                            left: include.dim,
                            right: UnitDim::NONE,
                        }));
                    }
                    let flag = include.value.raw_i64();
                    let one = Fixed64::from_i64(1).raw_i64();
                    if flag != 0 && flag != one {
                        return Err("끝포함은 0 또는 1이어야 합니다".to_string().into());
                    }
                    let include_end = flag == one;
                    let step = UnitValue {
                        value: Fixed64::from_i64(1),
                        dim: start.dim,
                    };
                    let mut items = Vec::new();
                    let mut current = start.value;
                    if start.value.raw_i64() <= end.value.raw_i64() {
                        if include_end {
                            while current.raw_i64() <= end.value.raw_i64() {
                                items.push(unit_value_to_value(UnitValue {
                                    value: current,
                                    dim: start.dim,
                                }));
                                if current == end.value {
                                    break;
                                }
                                current = current.checked_add(step.value)?;
                            }
                        } else {
                            while current.raw_i64() < end.value.raw_i64() {
                                items.push(unit_value_to_value(UnitValue {
                                    value: current,
                                    dim: start.dim,
                                }));
                                if current == end.value {
                                    break;
                                }
                                current = current.checked_add(step.value)?;
                            }
                        }
                    }
                    Ok(Value::List(items))
                }),
                "대문자로바꾸기" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("대문자로바꾸기는 인자 1개를 받습니다".to_string().into());
                    }
                    let Value::String(text) = &args[0] else {
                        return Err("대문자로바꾸기는 글 인자를 받습니다".to_string().into());
                    };
                    Ok(Value::String(text.to_uppercase()))
                }),
                "소문자로바꾸기" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("소문자로바꾸기는 인자 1개를 받습니다".to_string().into());
                    }
                    let Value::String(text) = &args[0] else {
                        return Err("소문자로바꾸기는 글 인자를 받습니다".to_string().into());
                    };
                    Ok(Value::String(text.to_lowercase()))
                }),
                "다듬기" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("다듬기는 인자 1개를 받습니다".to_string().into());
                    }
                    let Value::String(text) = &args[0] else {
                        return Err("다듬기는 글 인자를 받습니다".to_string().into());
                    };
                    Ok(Value::String(text.trim().to_string()))
                }),
                "되풀이하기" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("되풀이하기는 인자 2개를 받습니다".to_string().into());
                    }
                    let Value::String(text) = &args[0] else {
                        return Err("되풀이하기 첫 인자는 글이어야 합니다".to_string().into());
                    };
                    let count = value_to_i64(&args[1])?;
                    if count < 0 {
                        return Err("되풀이하기 횟수는 0 이상이어야 합니다".to_string().into());
                    }
                    Ok(Value::String(text.repeat(count as usize)))
                }),
                "글자뽑기" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("글자뽑기는 인자 2개를 받습니다".to_string().into());
                    }
                    let Value::String(text) = &args[0] else {
                        return Err("글자뽑기는 글 인자를 받습니다".to_string().into());
                    };
                    let idx = value_to_index(&args[1])?;
                    let ch = text.chars().nth(idx).map(|c| c.to_string());
                    Ok(ch.map(Value::String).unwrap_or(Value::None))
                }),
                "찾기" => {
                    if args.len() != 2 {
                        return Err("찾기는 인자 2개를 받습니다".to_string().into());
                    }
                    // RD-476: canonical safe lookup for maps; retain the existing
                    // text-pattern overload independently.
                    if let Value::Map(map) = &args[0] {
                        return checked_optional_map_get(map, &args[1]);
                    }
                    if let Value::List(items) = &args[0] {
                        let index = value_to_i64(&args[1])?;
                        let item = usize::try_from(index).ok().and_then(|index| items.get(index));
                        return match item {
                            Some(Value::None) => Err("E_RUNTIME_NESTED_OPTIONAL_UNSUPPORTED: 없음이 저장된 차림의 조회는 아직 지원하지 않습니다".to_string().into()),
                            Some(value) => Ok(value.clone()),
                            None => Ok(Value::None),
                        };
                    }
                    let Value::String(text) = &args[0] else {
                        return Err("찾기는 글 인자를 받습니다".to_string().into());
                    };
                    if self.program.program.source_epoch == ddonirang_lang::SourceEpoch::V25
                        || !matches!(&args[1], Value::String(_))
                    {
                        return Err("E_RUNTIME_TEXT_LOOKUP_UNSUPPORTED: 글 찾기는 아직 지원하지 않습니다. 고정된 유니코드 글자 경계 구현이 필요합니다".to_string().into());
                    }
                    let Value::String(pattern) = &args[1] else {
                        return Err("찾기는 글 인자를 받습니다".to_string().into());
                    };
                    let idx = text
                        .find(pattern)
                        .map(|byte_idx| text[..byte_idx].chars().count() as i64);
                    Ok(Value::Fixed64(Fixed64::from_i64(idx.unwrap_or(-1))))
                }
                "찾기?" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("찾기?는 인자 2개를 받습니다".to_string().into());
                    }
                    let Value::Map(map) = &args[0] else {
                        return Err("찾기?는 짝맞춤 인자를 받습니다".to_string().into());
                    };
                    checked_optional_map_get(map, &args[1])
                }),
                "있는가" | "없는가" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err(format!("{func}는 인자 1개를 받습니다").into());
                    }
                    let absent = matches!(args[0], Value::None);
                    Ok(Value::Bool(if func == "없는가" { absent } else { !absent }))
                }),
                "정규맞추기" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("정규맞추기는 인자 2개를 받습니다".to_string().into());
                    }
                    let Value::String(text) = &args[0] else {
                        return Err("정규맞추기 첫 인자는 글이어야 합니다".to_string().into());
                    };
                    let Value::Regex(regex) = &args[1] else {
                        return Err("정규맞추기 둘째 인자는 정규식이어야 합니다"
                            .to_string()
                            .into());
                    };
                    Ok(Value::Bool(regex_is_full_match(text, regex)?))
                }),
                "정규찾기" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("정규찾기는 인자 2개를 받습니다".to_string().into());
                    }
                    let Value::String(text) = &args[0] else {
                        return Err("정규찾기 첫 인자는 글이어야 합니다".to_string().into());
                    };
                    let Value::Regex(regex) = &args[1] else {
                        return Err("정규찾기 둘째 인자는 정규식이어야 합니다"
                            .to_string()
                            .into());
                    };
                    let matched = regex_find_first(text, regex)?;
                    Ok(matched.map(Value::String).unwrap_or(Value::None))
                }),
                "정규캡처하기" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("정규캡처하기는 인자 2개를 받습니다".to_string().into());
                    }
                    let Value::String(text) = &args[0] else {
                        return Err("정규캡처하기 첫 인자는 글이어야 합니다".to_string().into());
                    };
                    let Value::Regex(regex) = &args[1] else {
                        return Err("정규캡처하기 둘째 인자는 정규식이어야 합니다"
                            .to_string()
                            .into());
                    };
                    Ok(Value::List(
                        regex_capture_first(text, regex)?
                            .into_iter()
                            .map(Value::String)
                            .collect(),
                    ))
                }),
                "정규이름캡처하기" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("정규이름캡처하기는 인자 2개를 받습니다".to_string().into());
                    }
                    let Value::String(text) = &args[0] else {
                        return Err("정규이름캡처하기 첫 인자는 글이어야 합니다"
                            .to_string()
                            .into());
                    };
                    let Value::Regex(regex) = &args[1] else {
                        return Err("정규이름캡처하기 둘째 인자는 정규식이어야 합니다"
                            .to_string()
                            .into());
                    };
                    Ok(Value::Map(regex_named_capture_first(text, regex)?))
                }),
                "정규바꾸기" => in_call_arm_frame(|| {
                    if args.len() != 3 {
                        return Err("정규바꾸기는 인자 3개를 받습니다".to_string().into());
                    }
                    let Value::String(text) = &args[0] else {
                        return Err("정규바꾸기 첫 인자는 글이어야 합니다".to_string().into());
                    };
                    let Value::Regex(regex) = &args[1] else {
                        return Err("정규바꾸기 둘째 인자는 정규식이어야 합니다"
                            .to_string()
                            .into());
                    };
                    let Value::String(replacement) = &args[2] else {
                        return Err("정규바꾸기 셋째 인자는 글이어야 합니다".to_string().into());
                    };
                    Ok(Value::String(regex_replace_all(text, regex, replacement)?))
                }),
                "정규나누기" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("정규나누기는 인자 2개를 받습니다".to_string().into());
                    }
                    let Value::String(text) = &args[0] else {
                        return Err("정규나누기 첫 인자는 글이어야 합니다".to_string().into());
                    };
                    let Value::Regex(regex) = &args[1] else {
                        return Err("정규나누기 둘째 인자는 정규식이어야 합니다"
                            .to_string()
                            .into());
                    };
                    Ok(Value::List(
                        regex_split(text, regex)?
                            .into_iter()
                            .map(Value::String)
                            .collect(),
                    ))
                }),
                "첫번째" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("첫번째는 인자 1개를 받습니다".to_string().into());
                    }
                    let Value::List(items) = &args[0] else {
                        return Err("첫번째는 차림 인자를 받습니다".to_string().into());
                    };
                    if !items.is_empty() {
                        self.work_cost.borrow_mut().charge_element_visits(1)?;
                    }
                    Ok(items.first().cloned().unwrap_or(Value::None))
                }),
                "마지막" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("마지막은 인자 1개를 받습니다".to_string().into());
                    }
                    let Value::List(items) = &args[0] else {
                        return Err("마지막은 차림 인자를 받습니다".to_string().into());
                    };
                    if !items.is_empty() {
                        self.work_cost.borrow_mut().charge_element_visits(1)?;
                    }
                    Ok(items.last().cloned().unwrap_or(Value::None))
                }),
                "추가" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("추가는 인자 2개를 받습니다".to_string().into());
                    }
                    let Value::List(items) = &args[0] else {
                        return Err("추가는 차림 인자를 받습니다".to_string().into());
                    };
                    self.work_cost
                        .borrow_mut()
                        .charge_element_visits(items.len())?;
                    // 순서 있는 차림 복사는 장면의 비교/열쇠 사용이 아니다.
                    // 실제 복제되는 불투명 장면 내부의 정점도 계량한다.
                    for value in items.iter().chain(std::iter::once(&args[1])) {
                        self.work_cost.borrow_mut().charge_element_visits(crate::scene_value::clone_visits(value))?;
                    }
                    list_add(&args[0], args[1].clone()).map_err(runtime_error)
                }),
                "제거" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("제거는 인자 2개를 받습니다".to_string().into());
                    }
                    let _ = value_to_index(&args[1])?;
                    let Value::List(items) = &args[0] else {
                        return Err("제거는 차림 인자를 받습니다".to_string().into());
                    };
                    self.work_cost
                        .borrow_mut()
                        .charge_element_visits(items.len())?;
                    list_remove(&args[0], &args[1]).map_err(runtime_error)
                }),
                "정렬" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("정렬은 인자 2개를 받습니다".to_string().into());
                    }
                    let Value::List(items) = &args[0] else {
                        return Err("정렬은 차림 인자를 받습니다".to_string().into());
                    };
                    let func = args[1].clone();
                    let mut keyed = Vec::with_capacity(items.len());
                    for (idx, item) in items.iter().cloned().enumerate() {
                        self.work_cost.borrow_mut().charge_element_visits(1)?;
                        let key = self.eval_callable(
                            locals,
                            &func,
                            vec![item.clone()],
                            source_span.clone(),
                        )?;
                        keyed.push((idx, key, item));
                    }
                    keyed.sort_by(|a, b| value_cmp(&a.1, &b.1).then_with(|| a.0.cmp(&b.0)));
                    let items = keyed.into_iter().map(|(_, _, item)| item).collect();
                    Ok(Value::List(items))
                }),
                "거르기" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("거르기는 인자 2개를 받습니다".to_string().into());
                    }
                    let Value::List(items) = &args[0] else {
                        return Err("거르기는 차림 인자를 받습니다".to_string().into());
                    };
                    let func = args[1].clone();
                    let mut out = Vec::new();
                    for item in items.iter().cloned() {
                        self.work_cost.borrow_mut().charge_element_visits(1)?;
                        let verdict = self.eval_callable(
                            locals,
                            &func,
                            vec![item.clone()],
                            source_span.clone(),
                        )?;
                        if is_truthy(&verdict)? {
                            out.push(item);
                        }
                    }
                    Ok(Value::List(out))
                }),
                "변환" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("변환은 인자 2개를 받습니다".to_string().into());
                    }
                    let Value::List(items) = &args[0] else {
                        return Err("변환은 차림 인자를 받습니다".to_string().into());
                    };
                    let func = args[1].clone();
                    let mut out = Vec::with_capacity(items.len());
                    for item in items.iter().cloned() {
                        self.work_cost.borrow_mut().charge_element_visits(1)?;
                        let mapped =
                            self.eval_callable(locals, &func, vec![item], source_span.clone())?;
                        out.push(mapped);
                    }
                    Ok(Value::List(out))
                }),
                "바꾸기" => {
                    if args.len() == 3 {
                        let Value::String(text) = &args[0] else {
                            return Err("바꾸기 첫 인자는 글이어야 합니다".to_string().into());
                        };
                        let Value::String(from) = &args[1] else {
                            return Err("바꾸기 둘째 인자는 글이어야 합니다".to_string().into());
                        };
                        let Value::String(to) = &args[2] else {
                            return Err("바꾸기 셋째 인자는 글이어야 합니다".to_string().into());
                        };
                        return Ok(Value::String(text.replace(from, to)));
                    }
                    if args.len() != 2 {
                        return Err("바꾸기는 인자 2개 또는 3개를 받습니다".to_string().into());
                    }
                    let Value::List(items) = &args[0] else {
                        return Err("바꾸기는 차림 인자를 받습니다".to_string().into());
                    };
                    let func = args[1].clone();
                    let mut out = Vec::with_capacity(items.len());
                    for item in items.iter().cloned() {
                        self.work_cost.borrow_mut().charge_element_visits(1)?;
                        let mapped =
                            self.eval_callable(locals, &func, vec![item], source_span.clone())?;
                        out.push(mapped);
                    }
                    Ok(Value::List(out))
                }
                "토막내기" => in_call_arm_frame(|| {
                    if args.len() != 3 {
                        return Err("토막내기는 인자 3개를 받습니다".to_string().into());
                    }
                    let Value::List(items) = &args[0] else {
                        return Err("토막내기는 차림 인자를 받습니다".to_string().into());
                    };
                    let start = value_to_index(&args[1])?;
                    let end = value_to_index(&args[2])?;
                    if start>end || end>items.len() {
                        return Err("E_SELECTION_BOUNDS: 토막내기 범위가 차림을 벗어나거나 뒤집혔습니다".to_string().into());
                    }
                    self.work_cost
                        .borrow_mut()
                        .charge_element_visits(end - start)?;
                    Ok(Value::List(items[start..end].to_vec()))
                }),
                "들어있나" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("들어있나는 인자 2개를 받습니다".to_string().into());
                    }
                    let Value::List(items) = &args[0] else {
                        return Err("들어있나는 차림 인자를 받습니다".to_string().into());
                    };
                    let target = &args[1];
                    let mut exists = false;
                    for item in items {
                        self.work_cost.borrow_mut().charge_element_visits(1)?;
                        if self.eval_collection_search_equality(item, target)? {
                            exists = true;
                            break;
                        }
                    }
                    Ok(Value::Bool(exists))
                }),
                "찾아보기" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("찾아보기는 인자 2개를 받습니다".to_string().into());
                    }
                    let Value::List(items) = &args[0] else {
                        return Err("찾아보기는 차림 인자를 받습니다".to_string().into());
                    };
                    let target = &args[1];
                    let mut idx = -1;
                    for (index, item) in items.iter().enumerate() {
                        self.work_cost.borrow_mut().charge_element_visits(1)?;
                        if self.eval_collection_search_equality(item, target)? {
                            idx = index as i64;
                            break;
                        }
                    }
                    Ok(Value::Fixed64(Fixed64::from_i64(idx)))
                }),
                "뒤집기" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("뒤집기는 인자 1개를 받습니다".to_string().into());
                    }
                    let Value::List(items) = &args[0] else {
                        return Err("뒤집기는 차림 인자를 받습니다".to_string().into());
                    };
                    self.work_cost
                        .borrow_mut()
                        .charge_element_visits(items.len())?;
                    let mut out = items.clone();
                    out.reverse();
                    Ok(Value::List(out))
                }),
                "펼치기" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("펼치기는 인자 1개를 받습니다".to_string().into());
                    }
                    let Value::List(items) = &args[0] else {
                        return Err("펼치기는 차림 인자를 받습니다".to_string().into());
                    };
                    let mut out = Vec::new();
                    for item in items {
                        self.work_cost.borrow_mut().charge_element_visits(1)?;
                        let Value::List(inner) = item else {
                            return Err("펼치기는 차림<차림> 인자를 받습니다".to_string().into());
                        };
                        self.work_cost
                            .borrow_mut()
                            .charge_element_visits(inner.len())?;
                        out.extend(inner.clone());
                    }
                    Ok(Value::List(out))
                }),
                "각각돌며" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("각각돌며는 인자 2개를 받습니다".to_string().into());
                    }
                    let Value::List(items) = &args[0] else {
                        return Err("각각돌며는 차림 인자를 받습니다".to_string().into());
                    };
                    let func = args[1].clone();
                    for item in items.iter().cloned() {
                        self.work_cost.borrow_mut().charge_element_visits(1)?;
                        let _ =
                            self.eval_callable(locals, &func, vec![item], source_span.clone())?;
                    }
                    Ok(Value::None)
                }),
                "자르기" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("자르기는 인자 2개를 받습니다".to_string().into());
                    }
                    string_split(&args[0], &args[1]).map_err(runtime_error)
                }),
                "합치기" => {
                    if args.len() == 2 {
                        return match (&args[0], &args[1]) {
                            (Value::String(_), Value::String(_)) => {
                                let Value::String(left) = &args[0] else {
                                    unreachable!();
                                };
                                let Value::String(right) = &args[1] else {
                                    unreachable!();
                                };
                                self.work_cost
                                    .borrow_mut()
                                    .charge_canonical_utf8_bytes(left)?;
                                self.work_cost
                                    .borrow_mut()
                                    .charge_canonical_utf8_bytes(right)?;
                                string_concat(&args[0], &args[1]).map_err(runtime_error)
                            }
                            (Value::List(_), Value::String(_)) => {
                                if let Value::List(items) = &args[0] {
                                    let Value::String(delimiter) = &args[1] else {
                                        unreachable!();
                                    };
                                    for (index, item) in items.iter().enumerate() {
                                        self.work_cost.borrow_mut().charge_element_visits(1)?;
                                        let Value::String(text) = item else {
                                            return Err(runtime_error(
                                                RuntimeError::TypeMismatch {
                                                    expected: "차림<글>",
                                                },
                                            ));
                                        };
                                        if index > 0 {
                                            self.work_cost
                                                .borrow_mut()
                                                .charge_canonical_utf8_bytes(delimiter)?;
                                        }
                                        self.work_cost
                                            .borrow_mut()
                                            .charge_canonical_utf8_bytes(text)?;
                                    }
                                }
                                string_join(&args[0], &args[1]).map_err(runtime_error)
                            }
                            (Value::Pack(left), Value::Pack(right)) => {
                                self.work_cost
                                    .borrow_mut()
                                    .charge_element_visits(left.len())?;
                                let mut merged = left.clone();
                                for (key, value) in right {
                                    self.work_cost.borrow_mut().charge_element_visits(1)?;
                                    merged.insert(key.clone(), value.clone());
                                }
                                Ok(Value::Pack(merged))
                            }
                            _ => Err("합치기는 글/차림/묶음 조합만 지원합니다".to_string().into()),
                        };
                    }
                    if args.len() != 3 {
                        return Err("합치기는 인자 2개 또는 3개를 받습니다".to_string().into());
                    }
                    let Value::List(items) = &args[0] else {
                        return Err("합치기는 차림 인자를 받습니다".to_string().into());
                    };
                    let func = args[2].clone();
                    let mut acc = args[1].clone();
                    for item in items.iter().cloned() {
                        self.work_cost.borrow_mut().charge_element_visits(1)?;
                        acc = self.eval_callable(
                            locals,
                            &func,
                            vec![acc, item],
                            source_span.clone(),
                        )?;
                    }
                    Ok(acc)
                }
                "붙이기" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("붙이기는 인자 2개를 받습니다".to_string().into());
                    }
                    match (&args[0], &args[1]) {
                        (Value::List(items), Value::String(delimiter)) => {
                            for (index, item) in items.iter().enumerate() {
                                self.work_cost.borrow_mut().charge_element_visits(1)?;
                                let Value::String(text) = item else {
                                    return Err(runtime_error(RuntimeError::TypeMismatch {
                                        expected: "차림<글>",
                                    }));
                                };
                                if index > 0 {
                                    self.work_cost
                                        .borrow_mut()
                                        .charge_canonical_utf8_bytes(delimiter)?;
                                }
                                self.work_cost
                                    .borrow_mut()
                                    .charge_canonical_utf8_bytes(text)?;
                            }
                            string_join(&args[0], &args[1]).map_err(runtime_error)
                        }
                        (Value::List(left), Value::List(right)) => {
                            self.work_cost
                                .borrow_mut()
                                .charge_element_visits(left.len())?;
                            self.work_cost
                                .borrow_mut()
                                .charge_element_visits(right.len())?;
                            let mut out = left.clone();
                            out.extend(right.clone());
                            Ok(Value::List(out))
                        }
                        _ => Err("붙이기는 차림/글 조합 또는 차림/차림 조합만 지원합니다"
                            .to_string()
                            .into()),
                    }
                }),
                "포함하나" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("포함하나는 인자 2개를 받습니다".to_string().into());
                    }
                    string_contains(&args[0], &args[1]).map_err(runtime_error)
                }),
                "시작하나" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("시작하나는 인자 2개를 받습니다".to_string().into());
                    }
                    string_starts(&args[0], &args[1]).map_err(runtime_error)
                }),
                "끝나나" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("끝나나는 인자 2개를 받습니다".to_string().into());
                    }
                    string_ends(&args[0], &args[1]).map_err(runtime_error)
                }),
                "숫자로" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("숫자로는 인자 1개를 받습니다".to_string().into());
                    }
                    string_to_number(&args[0]).map_err(runtime_error)
                }),
                "글바꾸기" => {
                    if args.len() != 3 {
                        return Err("글바꾸기는 인자 3개를 받습니다".to_string().into());
                    }
                    let base = match &args[0] {
                        Value::String(s) => s.clone(),
                        _ => return Err("글바꾸기 첫 인자는 글이어야 합니다".to_string().into()),
                    };
                    let idx = value_to_index(&args[1])?;
                    let replacement = match &args[2] {
                        Value::String(s) => s.clone(),
                        _ => return Err("글바꾸기 새글은 글이어야 합니다".to_string().into()),
                    };
                    let mut out = String::new();
                    let mut replaced = false;
                    for (i, ch) in base.chars().enumerate() {
                        if i == idx {
                            out.push_str(&replacement);
                            replaced = true;
                        } else {
                            out.push(ch);
                        }
                    }
                    if !replaced {
                        return Ok(Value::String(base));
                    }
                    Ok(Value::String(out))
                }
                "바닥" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("바닥은 인자 1개를 받습니다".to_string().into());
                    }
                    let qty = unit_value_from_value(&args[0])?;
                    let value = fixed64_floor(qty.value);
                    Ok(unit_value_to_value(UnitValue {
                        value,
                        dim: qty.dim,
                    }))
                }),
                "천장" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("천장은 인자 1개를 받습니다".to_string().into());
                    }
                    let qty = unit_value_from_value(&args[0])?;
                    let value = fixed64_ceil(qty.value)?;
                    Ok(unit_value_to_value(UnitValue {
                        value,
                        dim: qty.dim,
                    }))
                }),
                "반올림" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("반올림은 인자 1개를 받습니다".to_string().into());
                    }
                    let qty = unit_value_from_value(&args[0])?;
                    let value = fixed64_round_even(qty.value)?;
                    Ok(unit_value_to_value(UnitValue {
                        value,
                        dim: qty.dim,
                    }))
                }),
                "합계" => {
                    if args.len() != 1 {
                        return Err("합계는 인자 1개를 받습니다".to_string().into());
                    }
                    let Value::List(items) = &args[0] else {
                        return Err("합계는 차림 인자를 받습니다".to_string().into());
                    };
                    if items.is_empty() {
                        return Ok(Value::Fixed64(Fixed64::from_i64(0)));
                    }
                    self.work_cost.borrow_mut().charge_element_visits(1)?;
                    let mut total = unit_value_from_value(&items[0])?;
                    for item in items.iter().skip(1) {
                        self.work_cost.borrow_mut().charge_element_visits(1)?;
                        let next = unit_value_from_value(item)?;
                        total = checked_unit_add(total, next)?;
                    }
                    Ok(unit_value_to_value(total))
                }
                "평균" => {
                    if args.len() != 1 {
                        return Err("평균은 인자 1개를 받습니다".to_string().into());
                    }
                    let Value::List(items) = &args[0] else {
                        return Err("평균은 차림 인자를 받습니다".to_string().into());
                    };
                    if items.is_empty() {
                        return Ok(Value::None);
                    }
                    self.work_cost.borrow_mut().charge_element_visits(1)?;
                    let mut total = unit_value_from_value(&items[0])?;
                    for item in items.iter().skip(1) {
                        self.work_cost.borrow_mut().charge_element_visits(1)?;
                        let next = unit_value_from_value(item)?;
                        total = checked_unit_add(total, next)?;
                    }
                    let count = Fixed64::from_i64(items.len() as i64);
                    let avg = total.div_scalar(count).map_err(unit_error)?;
                    Ok(unit_value_to_value(avg))
                }
                "지니" | "지니계수" => {
                    let values = expect_numeric_list_arg(&args, "지니", self.work_cost.as_ref())?;
                    if values.is_empty() {
                        return Ok(Value::None);
                    }
                    let mut total = Fixed64::ZERO;
                    for value in &values {
                        if value.value.raw_i64() < 0 {
                            return Err("지니 입력은 음수가 될 수 없습니다".to_string().into());
                        }
                        total = total.checked_add(value.value)?;
                    }
                    if total.raw_i64() == 0 {
                        return Ok(Value::Fixed64(Fixed64::ZERO));
                    }
                    let mut pair_sum = Fixed64::ZERO;
                    for i in 0..values.len() {
                        for j in (i + 1)..values.len() {
                            let diff = values[i].value.checked_sub(values[j].value)?;
                            pair_sum = pair_sum.checked_add(fixed64_abs(diff)?)?;
                        }
                    }
                    let count = Fixed64::from_i64(values.len() as i64);
                    let denom = count.checked_mul(total)?;
                    let gini = pair_sum.try_div(denom)?;
                    Ok(Value::Fixed64(gini))
                }
                "분위수" | "백분위수" => {
                    let (mut values, p, mode) =
                        expect_quantile_args(&args, self.work_cost.as_ref())?;
                    if values.is_empty() {
                        return Ok(Value::None);
                    }
                    if values.len() == 1 {
                        return Ok(unit_value_to_value(values[0]));
                    }
                    values.sort_by(|a, b| a.value.raw_i64().cmp(&b.value.raw_i64()));
                    match mode {
                        PercentileMode::Linear => {
                            let max_index = Fixed64::from_i64((values.len() - 1) as i64);
                            let pos = p.checked_mul(max_index)?;
                            let low = fixed64_floor(pos);
                            let high = fixed64_ceil(pos)?;
                            let low_idx = fixed64_to_nonnegative_index(low)?;
                            let high_idx = fixed64_to_nonnegative_index(high)?;
                            if low_idx >= values.len() || high_idx >= values.len() {
                                return Err("분위수 인덱스가 범위를 벗어났습니다"
                                    .to_string()
                                    .into());
                            }
                            if low_idx == high_idx {
                                return Ok(unit_value_to_value(values[low_idx]));
                            }
                            let frac = pos.checked_sub(low)?;
                            let base = values[low_idx];
                            let next = values[high_idx];
                            let delta = next.value.checked_sub(base.value)?;
                            let step = delta.checked_mul(frac)?;
                            let interpolated = UnitValue {
                                value: base.value.checked_add(step)?,
                                dim: base.dim,
                            };
                            Ok(unit_value_to_value(interpolated))
                        }
                        PercentileMode::NearestRank => {
                            let idx = nearest_rank_index(p, values.len())?;
                            Ok(unit_value_to_value(values[idx]))
                        }
                    }
                }
                "적분.오일러" => in_call_arm_frame(|| {
                    if args.len() != 3 {
                        return Err("적분.오일러는 인자 3개를 받습니다".to_string().into());
                    }
                    let value = unit_value_from_value(&args[0])?;
                    let rate = unit_value_from_value(&args[1])?;
                    let dt = unit_value_from_value(&args[2])?;
                    let delta = checked_unit_mul(rate, dt)?;
                    let out = checked_unit_add(value, delta)?;
                    Ok(unit_value_to_value(out))
                }),
                "적분.반암시적오일러" => in_call_arm_frame(|| {
                    if args.len() != 4 {
                        return Err("적분.반암시적오일러는 인자 4개를 받습니다"
                            .to_string()
                            .into());
                    }
                    let position = unit_value_from_value(&args[0])?;
                    let velocity = unit_value_from_value(&args[1])?;
                    let acceleration = unit_value_from_value(&args[2])?;
                    let dt = unit_value_from_value(&args[3])?;
                    let next_velocity = checked_unit_add(velocity, checked_unit_mul(acceleration, dt)?)?;
                    let next_position = checked_unit_add(position, checked_unit_mul(next_velocity, dt)?)?;
                    Ok(Value::List(vec![
                        unit_value_to_value(next_position),
                        unit_value_to_value(next_velocity),
                    ]))
                }),
                "보간.선형" => in_call_arm_frame(|| {
                    if args.len() != 3 {
                        return Err("보간.선형은 인자 3개를 받습니다".to_string().into());
                    }
                    let start = unit_value_from_value(&args[0])?;
                    let end = unit_value_from_value(&args[1])?;
                    let t = unit_value_from_value(&args[2])?;
                    if !t.is_dimensionless() {
                        return Err(unit_error(UnitError::DimensionMismatch {
                            left: t.dim,
                            right: UnitDim::NONE,
                        }));
                    }
                    let delta = checked_unit_sub(end, start)?;
                    let out = checked_unit_add(start, checked_unit_mul(delta, t)?)?;
                    Ok(unit_value_to_value(out))
                }),
                "보간.계단" => in_call_arm_frame(|| {
                    if args.len() != 4 {
                        return Err("보간.계단은 인자 4개를 받습니다".to_string().into());
                    }
                    let start = unit_value_from_value(&args[0])?;
                    let end = unit_value_from_value(&args[1])?;
                    if start.dim != end.dim {
                        return Err(unit_error(UnitError::DimensionMismatch {
                            left: start.dim,
                            right: end.dim,
                        }));
                    }
                    let t = unit_value_from_value(&args[2])?;
                    if !t.is_dimensionless() {
                        return Err(unit_error(UnitError::DimensionMismatch {
                            left: t.dim,
                            right: UnitDim::NONE,
                        }));
                    }
                    let threshold = unit_value_from_value(&args[3])?;
                    if !threshold.is_dimensionless() {
                        return Err(unit_error(UnitError::DimensionMismatch {
                            left: threshold.dim,
                            right: UnitDim::NONE,
                        }));
                    }
                    if t.value < threshold.value {
                        Ok(unit_value_to_value(start))
                    } else {
                        Ok(unit_value_to_value(end))
                    }
                }),
                "필터.이동평균" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("필터.이동평균은 인자 2개를 받습니다".to_string().into());
                    }
                    let Value::List(window) = &args[0] else {
                        return Err("필터.이동평균 첫 인자는 차림이어야 합니다"
                            .to_string()
                            .into());
                    };
                    let next = unit_value_from_value(&args[1])?;
                    let mut next_window = Vec::with_capacity(window.len() + 1);
                    let mut total = next.value;
                    for item in window {
                        let value = unit_value_from_value(item)?;
                        if value.dim != next.dim {
                            return Err(unit_error(UnitError::DimensionMismatch {
                                left: next.dim,
                                right: value.dim,
                            }));
                        }
                        total = total.checked_add(value.value)?;
                        next_window.push(unit_value_to_value(value));
                    }
                    next_window.push(unit_value_to_value(next));
                    let count = Fixed64::from_i64(next_window.len() as i64);
                    let avg = total
                        .try_div(count)
                        .map_err(|_| "필터.이동평균 계산 중 0으로 나눌 수 없습니다".to_string())?;
                    Ok(Value::List(vec![
                        Value::List(next_window),
                        unit_value_to_value(UnitValue {
                            value: avg,
                            dim: next.dim,
                        }),
                    ]))
                }),
                "필터.지수평활" => in_call_arm_frame(|| {
                    if args.len() != 3 {
                        return Err("필터.지수평활은 인자 3개를 받습니다".to_string().into());
                    }
                    let prev = unit_value_from_value(&args[0])?;
                    let next = unit_value_from_value(&args[1])?;
                    if prev.dim != next.dim {
                        return Err(unit_error(UnitError::DimensionMismatch {
                            left: prev.dim,
                            right: next.dim,
                        }));
                    }
                    let alpha = unit_value_from_value(&args[2])?;
                    if !alpha.is_dimensionless() {
                        return Err(unit_error(UnitError::DimensionMismatch {
                            left: alpha.dim,
                            right: UnitDim::NONE,
                        }));
                    }
                    if alpha.value.raw_i64() < 0 || alpha.value.raw_i64() > Fixed64::ONE.raw_i64() {
                        return Err("필터.지수평활 alpha는 0..1 범위여야 합니다"
                            .to_string()
                            .into());
                    }
                    let out = prev.value.checked_weighted_blend(next.value, alpha.value)?;
                    Ok(unit_value_to_value(UnitValue {
                        value: out,
                        dim: prev.dim,
                    }))
                }),
                "미분.중앙차분" => in_call_arm_frame(|| {
                    let (formula, var_name, point, step) =
                        expect_numeric_derivative_args(&args, "미분.중앙차분")?;
                    let prepared = prepare_numeric_formula(&formula, &var_name, "미분.중앙차분")?;
                    let two = Fixed64::from_i64(2);

                    let x_plus = UnitValue {
                        value: point.value.checked_add(step.value)?,
                        dim: point.dim,
                    };
                    let x_minus = UnitValue {
                        value: point.value.checked_sub(step.value)?,
                        dim: point.dim,
                    };
                    let y_plus = eval_numeric_formula_at_metered(
                        &prepared,
                        x_plus,
                        "미분.중앙차분",
                        &self.work_cost,
                    )?;
                    let y_minus = eval_numeric_formula_at_metered(
                        &prepared,
                        x_minus,
                        "미분.중앙차분",
                        &self.work_cost,
                    )?;
                    if y_plus.dim != y_minus.dim {
                        return Err(unit_error(UnitError::DimensionMismatch {
                            left: y_plus.dim,
                            right: y_minus.dim,
                        }));
                    }
                    let denom = step.value.checked_mul(two)?;
                    let d_h = y_plus
                        .value
                        .checked_sub(y_minus.value)?
                        .try_div(denom)?;

                    let step2 = step.value.checked_mul(two)?;
                    let x_plus2 = UnitValue {
                        value: point.value.checked_add(step2)?,
                        dim: point.dim,
                    };
                    let x_minus2 = UnitValue {
                        value: point.value.checked_sub(step2)?,
                        dim: point.dim,
                    };
                    let y_plus2 = eval_numeric_formula_at_metered(
                        &prepared,
                        x_plus2,
                        "미분.중앙차분",
                        &self.work_cost,
                    )?;
                    let y_minus2 = eval_numeric_formula_at_metered(
                        &prepared,
                        x_minus2,
                        "미분.중앙차분",
                        &self.work_cost,
                    )?;
                    if y_plus2.dim != y_minus2.dim {
                        return Err(unit_error(UnitError::DimensionMismatch {
                            left: y_plus2.dim,
                            right: y_minus2.dim,
                        }));
                    }
                    let denom2 = step2.checked_mul(two)?;
                    let d_2h = y_plus2
                        .value
                        .checked_sub(y_minus2.value)?
                        .try_div(denom2)?;
                    let three = Fixed64::from_i64(3);
                    let richardson_delta = d_h.checked_sub(d_2h)?;
                    let richardson_correction = richardson_delta.try_div(three)?;
                    let richardson_value = d_h.checked_add(richardson_correction)?;
                    let richardson_error = fixed64_abs(richardson_delta)?.try_div(three)?;
                    let diff_dim = y_plus.dim.sub(step.dim);
                    Ok(Value::List(vec![
                        unit_value_to_value(UnitValue {
                            value: richardson_value,
                            dim: diff_dim,
                        }),
                        unit_value_to_value(UnitValue {
                            value: richardson_error,
                            dim: diff_dim,
                        }),
                        Value::String("중앙차분".to_string()),
                    ]))
                }),
                "적분.사다리꼴" => in_call_arm_frame(|| {
                    let (formula, var_name, start, end, step) =
                        expect_numeric_integral_args(&args, "적분.사다리꼴")?;
                    let prepared = prepare_numeric_formula(&formula, &var_name, "적분.사다리꼴")?;
                    let (lower, upper, sign) = if end.value < start.value {
                        (end, start, -1_i64)
                    } else {
                        (start, end, 1_i64)
                    };
                    let interval = upper.value.checked_sub(lower.value)?;
                    let n_est = interval.try_div(step.value)?;
                    let mut n = fixed64_to_nonnegative_index(fixed64_ceil(n_est)?)?;
                    if n == 0 {
                        n = 1;
                    }
                    let coarse =
                        numeric_trapezoid_integral(&prepared, lower, upper, n, &self.work_cost)?;
                    let refined = numeric_trapezoid_integral(
                        &prepared,
                        lower,
                        upper,
                        n.saturating_mul(2).max(2),
                        &self.work_cost,
                    )?;
                    if coarse.dim != refined.dim {
                        return Err(unit_error(UnitError::DimensionMismatch {
                            left: coarse.dim,
                            right: refined.dim,
                        }));
                    }
                    let three = Fixed64::from_i64(3);
                    let richardson_delta = refined.value.checked_sub(coarse.value)?;
                    let richardson_correction = richardson_delta.try_div(three)?;
                    let richardson_value = refined.value.checked_add(richardson_correction)?;
                    let richardson_error = fixed64_abs(richardson_delta)?.try_div(three)?;
                    let mut approx = refined;
                    approx.value = richardson_value;
                    if sign < 0 {
                        approx.value = Fixed64::ZERO.checked_sub(approx.value)?;
                    }
                    let err = UnitValue {
                        value: richardson_error,
                        dim: coarse.dim,
                    };
                    Ok(Value::List(vec![
                        unit_value_to_value(approx),
                        unit_value_to_value(err),
                        Value::String("사다리꼴".to_string()),
                    ]))
                }),
                "수치해.이분법" => in_call_arm_frame(|| {
                    let (formula, var_name, lower, upper, iterations) =
                        expect_numeric_bisection_args(&args, "수치해.이분법")?;
                    let prepared = prepare_numeric_formula(&formula, &var_name, "수치해.이분법")?;
                    let (root, residual, used_iterations) = numeric_bisection_root(
                        &prepared,
                        lower,
                        upper,
                        iterations,
                        &self.work_cost,
                    )?;
                    Ok(Value::List(vec![
                        unit_value_to_value(root),
                        unit_value_to_value(residual),
                        unit_value_to_value(UnitValue {
                            value: Fixed64::from_i64(used_iterations as i64),
                            dim: UnitDim::NONE,
                        }),
                        Value::String("이분법".to_string()),
                    ]))
                }),
                "선형부등식.풀기" => in_call_arm_frame(|| eval_linear_inequality_solve(&args, &self.work_cost)),
                "abs" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("abs는 인자 1개를 받습니다".to_string().into());
                    }
                    let qty = unit_value_from_value(&args[0])?;
                    let value = fixed64_abs(qty.value)?;
                    Ok(unit_value_to_value(UnitValue {
                        value,
                        dim: qty.dim,
                    }))
                }),
                "min" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("min은 인자 2개를 받습니다".to_string().into());
                    }
                    let left = unit_value_from_value(&args[0])?;
                    let right = unit_value_from_value(&args[1])?;
                    if left.dim != right.dim {
                        return Err(unit_error(UnitError::DimensionMismatch {
                            left: left.dim,
                            right: right.dim,
                        }));
                    }
                    let out = if left.value <= right.value {
                        left
                    } else {
                        right
                    };
                    Ok(unit_value_to_value(out))
                }),
                "max" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("max는 인자 2개를 받습니다".to_string().into());
                    }
                    let left = unit_value_from_value(&args[0])?;
                    let right = unit_value_from_value(&args[1])?;
                    if left.dim != right.dim {
                        return Err(unit_error(UnitError::DimensionMismatch {
                            left: left.dim,
                            right: right.dim,
                        }));
                    }
                    let out = if left.value >= right.value {
                        left
                    } else {
                        right
                    };
                    Ok(unit_value_to_value(out))
                }),
                "clamp" => in_call_arm_frame(|| {
                    if args.len() != 3 {
                        return Err("clamp는 인자 3개를 받습니다".to_string().into());
                    }
                    let value = unit_value_from_value(&args[0])?;
                    let min = unit_value_from_value(&args[1])?;
                    let max = unit_value_from_value(&args[2])?;
                    if value.dim != min.dim {
                        return Err(unit_error(UnitError::DimensionMismatch {
                            left: value.dim,
                            right: min.dim,
                        }));
                    }
                    if value.dim != max.dim {
                        return Err(unit_error(UnitError::DimensionMismatch {
                            left: value.dim,
                            right: max.dim,
                        }));
                    }
                    let clamped = if value.value < min.value {
                        min
                    } else if value.value > max.value {
                        max
                    } else {
                        value
                    };
                    Ok(unit_value_to_value(clamped))
                }),
                "sqrt" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("sqrt는 인자 1개를 받습니다".to_string().into());
                    }
                    let qty = unit_value_from_value(&args[0])?;
                    if qty.value.raw_i64() < 0 {
                        return Err("음수의 제곱근은 지원하지 않습니다".to_string().into());
                    }
                    let Some(dim) = qty.dim.sqrt() else {
                        return Err("제곱근은 짝수 차원만 지원합니다".to_string().into());
                    };
                    let Some(value) = fixed64_sqrt(qty.value) else {
                        return Err("제곱근 계산 실패".to_string().into());
                    };
                    Ok(unit_value_to_value(UnitValue { value, dim }))
                }),
                "sin" | "cos" | "tan" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err(format!("{}는 인자 1개를 받습니다", func).into());
                    }
                    let qty = unit_value_from_value(&args[0])?;
                    if !(qty.is_dimensionless() || qty.dim == UnitDim::ANGLE) {
                        return Err(unit_error(UnitError::DimensionMismatch {
                            left: qty.dim,
                            right: UnitDim::ANGLE,
                        }));
                    }
                    let angle = fixed64_to_f64(qty.value);
                    let out = match func {
                        "sin" => libm::sin(angle),
                        "cos" => libm::cos(angle),
                        _ => libm::tan(angle),
                    };
                    Ok(Value::Fixed64(fixed64_from_host_f64_checked(out)?))
                }),
                "asin" | "acos" | "atan" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err(format!("{}는 인자 1개를 받습니다", func).into());
                    }
                    let qty = unit_value_from_value(&args[0])?;
                    if !qty.is_dimensionless() {
                        return Err(unit_error(UnitError::DimensionMismatch {
                            left: qty.dim,
                            right: UnitDim::NONE,
                        }));
                    }
                    let val = fixed64_to_f64(qty.value);
                    let out = match func {
                        "asin" => libm::asin(val),
                        "acos" => libm::acos(val),
                        _ => libm::atan(val),
                    };
                    Ok(Value::Fixed64(fixed64_from_host_f64_checked(out)?))
                }),
                "atan2" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("atan2는 인자 2개를 받습니다".to_string().into());
                    }
                    let y = unit_value_from_value(&args[0])?;
                    let x = unit_value_from_value(&args[1])?;
                    let out = libm::atan2(fixed64_to_f64(y.value), fixed64_to_f64(x.value));
                    Ok(Value::Fixed64(fixed64_from_host_f64_checked(out)?))
                }),
                "powi" => {
                    if args.len() != 2 {
                        return Err("powi는 인자 2개를 받습니다".to_string().into());
                    }
                    if numeric_pack_kind(&args[0]) == Some(NUMERIC_KIND_FACTOR) {
                        let Some(base) = exact_numeric_from_value(&args[0])? else {
                            return Err("곱수 값이 필요합니다".to_string().into());
                        };
                        if !base.is_integer() {
                            return Err("곱수 거듭제곱은 정수 값만 지원합니다".to_string().into());
                        }
                        let exp = value_to_i64(&args[1])?;
                        if exp < 0 {
                            return Err("곱수 거듭제곱은 음수 지수를 지원하지 않습니다"
                                .to_string()
                                .into());
                        }
                        let raw = base.value.to_integer().pow(exp as u32);
                        let (value, status) =
                            make_factor_value(&make_big_int_pack_from_bigint(&raw))?;
                        self.note_factor_route_from_value(&value);
                        if status == FACTOR_DECOMP_STATUS_DEFERRED {
                            self.emit_factor_decomposition_deferred_diag(&value, source_span);
                        }
                        return Ok(value);
                    }
                    let base = unit_value_from_value(&args[0])?;
                    let exp = unit_value_from_value(&args[1])?;
                    if !exp.is_dimensionless() {
                        return Err(unit_error(UnitError::DimensionMismatch {
                            left: exp.dim,
                            right: UnitDim::NONE,
                        }));
                    }
                    if exp.value.frac_part() != 0 {
                        return Err("powi는 정수 지수만 지원합니다".to_string().into());
                    }
                    let value = unit_powi(base, exp.value.int_part())?;
                    Ok(unit_value_to_value(value))
                }
                "글로" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("글로는 인자 1개를 받습니다".to_string().into());
                    }
                    Ok(Value::String(value_to_string(&args[0])))
                }),
                "자원" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("자원은 인자 1개를 받습니다".to_string().into());
                    }
                    match &args[0] {
                        Value::String(path) => {
                            Ok(Value::ResourceHandle(ResourceHandle::from_path(path)))
                        }
                        Value::ResourceHandle(handle) => Ok(Value::ResourceHandle(*handle)),
                        _ => Err("자원은 글 인자를 받습니다".to_string().into()),
                    }
                }),
                "자료.열기" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("E_DATASET_OPEN_ARITY: 자료.열기는 자원핸들 1개를 받습니다"
                            .to_string()
                            .into());
                    }
                    let handle = match args.first() {
                        Some(Value::ResourceHandle(handle)) => *handle,
                        _ => {
                            return Err(
                                "E_DATASET_OPEN_TYPE: 자료.열기는 검증된 자원핸들만 받습니다"
                                    .to_string()
                                    .into(),
                            )
                        }
                    };
                    let opened = match self.project_asset_registry {
                        Some(registry) => registry.open_sealed_dataset(handle),
                        None => gate0_registry::open_sealed_dataset(handle),
                    }
                    .map_err(EvalError::from)?;
                    Ok(sealed_dataset_runtime_value(&opened))
                }),
                "눌렸나" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("눌렸나는 인자 1개를 받습니다".to_string().into());
                    }
                    let key = value_to_string(&args[0]);
                    Ok(input_pressed(&self.input, &key))
                }),
                "막눌렸나" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("막눌렸나는 인자 1개를 받습니다".to_string().into());
                    }
                    let key = value_to_string(&args[0]);
                    Ok(match self.control_pressed {
                        Some(pressed) => input_pressed(&InputState::new(pressed, 0), &key),
                        None => input_just_pressed(&self.input, &key),
                    })
                }),
                "입력사상.만들기" => {
                    if self.logical_input.is_some() && !args.is_empty() {
                        return Err("E_LOGICAL_INPUT_PHYSICAL_UNSUPPORTED: 키 사상은 호스트에서 결속합니다".to_string().into());
                    }
                    if args.is_empty() {
                        return Ok(make_std_input_map(BTreeMap::new()));
                    }
                    if args.len() != 1 {
                        return Err("입력사상.만들기는 인자 0개 또는 1개를 받습니다"
                            .to_string()
                            .into());
                    }
                    let Value::Pack(fields) = &args[0] else {
                        return Err("입력사상.만들기는 묶음 인자를 받습니다".to_string().into());
                    };
                    Ok(make_std_input_map(fields.clone()))
                }
                "입력사상.방향" => in_call_arm_frame(|| {
                    if self.logical_input.is_some() {
                        return Err("E_LOGICAL_INPUT_PHYSICAL_UNSUPPORTED: 방향 키 사상 대신 결속된 행동을 조회하세요".to_string().into());
                    }
                    if args.len() != 1 {
                        return Err("입력사상.방향은 인자 1개를 받습니다".to_string().into());
                    }
                    let map = std_input_map_fields(&args[0])?;
                    let left = input_map_action_pressed(&self.input, &map, "왼쪽") as i64;
                    let right = input_map_action_pressed(&self.input, &map, "오른쪽") as i64;
                    let up = input_map_action_pressed(&self.input, &map, "위") as i64;
                    let down = input_map_action_pressed(&self.input, &map, "아래") as i64;
                    Ok(Value::List(vec![
                        fixed_value(right - left),
                        fixed_value(down - up),
                    ]))
                }),
                "입력사상.동작" => {
                    if args.len() != 2 {
                        return Err("입력사상.동작은 인자 2개를 받습니다".to_string().into());
                    }
                    let map = std_input_map_fields(&args[0])?;
                    let action = value_to_string(&args[1]);
                    if let Some(input) = &self.logical_input {
                        if map.keys().any(|key| key != "__kind") {
                            return Err("E_LOGICAL_INPUT_PHYSICAL_UNSUPPORTED: 논리 조회에 키 사상을 넘길 수 없습니다".to_string().into());
                        }
                        self.input_reads.borrow_mut().insert(format!("logical.held:{action}"));
                        return input.is_held(&action).map(Value::Bool)
                            .map_err(|_| "E_LOGICAL_INPUT_CHANNEL: 입력 계약에 없는 조작 채널입니다".to_string().into());
                    }
                    Ok(Value::Bool(input_map_action_pressed(
                        &self.input,
                        &map,
                        &action,
                    )))
                }
                "입력사상.알림결속" => {
                    if self.logical_input.is_none() {
                        return Err("E_LOGICAL_INPUT_PROFILE_REQUIRED: 알림 결속에는 논리 입력 프로필이 필요합니다".to_string().into());
                    }
                    if args.len()!=3 || args.iter().any(|value|!matches!(value,Value::String(_))) {
                        return Err("E_LOGICAL_REACTION_BINDING: 채널·알림씨·받는이 이름을 글로 명시하세요".to_string().into());
                    }
                    if !self.root_program_active {return Err("E_LOGICAL_REACTION_BINDING: 알림 결속은 최상위에서만 선언합니다".to_string().into());}
                    let direct = self.program.root_stmts.iter().any(|stmt| {
                        let Stmt::Expr {expr,..}=stmt else {return false};
                        matches!(&expr.kind, ExprKind::Call {func,..} if func=="입력사상.알림결속")
                            && source_span.is_some() && self.source_span_for_expr(expr)==source_span
                    });
                    if !direct {return Err("E_LOGICAL_REACTION_BINDING: 알림 결속은 최상위의 직접 선언이어야 합니다".to_string().into());}
                    Ok(Value::None)
                }
                "입력사상.사건들" => in_call_arm_frame(|| {
                    if !args.is_empty() {
                        return Err("입력사상.사건들은 인자 없이 해당 마디의 사건을 읽습니다".to_string().into());
                    }
                    let input = self.logical_input.as_ref().ok_or_else(||
                        EvalError::from("E_LOGICAL_INPUT_PROFILE_REQUIRED: 사건 조회에는 논리 입력 프로필이 필요합니다".to_string()))?;
                    self.input_reads.borrow_mut().insert("logical.events".into());
                    self.work_cost.borrow_mut().charge_element_visits(input.events().len())?;
                    let mut values = Vec::with_capacity(input.events().len());
                    for event in input.events() {
                        let kind = match event.kind {
                            ddonirang_core::input::LogicalInputEventKind::Press => "누름",
                            ddonirang_core::input::LogicalInputEventKind::Repeat => "반복",
                            ddonirang_core::input::LogicalInputEventKind::Release => "뗌",
                            ddonirang_core::input::LogicalInputEventKind::Number(..) => "수",
                            ddonirang_core::input::LogicalInputEventKind::Coordinate(..) => "좌표",
                            _ => return Err("E_LOGICAL_INPUT_TEXT_UNSUPPORTED: 알파2는 글자 입력을 지원하지 않습니다".to_string().into()),
                        };
                        let ordinal = i64::try_from(event.receive_ordinal).map_err(|_|
                            EvalError::from("E_LOGICAL_INPUT_ORDER: 사건 순서가 범위를 넘었습니다".to_string()))?;
                        let mut fields = BTreeMap::from([
                            ("채널".into(), Value::String(event.channel.clone())),
                            ("종류".into(), Value::String(kind.into())),
                            ("순서".into(), fixed_value(ordinal)),
                        ]);
                        match &event.kind {
                            ddonirang_core::input::LogicalInputEventKind::Number(raw,phase) => {
                                fields.insert("수값".into(), Value::Fixed64(Fixed64::from_raw_i64(*raw)));
                                fields.insert("단계".into(), Value::String(manipulation_phase_text(*phase).into()));
                            }
                            ddonirang_core::input::LogicalInputEventKind::Coordinate(raw,phase) => {
                                fields.insert("좌표값".into(), Value::List(raw.iter().map(|value|
                                    Value::Fixed64(Fixed64::from_raw_i64(*value))).collect()));
                                fields.insert("단계".into(), Value::String(manipulation_phase_text(*phase).into()));
                            }
                            _ => {}
                        }
                        values.push(Value::Pack(fields));
                    }
                    Ok(Value::List(values))
                }),
                "무작위" => in_call_arm_frame(|| {
                    if !args.is_empty() {
                        return Err("무작위는 인자 0개를 받습니다".to_string().into());
                    }
                    Ok(Value::Fixed64(self.next_rng_fixed64()))
                }),
                "무작위정수" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("무작위정수는 인자 2개를 받습니다".to_string().into());
                    }
                    let min = value_to_i64(&args[0])?;
                    let max = value_to_i64(&args[1])?;
                    if min > max {
                        return Err("무작위정수의 최소/최대가 올바르지 않습니다"
                            .to_string()
                            .into());
                    }
                    let range = (max - min + 1) as u64;
                    let value = (self.next_rng_u64() % range) as i64 + min;
                    Ok(Value::Fixed64(Fixed64::from_i64(value)))
                }),
                "무작위선택" => {
                    if args.len() != 1 {
                        return Err("무작위선택은 인자 1개를 받습니다".to_string().into());
                    }
                    let Value::List(items) = &args[0] else {
                        return Err("무작위선택은 차림 인자를 받습니다".to_string().into());
                    };
                    if items.is_empty() {
                        return Ok(Value::None);
                    }
                    let idx = (self.next_rng_u64() % items.len() as u64) as usize;
                    Ok(items[idx].clone())
                }
                "무작위가방.만들기" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("무작위가방.만들기는 인자 2개를 받습니다".to_string().into());
                    }
                    let seed = value_to_i64(&args[0])? as u64;
                    let Value::List(candidates) = &args[1] else {
                        return Err("무작위가방.만들기는 후보 차림을 받습니다"
                            .to_string()
                            .into());
                    };
                    if candidates.is_empty() {
                        return Err("무작위가방 후보들은 비어 있을 수 없습니다"
                            .to_string()
                            .into());
                    }
                    Ok(make_std_random_bag(
                        seed,
                        seed,
                        candidates.clone(),
                        candidates.clone(),
                        0,
                    ))
                }),
                "무작위가방.꺼내기" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("무작위가방.꺼내기는 인자 1개를 받습니다".to_string().into());
                    }
                    let (value, bag) = std_random_bag_draw_once(&args[0])?;
                    let mut fields = BTreeMap::new();
                    fields.insert(
                        STD_RANDOM_BAG_KIND_FIELD.to_string(),
                        Value::String(STD_RANDOM_BAG_DRAW_KIND.to_string()),
                    );
                    fields.insert(STD_RANDOM_BAG_VALUE_FIELD.to_string(), value);
                    fields.insert(STD_RANDOM_BAG_BAG_FIELD.to_string(), bag);
                    Ok(Value::Pack(fields))
                }),
                "무작위가방.미리보기" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("무작위가방.미리보기는 인자 2개를 받습니다"
                            .to_string()
                            .into());
                    }
                    let count = value_to_i64(&args[1])?;
                    if count < 0 {
                        return Err("무작위가방.미리보기 개수는 0 이상이어야 합니다"
                            .to_string()
                            .into());
                    }
                    let mut bag = args[0].clone();
                    let mut values = Vec::new();
                    for _ in 0..count {
                        let (value, next_bag) = std_random_bag_draw_once(&bag)?;
                        values.push(value);
                        bag = next_bag;
                    }
                    Ok(Value::List(values))
                }),
                "무작위가방.남은것" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("무작위가방.남은것은 인자 1개를 받습니다".to_string().into());
                    }
                    let fields = std_random_bag_fields(&args[0])?;
                    Ok(Value::List(std_random_bag_list_field(
                        fields,
                        STD_RANDOM_BAG_REMAINING_FIELD,
                    )?))
                }),
                "무작위가방.비었나" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("무작위가방.비었나는 인자 1개를 받습니다".to_string().into());
                    }
                    let fields = std_random_bag_fields(&args[0])?;
                    Ok(Value::Bool(
                        std_random_bag_list_field(fields, STD_RANDOM_BAG_REMAINING_FIELD)?
                            .is_empty(),
                    ))
                }),
                "격자게임상태.초기화" => in_call_arm_frame(|| {
                    if !args.is_empty() {
                        return Err("격자게임상태.초기화는 인자 0개를 받습니다"
                            .to_string()
                            .into());
                    }
                    Ok(make_std_grid_game_state("준비", Value::None, 0))
                }),
                "격자게임상태.만들기" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("격자게임상태.만들기는 인자 1개를 받습니다"
                            .to_string()
                            .into());
                    }
                    let state = value_to_grid_game_state_name(&args[0])?;
                    Ok(make_std_grid_game_state(&state, Value::None, 0))
                }),
                "격자게임상태.상태" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("격자게임상태.상태는 인자 1개를 받습니다".to_string().into());
                    }
                    Ok(Value::String(std_grid_game_state_state(
                        std_grid_game_state_fields(&args[0])?,
                    )?))
                }),
                "격자게임상태.틱" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("격자게임상태.틱은 인자 1개를 받습니다".to_string().into());
                    }
                    Ok(fixed_value(std_grid_game_state_tick(
                        std_grid_game_state_fields(&args[0])?,
                    )?))
                }),
                "격자게임상태.상태인가" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("격자게임상태.상태인가는 인자 2개를 받습니다"
                            .to_string()
                            .into());
                    }
                    let current = std_grid_game_state_state(std_grid_game_state_fields(&args[0])?)?;
                    let expected = value_to_grid_game_state_name(&args[1])?;
                    Ok(Value::Bool(current == expected))
                }),
                "격자게임상태.바꾸기" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("격자게임상태.바꾸기는 인자 2개를 받습니다"
                            .to_string()
                            .into());
                    }
                    let fields = std_grid_game_state_fields(&args[0])?;
                    let state = value_to_grid_game_state_name(&args[1])?;
                    Ok(make_std_grid_game_state(
                        &state,
                        Value::String(std_grid_game_state_state(fields)?),
                        std_grid_game_state_tick(fields)?.saturating_add(1),
                    ))
                }),
                "격자게임상태.멈춤" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("격자게임상태.멈춤은 인자 1개를 받습니다".to_string().into());
                    }
                    let fields = std_grid_game_state_fields(&args[0])?;
                    let state = std_grid_game_state_state(fields)?;
                    let previous = if state == "멈춤" {
                        std_grid_game_state_previous(fields)
                    } else {
                        Value::String(state)
                    };
                    Ok(make_std_grid_game_state(
                        "멈춤",
                        previous,
                        std_grid_game_state_tick(fields)?.saturating_add(1),
                    ))
                }),
                "격자게임상태.재개" => {
                    if args.len() != 1 {
                        return Err("격자게임상태.재개는 인자 1개를 받습니다".to_string().into());
                    }
                    let fields = std_grid_game_state_fields(&args[0])?;
                    let state = std_grid_game_state_state(fields)?;
                    if state != "멈춤" {
                        return Ok(args[0].clone());
                    }
                    let previous_value = std_grid_game_state_previous(fields);
                    let previous = match &previous_value {
                        Value::String(candidate)
                            if is_allowed_grid_game_state(&candidate) && candidate != "멈춤" =>
                        {
                            candidate.clone()
                        }
                        _ => "진행".to_string(),
                    };
                    Ok(make_std_grid_game_state(
                        &previous,
                        Value::None,
                        std_grid_game_state_tick(fields)?.saturating_add(1),
                    ))
                }
                "테트로미노.이름목록" => in_call_arm_frame(|| {
                    if !args.is_empty() {
                        return Err("테트로미노.이름목록은 인자 0개를 받습니다"
                            .to_string()
                            .into());
                    }
                    Ok(Value::List(
                        // F5-GA[KNOWN-PIECE-LITERAL]: 고정 I/O/T/S/Z/J/L만 생성하며 cells가 모두 처리한다.
                        tetromino_names()
                            .into_iter()
                            .map(|name| Value::String(name.to_string()))
                            .collect(),
                    ))
                }),
                "테트로미노.만들기" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("테트로미노.만들기는 인자 1개를 받습니다".to_string().into());
                    }
                    let name = value_to_tetromino_name(&args[0])?;
                    Ok(make_std_block_piece(tetromino_cells(&name)?))
                }),
                "테트로미노.목록" => in_call_arm_frame(|| {
                    if !args.is_empty() {
                        return Err("테트로미노.목록은 인자 0개를 받습니다".to_string().into());
                    }
                    Ok(Value::List(
                        tetromino_names()
                            .into_iter()
                            .map(|name| {
                                make_std_block_piece(
                                    tetromino_cells(name).expect("known tetromino"),
                                )
                            })
                            .collect(),
                    ))
                }),
                "격자줄.찬줄목록" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("격자줄.찬줄목록은 인자 2개를 받습니다".to_string().into());
                    }
                    Ok(Value::List(
                        std_grid_line_full_rows(&args[0], &args[1])?
                            .into_iter()
                            .map(fixed_value)
                            .collect(),
                    ))
                }),
                "격자줄.지우기" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("격자줄.지우기는 인자 2개를 받습니다".to_string().into());
                    }
                    std_grid_line_clear(&args[0], args[1].clone())
                }),
                "낙하조각.만들기" | "격자게임.스폰" => in_call_arm_frame(|| {
                    if args.len() != 3 {
                        return Err("낙하조각.만들기는 인자 3개를 받습니다".to_string().into());
                    }
                    let _ = std_block_piece_cells(&args[0])?;
                    let x = value_to_i64(&args[1])?;
                    let y = value_to_i64(&args[2])?;
                    Ok(make_std_falling_piece(args[0].clone(), x, y))
                }),
                "낙하조각.조각" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("낙하조각.조각은 인자 1개를 받습니다".to_string().into());
                    }
                    let (piece, _, _) = std_falling_piece_parts(&args[0])?;
                    Ok(piece)
                }),
                "낙하조각.위치" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("낙하조각.위치는 인자 1개를 받습니다".to_string().into());
                    }
                    let (_, x, y) = std_falling_piece_parts(&args[0])?;
                    Ok(Value::List(vec![fixed_value(x), fixed_value(y)]))
                }),
                "낙하조각.배치" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("낙하조각.배치는 인자 1개를 받습니다".to_string().into());
                    }
                    Ok(std_block_piece_cells_value(&std_falling_piece_cells(
                        &args[0],
                    )?))
                }),
                "낙하조각.이동" => in_call_arm_frame(|| {
                    if args.len() != 3 {
                        return Err("낙하조각.이동은 인자 3개를 받습니다".to_string().into());
                    }
                    let (piece, x, y) = std_falling_piece_parts(&args[0])?;
                    Ok(make_std_falling_piece(
                        piece,
                        x + value_to_i64(&args[1])?,
                        y + value_to_i64(&args[2])?,
                    ))
                }),
                "낙하조각.회전" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("낙하조각.회전은 인자 2개를 받습니다".to_string().into());
                    }
                    let Value::String(direction) = &args[1] else {
                        return Err("낙하조각 회전 방향은 글이어야 합니다".to_string().into());
                    };
                    std_falling_piece_rotate(&args[0], direction)
                }),
                "격자게임.놓을수있나" => in_call_arm_frame(|| {
                    if args.len() != 3 {
                        return Err("격자게임.놓을수있나는 인자 3개를 받습니다"
                            .to_string()
                            .into());
                    }
                    Ok(Value::Bool(std_grid_game_placeable(
                        &args[0], &args[1], &args[2],
                    )?))
                }),
                "격자게임.입력적용" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("격자게임.입력적용은 인자 2개를 받습니다".to_string().into());
                    }
                    std_grid_game_apply_input(&args[0], &args[1], &self.input)
                }),
                "격자게임.중력틱" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("격자게임.중력틱은 인자 1개를 받습니다".to_string().into());
                    }
                    let (piece, x, y) = std_falling_piece_parts(&args[0])?;
                    Ok(make_std_falling_piece(piece, x, y + 1))
                }),
                "격자게임.고정" => in_call_arm_frame(|| {
                    if args.len() != 3 {
                        return Err("격자게임.고정은 인자 3개를 받습니다".to_string().into());
                    }
                    std_block_piece_lock(
                        &std_falling_piece_block(&args[0])?,
                        &args[1],
                        args[2].clone(),
                    )
                }),
                "격자게임.다음조각" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("격자게임.다음조각은 인자 1개를 받습니다".to_string().into());
                    }
                    std_grid_game_next_piece(&args[0])
                }),
                "격자게임.한틱" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("격자게임.한틱은 인자 2개를 받습니다".to_string().into());
                    }
                    std_grid_game_tick(&args[0], &args[1], &self.input)
                }),
                "격자게임.회전시도" => in_call_arm_frame(|| {
                    if args.len() != 4 {
                        return Err("격자게임.회전시도는 인자 4개를 받습니다".to_string().into());
                    }
                    let Value::String(direction) = &args[3] else {
                        return Err("격자게임.회전시도 방향은 글이어야 합니다"
                            .to_string()
                            .into());
                    };
                    std_grid_game_rotation_try(&args[0], &args[1], &args[2], direction)
                }),
                "격자게임홀드.초기화" => in_call_arm_frame(|| {
                    if !args.is_empty() {
                        return Err("격자게임홀드.초기화는 인자 0개를 받습니다"
                            .to_string()
                            .into());
                    }
                    Ok(make_std_grid_game_hold(Value::None, false))
                }),
                "격자게임홀드.칸" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("격자게임홀드.칸은 인자 1개를 받습니다".to_string().into());
                    }
                    std_grid_game_hold_piece(std_grid_game_hold_fields(&args[0])?)
                }),
                "격자게임홀드.썼나" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("격자게임홀드.썼나는 인자 1개를 받습니다".to_string().into());
                    }
                    Ok(Value::Bool(std_grid_game_hold_used(
                        std_grid_game_hold_fields(&args[0])?,
                    )?))
                }),
                "격자게임홀드.교체" => in_call_arm_frame(|| {
                    if args.len() != 5 {
                        return Err("격자게임홀드.교체는 인자 5개를 받습니다".to_string().into());
                    }
                    std_grid_game_hold_swap(
                        &args[0],
                        &args[1],
                        &args[2],
                        value_to_i64(&args[3])?,
                        value_to_i64(&args[4])?,
                    )
                }),
                "격자게임홀드.초기화턴" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("격자게임홀드.초기화턴은 인자 1개를 받습니다"
                            .to_string()
                            .into());
                    }
                    let piece = std_grid_game_hold_piece(std_grid_game_hold_fields(&args[0])?)?;
                    Ok(make_std_grid_game_hold(piece, false))
                }),
                "격자게임점수.초기화" => in_call_arm_frame(|| {
                    if !args.is_empty() {
                        return Err("격자게임점수.초기화는 인자 0개를 받습니다"
                            .to_string()
                            .into());
                    }
                    Ok(make_std_grid_game_score(0, 0))
                }),
                "격자게임점수.더하기" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("격자게임점수.더하기는 인자 2개를 받습니다"
                            .to_string()
                            .into());
                    }
                    std_grid_game_score_add(&args[0], value_to_i64(&args[1])?)
                }),
                "격자게임점수.점수" | "격자게임점수.줄수" | "격자게임점수.레벨" =>
                in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("격자게임점수 접근자는 인자 1개를 받습니다"
                            .to_string()
                            .into());
                    }
                    let field = match func {
                        "격자게임점수.점수" => "점수",
                        "격자게임점수.줄수" => "줄수",
                        _ => "레벨",
                    };
                    Ok(fixed_value(std_score_int_field(
                        std_grid_game_score_fields(&args[0])?,
                        field,
                    )?))
                }),
                "격자게임세션.만들기" => in_call_arm_frame(|| {
                    if args.len() != 5 {
                        return Err("격자게임세션.만들기는 인자 5개를 받습니다"
                            .to_string()
                            .into());
                    }
                    Ok(make_std_grid_game_session(
                        args[0].clone(),
                        args[1].clone(),
                        args[2].clone(),
                        args[3].clone(),
                        args[4].clone(),
                    ))
                }),
                "격자게임세션.격자"
                | "격자게임세션.가방"
                | "격자게임세션.상태"
                | "격자게임세션.점수"
                | "격자게임세션.낙하조각" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("격자게임세션 접근자는 인자 1개를 받습니다"
                            .to_string()
                            .into());
                    }
                    let field = match func {
                        "격자게임세션.격자" => "격자",
                        "격자게임세션.가방" => "가방",
                        "격자게임세션.상태" => "상태",
                        "격자게임세션.점수" => "점수",
                        _ => "낙하조각",
                    };
                    std_session_field(std_grid_game_session_fields(&args[0])?, field)
                }),
                "격자게임세션.바꾸기" => in_call_arm_frame(|| {
                    if args.len() != 6 {
                        return Err("격자게임세션.바꾸기는 인자 6개를 받습니다"
                            .to_string()
                            .into());
                    }
                    let _ = std_grid_game_session_fields(&args[0])?;
                    Ok(make_std_grid_game_session(
                        args[1].clone(),
                        args[2].clone(),
                        args[3].clone(),
                        args[4].clone(),
                        args[5].clone(),
                    ))
                }),
                "격자게임보기.칸목록" => in_call_arm_frame(|| {
                    if args.len() != 3 {
                        return Err("격자게임보기.칸목록은 인자 3개를 받습니다"
                            .to_string()
                            .into());
                    }
                    std_grid_game_view_cells(&args[0], &args[1], args[2].clone())
                }),
                "격자게임보기.문자판" => in_call_arm_frame(|| {
                    if args.len() != 3 {
                        return Err("격자게임보기.문자판은 인자 3개를 받습니다"
                            .to_string()
                            .into());
                    }
                    std_grid_game_view_text(&args[0], &args[1], args[2].clone())
                }),
                "격자게임보기.상태요약" => in_call_arm_frame(|| {
                    if args.len() != 1 {
                        return Err("격자게임보기.상태요약은 인자 1개를 받습니다"
                            .to_string()
                            .into());
                    }
                    std_grid_game_view_summary(&args[0])
                }),
                "격자게임보기.유령조각" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("격자게임보기.유령조각은 인자 2개를 받습니다"
                            .to_string()
                            .into());
                    }
                    std_grid_game_ghost_piece(&args[0], &args[1])
                }),
                "격자게임보기.유령보개목록" => in_call_arm_frame(|| {
                    if args.len() != 6 {
                        return Err("격자게임보기.유령보개목록은 인자 6개를 받습니다"
                            .to_string()
                            .into());
                    }
                    std_grid_game_ghost_bogae_drawlist(
                        &args[0],
                        &args[1],
                        args[2].clone(),
                        args[3].clone(),
                        &args[4],
                        &args[5],
                    )
                }),
                "격자게임보기.보개목록" => in_call_arm_frame(|| {
                    if args.len() != 4 {
                        return Err("격자게임보기.보개목록은 인자 4개를 받습니다"
                            .to_string()
                            .into());
                    }
                    std_grid_game_bogae_drawlist(&args[0], &args[1], args[2].clone(), &args[3])
                }),
                "격자게임보기.보개크기" => in_call_arm_frame(|| {
                    if args.len() != 2 {
                        return Err("격자게임보기.보개크기는 인자 2개를 받습니다"
                            .to_string()
                            .into());
                    }
                    std_grid_game_bogae_size(&args[0], &args[1])
                }),

                _ => in_call_arm_frame(|| {
                    let seed = match ordinary_overload {
                        Some(seed) => seed,
                        None => self
                            .program
                            .functions
                            .get(func)
                            .ok_or_else(|| format!("함수 '{}'를 찾을 수 없습니다", func))?
                            .clone(),
                    };
                    self.work_cost.borrow_mut().enter_call()?;
                    let result = self.eval_seed_after_dispatch(&seed, args.clone());
                    self.work_cost.borrow_mut().exit_call();
                    result
                }),
            }?;
            if let Some(rule) = rule {
                for argument in &args {
                    self.charge_builtin_value_visits(argument, rule)?;
                }
                if rule.bigint_rational_operand_and_result_bytes {
                    self.charge_big_number_parts(&[&value])?;
                }
            }
            Ok(value)
        })();
        result
    }

}
