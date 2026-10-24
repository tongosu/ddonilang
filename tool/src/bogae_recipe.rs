//! Conservative product seal for RD-135/520. No world reads or hidden captures.
use ddonirang_lang::{Expr, ExprKind, Literal, SeedDef, SeedKind, Stmt, TypeRef};
use ddonirang_lang::runtime::{SealedViewRecipe, Value};
use serde_json::{json, Value as Json};
use std::collections::{HashMap, HashSet};

pub fn seal(seed: &SeedDef, source: &str, functions: &HashMap<String, SeedDef>, overloaded: &HashSet<String>)
    -> Result<SealedViewRecipe, String> {
    if seed.params.len() != 1 || seed.params[0].optional || seed.params[0].default_value.is_some()
        || matches!(seed.params[0].type_ref, TypeRef::Infer)
        || seed.result_type != Some(TypeRef::Named("보기값".into())) {
        return Err("E_BOGAE_RECIPE_TYPE: 보기법은 명시 대상 씨 하나를 받아 보기값을 내는 셈씨여야 합니다".into());
    }
    let mut visiting = HashSet::new();
    let mut budget = 4096;
    if !pure_definition(seed, functions, overloaded, &mut visiting, &mut budget, 32) {
        return Err("E_BOGAE_RECIPE_UNSEALED: 세계 읽기·효과·숨은 포획이 없는 보기법만 봉인할 수 있습니다".into());
    }
    let parameter_type = format!("{:?}", seed.params[0].type_ref);
    let seal = crate::runtime_surface::sha256_hex(format!(
        "ddn.sealed_view_recipe.v1\0{source}\0{}\0{}:{}\0{parameter_type}",
        seed.canonical_name, seed.span.start, seed.span.end).as_bytes());
    Ok(SealedViewRecipe {source_identity:source.into(), definition_name:seed.canonical_name.clone(),
        definition_span:[seed.span.start,seed.span.end], parameter_type, seal,validation_visits:4096-budget})
}

fn pure_definition(seed: &SeedDef, functions: &HashMap<String, SeedDef>, overloaded:&HashSet<String>,
    visiting: &mut HashSet<String>, budget: &mut usize, depth: usize) -> bool {
    if depth == 0 || overloaded.contains(&seed.canonical_name) || !seed.modifiers.is_empty() || seed.params.iter().any(|pin|pin.optional || pin.default_value.is_some())
        || !matches!(seed.seed_kind, SeedKind::Semssi | SeedKind::ValueFunc)
        || !visiting.insert(seed.canonical_name.clone()) { return false; }
    let mut locals: HashSet<_> = seed.params.iter().map(|pin|pin.pin_name.clone()).collect();
    let result = seed.body.as_ref().is_some_and(|body| body.stmts.iter().all(|stmt| {
        if *budget == 0 { return false; } *budget -= 1;
        match stmt {
            Stmt::Definition {name,value,..} => {
                let valid=pure_expr(value,&locals,functions,overloaded,visiting,budget,depth-1);
                if valid { locals.insert(name.clone()); } valid
            }
            Stmt::Mutate {target,value,deferred:false,..} => {
                let ExprKind::Var(name)=&target.kind else {return false;};
                let valid=pure_expr(value,&locals,functions,overloaded,visiting,budget,depth-1);
                if valid && name != &seed.canonical_name { locals.insert(name.clone()); } valid
            }
            Stmt::Return {value,..} | Stmt::Expr {expr:value,..} =>
                pure_expr(value,&locals,functions,overloaded,visiting,budget,depth-1),
            _ => false,
        }
    }));
    visiting.remove(&seed.canonical_name);
    result
}

fn pure_expr(expr: &Expr, locals: &HashSet<String>, functions: &HashMap<String, SeedDef>, overloaded:&HashSet<String>,
    visiting: &mut HashSet<String>, budget: &mut usize, depth: usize) -> bool {
    if depth == 0 || *budget == 0 {return false;} *budget -= 1;
    let mut child=|expr|pure_expr(expr,locals,functions,overloaded,visiting,budget,depth-1);
    match &expr.kind {
        ExprKind::Literal(Literal::Int(_) | Literal::Fixed64(_) | Literal::String(_)
            | Literal::Atom(_) | Literal::Bool(_) | Literal::None) => true,
        ExprKind::Var(name) => locals.contains(name) || matches!(name.as_str(),"참"|"거짓"|"없음"),
        ExprKind::FieldAccess {target,..} => child(target),
        ExprKind::Infix {left,right,..} => child(left) && child(right),
        ExprKind::CollectionConstructor {elements} => elements.iter().all(child),
        ExprKind::Pack {fields} => fields.iter().all(|(_,expr)|child(expr)),
        ExprKind::Call {args,func} => {
            if overloaded.contains(func) {return false;}
            if !args.iter().all(|arg|child(&arg.expr)) {return false;}
            matches!(func.as_str(),"표준장면.원"|"표준장면.다각형"|"표준장면.글"|"표준장면.평면"
                |"표준보기.장면"|"표준보기.결과표"|"차림.값")
                || functions.get(func).is_some_and(|seed|
                    pure_definition(seed,functions,overloaded,visiting,budget,depth-1))
        }
        _ => false,
    }
}

/// Exact typed target projection; no DOM target, renderer callback or display-text key.
pub fn project_target(value: &Value) -> Result<Json,String> {
    project_target_with_visits(value).map(|(value,_)|value)
}

pub fn project_target_with_visits(value: &Value) -> Result<(Json,usize),String> {
    fn walk(value:&Value, depth:usize, budget:&mut usize)->Result<Json,String> {
        if depth==0 || *budget==0 {return Err("E_BOGAE_TARGET_UNSUPPORTED: 대상 크기가 봉인 한도를 넘습니다".into());}
        *budget-=1;
        Ok(match value {
            Value::None => json!({"kind":"none"}),
            Value::Bool(value) => json!({"kind":"bool","value":value}),
            Value::Fixed64(value) => json!({"kind":"fixed64","raw":value.raw_i64().to_string()}),
            Value::String(value) => json!({"kind":"text","value":value}),
            Value::SceneElement(element) => {
                if !matches!(element,ddonirang_lang::runtime::PlaneElement::Circle {..}) {
                    return Err("E_BOGAE_TARGET_UNSUPPORTED: 현재 으뜸 대상 선택은 원 장면요소값만 지원합니다".into());
                }
                let visits=crate::scene_value::clone_visits(value);
                if visits>*budget {return Err("E_BOGAE_TARGET_UNSUPPORTED: 대상 크기가 봉인 한도를 넘습니다".into());}
                *budget-=visits;
                json!({"kind":"scene_element","element":crate::scene_value::project_element(element)})
            },
            Value::List(values) => json!({"kind":"list","values":values.iter()
                .map(|value|walk(value,depth-1,budget)).collect::<Result<Vec<_>,_>>()?}),
            Value::Pack(values) => json!({"kind":"pack","fields":values.iter()
                .map(|(name,value)|Ok((name.clone(),walk(value,depth-1,budget)?)))
                .collect::<Result<std::collections::BTreeMap<_,_>,String>>()?}),
            _ => return Err("E_BOGAE_TARGET_UNSUPPORTED: 이 대상 씨는 아직 돋 대상 봉인을 지원하지 않습니다".into()),
        })
    }
    let mut budget=4096;
    let projected=walk(value,64,&mut budget)?;
    Ok((projected,4096-budget))
}
