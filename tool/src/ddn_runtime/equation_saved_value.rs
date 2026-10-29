//! 풀이 생산자에 결속된 저장 소비자. 이 함수 자체는 원문 권한을 발급하지 않는다.
//! 기존 Map/RationalV1 바이트만 사용한다. 호출자는 정적 생산자 근거를 먼저 확인한다.
use super::*;

fn invalid() -> EvalError {
    "E_EQUATION_RESULT_STORAGE: 원문 풀이 생산자의 결과 형식과 값이 다릅니다".to_string().into()
}
fn pack(fields: BTreeMap<String, ResourceValue>) -> ResourceValue {
    ResourceValue::Map(fields.into_iter().map(|(name,value)| {
        let key=ResourceValue::String(name);
        (key.canon_key(), ddonirang_core::platform::ResourceMapEntry{key,value})
    }).collect())
}
fn number(value:&Value)->Result<ResourceValue,EvalError> {
    exact_numeric_from_value(value)?.ok_or_else(invalid)?;
    match value {
        Value::Rational(value)=>Ok(ResourceValue::RationalV1(value.clone())),
        _=>resource_value_from_value(value),
    }
}
pub(super) fn encode(value:&Value)->Result<ResourceValue,EvalError> {
    let Value::Pack(fields)=value else{return Err(invalid())};
    let Some(Value::String(status))=fields.get(RELATION_SOLVE_RESULT_KIND_FIELD) else{return Err(invalid())};
    if status==RELATION_SOLVE_RESULT_FAILURE {
        if fields.len()!=2 || !matches!(fields.get(RELATION_SOLVE_REASON_FIELD),Some(Value::String(_))) {return Err(invalid())}
        return resource_value_from_value(value);
    }
    if status!=RELATION_SOLVE_RESULT_SUCCESS {return Err(invalid())}
    let Some(Value::Pack(bindings))=fields.get(RELATION_SOLVE_BINDINGS_FIELD) else{return Err(invalid())};
    let has_inputs=fields.contains_key("왼쪽입력")||fields.contains_key("오른쪽입력");
    let expected=2+if bindings.len()==1 {2} else {0}+if has_inputs {2} else {0};
    if fields.len()!=expected {return Err(invalid())}
    let mut result=BTreeMap::from([(RELATION_SOLVE_RESULT_KIND_FIELD.into(),ResourceValue::String(status.clone()))]);
    let mut numbers=BTreeMap::new();
    for (name,value) in bindings {numbers.insert(name.clone(),number(value)?);}
    result.insert(RELATION_SOLVE_BINDINGS_FIELD.into(),pack(numbers));
    if bindings.len()==1 {
        let (name,binding)=bindings.iter().next().ok_or_else(invalid)?;
        let Some(Value::String(actual))=fields.get(RELATION_SOLVE_VAR_FIELD) else{return Err(invalid())};
        let answer=fields.get(RELATION_SOLVE_VALUE_FIELD).ok_or_else(invalid)?;
        if name!=actual || exact_numeric_from_value(answer)?.ok_or_else(invalid)?.value!=exact_numeric_from_value(binding)?.ok_or_else(invalid)?.value {return Err(invalid())}
        result.insert(RELATION_SOLVE_VAR_FIELD.into(),ResourceValue::String(name.clone()));
        result.insert(RELATION_SOLVE_VALUE_FIELD.into(),number(answer)?);
    }
    if has_inputs {
        let mut supplied=BTreeSet::new();
        for field in ["왼쪽입력","오른쪽입력"] {
            let Some(Value::Pack(inputs))=fields.get(field) else{return Err(invalid())};
            let mut converted=BTreeMap::new();
            for (name,value) in inputs {
                let Value::Rational(_) = value else{return Err(invalid())};
                let binding=bindings.get(name).ok_or_else(invalid)?;
                if exact_numeric_from_value(value)?.ok_or_else(invalid)?.value!=exact_numeric_from_value(binding)?.ok_or_else(invalid)?.value {return Err(invalid())}
                supplied.insert(name.clone());converted.insert(name.clone(),number(value)?);
            }
            result.insert(field.into(),pack(converted));
        }
        if supplied!=bindings.keys().cloned().collect() {return Err(invalid())}
    }
    Ok(pack(result))
}
// Map을 일반적으로 묶음으로 추측하지 않는다. 정적 풀이 생산자가
// 지정한 결과/해/주입 위치에서만 기존 Map 바이트를 복원한다.
fn string_fields(value:&ResourceValue)->Result<BTreeMap<String,&ResourceValue>,EvalError> {
    let ResourceValue::Map(entries)=value else{return Err(invalid())};
    let mut fields=BTreeMap::new();
    for (index,entry) in entries {
        let ResourceValue::String(name)=&entry.key else{return Err(invalid())};
        if index!=&entry.key.canon_key() || fields.insert(name.clone(),&entry.value).is_some() {return Err(invalid())}
    }
    Ok(fields)
}
fn decode_number(value:&ResourceValue)->Result<Value,EvalError> {
    if matches!(value,ResourceValue::Map(_)) {
        Ok(Value::Pack(string_fields(value)?.into_iter().map(|(key,value)|(key,value_from_resource_value(value))).collect()))
    }else{Ok(value_from_resource_value(value))}
}
pub(super) fn decode(value:&ResourceValue)->Result<Value,EvalError> {
    let mut fields=BTreeMap::new();
    for (key,value) in string_fields(value)? {
        let decoded=if matches!(key.as_str(),"해"|"왼쪽입력"|"오른쪽입력") {
            let mut bindings=BTreeMap::new();
            for (name,value) in string_fields(value)? {bindings.insert(name,decode_number(value)?);}
            Value::Pack(bindings)
        }else if key==RELATION_SOLVE_VALUE_FIELD {decode_number(value)?}
        else{value_from_resource_value(value)};
        fields.insert(key,decoded);
    }
    Ok(Value::Pack(fields))
}
pub(super) fn validate_saved(value:&ResourceValue)->Result<(),String> {
    let restored=decode(value).map_err(|error|error.to_string())?;
    let canonical=encode(&restored).map_err(|error|error.to_string())?;
    if &canonical!=value {return Err("E_EQUATION_RESULT_STORAGE: 저장 결과의 정규 형식이 다릅니다".into())}
    Ok(())
}
