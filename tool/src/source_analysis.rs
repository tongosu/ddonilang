use std::collections::{BTreeMap, BTreeSet};

use ddonirang_lang::{
    CanonProgram, Expr, ExprKind, Lexer, Literal, ParseMode, Stmt, TokenKind, TopLevelItem, TypeRef,
};
use serde::{Deserialize, Serialize};

use crate::ddn_runtime::{
    resolve_supported_source_profile, DdnParseWarning, DdnProgram,
    V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
};

pub const SOURCE_ANALYSIS_SNAPSHOT_SCHEMA: &str = "ddn.source_analysis_snapshot.candidate.v1";
pub const STRICT_SOURCE_PROFILE_IDENTITY: &str = "strict";
pub const REPAIR_PROPOSAL_SCHEMA: &str = "ddn.source_analysis_repair_proposal.candidate.v1";
pub const REMOVE_SETUP_COLON_FIX_ID: &str = "FIX_SOURCE_REMOVE_SETUP_COLON";

#[derive(Clone, Debug, PartialEq, Eq)]
enum StaticType {
    Number,
    Text,
    Bytes,
    Bool,
    Function,
    Collection(Box<StaticType>),
    Tuple(Vec<StaticType>),
    None,
    Unknown,
}

impl StaticType {
    fn label(&self) -> &'static str {
        match self {
            Self::Number => "수",
            Self::Text => "글",
            Self::Bytes => "바이트열",
            Self::Bool => "참거짓",
            Self::Function => "함수값",
            Self::Collection(_) => "차림",
            Self::Tuple(_) => "위치형 값꾸러미",
            Self::None => "없음",
            Self::Unknown => "알수없음",
        }
    }
}

// Unknown is an unresolved static component, not permission to erase an
// already known element type. Compare each known component without requiring
// an empty or dynamically produced collection to invent an element type.
fn static_types_compatible(left: &StaticType, right: &StaticType) -> bool {
    if let (StaticType::Tuple(a),StaticType::Tuple(b))=(left,right) {
        return a.len()==b.len() && a.iter().zip(b).all(|(a,b)|static_types_compatible(a,b));
    }
    let mut pair = (left, right);
    loop {
        match pair {
            (StaticType::Unknown, _) | (_, StaticType::Unknown) => return true,
            (StaticType::Collection(a), StaticType::Collection(b)) => pair = (a, b),
            (a, b) => return a == b,
        }
    }
}

fn is_range_expression(expr: &Expr) -> bool {
    matches!(&expr.kind, ExprKind::Call { func, .. } if func == "표준.범위")
}

pub(crate) fn static_tuple_index(expr:&Expr)->Option<usize> {
    // Keep existing literal admission and use only closed arithmetic trees
    // for the additional compile-time case. No names, calls, world values or
    // input reads are evaluated here. Runtime arithmetic owns every operation.
    match &expr.kind {
        ExprKind::Literal(Literal::Int(n))=>return usize::try_from(*n).ok(),
        ExprKind::Literal(Literal::Fixed64(n)) if n.raw_i64()>=0 && n.frac_part()==0
            =>return usize::try_from(n.int_part()).ok(),
        ExprKind::Infix { .. }=>{},
        _=>return None,
    }
    use ddonirang_core::fixed64::Fixed64;
    enum Job<'a> { Evaluate(&'a Expr), Apply(&'a str) }
    let mut pending=vec![Job::Evaluate(expr)];
    let mut values=Vec::<Fixed64>::new();
    while let Some(job)=pending.pop() {
        match job {
            Job::Evaluate(expr)=>match &expr.kind {
                ExprKind::Literal(Literal::Int(n))=>values.push(Fixed64::from_i64(*n)),
                ExprKind::Literal(Literal::Fixed64(n))=>values.push(*n),
                ExprKind::Infix { left,op,right } if matches!(op.as_str(),"+"|"-"|"*"|"/")=>{
                    pending.push(Job::Apply(op));
                    pending.push(Job::Evaluate(right));
                    pending.push(Job::Evaluate(left));
                },
                _=>return None,
            },
            Job::Apply(op)=>{
                let right=values.pop()?;
                let left=values.pop()?;
                let value=match op {
                    "+"=>left.checked_add(right),
                    "-"=>left.checked_sub(right),
                    "*"=>left.checked_mul(right),
                    "/"=>left.try_div(right),
                    _=>return None,
                }.ok()?;
                values.push(value);
            },
        }
    }
    let value=values.pop()?;
    if !values.is_empty() || value.raw_i64()<0 || value.frac_part()!=0 { return None; }
    usize::try_from(value.int_part()).ok()
}

fn tuple_selection_type(items:&[StaticType],selectors:&[ddonirang_lang::TensorAxisSelector],location:&Expr)->Result<StaticType,StaticDiagnostic> {
    let failure=||static_error("E_CHECK_TUPLE_SELECTION","위치형 값꾸러미는 범위 안의 정적 위치 또는 반열림 범위로 골라야 합니다",location);
    match selectors {
        [ddonirang_lang::TensorAxisSelector::Index(index)]=>{
            let index=static_tuple_index(index).ok_or_else(failure)?;
            items.get(index).cloned().ok_or_else(failure)
        }
        [ddonirang_lang::TensorAxisSelector::Range{start,end}]=>{
            let start=usize::try_from(*start).map_err(|_|failure())?;
            let end=usize::try_from(*end).map_err(|_|failure())?;
            if start>end||end>items.len(){return Err(failure());}
            Ok(StaticType::Tuple(items[start..end].to_vec()))
        }
        _=>Err(failure()),
    }
}

fn is_scalar_selection(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::SelectionPlace { index, .. } => !is_range_expression(index),
        ExprKind::MultiAxisSelection(selection) => matches!(selection.selectors.as_slice(),
            [ddonirang_lang::TensorAxisSelector::Index(index)] if !is_range_expression(index)),
        _ => false,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StaticDiagnostic {
    code: &'static str,
    message: String,
    start: usize,
    end: usize,
}

fn type_ref_kind(type_ref: &TypeRef) -> StaticType {
    match type_ref {
        TypeRef::Named(name) => match name.as_str() {
            "수" | "셈수" | "바른수" | "큰바른수" | "나눔수" => StaticType::Number,
            "글" | "이름씨" => StaticType::Text,
            "바이트열" => StaticType::Bytes,
            "참거짓" | "논" => StaticType::Bool,
            "없음" => StaticType::None,
            _ => StaticType::Unknown,
        },
        TypeRef::Function {..} => StaticType::Function,
        TypeRef::Applied { name, args } if name == "차림" => StaticType::Collection(Box::new(
            args.first().map(type_ref_kind).unwrap_or(StaticType::Unknown))),
        _ => StaticType::Unknown,
    }
}

fn static_error(code: &'static str, message: impl Into<String>, expr: &Expr) -> StaticDiagnostic {
    StaticDiagnostic {
        code,
        message: message.into(),
        start: expr.span.start,
        end: expr.span.end,
    }
}

fn check_logical_operand(
    operator: &str,
    kind: &StaticType,
    expr: &Expr,
) -> Result<(), StaticDiagnostic> {
    if matches!(kind, StaticType::Bool | StaticType::Unknown) {
        return Ok(());
    }
    Err(static_error(
        "E_CHECK_TYPE_MISMATCH",
        format!(
            "{operator}의 피연산자는 참거짓이어야 합니다: {}",
            kind.label()
        ),
        expr,
    ))
}

struct StaticCallContext<'a> {
    results: BTreeMap<String, StaticType>,
    program: Option<&'a CanonProgram>,
}
impl std::ops::Deref for StaticCallContext<'_> {
    type Target=BTreeMap<String, StaticType>;
    fn deref(&self)->&Self::Target { &self.results }
}
fn contains_function_kind(kind:&StaticType)->bool {
    let mut pending=vec![kind];
    while let Some(kind)=pending.pop(){match kind {
        StaticType::Function=>return true,
        StaticType::Tuple(items)=>pending.extend(items),
        StaticType::Collection(item)=>pending.push(item),_=>{},
    }}
    false
}

// Source-kind evidence only. Inspect value-preserving constructors, never
// evaluate a selector/call or infer function identity from its spelling.
fn contains_function_data(expr:&Expr,kind:&StaticType,symbols:&BTreeMap<String,StaticType>,calls:&StaticCallContext<'_>)->bool {
    if contains_function_kind(kind){return true;}
    let mut pending=vec![expr];
    while let Some(expr)=pending.pop(){match &expr.kind {
        ExprKind::Var(name) if symbols.get(name).is_some_and(contains_function_kind)=>return true,
        ExprKind::CollectionConstructor{elements}|ExprKind::TupleConstructor{elements}=>pending.extend(elements),
        ExprKind::Call{func,..} if calls.get(func).is_some_and(contains_function_kind)=>return true,
        _=>{},
    }}
    false
}

fn checked_table_descriptor()->Result<ddonirang_lang::stdlib::CheckedResultTableDescriptor,String> {
    let (raw,digest)=ddonirang_lang::stdlib::result_table_registration_material(1).ok_or("등록 판본 없음")?;
    let material:serde_json::Value=serde_json::from_str(raw).map_err(|e|e.to_string())?;
    let id=|key:&str|->Result<[u8;32],String> {
        hex::decode(material[key].as_str().ok_or("등록 정체 없음")?).map_err(|e|e.to_string())?
            .try_into().map_err(|_|"등록 정체 크기 불일치".into())
    };
    ddonirang_lang::stdlib::lookup_result_table_registration(&id("owner")?,&id("definition")?,1,digest,&[])
}

fn check_known_collection_members(
    value: &Expr,
    expected: &StaticType,
    symbols: &BTreeMap<String, StaticType>,
    call_results: &StaticCallContext<'_>,
) -> Result<(), StaticDiagnostic> {
    let mut pending = vec![(value, expected)];
    while let Some((value, expected)) = pending.pop() {
        if let (ExprKind::CollectionConstructor { elements }, StaticType::Collection(element)) = (&value.kind, expected) {
            // Preserve authored order and report the first proven mismatch.
            // Unresolved members do not hide a later known incompatible value.
            for member in elements.iter().rev() {
                pending.push((member, element));
            }
        } else {
            let actual = infer_expr(value, symbols, call_results, false)?;
            if !static_types_compatible(expected, &actual) {
                return Err(static_error("E_CHECK_TYPE_MISMATCH",
                    format!("차림 원소의 씨가 맞지 않습니다: 기대 {}, 실제 {}",
                        expected.label(), actual.label()), value));
            }
        }
    }
    Ok(())
}

fn infer_expected_function(value:&Expr,kind:&TypeRef,symbols:&BTreeMap<String,StaticType>,calls:&StaticCallContext<'_>,unknown_names_allowed:bool)->Result<StaticType,StaticDiagnostic>{
    let TypeRef::Function{params,result}=kind else{return infer_expr(value,symbols,calls,unknown_names_allowed)};
    match &value.kind{
        ExprKind::SeedLiteral{param,body}=>{
            if calls.program.is_some_and(|p|p.source_epoch==ddonirang_lang::SourceEpoch::V25){
                let bound=param.split(',').map(str::trim).filter(|n|!n.is_empty()).map(str::to_owned).collect();
                anonymous_capture_free_names(body,&bound,&mut std::collections::HashSet::new()).map_err(|message|static_error("E_CHECK_FUNCTION_BODY",message,body))?;
            }
            let names:Vec<_>=param.split(',').map(str::trim).filter(|n|!n.is_empty()).collect();
            if names.len()!=params.len(){return Err(static_error("E_CHECK_FUNCTION_ARITY","함수씨와 받는자리 수가 다릅니다",value));}
            let mut nested=symbols.clone();for (name,kind) in names.iter().zip(params){nested.insert((*name).into(),type_ref_kind(kind));}
            let actual=infer_expr(body,&nested,calls,unknown_names_allowed)?;let expected=type_ref_kind(result);
            if actual!=StaticType::Unknown&&expected!=StaticType::Unknown&&!static_types_compatible(&expected,&actual){
                let site=if let ExprKind::FunctionBlock{result,..}=&body.kind{result.as_ref()}else{body.as_ref()};
                return Err(static_error("E_CHECK_FUNCTION_RESULT_TYPE","함수 결과씨가 다릅니다",site));
            }
            Ok(StaticType::Function)
        },
        ExprKind::Var(target) if !symbols.contains_key(target)=>{
            let program=calls.program.ok_or_else(||static_error("E_CHECK_FUNCTION_SOURCE","함수값의 실제 원문 문맥이 없습니다",value))?;
            ddonirang_lang::runtime::resolve_source_function_definition(program,target,kind).map_err(|message|static_error("E_CHECK_FUNCTION_SOURCE",message,value))?;
            Ok(StaticType::Function)
        },
        _=>{let actual=infer_expr(value,symbols,calls,unknown_names_allowed)?;
            if !matches!(actual,StaticType::Function|StaticType::Unknown){return Err(static_error("E_CHECK_FUNCTION_SOURCE","실제 함수값이 필요합니다",value));}Ok(actual)},
    }
}

fn infer_expr(
    expr: &Expr,
    symbols: &BTreeMap<String, StaticType>,
    call_results: &StaticCallContext<'_>,
    unknown_names_allowed: bool,
) -> Result<StaticType, StaticDiagnostic> {
    match &expr.kind {
        ExprKind::SumBranchValue {payload,..} => {
            for value in payload {infer_expr(&value.value,symbols,call_results,unknown_names_allowed)?;}
            Ok(StaticType::Unknown)
        }
        ExprKind::Literal(value) => Ok(match value {
            Literal::Int(_) | Literal::Fixed64(_) => StaticType::Number,
            Literal::String(_) | Literal::Atom(_) => StaticType::Text,
            Literal::Bool(_) => StaticType::Bool,
            Literal::None => StaticType::None,
            Literal::Bytes(_) => StaticType::Bytes,
            Literal::Regex(_) | Literal::Resource(_) => StaticType::Unknown,
        }),
        ExprKind::Var(name) => match name.as_str() {
            "참" | "거짓" => Ok(StaticType::Bool),
            "없음" => Ok(StaticType::None),
            _ => symbols
                .get(name)
                .cloned()
                .map(|kind| if unknown_names_allowed && kind == StaticType::None { StaticType::Unknown } else { kind })
                .or_else(|| unknown_names_allowed.then_some(StaticType::Unknown))
                .ok_or_else(|| {
                    static_error(
                        "E_CHECK_UNKNOWN_NAME",
                        format!("정의되지 않은 이름입니다: {name}"),
                        expr,
                    )
                }),
        },
        ExprKind::Infix { left, op, right } => {
            let left_type = infer_expr(left, symbols, call_results, unknown_names_allowed)?;
            let right_type = infer_expr(right, symbols, call_results, unknown_names_allowed)?;
            if matches!(op.as_str(), "그리고" | "또는" | "&&" | "||") {
                // 실행의 단락 평가와 달리 정적 검사는 양쪽 식을 모두 검사한다.
                check_logical_operand(op, &left_type, left)?;
                check_logical_operand(op, &right_type, right)?;
                return Ok(StaticType::Bool);
            }
            if matches!(op.as_str(), "==" | "!=" | "<" | "<=" | ">" | ">=") {
                return Ok(StaticType::Bool);
            }
            if op == "+" && left_type == StaticType::Text && right_type == StaticType::Text {
                return Ok(StaticType::Text);
            }
            if matches!(op.as_str(), "+" | "-" | "*" | "/") {
                if left_type == StaticType::Number && right_type == StaticType::Number {
                    return Ok(StaticType::Number);
                }
                if left_type != StaticType::Unknown && right_type != StaticType::Unknown {
                    return Err(static_error(
                        "E_CHECK_TYPE_MISMATCH",
                        format!(
                            "연산자 {op}의 피연산자 형식이 맞지 않습니다: {} {op} {}",
                            left_type.label(),
                            right_type.label()
                        ),
                        expr,
                    ));
                }
            }
            Ok(StaticType::Unknown)
        }
        ExprKind::MultiAxisSelection(selection) => {
            let target_type = infer_expr(&selection.target, symbols, call_results, unknown_names_allowed)?;
            if let StaticType::Tuple(items)=&target_type {
                return tuple_selection_type(items,&selection.selectors,expr);
            }
            for selector in &selection.selectors {
                if let ddonirang_lang::TensorAxisSelector::Index(index) = selector {
                    let index_type = infer_expr(index, symbols, call_results, unknown_names_allowed)?;
                    if matches!(target_type, StaticType::Collection(_) | StaticType::Text)
                        && index_type != StaticType::Number && index_type != StaticType::Unknown {
                        return Err(static_error("E_CHECK_TYPE_MISMATCH",
                            format!("차림 위치는 수여야 합니다: {}", index_type.label()), index));
                    }
                }
            }
            // RD-352/354: scalar List selection returns its element type;
            // half-open ranges retain the List type. Tensor and unknown target
            // contracts remain owned by their existing common-runtime checks.
            match (target_type, selection.selectors.as_slice()) {
                (StaticType::Text, [ddonirang_lang::TensorAxisSelector::Index(_)]) => Ok(StaticType::Text),
                (StaticType::Collection(element), [ddonirang_lang::TensorAxisSelector::Index(index)]) => {
                    if is_range_expression(index) {
                        Ok(StaticType::Collection(element))
                    } else { Ok(*element) }
                }
                (collection @ StaticType::Collection(_), [ddonirang_lang::TensorAxisSelector::Range { .. }]) => Ok(collection),
                _ => Ok(StaticType::Unknown),
            }
        }
        ExprKind::Call { args, func } => {
            let expected_call=call_results.program.map(|program|crate::function_call_graph::SourceCall::expected_arguments(program,expr,symbols.contains_key(func)))
                .transpose().map_err(|message|static_error("E_CHECK_FUNCTION_SOURCE",message,expr))?.flatten();
            let expected_types=expected_call.as_ref().map(|binding|binding.authored_types()).transpose()
                .map_err(|message|static_error("E_CHECK_FUNCTION_SOURCE",message,expr))?;
            let mut selected_argument=None;
            if func=="표준보기.결과표" && args.iter().any(|arg|arg.resolved_pin.as_deref()==Some("열들")) {
                if let Some(program)=call_results.program {
                    if symbols.contains_key("관찰") {return Err(static_error("E_CHECK_TABLE_SELECTION","가려진 관찰 이름은 공개 참조가 아닙니다",expr));}
                    let registration=checked_table_descriptor().map_err(|message|static_error("E_CHECK_TABLE_SELECTION",message,expr))?;
                    let shadowed=symbols.contains_key(func)||symbols.contains_key("표준보기");
                    if let Some(admitted)=crate::numeric_constraint_plan::resolve_registered_result_table_call(program,expr,&registration,shadowed)
                        .map_err(|message|static_error("E_CHECK_TABLE_SELECTION",message,expr))? {
                        admitted.into_parts(program,expr).map_err(|message|static_error("E_CHECK_TABLE_SELECTION",message,expr))?;
                        selected_argument=args.iter().position(|arg|arg.resolved_pin.as_deref()==Some("열들"));
                    }
                }
            }
            let mut xy_arguments=std::collections::BTreeSet::new();
            if matches!(func.as_str(),"표준보기.흩어진점"|"표준보기.좌표선") {
                if let Some(program)=call_results.program {
                    if symbols.contains_key("관찰") {return Err(static_error("E_CHECK_XY_SELECTION","가려진 관찰 이름은 공개 참조가 아닙니다",expr));}
                    let shadowed=symbols.contains_key(func)||symbols.contains_key("표준보기");
                    if crate::authored_bogae::xy::resolve_call(program,expr,shadowed)
                        .map_err(|message|static_error("E_CHECK_XY_SELECTION",message,expr))?.is_some() {
                        xy_arguments.extend(0..args.len());
                    }
                }
            }
            for (index,arg) in args.iter().enumerate() {
                if selected_argument==Some(index)||xy_arguments.contains(&index) {continue;}
                // RD-520: the recipe pin refers to an authored named callable,
                // not a world variable. Only this exact pin admits a definition
                // reference; its type and capture-free seal are checked by the
                // shared runtime. Ordinary unknown variable errors stay intact.
                if func=="표준보개.돋" && (arg.resolved_pin.as_deref()==Some("보기법")
                    || (arg.resolved_pin.is_none() && index==1)) {
                    if let ExprKind::Var(name)=&arg.expr.kind {
                        if !symbols.contains_key(name) && call_results.contains_key(name) {continue;}
                    }
                }
                let kind=if let Some(kind)=expected_types.as_ref().and_then(|types|types.get(index)).filter(|kind|matches!(kind,TypeRef::Function{..})){
                    infer_expected_function(&arg.expr,kind,symbols,call_results,unknown_names_allowed)?
                }else{infer_expr(&arg.expr,symbols,call_results,unknown_names_allowed)?};
                let canonical=ddonirang_lang::stdlib::canonicalize_stdlib_alias(func);
                let data_key=match canonical {"모음"=>true,"짝맞춤"=>index%2==0,
                    "짝맞춤.값"|"짝맞춤.바꾼값"=>index==1,"정렬"=>index==0,_=>false};
                if data_key && contains_function_data(&arg.expr,&kind,symbols,call_results) {
                    return Err(static_error("E_TYPE_MISMATCH",
                        "함수값은 모음 원소·짝맞춤 열쇠·일반 정렬 대상이 아닙니다",&arg.expr));
                }
                if matches!(func.as_str(), "그리고" | "또는") {
                    check_logical_operand(func, &kind, &arg.expr)?;
                }
            }
            if matches!(func.as_str(), "그리고" | "또는") {
                return Ok(StaticType::Bool);
            }
            Ok(call_results
                .get(func)
                .cloned()
                .unwrap_or(StaticType::Unknown))
        }
        ExprKind::TupleConstructor { elements } => {
            Ok(StaticType::Tuple(elements.iter().map(|element|
                infer_expr(element,symbols,call_results,unknown_names_allowed)).collect::<Result<Vec<_>,_>>()?))
        }
        ExprKind::CollectionConstructor { elements } => {
            let mut element_type = None;
            for element in elements {
                let actual = infer_expr(element, symbols, call_results, unknown_names_allowed)?;
                element_type = Some(match element_type {
                    None => actual,
                    Some(previous) if previous == actual => previous,
                    // Do not manufacture a heterogeneous positional Tuple from
                    // a List constructor. Its unsupported type inference stays
                    // conservative and its value contract stays runtime-owned.
                    Some(_) => StaticType::Unknown,
                });
            }
            Ok(StaticType::Collection(Box::new(element_type.unwrap_or(StaticType::Unknown))))
        }
        ExprKind::SelectionPlace { target, index } => {
            let target_type = infer_expr(target, symbols, call_results, unknown_names_allowed)?;
            if let StaticType::Tuple(items)=&target_type {
                return tuple_selection_type(items,&[ddonirang_lang::TensorAxisSelector::Index(Box::new((**index).clone()))],expr);
            }
            let index_type = infer_expr(index, symbols, call_results, unknown_names_allowed)?;
            // RD-351/353: only a known positional target requires a numeric
            // selector. Map keys are owned by the target, not by list syntax.
            // Unknown target typing is checked by the common runtime; it is
            // not permission to insert a missing key or default a failed read.
            if matches!(target_type, StaticType::Collection(_) | StaticType::Text)
                && index_type != StaticType::Number && index_type != StaticType::Unknown {
                return Err(static_error(
                    "E_CHECK_TYPE_MISMATCH",
                    format!("차림 위치는 수여야 합니다: {}", index_type.label()),
                    index,
                ));
            }
            // RD-354: a single grapheme remains Text, not an unknown scalar.
            Ok(match target_type {
                StaticType::Text => StaticType::Text,
                StaticType::Collection(element) if is_range_expression(index) => StaticType::Collection(element),
                StaticType::Collection(element) => *element,
                _ => StaticType::Unknown,
            })
        }
        ExprKind::FieldAccess { target, .. } => {
            infer_expr(target, symbols, call_results, unknown_names_allowed)?;
            Ok(StaticType::Unknown)
        }
        ExprKind::Suffix { value, .. } | ExprKind::Nuance { expr: value, .. } => {
            infer_expr(value, symbols, call_results, unknown_names_allowed)
        }
        ExprKind::Pack { fields } => {
            for (_, value) in fields {
                infer_expr(value, symbols, call_results, unknown_names_allowed)?;
            }
            Ok(StaticType::Unknown)
        }
        ExprKind::TemplateRender { inject, .. } | ExprKind::FormulaEval { inject, .. } => {
            for (_, value) in inject {
                infer_expr(value, symbols, call_results, unknown_names_allowed)?;
            }
            Ok(StaticType::Unknown)
        }
        ExprKind::SeedLiteral { param, body } => {
            let mut local = symbols.clone();
            local.insert(param.clone(), StaticType::Unknown);
            infer_expr(body, &local, call_results, true)?;
            Ok(StaticType::Function)
        }
        ExprKind::FunctionBlock{statements,result}=>{
            let mut nested=symbols.clone();
            validate_root_statements(&statements.stmts,&mut nested,call_results,unknown_names_allowed)?;
            infer_expr(result,&nested,call_results,unknown_names_allowed)
        }
        ExprKind::Thunk(body) => {
            validate_root_statements(&body.stmts, &mut symbols.clone(), call_results, true)?;
            Ok(StaticType::Unknown)
        }
        ExprKind::Eval { thunk, mode } => {
            infer_expr(thunk, symbols, call_results, unknown_names_allowed)?;
            // D6's braced condition is represented as a Bool-evaluated thunk.
            // Preserve the existing static rejection of known non-Bool values.
            if matches!(mode, ddonirang_lang::ThunkEvalMode::Bool | ddonirang_lang::ThunkEvalMode::Not) {
                if let ExprKind::Thunk(body) = &thunk.kind {
                    let mut local = symbols.clone();
                    validate_root_statements(&body.stmts, &mut local, call_results, true)?;
                    if let Some(Stmt::Expr { expr: result, .. }) = body.stmts.last() {
                        let actual = infer_expr(result, &local, call_results, true)?;
                        if actual != StaticType::Bool && actual != StaticType::Unknown {
                            return Err(static_error("E_CHECK_TYPE_MISMATCH",
                                format!("조건은 참거짓이어야 합니다: {}", actual.label()), result));
                        }
                    }
                }
                return Ok(StaticType::Bool);
            }
            Ok(StaticType::Unknown)
        }
        ExprKind::Pipe { stages } => {
            for stage in stages {
                infer_expr(stage, symbols, call_results, unknown_names_allowed)?;
            }
            Ok(StaticType::Unknown)
        }
        _ => Ok(StaticType::Unknown),
    }
}

fn validate_root_statements(
    statements: &[Stmt],
    symbols: &mut BTreeMap<String, StaticType>,
    call_results: &StaticCallContext<'_>,
    unknown_names_allowed: bool,
) -> Result<(), StaticDiagnostic> {
    for statement in statements {
        match statement {
            Stmt::SumDefinition {..} => {},
            Stmt::SumMatch {subject,arms,default_body,..} => {
                infer_expr(subject,symbols,call_results,unknown_names_allowed)?;
                for arm in arms {
                    let mut scoped=symbols.clone();
                    for binder in &arm.binder {scoped.insert(binder.pin.clone(),StaticType::Unknown);}
                    validate_root_statements(&arm.body.stmts,&mut scoped,call_results,unknown_names_allowed)?;
                }
                if let Some(body)=default_body {validate_root_statements(&body.stmts,&mut symbols.clone(),call_results,unknown_names_allowed)?;}
            }
            Stmt::Definition {
                name,
                type_ref,
                value,
                ..
            } => {
                if matches!(type_ref,TypeRef::Infer) && matches!(value.kind,ExprKind::SeedLiteral{..})
                    && call_results.program.is_some_and(|p|p.source_epoch==ddonirang_lang::SourceEpoch::V25) {
                    return Err(static_error("E_CHECK_FUNCTION_TYPE_REQUIRED","보관할 함수값의 함수씨를 명시하세요",value));
                }
                let actual=if matches!(type_ref,TypeRef::Function{..}){
                    infer_expected_function(value,type_ref,symbols,call_results,unknown_names_allowed)?
                }else{infer_expr(value,symbols,call_results,unknown_names_allowed)?};
                let expected = type_ref_kind(type_ref);
                if !unknown_names_allowed && matches!(expected, StaticType::Collection(_)) {
                    check_known_collection_members(value, &expected, symbols, call_results)?;
                }
                // Existing callable-body declared-type diagnostics are emitted
                // by the runtime with pin/expected/actual fields. Keep that
                // contract; this conservative body pass adds logical checking.
                if !unknown_names_allowed && !matches!(expected,StaticType::Unknown|StaticType::Function)
                    && actual != StaticType::Unknown
                    && !static_types_compatible(&expected, &actual)
                {
                    return Err(static_error(
                        "E_CHECK_TYPE_MISMATCH",
                        format!(
                            "{name}의 정의 형식이 맞지 않습니다: 기대 {}, 실제 {}",
                            expected.label(),
                            actual.label()
                        ),
                        value,
                    ));
                }
                symbols.insert(
                    name.clone(),
                    if expected == StaticType::Unknown {
                        actual
                    } else {
                        expected
                    },
                );
            }
            Stmt::DerivedBinding{name,value,..}=>{
                let actual=infer_expr(value,symbols,call_results,unknown_names_allowed)?;
                symbols.insert(name.clone(),actual);
            }
            Stmt::Mutate { target, value, .. } => {
                let actual = infer_expr(value, symbols, call_results, unknown_names_allowed)?;
                if let ExprKind::Var(name) = &target.kind {
                    if let Some(previous) = symbols.get(name).cloned() {
                        if !unknown_names_allowed && matches!(previous, StaticType::Collection(_)) {
                            check_known_collection_members(value, &previous, symbols, call_results)?;
                        }
                        // 첫 없음의 후속 씨 고정 정책은 이 변경에서 결정하지 않는다.
                        if unknown_names_allowed && previous == StaticType::None {
                            continue;
                        }
                        if previous != StaticType::Unknown
                            && actual != StaticType::Unknown
                            && !static_types_compatible(&previous, &actual)
                        {
                            return Err(static_error(
                                "E_CHECK_TYPE_MISMATCH",
                                format!(
                                    "{name}의 재결속 형식이 맞지 않습니다: 기대 {}, 실제 {}",
                                    previous.label(),
                                    actual.label()
                                ),
                                value,
                            ));
                        }
                        // 동적으로만 알 수 있는 중간 값이 이미 추론한 씨를 지우지 않는다.
                        if previous != StaticType::Unknown {
                            continue;
                        }
                    }
                    symbols.insert(name.clone(), actual);
                } else {
                    let expected = infer_expr(target, symbols, call_results, unknown_names_allowed)?;
                    if !unknown_names_allowed && is_scalar_selection(target)
                        && matches!(expected, StaticType::Collection(_)) {
                        check_known_collection_members(value, &expected, symbols, call_results)?;
                    }
                    // Preserve the admitted runtime range-replacement family:
                    // size, target and selector failures are checked together.
                    if (is_scalar_selection(target) || matches!(expected,StaticType::Tuple(_)))
                        && !static_types_compatible(&expected, &actual) {
                        return Err(static_error("E_CHECK_TYPE_MISMATCH",
                            format!("선택 위치의 교체 형식이 맞지 않습니다: 기대 {}, 실제 {}",
                                expected.label(), actual.label()), value));
                    }
                }
            }
            Stmt::ResultFlow {
                producer,
                destination,
                ..
            } => {
                let actual = infer_expr(producer, symbols, call_results, unknown_names_allowed)?;
                let expected = infer_expr(destination, symbols, call_results, unknown_names_allowed)?;
                if !unknown_names_allowed && is_scalar_selection(destination)
                    && matches!(expected, StaticType::Collection(_)) {
                    check_known_collection_members(producer, &expected, symbols, call_results)?;
                }
                if is_scalar_selection(destination)
                    && !static_types_compatible(&expected, &actual) {
                    return Err(static_error("E_CHECK_TYPE_MISMATCH",
                        format!("선택 결과 자리의 형식이 맞지 않습니다: 기대 {}, 실제 {}",
                            expected.label(), actual.label()), producer));
                }
            }
            Stmt::Expr { expr, .. }
            | Stmt::Show { expr, .. }
            | Stmt::Inspect { expr, .. }
            | Stmt::Return { value: expr, .. } => {
                infer_expr(expr, symbols, call_results, unknown_names_allowed)?;
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                let condition_type =
                    infer_expr(condition, symbols, call_results, unknown_names_allowed)?;
                if condition_type != StaticType::Bool && condition_type != StaticType::Unknown {
                    return Err(static_error(
                        "E_CHECK_TYPE_MISMATCH",
                        format!(
                            "만약 조건은 참거짓이어야 합니다: {}",
                            condition_type.label()
                        ),
                        condition,
                    ));
                }
                let mut then_symbols = symbols.clone();
                validate_root_statements(
                    &then_body.stmts,
                    &mut then_symbols,
                    call_results,
                    unknown_names_allowed,
                )?;
                if let Some(else_body) = else_body {
                    let mut else_symbols = symbols.clone();
                    validate_root_statements(
                        &else_body.stmts,
                        &mut else_symbols,
                        call_results,
                        unknown_names_allowed,
                    )?;
                }
            }
            Stmt::DeclBlock { items, .. } => {
                for item in items {
                    let actual = item
                        .value
                        .as_ref()
                        .map(|value| {
                            infer_expr(value, symbols, call_results, unknown_names_allowed)
                        })
                        .transpose()?
                        .unwrap_or(StaticType::Unknown);
                    let declared = type_ref_kind(&item.type_ref);
                    symbols.insert(
                        item.name.clone(),
                        if declared == StaticType::Unknown {
                            if unknown_names_allowed && actual == StaticType::None {
                                StaticType::Unknown
                            } else {
                                actual
                            }
                        } else {
                            declared
                        },
                    );
                }
            }
            Stmt::While {
                condition, body, ..
            }
            | Stmt::Guard {
                condition, body, ..
            }
            | Stmt::HookWhenBecomes {
                condition, body, ..
            }
            | Stmt::HookWhile {
                condition, body, ..
            } => {
                infer_expr(condition, symbols, call_results, true)?;
                validate_root_statements(&body.stmts, &mut symbols.clone(), call_results, true)?;
            }
            Stmt::Repeat { body, .. } | Stmt::BeatBlock { body, .. } | Stmt::Hook { body, .. } => {
                validate_root_statements(&body.stmts, &mut symbols.clone(), call_results, true)?;
            }
            Stmt::ForEach {
                item,
                item_type,
                iterable,
                body,
                ..
            } => {
                infer_expr(iterable, symbols, call_results, unknown_names_allowed)?;
                let mut nested = symbols.clone();
                nested.insert(
                    item.clone(),
                    item_type
                        .as_ref()
                        .map(type_ref_kind)
                        .unwrap_or(StaticType::Unknown),
                );
                validate_root_statements(&body.stmts, &mut nested, call_results, true)?;
            }
            Stmt::Quantifier {
                variable,
                domain,
                body,
                ..
            } => {
                let mut nested = symbols.clone();
                nested.insert(variable.clone(), type_ref_kind(domain));
                validate_root_statements(&body.stmts, &mut nested, call_results, true)?;
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                for branch in branches {
                    infer_expr(
                        &branch.condition,
                        symbols,
                        call_results,
                        unknown_names_allowed,
                    )?;
                    validate_root_statements(
                        &branch.body.stmts,
                        &mut symbols.clone(),
                        call_results,
                        true,
                    )?;
                }
                validate_root_statements(
                    &else_body.stmts,
                    &mut symbols.clone(),
                    call_results,
                    true,
                )?;
            }
            Stmt::Try { action, body, .. } => {
                infer_expr(action, symbols, call_results, unknown_names_allowed)?;
                validate_root_statements(&body.stmts, &mut symbols.clone(), call_results, true)?;
            }
            Stmt::Contract {
                condition,
                then_body,
                else_body,
                ..
            } => {
                infer_expr(condition, symbols, call_results, unknown_names_allowed)?;
                if let Some(body) = then_body {
                    validate_root_statements(
                        &body.stmts,
                        &mut symbols.clone(),
                        call_results,
                        true,
                    )?;
                }
                validate_root_statements(
                    &else_body.stmts,
                    &mut symbols.clone(),
                    call_results,
                    true,
                )?;
            }
            Stmt::Receive {
                sender,
                condition,
                binding,
                body,
                ..
            } => {
                for expr in sender.iter().chain(condition.iter()) {
                    infer_expr(expr, symbols, call_results, true)?;
                }
                let mut nested = symbols.clone();
                if let Some(name) = binding {
                    nested.insert(name.clone(), StaticType::Unknown);
                }
                validate_root_statements(&body.stmts, &mut nested, call_results, true)?;
            }
            Stmt::Send {
                sender,
                payload,
                receiver,
                ..
            } => {
                for expr in sender.iter().chain([payload, receiver]) {
                    infer_expr(expr, symbols, call_results, unknown_names_allowed)?;
                }
            }
            Stmt::PublicObservation { fields, .. } => {
                for field in fields {
                    infer_expr(&field.value, symbols, call_results, unknown_names_allowed)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

// Lifecycle bodies can bind world resources before authored root views read
// them. Admit those names, not arbitrary unknown names or callable locals.
// Their value/type remains runtime-checked; this does not initialize a resource.
pub(crate) fn collect_world_bindings(
    statements: &[Stmt],
    locals: &BTreeSet<String>,
    names: &mut BTreeSet<String>,
) {
    let mut locals = locals.clone();
    for statement in statements {
        match statement {
            Stmt::Definition { name, .. } => { locals.insert(name.clone()); }
            Stmt::DeclBlock { items, .. } => {
                for item in items { locals.insert(item.name.clone()); }
            }
            Stmt::Mutate { target, .. } => {
                if let ExprKind::Var(name) = &target.kind {
                    if !locals.contains(name) { names.insert(name.clone()); }
                }
            }
            Stmt::If { then_body, else_body, .. } => {
                collect_world_bindings(&then_body.stmts, &locals, names);
                if let Some(body) = else_body { collect_world_bindings(&body.stmts, &locals, names); }
            }
            Stmt::While { body, .. } | Stmt::Guard { body, .. }
            | Stmt::HookWhenBecomes { body, .. } | Stmt::HookWhile { body, .. }
            | Stmt::Repeat { body, .. } | Stmt::BeatBlock { body, .. }
            | Stmt::Hook { body, .. } => collect_world_bindings(&body.stmts, &locals, names),
            Stmt::ForEach { item, body, .. } => {
                let mut nested = locals.clone(); nested.insert(item.clone());
                collect_world_bindings(&body.stmts, &nested, names);
            }
            Stmt::Quantifier { variable, body, .. } => {
                let mut nested = locals.clone(); nested.insert(variable.clone());
                collect_world_bindings(&body.stmts, &nested, names);
            }
            Stmt::Choose { branches, else_body, .. } => {
                for branch in branches { collect_world_bindings(&branch.body.stmts, &locals, names); }
                collect_world_bindings(&else_body.stmts, &locals, names);
            }
            Stmt::Try { body, .. } => {
                let mut nested = locals.clone(); nested.insert("그것".to_string());
                collect_world_bindings(&body.stmts, &nested, names);
            }
            Stmt::Contract { then_body, else_body, .. } => {
                if let Some(body) = then_body { collect_world_bindings(&body.stmts, &locals, names); }
                collect_world_bindings(&else_body.stmts, &locals, names);
            }
            // No expression/thunk/callable traversal: their locals do not
            // become names visible in a root authored view.
            _ => {}
        }
    }
}

fn validate_minimal_root_program(program: &CanonProgram) -> Result<(), StaticDiagnostic> {
    let call_results = program
        .items
        .iter()
        .map(|item| {
            let TopLevelItem::SeedDef(seed) = item;
            (
                seed.canonical_name.clone(),
                seed.result_type
                    .as_ref()
                    .map(type_ref_kind)
                    .unwrap_or(StaticType::Unknown),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let call_results=StaticCallContext {results:call_results,program:Some(program)};
    let mut symbols = BTreeMap::new();
    // The parser relocates authored top-level setup into a tick seed. The
    // runtime executes that setup before root statements; analysis must keep
    // its names and declared types visible there without exporting locals
    // from an ordinary callable's own declaration block.
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if !matches!(seed.canonical_name.as_str(), "매마디" | "매틱") { continue; }
        let Some(body) = &seed.body else { continue; };
        for statement in &body.stmts {
            if seed.is_relocated_top_level_setup(statement) {
                validate_root_statements(std::slice::from_ref(statement), &mut symbols, &call_results, false)?;
            }
        }
    }
    let mut world_names = BTreeSet::new();
    for statement in &program.root_stmts {
        match statement {
            Stmt::DerivedBinding{name,..}=>{world_names.insert(name.clone());},
            Stmt::Hook { body, .. } | Stmt::HookWhenBecomes { body, .. }
            | Stmt::HookWhile { body, .. } =>
                collect_world_bindings(&body.stmts, &BTreeSet::new(), &mut world_names),
            _ => {}
        }
    }
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if matches!(seed.canonical_name.as_str(), "매마디" | "매틱") {
            if let Some(body) = &seed.body {
                let params = seed.params.iter().map(|param| param.pin_name.clone()).collect();
                collect_world_bindings(&body.stmts, &params, &mut world_names);
            }
        }
        // An explicit project entry may own the same lifecycle hooks as a
        // root program. Only their world bindings are visible to root views;
        // ordinary callable locals and parameter names remain private.
        if let Some(body) = &seed.body {
            let params = seed.params.iter().map(|param| param.pin_name.clone()).collect();
            for statement in &body.stmts {
                if let Stmt::Hook { body, .. } = statement {
                    collect_world_bindings(&body.stmts, &params, &mut world_names);
                }
            }
        }
    }
    for name in world_names {
        symbols.entry(name).or_insert(StaticType::Unknown);
    }
    for statement in &program.root_stmts {
        if let Some(role)=ddonirang_lang::proof_authoring::identity_declaration_role(program,statement) {
            // Exact static carrier/operation references are not runtime values.
            // Ordinary unknown names and calls still use the unchanged pass.
            symbols.insert(role.name().to_string(),StaticType::Unknown);
        } else {
            validate_root_statements(std::slice::from_ref(statement),&mut symbols,&call_results,false)?;
        }
    }
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        let mut local = symbols.clone();
        for param in &seed.params {
            local.insert(param.pin_name.clone(), type_ref_kind(&param.type_ref));
        }
        if let Some(body) = &seed.body {
            validate_root_statements(&body.stmts, &mut local, &call_results, true)?;
        }
    }
    Ok(())
}

pub(crate) fn validate_program_for_execution(program: &CanonProgram) -> Result<(), String> {
    validate_minimal_root_program(program)
        .map_err(|error| format!("{}: {}", error.code, error.message))
}

/// A rejected numeric projection must not mask a more precise existing
/// source-category diagnostic. This never retries or admits execution.
pub(crate) fn existing_static_rejection(program:&CanonProgram)->Option<String> {
    existing_static_rejection_diagnostic(program, "").map(|diagnostic| diagnostic.message)
}

/// Typed facts from the same canonical program and static checker invocation.
/// No source reparse or formatted-message recovery is used by the consumer.
pub(crate) fn existing_static_rejection_diagnostic(program:&CanonProgram,path:&str)->Option<crate::preprocess::ExecutionDiagnostic> {
    let error=validate_minimal_root_program(program).err()?;
    let source=&program.origin.source;
    let prefix=source.get(..error.start)?;
    let line=prefix.bytes().filter(|byte|*byte==b'\n').count()+1;
    let line_start=prefix.rfind('\n').map_or(0,|offset|offset+1);
    let column=prefix[line_start..].chars().count()+1;
    let line_end=source[line_start..].find('\n').map_or(source.len(),|offset|line_start+offset);
    Some(crate::preprocess::ExecutionDiagnostic {
        code:error.code,
        message:format!("{}: {} ({}:{})\n{}\n{}^",error.code,error.message,line,column,&source[line_start..line_end]," ".repeat(column-1)),
        source_path:path.into(),span:Some((error.start,error.end)),phase:"execution_parse",
        cause:None,offending_field:None,expected:None,actual:None,
    })
}

#[derive(Clone, Debug, Deserialize)]
pub struct SourceAnalysisRequest {
    pub source_text: String,
    #[serde(default = "default_path")]
    pub path: String,
    pub analysis_generation: u64,
    #[serde(default = "default_profile")]
    pub source_profile_identity: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceAnalysisSpan {
    pub encoding: &'static str,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceAnalysisOriginalSourceSpan {
    pub coordinate_space: &'static str,
    pub encoding: &'static str,
    pub half_open: bool,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceAnalysisRepairProposal {
    pub schema: &'static str,
    pub fix_id: &'static str,
    pub source_revision_identity: String,
    pub analysis_generation: u64,
    pub diagnostic_identity: String,
    pub original_source_span: SourceAnalysisOriginalSourceSpan,
    pub expected_text: String,
    pub replacement_text: String,
    pub proposal_identity: String,
    pub explanation: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceAnalysisDiagnostic {
    pub code: String,
    pub severity: &'static str,
    pub message: String,
    pub path: String,
    pub span: SourceAnalysisSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repair_proposal: Option<SourceAnalysisRepairProposal>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceAnalysisSnapshot {
    pub schema: &'static str,
    pub source_revision_identity: String,
    pub analysis_generation: u64,
    pub source_profile_identity: String,
    pub outcome: &'static str,
    pub diagnostics: Vec<SourceAnalysisDiagnostic>,
    /// Read-only parsed-source fact; absence means unknown, never proved.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_source_present: Option<bool>,
}

fn parsed_proof_source_present(program: &CanonProgram) -> bool {
    fn proof_type(ty: &TypeRef) -> bool {
        match ty {
            TypeRef::AlgebraEquality { .. } | TypeRef::CardCallEquality { .. } | TypeRef::AlgebraNonzero { .. }
                | TypeRef::StructureLaw { .. } | TypeRef::PropositionConjunction(_) => true,
            TypeRef::Tuple(items) => items.iter().any(proof_type),
            TypeRef::Applied { args, .. } => args.iter().any(proof_type),
            TypeRef::Function { params, result } => params.iter().any(proof_type) || proof_type(result),
            TypeRef::Tensor { element, .. } => proof_type(element),
            _ => false,
        }
    }
    program.items.iter().any(|item| {
        let TopLevelItem::SeedDef(seed) = item;
        seed.proof_header.is_some() || seed.result_type.as_ref().is_some_and(proof_type)
            || seed.params.iter().any(|pin| proof_type(&pin.type_ref))
    })
}

fn default_path() -> String {
    "lesson.ddn".to_string()
}

fn default_profile() -> String {
    String::new()
}

fn source_revision_identity(source: &str) -> String {
    format!("blake3:{}", blake3::hash(source.as_bytes()).to_hex())
}

fn deterministic_identity(domain: &str, parts: &[&[u8]]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        hasher.update(&(part.len() as u64).to_be_bytes());
        hasher.update(part);
    }
    format!("{domain}:blake3:{}", hasher.finalize().to_hex())
}

fn raw_chaebi_header_colon_spans(source: &str) -> Option<Vec<(usize, usize)>> {
    let tokens = Lexer::new(source).tokenize().ok()?;
    let mut spans = Vec::new();
    for window in tokens.windows(3) {
        let is_chaebi = match &window[0].kind {
            TokenKind::Ident(name) | TokenKind::Josa(name) => name == "채비",
            _ => false,
        };
        if is_chaebi
            && matches!(window[1].kind, TokenKind::Colon)
            && matches!(window[2].kind, TokenKind::LBrace)
        {
            spans.push((window[1].span.start, window[1].span.end));
        }
    }
    Some(spans)
}

fn is_chaebi_header_warning(warning: &DdnParseWarning) -> bool {
    warning.code == "W_BLOCK_HEADER_COLON_DEPRECATED" && warning.message.contains("`채비:`")
}

fn authoritative_repair_proposal(
    source: &str,
    revision: &str,
    generation: u64,
    occurrence_index: usize,
    original_span: (usize, usize),
) -> Option<(String, SourceAnalysisRepairProposal)> {
    let (start, end) = original_span;
    if source.get(start..end)? != ":" {
        return None;
    }
    let generation_bytes = generation.to_be_bytes();
    let occurrence_bytes = (occurrence_index as u64).to_be_bytes();
    let start_bytes = (start as u64).to_be_bytes();
    let end_bytes = (end as u64).to_be_bytes();
    let diagnostic_identity = deterministic_identity(
        "ddn.source_analysis_diagnostic.candidate.v1",
        &[
            revision.as_bytes(),
            &generation_bytes,
            b"W_BLOCK_HEADER_COLON_DEPRECATED",
            &occurrence_bytes,
            &start_bytes,
            &end_bytes,
        ],
    );
    let proposal_identity = deterministic_identity(
        "ddn.source_analysis_repair_proposal.candidate.v1",
        &[
            revision.as_bytes(),
            diagnostic_identity.as_bytes(),
            &start_bytes,
            &end_bytes,
            b":",
            b"",
        ],
    );
    let proposal = SourceAnalysisRepairProposal {
        schema: REPAIR_PROPOSAL_SCHEMA,
        fix_id: REMOVE_SETUP_COLON_FIX_ID,
        source_revision_identity: revision.to_string(),
        analysis_generation: generation,
        diagnostic_identity: diagnostic_identity.clone(),
        original_source_span: SourceAnalysisOriginalSourceSpan {
            coordinate_space: "original-source",
            encoding: "utf-8-byte",
            half_open: true,
            start,
            end,
        },
        expected_text: ":".to_string(),
        replacement_text: String::new(),
        proposal_identity,
        explanation: "비권장 블록 헤더의 콜론을 제거합니다.",
    };
    Some((diagnostic_identity, proposal))
}

fn accepted_diagnostics(
    source: &str,
    path: &str,
    revision: &str,
    generation: u64,
    warnings: &[DdnParseWarning],
) -> Vec<SourceAnalysisDiagnostic> {
    let repairable_count = warnings
        .iter()
        .filter(|warning| is_chaebi_header_warning(warning))
        .count();
    let raw_spans =
        raw_chaebi_header_colon_spans(source).filter(|spans| spans.len() == repairable_count);
    let mut repairable_ordinal = 0usize;

    warnings
        .iter()
        .map(|warning| {
            let repair = if is_chaebi_header_warning(warning) {
                let ordinal = repairable_ordinal;
                repairable_ordinal += 1;
                raw_spans.as_ref().and_then(|spans| {
                    spans.get(ordinal).and_then(|span| {
                        authoritative_repair_proposal(source, revision, generation, ordinal, *span)
                    })
                })
            } else {
                None
            };
            let (diagnostic_identity, repair_proposal) = match repair {
                Some((identity, proposal)) => (Some(identity), Some(proposal)),
                None => (None, None),
            };
            SourceAnalysisDiagnostic {
                code: warning.code.clone(),
                severity: "warning",
                message: warning.message.clone(),
                path: path.to_string(),
                span: SourceAnalysisSpan {
                    encoding: "utf-8-byte",
                    start: warning.span_start,
                    end: warning.span_end,
                },
                diagnostic_identity,
                repair_proposal,
            }
        })
        .collect()
}

fn diagnostic_code(message: &str) -> String {
    let Some(start) = message.find("E_") else {
        return "E_PARSE".to_string();
    };
    let tail = &message[start..];
    let end = tail
        .char_indices()
        .find_map(|(index, ch)| {
            if index > 0 && !(ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_') {
                Some(index)
            } else {
                None
            }
        })
        .unwrap_or(tail.len());
    tail[..end].to_string()
}

fn byte_offset_for_line_column(source: &str, target_line: usize, target_column: usize) -> usize {
    let mut line = 1usize;
    let mut column = 1usize;
    for (offset, ch) in source.char_indices() {
        if line == target_line && column >= target_column {
            return offset;
        }
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    source.len()
}

pub(crate) fn formatted_error_span(source: &str, message: &str) -> SourceAnalysisSpan {
    let mut start = 0usize;
    if let Some(location_end) = message.find(")\n") {
        if let Some(location_start) = message[..location_end].rfind('(') {
            let location = &message[location_start + 1..location_end];
            if let Some((line, column)) = location.split_once(':') {
                if let (Ok(line), Ok(column)) = (line.parse::<usize>(), column.parse::<usize>()) {
                    start = byte_offset_for_line_column(source, line, column);
                }
            }
        }
    }
    let end = source[start..]
        .chars()
        .next()
        .map(|ch| start + ch.len_utf8())
        .unwrap_or(start);
    SourceAnalysisSpan {
        encoding: "utf-8-byte",
        start,
        end,
    }
}

pub fn analyze_source(request: SourceAnalysisRequest) -> SourceAnalysisSnapshot {
    let revision = source_revision_identity(&request.source_text);
    // Model syntax has its own existing typed frontdoor. Do not send its
    // '= 모형' declaration through the ordinary v25 definition migration check.
    if request.source_profile_identity == V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY
        && ddonirang_lang::has_model_surface(&request.source_text)
    {
        return analyze_model_source(request, revision);
    }
    let program = if request.source_profile_identity == STRICT_SOURCE_PROFILE_IDENTITY {
        DdnProgram::from_source_with_mode(&request.source_text, &request.path, ParseMode::Strict)
    } else {
        let profile = match resolve_supported_source_profile(Some(&request.source_profile_identity))
        {
            Ok(profile) => profile,
            Err(error) => {
                return SourceAnalysisSnapshot {
                    schema: SOURCE_ANALYSIS_SNAPSHOT_SCHEMA,
                    source_revision_identity: revision,
                    analysis_generation: request.analysis_generation,
                    source_profile_identity: request.source_profile_identity,
                    outcome: "UNSUPPORTED",
                    proof_source_present: None,
                    diagnostics: vec![SourceAnalysisDiagnostic {
                        code: error.code.to_string(),
                        severity: "error",
                        message: error.message,
                        path: request.path,
                        span: SourceAnalysisSpan {
                            encoding: "utf-8-byte",
                            start: 0,
                            end: 0,
                        },
                        diagnostic_identity: None,
                        repair_proposal: None,
                    }],
                };
            }
        };
        let program=DdnProgram::from_supported_source_profile_diagnostic_with_host(
            &request.source_text,&request.path,ParseMode::Strict,profile,
            crate::ddn_runtime::default_age_target(),None);
        return finish_typed_source_analysis(request,revision,program);
    };

    finish_source_analysis(request, revision, program)
}

fn analyze_model_source(request: SourceAnalysisRequest, revision: String) -> SourceAnalysisSnapshot {
    let diagnostics = match ddonirang_lang::parse_model_program_with_source_file(&request.source_text, &request.path) {
        Ok(program) => program.ir.diagnostics.iter().map(|item| SourceAnalysisDiagnostic {
            code: item.candidate_code.to_string(), severity: match item.severity {
                "error_candidate" => "error",
                severity => severity,
            },
            message: item.message.clone(), path: request.path.clone(),
            span: SourceAnalysisSpan { encoding: "utf-8-byte", start: item.provenance.start, end: item.provenance.end },
            diagnostic_identity: None, repair_proposal: None,
        }).collect::<Vec<_>>(),
        Err(error) => vec![SourceAnalysisDiagnostic {
            code: error.candidate_code.to_string(), severity: "error", message: error.message,
            path: request.path.clone(),
            span: SourceAnalysisSpan { encoding: "utf-8-byte", start: error.span.start, end: error.span.end },
            diagnostic_identity: None, repair_proposal: None,
        }],
    };
    let outcome = if diagnostics.iter().any(|item| item.severity == "error") { "REJECTED" } else { "ACCEPTED" };
    SourceAnalysisSnapshot { schema: SOURCE_ANALYSIS_SNAPSHOT_SCHEMA, source_revision_identity: revision,
        analysis_generation: request.analysis_generation, source_profile_identity: request.source_profile_identity,
        proof_source_present: if outcome == "ACCEPTED" { Some(false) } else { None }, outcome, diagnostics }
}

/// Compilation-local facts stay bound to the actual supplied context. The
/// existing snapshot wire contains no durable language/profile authority.
pub struct FirstCallableSourceAnalysis {
    snapshot: SourceAnalysisSnapshot,
    context: std::sync::Arc<ddonirang_lang::parser::RegisteredFirstCallableContext>,
}

impl FirstCallableSourceAnalysis {
    pub fn snapshot(&self) -> &SourceAnalysisSnapshot { &self.snapshot }
    pub fn is_current_for(&self, source: &str, generation: u64,
        context: &std::sync::Arc<ddonirang_lang::parser::RegisteredFirstCallableContext>) -> bool {
        self.snapshot.source_revision_identity == source_revision_identity(source)
            && self.snapshot.analysis_generation == generation
            && std::sync::Arc::ptr_eq(&self.context, context)
    }
    pub fn original_declaration(&self) -> Result<ddonirang_lang::ddl::FirstCallableJaDeclarationBinding<'_>, ddonirang_lang::ddl::DdlError> {
        let registration = ddonirang_lang::ddl::FIRST_CALLABLE_JA_REGISTRATION;
        ddonirang_lang::ddl::bind_first_callable_ja_declaration(self.context.original_public(),
            registration.language_profile_identity, registration.registration_revision)
    }
}

pub fn analyze_source_with_first_callable_context(request: SourceAnalysisRequest,
    context: std::sync::Arc<ddonirang_lang::parser::RegisteredFirstCallableContext>) -> FirstCallableSourceAnalysis {
    let snapshot = if resolve_supported_source_profile(Some(&request.source_profile_identity)).is_err() {
        analyze_source(request)
    } else {
        let revision = source_revision_identity(&request.source_text);
        let program = DdnProgram::from_first_callable_registered_source(&request.source_text, &request.path, context.clone());
        finish_source_analysis(request, revision, program)
    };
    FirstCallableSourceAnalysis { snapshot, context }
}

fn finish_typed_source_analysis(request:SourceAnalysisRequest,revision:String,
    program:Result<DdnProgram,crate::preprocess::ExecutionDiagnostic>)->SourceAnalysisSnapshot {
    match program {
        Err(error) if error.span.is_some()=>{
            let (start,end)=error.span.expect("matched mechanical span");
            SourceAnalysisSnapshot {schema:SOURCE_ANALYSIS_SNAPSHOT_SCHEMA,
                source_revision_identity:revision,analysis_generation:request.analysis_generation,
                source_profile_identity:request.source_profile_identity,outcome:"REJECTED",proof_source_present:None,
                diagnostics:vec![SourceAnalysisDiagnostic {code:error.code.into(),severity:"error",message:error.message,
                    path:error.source_path,span:SourceAnalysisSpan{encoding:"utf-8-byte",start,end},
                    diagnostic_identity:None,repair_proposal:None}]}
        },
        Err(error)=>finish_source_analysis(request,revision,Err(error.message)),
        Ok(program)=>finish_source_analysis(request,revision,Ok(program)),
    }
}

fn finish_source_analysis(request: SourceAnalysisRequest, revision: String,
    program: Result<DdnProgram, String>) -> SourceAnalysisSnapshot {
    match program {
        Ok(program) => {
            let mut diagnostics = accepted_diagnostics(
                &request.source_text,
                &request.path,
                &revision,
                request.analysis_generation,
                program.parse_warnings(),
            );
            let mut outcome = "ACCEPTED";
            if request.source_profile_identity == V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY {
                if let Err(error) = validate_minimal_root_program(program.canonical_program()) {
                    outcome = "REJECTED";
                    diagnostics.push(SourceAnalysisDiagnostic {
                        code: error.code.to_string(),
                        severity: "error",
                        message: error.message,
                        path: request.path.clone(),
                        span: SourceAnalysisSpan {
                            encoding: "utf-8-byte",
                            start: error.start,
                            end: error.end,
                        },
                        diagnostic_identity: None,
                        repair_proposal: None,
                    });
                }
            }
            SourceAnalysisSnapshot {
                schema: SOURCE_ANALYSIS_SNAPSHOT_SCHEMA,
                source_revision_identity: revision.clone(),
                analysis_generation: request.analysis_generation,
                source_profile_identity: request.source_profile_identity,
                outcome,
                proof_source_present: Some(parsed_proof_source_present(program.canonical_program())),
                diagnostics,
            }
        }
        Err(message) => {
            let generic=crate::numeric_constraint_plan::generic_source_diagnostic(&request.source_text,&message);
            let code=generic.as_ref().map(|facts|facts.code.to_owned()).unwrap_or_else(||diagnostic_code(&message));
            let spans=generic.map(|facts|facts.spans.into_iter().map(|span|SourceAnalysisSpan {
                encoding:"utf-8-byte",start:span.start,end:span.end,
            }).collect::<Vec<_>>()).unwrap_or_else(||vec![formatted_error_span(&request.source_text,&message)]);
            SourceAnalysisSnapshot {
            schema: SOURCE_ANALYSIS_SNAPSHOT_SCHEMA,
            source_revision_identity: revision,
            analysis_generation: request.analysis_generation,
            source_profile_identity: request.source_profile_identity,
            outcome: if code == "E_MULTILANG_CONTEXT_REQUIRED" { "UNSUPPORTED" } else { "REJECTED" },
            proof_source_present: None,
            diagnostics: spans.into_iter().map(|span|SourceAnalysisDiagnostic {
                code: code.clone(),
                severity: "error",
                span,
                message: message.clone(),
                path: request.path.clone(),
                diagnostic_identity: None,
                repair_proposal: None,
            }).collect(),
        }},
    }
}

pub fn source_analysis_snapshot_json(request_json: &str) -> Result<String, String> {
    let request: SourceAnalysisRequest = serde_json::from_str(request_json)
        .map_err(|error| format!("E_SOURCE_ANALYSIS_REQUEST_INVALID {error}"))?;
    serde_json::to_string(&analyze_source(request))
        .map_err(|error| format!("E_SOURCE_ANALYSIS_SERIALIZE_FAILED {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsed_proof_presence_is_syntax_not_word_or_verdict() {
        let ordinary=analyze_source(request("글 := \"보임\". 글 보여주기.",7,V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
        assert_eq!(ordinary.proof_source_present,Some(false));
        let proof=analyze_source(request("(값:나눔수~을, 근거:(값 =:= 값)보임~로) 확인:(값 =:= 값)보임:셈씨 := { 확인 <- 근거. }.",8,V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
        assert_eq!(proof.proof_source_present,Some(true),"{:?}",proof.diagnostics);
        assert_eq!(proof.analysis_generation,8);
        let broken=analyze_source(request("(값:나눔수~을,",9,V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
        assert_eq!(broken.proof_source_present,None);
    }

    #[test]
    fn vc13_tuple_static_selection_has_position_specific_types() {
        for source in [
            "묶음 := (1,\"가\",참). 결과 := 묶음[0] + 2.",
            "묶음 := (1,\"가\",참). 결과 := 묶음[1..2][0] + \"나\".",
            "묶음 := (1,\"가\"). 묶음[0..2] <- (2,\"나\").",
        ] {
            let snapshot=analyze_source(request(source,0,V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
            assert_eq!(snapshot.outcome,"ACCEPTED","{source}: {:?}",snapshot.diagnostics);
        }
        for source in [
            "묶음 := (1,\"가\"). 위치 := 0. 결과 := 묶음[위치].",
            "묶음 := (1,\"가\"). 결과 := 묶음[2].",
            "묶음 := (1,\"가\"). 결과 := 묶음[1] * 2.",
            "묶음 := (1,\"가\"). 묶음[0..2] <- (2,참).",
        ] {
            let snapshot=analyze_source(request(source,0,V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
            assert_eq!(snapshot.outcome,"REJECTED","{source}: {:?}",snapshot.diagnostics);
        }
    }

    #[test]
    fn vc13_tuple_constant_arithmetic_rejects_dynamic_or_failed_numbers() {
        for expression in ["1 + 0", "2 / 2", "1.5 - 0.5", "0.5 * 2"] {
            let source=format!("묶음 := (1,\"가\"). 결과 := 묶음[({expression})] + \"나\".");
            let snapshot=analyze_source(request(&source,0,V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
            assert_eq!(snapshot.outcome,"ACCEPTED","{source}: {:?}",snapshot.diagnostics);
        }
        for expression in ["위치 + 0", "3 / 2", "1 - 2", "4 / 2", "2 / 0", "2147483647 + 1"] {
            let source=format!("위치 := 1. 묶음 := (1,\"가\"). 결과 := 묶음[({expression})].");
            let snapshot=analyze_source(request(&source,0,V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
            assert_eq!(snapshot.outcome,"REJECTED","{source}: {:?}",snapshot.diagnostics);
            assert_eq!(snapshot.diagnostics[0].code,"E_CHECK_TUPLE_SELECTION","{source}");
            assert!(snapshot.diagnostics[0].span.start>0,"selection diagnostic must not point to the document start");
        }
    }

    #[test]
    fn declared_collection_does_not_hide_known_mixed_members_as_unknown() {
        for source in [
            "목록:(수)차림 := 차림[1,\"가\"].",
            "목록:(수)차림 := 차림[1,2]. 목록 <- 차림[3,\"가\"].",
            "목록:((수)차림)차림 := 차림[차림[1,\"가\"]].",
            "목록:((수)차림)차림 := 차림[차림[1,2]]. 목록[0] <- 차림[3,\"가\"].",
        ] {
            let snapshot = analyze_source(request(source, 0, V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
            assert_eq!(snapshot.outcome, "REJECTED", "{source}: {:?}", snapshot.diagnostics);
            assert_eq!(snapshot.diagnostics[0].code, "E_CHECK_TYPE_MISMATCH");
        }
        for source in [
            "목록:(수)차림 := 차림[1,2]. 목록 <- 차림[3,4].",
            "목록:((수)차림)차림 := 차림[차림[1,2]]. 목록[0] <- 차림[3,4].",
        ] {
            let snapshot = analyze_source(request(source, 0, V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
            assert_eq!(snapshot.outcome, "ACCEPTED", "{source}: {:?}", snapshot.diagnostics);
        }
    }

    #[test]
    fn homogeneous_collection_selection_preserves_element_type() {
        for source in [
            "목록:(수)차림 := 차림[1,2]. 결과 := 목록[0] + \"가\".",
            "목록 := 차림[\"가\",\"나\"]. 결과 := 목록[0] * 2.",
            "목록 := 차림[차림[\"가\"]]. 결과 := 목록[0][0] * 2.",
            "(목록:(수)차림) 계산:수:셈씨 := { 계산 <- 목록[0] + \"가\". }.",
            "목록:(수)차림 := 차림[\"가\"].",
            "목록:(수)차림 := 차림[1,2]. 목록[0] <- \"가\".",
            "목록:(수)차림 := 차림[1,2]. \"가\" -> 목록[0].",
            "목록 := 차림[1,2]. 부분 := 목록[0..1]. 결과 := 부분[0] + \"가\".",
        ] {
            let snapshot = analyze_source(request(source, 0, V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
            assert_eq!(snapshot.outcome, "REJECTED", "{source}: {:?}", snapshot.diagnostics);
            assert_eq!(snapshot.diagnostics[0].code, "E_CHECK_TYPE_MISMATCH");
        }
        for source in [
            "목록:(수)차림 := 차림[1,2]. 결과 := 목록[0] + 3.",
            "목록:(수)차림 := () 차림.",
            "목록 := 차림[차림[\"가\"]]. 결과 := 목록[0][0] + \"나\".",
            "목록 := 차림[1,2]. 부분 := 목록[0..1]. 결과 := 부분[0] + 3.",
            "목록:(수)차림 := 차림[1,2]. 목록[0] <- 3. 4 -> 목록[1].",
        ] {
            let snapshot = analyze_source(request(source, 0, V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
            assert_eq!(snapshot.outcome, "ACCEPTED", "{source}: {:?}", snapshot.diagnostics);
        }
    }

    #[test]
    fn vc04_b01_range_preserves_live_homogeneous_element_owner() {
        for source in [
            "목록 := 차림[1,2]. 부분 := 목록[0..1]. 결과 := 부분[0] + 3. 결과 보여주기.",
            "목록 := 차림[1,2,3]. 별칭 := 목록. 부분 := 별칭[1..3]. 다시 := 부분[0..1]. 결과 := 다시[0] + 3. 결과 보여주기.",
            "목록 := 차림[차림[1,2],차림[3,4]]. 부분 := 목록[1..2]. 결과 := 부분[0][0] + 3. 결과 보여주기.",
            "목록 := 차림[\"가\",\"나\"]. 부분 := 목록[1..2]. 결과 := 부분[0] + \"다\". 결과 보여주기.",
            "목록 := 차림[1,2]. 부분 := 목록[0..1]. 결과 := 부분[0] + 0.2. 결과 보여주기.",
            "목록 := 차림[1,2]. 부분 := 목록[0..1]. 결과:나눔수 := 부분[0] + 1/3. 결과 보여주기.",
            "목록 := 차림[1,2]. 부분 := 목록[0..1]. (부분[0] + 0.2) 보여주기.",
            "목록 := 차림[1,2]. 부분 := 목록[0..2]. 결과 := 부분[1] + 3. 결과 보여주기.",
            "목록 := 차림[10,20,30,40]. 부분 := 목록[1..4]. 결과 := 부분[2] + 3. 결과 보여주기.",
        ] {
            let snapshot=analyze_source(request(source,0,V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
            assert_eq!(snapshot.outcome,"ACCEPTED","{source}: {:?}",snapshot.diagnostics);
        }
        for source in [
            "목록 := 차림[1,2]. 부분 := 목록[0..1]. 결과 := 부분[0] + \"가\".",
            "목록 := 차림[1,\"가\"]. 부분 := 목록[1..2]. 결과 := 부분[0] + 3.",
            "목록 := 차림[1,2]. 부분 := 목록[0..1]. 목록 <- 차림[\"가\",\"나\"]. 결과 := 부분[0] + 3.",
            "목록 := 차림[]. 부분 := 목록[0..0]. 결과 := 부분[0] + 3.",
            "가방 <- (0,차림[1]) 무작위가방.만들기. 미리 <- (가방,1) 무작위가방.미리보기. 부분 := 미리[0..1]. 결과:셈수 := 부분[0].",
        ] {
            let snapshot=analyze_source(request(source,0,V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
            assert_eq!(snapshot.outcome,"REJECTED","{source}: {:?}",snapshot.diagnostics);
        }
    }

    #[test]
    fn text_grapheme_selection_preserves_known_result_type() {
        for source in [
            "글 := \"👨‍👩‍👧‍👦가\". 결과 := 글[0] + 1.",
            "글 := \"가\". 결과 := 글[0] 그리고 참.",
        ] {
            let snapshot = analyze_source(request(source, 0, V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
            assert_eq!(snapshot.outcome, "REJECTED", "{source}: {:?}", snapshot.diagnostics);
            assert_eq!(snapshot.diagnostics[0].code, "E_CHECK_TYPE_MISMATCH");
        }
        for source in [
            "글 := \"👨‍👩‍👧‍👦가\". 결과 := 글[0] + \"나\".",
            "계산:움직씨 := { 결과 := 외부값[0]. }",
        ] {
            let snapshot = analyze_source(request(source, 0, V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
            assert_eq!(snapshot.outcome, "ACCEPTED", "{source}: {:?}", snapshot.diagnostics);
        }
    }

    fn request(source: &str, generation: u64, profile: &str) -> SourceAnalysisRequest {
        SourceAnalysisRequest {
            source_text: source.to_string(),
            path: "lesson.ddn".to_string(),
            analysis_generation: generation,
            source_profile_identity: profile.to_string(),
        }
    }

    #[test]
    fn alpha3_rd136_time_constructors_are_not_rejected_by_retired_alpha2_guard() {
        for source in [
            "시간 := (2) 표준시간.고정마디.",
            "시간 := (-1) 표준시간.상대마디.",
            "시간 := (번호=2) 표준시간.고정마디.",
            "시간 := (차이=-1) 표준시간.상대마디.",
            "계산:움직씨 := { 시간 := (2) 표준시간.고정마디. }.",
        ] {
            let snapshot = analyze_source(request(source, 0, V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
            assert_ne!(snapshot.outcome, "REJECTED", "{source}: {:?}",snapshot.diagnostics);
        }
        let text = analyze_source(request("글 := \"표준시간.고정마디\".", 0, V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
        assert_ne!(text.outcome, "REJECTED");
    }

    #[test]
    fn logical_static_checks_do_not_skip_known_bad_rhs() {
        for source in [
            "결과 := (거짓 그리고 1).",
            "결과 := (참 또는 1).",
            "결과 := (1 그리고 참).",
            "결과 := (거짓 && 1).",
            "결과 := (참 || 1).",
            "계산:움직씨 := { 결과 := (거짓 그리고 1). }",
            "계산:움직씨 := { { 참 }이면 { 결과 := (참 또는 1). }. }",
            "(매마디)마다 { 결과 := (거짓 그리고 1). }",
        ] {
            let snapshot = analyze_source(request(
                source,
                0,
                V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
            ));
            assert_eq!(snapshot.outcome, "REJECTED", "{source}");
            assert_eq!(
                snapshot.diagnostics[0].code, "E_CHECK_TYPE_MISMATCH",
                "{source}: {:?}",
                snapshot.diagnostics
            );
        }
    }

    #[test]
    fn logical_static_checks_accept_boolean_and_runtime_unknown_operands() {
        for source in [
            "결과 := (거짓 그리고 참).",
            "결과 := (참 또는 거짓).",
            "계산:움직씨 := { 결과 := (외부값 그리고 참). }",
            "계산:움직씨 := { 결과 := (외부값 또는 거짓). }",
        ] {
            let snapshot = analyze_source(request(
                source,
                0,
                V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
            ));
            assert_eq!(
                snapshot.outcome, "ACCEPTED",
                "{source}: {:?}",
                snapshot.diagnostics
            );
        }
    }

    #[test]
    fn logical_call_source_is_not_an_accepted_surface() {
        for source in ["결과 := (거짓, 참) 그리고.", "결과 := (참, 거짓) 또는."] {
            let snapshot = analyze_source(request(
                source,
                0,
                V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
            ));
            assert_eq!(snapshot.outcome, "REJECTED");
            assert_eq!(snapshot.diagnostics[0].code, "E_PARSE");
        }
    }

    #[test]
    fn logical_internal_call_ast_checks_both_operands() {
        use ddonirang_lang::{ArgBinding, BindingReason, Span};
        for operator in ["그리고", "또는"] {
            for (rhs, accepted) in [(Literal::Bool(true), true), (Literal::Int(1), false)] {
                let args = [Literal::Bool(false), rhs]
                    .into_iter()
                    .enumerate()
                    .map(|(id, literal)| ArgBinding {
                        id: id as u64,
                        span: Span::new(0, 0),
                        expr: Expr::new(id as u64, Span::new(0, 0), ExprKind::Literal(literal)),
                        josa: None,
                        resolved_pin: None,
                        binding_reason: BindingReason::Positional,
                    })
                    .collect();
                let expr = Expr::new(
                    3,
                    Span::new(0, 0),
                    ExprKind::Call {
                        args,
                        func: operator.to_string(),
                    },
                );
                let result = infer_expr(&expr, &BTreeMap::new(), &StaticCallContext {results:BTreeMap::new(),program:None}, false);
                if accepted {
                    assert_eq!(result.unwrap(), StaticType::Bool);
                } else {
                    assert_eq!(result.unwrap_err().code, "E_CHECK_TYPE_MISMATCH");
                }
            }
        }
    }

    #[test]
    fn unknown_intermediate_rebind_does_not_erase_first_inferred_type() {
        for first in ["값 := 1.", "값 <- 1."] {
            let source = format!("{first} 값 <- (\"a\", 1) 짝맞춤. 값 <- \"가\".");
            let snapshot = analyze_source(request(
                &source,
                0,
                V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
            ));
            assert_eq!(snapshot.outcome, "REJECTED", "{source}");
            assert_eq!(snapshot.diagnostics[0].code, "E_CHECK_TYPE_MISMATCH");
        }
    }

    #[test]
    fn body_checks_preserve_first_none_and_local_definition_shadowing() {
        for source in [
            "계산:움직씨 := { 값 := 없음. 값 <- 1. }",
            "계산:움직씨 := { 값 <- 없음. 값 <- 1. }",
            "계산:움직씨 := { 값 <- 없음. 값 <- 1. 값 <- \"가\". }",
            "값 := 1. 계산:움직씨 := { 값 := \"가\". 값 <- \"나\". }",
        ] {
            let snapshot = analyze_source(request(
                source,
                0,
                V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
            ));
            assert_eq!(
                snapshot.outcome, "ACCEPTED",
                "{source}: {:?}",
                snapshot.diagnostics
            );
        }
        let root = analyze_source(request(
            "값 := 없음. 값 <- 1.",
            0,
            V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
        ));
        assert_eq!(root.outcome, "REJECTED");
        assert_eq!(root.diagnostics[0].code, "E_CHECK_TYPE_MISMATCH");
    }

    #[test]
    fn current_minimal_root_program_rejects_mixed_addition_before_execution() {
        let snapshot = analyze_source(request(
            "\"점수: \" + 10 보여주기.\n",
            0,
            V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
        ));
        assert_eq!(snapshot.outcome, "REJECTED");
        assert_eq!(snapshot.diagnostics[0].code, "E_CHECK_TYPE_MISMATCH");
    }

    #[test]
    fn lifecycle_world_bindings_are_visible_without_callable_local_leaks() {
        for source in [
            "(시작)할때 { 초기화됨 <- 0. }. (매마디)마다 { { 초기화됨 == 0 }이면 { 시간 <- 0. 초기화됨 <- 1. }. 시간 <- 시간 + 1. x <- 시간 * 2. }. x 보여주기.",
            "(시작)할때 { x <- 1. }. x 보여주기.",
        ] {
            let snapshot = analyze_source(request(source, 0, V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
            assert_eq!(snapshot.outcome, "ACCEPTED", "{source}: {:?}", snapshot.diagnostics);
        }
        for source in [
            "계산:움직씨 := { 지역값 := 1. }. 지역값 보여주기.",
            "(매마디)마다 { 지역값 := 1. }. 지역값 보여주기.",
            "(매마디)마다 { x <- 1. }. 없는값 보여주기.",
        ] {
            let snapshot = analyze_source(request(source, 0, V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
            assert_eq!(snapshot.outcome, "REJECTED", "{source}");
            assert_eq!(snapshot.diagnostics[0].code, "E_CHECK_UNKNOWN_NAME", "{source}: {:?}", snapshot.diagnostics);
        }
    }

    #[test]
    fn current_minimal_root_program_rejects_unknown_name_before_execution() {
        let snapshot = analyze_source(request(
            "없는값 보여주기.\n",
            0,
            V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
        ));
        assert_eq!(snapshot.outcome, "REJECTED");
        assert_eq!(snapshot.diagnostics[0].code, "E_CHECK_UNKNOWN_NAME");
    }

    #[test]
    fn current_minimal_root_program_accepts_checked_arithmetic() {
        let snapshot = analyze_source(request(
            "결과 <- 3 + 4.\n결과 보여주기.\n",
            0,
            V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
        ));
        assert_eq!(snapshot.outcome, "ACCEPTED", "{:?}", snapshot.diagnostics);
    }

    #[test]
    fn root_program_reads_parser_relocated_top_level_setup() {
        // A3-P3: top-level setup remains world-owned after the parser moves
        // its declaration block into the tick seed. It is not a local name.
        for source in [
            "채비 { r:나눔수 := (6,9) 나눔수. f:곱수 := (72) 곱수. }. r 보여주기. f 보여주기.",
            "채비 { 값:수 := 2. }. (매마디)마다 { 값 <- 값 + 1. }. 값 보여주기.",
        ] {
            let snapshot = analyze_source(request(source, 0, V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
            assert_eq!(snapshot.outcome, "ACCEPTED", "{:?}", snapshot.diagnostics);
        }
    }

    #[test]
    fn root_setup_preserves_types_and_does_not_export_callable_locals() {
        for (source, code) in [
            ("채비 { 값:글 := \"가\". }. (값 + 1) 보여주기.", "E_CHECK_TYPE_MISMATCH"),
            ("실행:움직씨 := { 채비 { 숨김:수 := 2. }. }. 숨김 보여주기.", "E_CHECK_UNKNOWN_NAME"),
        ] {
            let snapshot = analyze_source(request(source, 0, V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY));
            assert_eq!(snapshot.outcome, "REJECTED", "{:?}", snapshot.diagnostics);
            assert_eq!(snapshot.diagnostics[0].code, code);
        }
    }

    #[test]
    fn current_minimal_root_program_recognizes_runtime_builtin_values() {
        for source in [
            "{ 참 }이면 { 100 보여주기. } 아니면 { 200 보여주기. }.",
            "{ 거짓 }이면 { 100 보여주기. } 아니면 { 200 보여주기. }.",
            "없음 보여주기.",
        ] {
            let snapshot = analyze_source(request(
                source,
                0,
                V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
            ));
            assert_eq!(
                snapshot.outcome, "ACCEPTED",
                "{source}: {:?}",
                snapshot.diagnostics
            );
        }
    }

    #[test]
    fn accepted_snapshot_preserves_identity_generation_and_utf8_warning_span() {
        let source = "매틱:움직씨 = {\n  채비: { 값:수 <- 0. }.\n}";
        let snapshot = analyze_source(request(source, 7, STRICT_SOURCE_PROFILE_IDENTITY));
        assert_eq!(snapshot.outcome, "ACCEPTED");
        assert_eq!(snapshot.analysis_generation, 7);
        assert_eq!(snapshot.source_profile_identity, "strict");
        assert!(snapshot.source_revision_identity.starts_with("blake3:"));
        let warning = snapshot
            .diagnostics
            .iter()
            .find(|row| row.code == "W_BLOCK_HEADER_COLON_DEPRECATED")
            .expect("authoritative warning");
        assert_eq!(warning.severity, "warning");
        assert_eq!(warning.span.encoding, "utf-8-byte");
        assert!(warning.span.end > warning.span.start);
        let proposal = warning
            .repair_proposal
            .as_ref()
            .expect("authoritative repair proposal");
        let original_colon = source.find("채비:").unwrap() + "채비".len();
        assert_eq!(proposal.original_source_span.start, original_colon);
        assert_eq!(proposal.original_source_span.end, original_colon + 1);
        assert_eq!(
            proposal.original_source_span.coordinate_space,
            "original-source"
        );
        assert_eq!(proposal.expected_text, ":");
        assert_eq!(proposal.replacement_text, "");
        assert_eq!(
            warning.diagnostic_identity.as_deref(),
            Some(proposal.diagnostic_identity.as_str())
        );
    }

    fn proposal_spans(snapshot: &SourceAnalysisSnapshot) -> Vec<(usize, usize)> {
        snapshot
            .diagnostics
            .iter()
            .filter_map(|row| row.repair_proposal.as_ref())
            .map(|proposal| {
                (
                    proposal.original_source_span.start,
                    proposal.original_source_span.end,
                )
            })
            .collect()
    }

    #[test]
    fn authoritative_repair_proposals_match_original_a_to_h() {
        let cases: Vec<(&str, String, Vec<usize>)> = vec![
            (
                "A_ASCII",
                "매틱:움직씨 = { 채비: { 값:수 <- 1. }. }".to_string(),
                Vec::new(),
            ),
            (
                "B_KOREAN_COMMENT",
                "// 앞한글\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }".to_string(),
                Vec::new(),
            ),
            (
                "C_NON_BMP_COMMENT",
                "// 😀\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }".to_string(),
                Vec::new(),
            ),
            (
                "D_COMBINING_COMMENT",
                "// e\u{301}\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }".to_string(),
                Vec::new(),
            ),
            (
                "E_MULTIPLE",
                "매틱:움직씨 = { 채비: { 값:수 <- 1. }. 채비: { 둘:수 <- 2. }. }"
                    .to_string(),
                Vec::new(),
            ),
            (
                "F_STRING_DECOY",
                "설정 { 제목: \"채비: {\". }.\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }"
                    .to_string(),
                Vec::new(),
            ),
            (
                "G_COMMENT_DECOY",
                "// 채비: { 주석 decoy\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }"
                    .to_string(),
                Vec::new(),
            ),
            (
                "H_OPAQUE_DECOY",
                "매틱:움직씨 = { 채비 { 이름:글 <- (x=1) 글무늬{\"채비:\"}. }. 채비: { 값:수 <- 1. }. }"
                    .to_string(),
                Vec::new(),
            ),
        ];

        for (case_id, source, _) in cases {
            let expected = if case_id == "E_MULTIPLE" {
                source
                    .match_indices("채비:")
                    .map(|(start, _)| start + "채비".len())
                    .collect::<Vec<_>>()
            } else {
                vec![source.rfind("채비:").unwrap() + "채비".len()]
            };
            let snapshot = analyze_source(request(&source, 41, STRICT_SOURCE_PROFILE_IDENTITY));
            assert_eq!(snapshot.outcome, "ACCEPTED", "{case_id}");
            assert_eq!(
                proposal_spans(&snapshot),
                expected
                    .iter()
                    .map(|start| (*start, *start + 1))
                    .collect::<Vec<_>>(),
                "{case_id}"
            );
            for diagnostic in snapshot
                .diagnostics
                .iter()
                .filter(|row| row.code == "W_BLOCK_HEADER_COLON_DEPRECATED")
            {
                let proposal = diagnostic.repair_proposal.as_ref().expect(case_id);
                assert_eq!(
                    source.get(
                        proposal.original_source_span.start..proposal.original_source_span.end
                    ),
                    Some(":"),
                    "{case_id}"
                );
            }
        }
    }

    #[test]
    fn multiple_occurrence_identities_are_unique_and_deterministic() {
        let source = "매틱:움직씨 = { 채비: { 값:수 <- 1. }. 채비: { 둘:수 <- 2. }. }";
        let first = analyze_source(request(source, 42, STRICT_SOURCE_PROFILE_IDENTITY));
        let repeated = analyze_source(request(source, 42, STRICT_SOURCE_PROFILE_IDENTITY));
        assert_eq!(first, repeated);
        let diagnostics = first
            .diagnostics
            .iter()
            .filter(|row| row.repair_proposal.is_some())
            .collect::<Vec<_>>();
        assert_eq!(diagnostics.len(), 2);
        assert_ne!(
            diagnostics[0].diagnostic_identity,
            diagnostics[1].diagnostic_identity
        );
        assert_ne!(
            diagnostics[0]
                .repair_proposal
                .as_ref()
                .unwrap()
                .proposal_identity,
            diagnostics[1]
                .repair_proposal
                .as_ref()
                .unwrap()
                .proposal_identity
        );
        let first_span = &diagnostics[0]
            .repair_proposal
            .as_ref()
            .unwrap()
            .original_source_span;
        let second_span = &diagnostics[1]
            .repair_proposal
            .as_ref()
            .unwrap()
            .original_source_span;
        assert_ne!(first_span, second_span);
        assert_eq!(source.get(first_span.start..first_span.end), Some(":"));
        assert_eq!(source.get(second_span.start..second_span.end), Some(":"));
        assert!(first_span.start < second_span.start);
        let next_generation = analyze_source(request(source, 43, STRICT_SOURCE_PROFILE_IDENTITY));
        assert_ne!(
            diagnostics[0].diagnostic_identity,
            next_generation.diagnostics[0].diagnostic_identity
        );
    }

    #[test]
    fn string_decoy_is_not_mutated_and_reanalysis_advances_revision() {
        let source = "설정 { 제목: \"채비: {\". }.\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }";
        let snapshot = analyze_source(request(source, 44, STRICT_SOURCE_PROFILE_IDENTITY));
        let proposal = snapshot.diagnostics[0].repair_proposal.as_ref().unwrap();
        let mut edited = source.to_string();
        edited.replace_range(
            proposal.original_source_span.start..proposal.original_source_span.end,
            &proposal.replacement_text,
        );
        assert!(edited.contains("제목: \"채비: {\""));
        assert!(edited.contains("매틱:움직씨 = { 채비 {"));
        assert!(!edited.contains("매틱:움직씨 = { 채비: {"));
        let reanalyzed = analyze_source(request(&edited, 45, STRICT_SOURCE_PROFILE_IDENTITY));
        assert_ne!(
            snapshot.source_revision_identity,
            reanalyzed.source_revision_identity
        );
        assert!(!reanalyzed
            .diagnostics
            .iter()
            .any(|row| row.code == "W_BLOCK_HEADER_COLON_DEPRECATED"));
    }

    #[test]
    fn native_payload_matches_t4h_cross_projection_seal() {
        let source = "설정 { 제목: \"채비: {\". }.\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }";
        let snapshot = analyze_source(request(source, 270, STRICT_SOURCE_PROFILE_IDENTITY));
        let diagnostic = &snapshot.diagnostics[0];
        let proposal = diagnostic.repair_proposal.as_ref().unwrap();
        assert_eq!(
            snapshot.source_revision_identity,
            "blake3:3159cfef3cdbeeeb74ed267fe9cae2a12944661234c0a942b38c1339057abaab"
        );
        assert_eq!(
            diagnostic.diagnostic_identity.as_deref(),
            Some("ddn.source_analysis_diagnostic.candidate.v1:blake3:9ac895408889cb0cd9b7b2aaf73a98eb72637242512f8204fbdc9649c6e79e1f")
        );
        assert_eq!(
            proposal.proposal_identity,
            "ddn.source_analysis_repair_proposal.candidate.v1:blake3:b36a7c471fdcb01548dd56902ec27e449faeaf9bac72da74a7c67c5a248f7158"
        );
        assert_eq!(proposal.original_source_span.start, 60);
        assert_eq!(proposal.original_source_span.end, 61);
        assert_eq!(proposal.expected_text, ":");
        assert_eq!(proposal.replacement_text, "");
    }

    #[test]
    fn association_mismatch_preserves_diagnostic_without_proposal() {
        let source = "매틱:움직씨 = { 채비: { 값:수 <- 1. }. 채비: { 둘:수 <- 2. }. }";
        let warnings = vec![DdnParseWarning {
            code: "W_BLOCK_HEADER_COLON_DEPRECATED".to_string(),
            message: "블록 헤더의 `채비:` 표기는 예정된 비권장입니다.".to_string(),
            span_start: 0,
            span_end: 1,
        }];
        let diagnostics =
            accepted_diagnostics(source, "lesson.ddn", "blake3:mismatch", 46, &warnings);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].diagnostic_identity.is_none());
        assert!(diagnostics[0].repair_proposal.is_none());
    }

    #[test]
    fn rejected_snapshot_is_fail_closed() {
        let snapshot = analyze_source(request(
            "매틱:움직씨 = { 값:수 <- . }",
            8,
            STRICT_SOURCE_PROFILE_IDENTITY,
        ));
        assert_eq!(snapshot.outcome, "REJECTED");
        assert_eq!(snapshot.diagnostics.len(), 1);
        assert_eq!(snapshot.diagnostics[0].severity, "error");
        assert_eq!(snapshot.diagnostics[0].span.encoding, "utf-8-byte");
    }

    #[test]
    fn unsupported_profile_is_not_downgraded() {
        let snapshot = analyze_source(request("값:수 <- 1.", 9, "unknown-profile"));
        assert_eq!(snapshot.outcome, "UNSUPPORTED");
        assert_eq!(snapshot.diagnostics[0].code, "E_SOURCE_PROFILE_UNSUPPORTED");
    }

    const P1A_V1_CORE_PROFILE: &str = "v1-core-v25";

    #[test]
    fn p1a_v1_core_profile_accepts_definition_and_rebinding() {
        let source = "계산:움직씨 := {\n    값:수 := 1.\n    값 <- 2.\n    값 보여주기.\n}\n";
        let snapshot = analyze_source(request(source, 101, P1A_V1_CORE_PROFILE));
        assert_eq!(snapshot.outcome, "ACCEPTED");
        assert!(snapshot.diagnostics.is_empty());
    }

    #[test]
    fn p1a_v1_core_profile_reports_exact_legacy_definition_migration() {
        let source = "계산:움직씨 = {\n}\n";
        let snapshot = analyze_source(request(source, 102, P1A_V1_CORE_PROFILE));
        assert_eq!(snapshot.outcome, "REJECTED");
        assert_eq!(
            snapshot.diagnostics[0].code,
            "E_V25_DEFINITION_MIGRATION_REQUIRED"
        );
    }

    #[test]
    fn p1a_v1_core_profile_rejects_direct_and_mutual_recursion_deterministically() {
        let direct = "첫계산:움직씨 := {\n    () 첫계산하기.\n}\n";
        let mutual = "첫계산:움직씨 := {\n    () 둘계산하기.\n}\n둘계산:움직씨 := {\n    () 첫계산하기.\n}\n";
        for source in [direct, mutual] {
            let first = analyze_source(request(source, 103, P1A_V1_CORE_PROFILE));
            let repeated = analyze_source(request(source, 103, P1A_V1_CORE_PROFILE));
            assert_eq!(first, repeated);
            assert_eq!(first.outcome, "REJECTED");
            assert_eq!(
                first.diagnostics[0].code, "E_V1_CORE_RECURSION_UNSUPPORTED",
                "diagnostic={:?}",
                first.diagnostics
            );
        }
    }

    #[test]
    fn p1a_v1_core_profile_omission_fails_closed_without_default_or_retry() {
        let request_json = serde_json::json!({
            "source_text": "계산:움직씨 := {\n    값:수 := 1.\n}\n",
            "path": "omitted-profile.ddn",
            "analysis_generation": 104
        })
        .to_string();
        let response = source_analysis_snapshot_json(&request_json).expect("structured response");
        let snapshot: serde_json::Value =
            serde_json::from_str(&response).expect("snapshot response");
        assert_eq!(snapshot["outcome"], "UNSUPPORTED");
        assert_eq!(
            snapshot["diagnostics"][0]["code"],
            "E_SOURCE_PROFILE_REQUIRED"
        );
    }
    #[test]
    fn model_current_surface_analysis_reaches_the_existing_shared_result_table() {
        let source = include_str!("../tests/fixtures/model_current_surface.또니");
        let snapshot = analyze_source(request(source, 301, P1A_V1_CORE_PROFILE));
        assert_eq!(snapshot.outcome, "ACCEPTED", "{:?}", snapshot.diagnostics);
        assert_eq!(snapshot.source_revision_identity, source_revision_identity(source));
        assert_eq!(snapshot.analysis_generation, 301);
        let summary = crate::runtime_surface::run_summary_from_supported_source_profile(source, "example.ddn", Some(1), Some(P1A_V1_CORE_PROFILE)).expect("same source, shared runtime");
        let projection = &summary["public_observation_table_projection"];
        assert_eq!(projection["tables"].as_array().expect("real result tables").len(), 1);
    }

    #[test]
    fn model_current_surface_keeps_real_unit_and_parse_diagnostics() {
        let source = "계산=모형{입력:수@m::(들임). 결과:수@s::(그자리,내줌). 결과::구함. 결과 =:= 입력.}.";
        let snapshot = analyze_source(request(source, 302, P1A_V1_CORE_PROFILE));
        assert_eq!(snapshot.outcome, "REJECTED");
        assert!(snapshot.diagnostics.iter().any(|item| item.code == "E_MODEL_UNIT_MISMATCH_CANDIDATE"));
        for item in &snapshot.diagnostics { assert!(item.span.end <= source.len()); }
        let broken = "계산 = 모형 {";
        let snapshot = analyze_source(request(broken, 303, P1A_V1_CORE_PROFILE));
        assert_eq!(snapshot.outcome, "REJECTED");
        assert!(snapshot.diagnostics.iter().all(|item| item.code != "E_V25_DEFINITION_MIGRATION_REQUIRED"));
    }

    #[test]
    fn model_current_surface_does_not_relax_ordinary_definition_or_profile_guards() {
        let ordinary = analyze_source(request("계산:움직씨 = { }", 304, P1A_V1_CORE_PROFILE));
        assert_eq!(ordinary.outcome, "REJECTED");
        assert_eq!(ordinary.diagnostics[0].code, "E_V25_DEFINITION_MIGRATION_REQUIRED");
        let unsupported = analyze_source(request("계산=모형{}.", 305, "unsupported-profile"));
        assert_eq!(unsupported.outcome, "UNSUPPORTED");
    }

}

#[cfg(test)]
mod function_data_same_attempt_span_tests {
 use super::*;
 const BASE:&str="기준요금 (사용량:나눔수~을 --> 나눔수) := 조건별 { 정의역:0 <= 사용량 <= 6. 10 * 사용량:0 <= 사용량 <= 3. 30 + 20 * (사용량 - 3):3 < 사용량 <= 6. }.\n함수:(나눔수 --> 나눔수) := 기준요금.\n";
 #[test]
 fn function_data_key_rejection_uses_actual_original_utf8_expression(){
  for prefix in ["","안내 := \"한글😀\".\n","#제목: 한글 머리\r\n"] {
   for tail in ["(함수) 모음 보여주기.","(함수,3) 짝맞춤 보여주기."] {
    let source=format!("{prefix}{BASE}{tail}");
    let snapshot=analyze_source(SourceAnalysisRequest{source_text:source.clone(),path:"원문위치.또니".into(),source_profile_identity:V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY.into(),analysis_generation:17});
    assert_eq!(snapshot.outcome,"REJECTED","{:?}",snapshot.diagnostics);
    let d=&snapshot.diagnostics[0];assert_eq!(d.code,"E_TYPE_MISMATCH");assert_eq!(d.path,"원문위치.또니");
    assert_eq!(&source[d.span.start..d.span.end],"함수","{source}: {d:?}");
    let expected=source.rfind("(함수").unwrap()+1;assert_eq!((d.span.start,d.span.end),(expected,expected+"함수".len()));
    assert_eq!(snapshot.analysis_generation,17);
   }
  }
 }
 #[test]
 fn corrected_source_and_new_generation_do_not_reuse_the_old_failure(){
  let source=format!("{BASE}([2,4],함수) 바꾸기 보여주기.");
  let snapshot=analyze_source(SourceAnalysisRequest{source_text:source,path:"새원문.또니".into(),source_profile_identity:V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY.into(),analysis_generation:18});
  assert_eq!(snapshot.outcome,"ACCEPTED","{:?}",snapshot.diagnostics);assert_eq!(snapshot.analysis_generation,18);assert!(snapshot.diagnostics.iter().all(|d|d.severity!="error"));
 }
}

// Static lexical capture/effect admission, shared with actual value creation.
// This never issues IDs or evaluates source; runtime captures real resolved values.
pub(crate) fn anonymous_capture_free_names(expr:&Expr,bound:&std::collections::HashSet<String>,out:&mut std::collections::HashSet<String>)->Result<(),String>{
    collect_anonymous_inventory(expr,bound,out,&mut BTreeSet::new(),&mut BTreeMap::new())
}
pub(crate) fn function_expression_inventory(expr:&Expr,bound:&std::collections::HashSet<String>)->Result<(std::collections::HashSet<String>,BTreeSet<ddonirang_lang::NodeId>),String>{
    let mut free=std::collections::HashSet::new();let mut calls=BTreeSet::new();collect_anonymous_inventory(expr,bound,&mut free,&mut calls,&mut BTreeMap::new())?;Ok((free,calls))
}
pub(crate) fn function_statement_inventory(stmts:&[Stmt],bound:&std::collections::HashSet<String>)->Result<(std::collections::HashSet<String>,BTreeSet<ddonirang_lang::NodeId>),String>{
    let mut free=std::collections::HashSet::new();let mut calls=BTreeSet::new();anonymous_block_capture_names(stmts,&mut bound.clone(),&mut free,&mut calls,&mut BTreeMap::new())?;Ok((free,calls))
}
pub(crate) fn function_statement_capture_sites(stmts:&[Stmt],bound:&std::collections::HashSet<String>)->Result<(BTreeMap<ddonirang_lang::NodeId,String>,BTreeSet<ddonirang_lang::NodeId>),String>{
    let mut free=std::collections::HashSet::new();let mut calls=BTreeSet::new();let mut reads=BTreeMap::new();
    anonymous_block_capture_names(stmts,&mut bound.clone(),&mut free,&mut calls,&mut reads)?;Ok((reads,calls))
}
fn collect_anonymous_inventory(expr:&Expr,bound:&std::collections::HashSet<String>,out:&mut std::collections::HashSet<String>,calls:&mut BTreeSet<ddonirang_lang::NodeId>,reads:&mut BTreeMap<ddonirang_lang::NodeId,String>)->Result<(),String>{
    match &expr.kind{
        ExprKind::Literal(Literal::Resource(_))|ExprKind::Suffix{at:ddonirang_lang::AtSuffix::Asset(_),..}=>return Err("E_FUNCTION_BODY_EFFECT: 자원 접근의 실제 효과·권한 공급이 필요합니다".into()),
        ExprKind::Literal(_)=>{},
        ExprKind::Var(name)=>{if !bound.contains(name){out.insert(name.clone());reads.insert(expr.id,name.clone());}},
        ExprKind::Infix{left,right,..}=>{collect_anonymous_inventory(left,bound,out,calls,reads)?;collect_anonymous_inventory(right,bound,out,calls,reads)?;},
        ExprKind::Suffix{value,..}=>collect_anonymous_inventory(value,bound,out,calls,reads)?,
        ExprKind::FieldAccess{target,..}=>collect_anonymous_inventory(target,bound,out,calls,reads)?,
        ExprKind::SelectionPlace{target,index}=>{collect_anonymous_inventory(target,bound,out,calls,reads)?;collect_anonymous_inventory(index,bound,out,calls,reads)?;},
        ExprKind::MultiAxisSelection(selection)=>for child in selection.children(){collect_anonymous_inventory(child,bound,out,calls,reads)?;},
        ExprKind::CollectionConstructor{elements}|ExprKind::TupleConstructor{elements}=>for e in elements{collect_anonymous_inventory(e,bound,out,calls,reads)?;},
        ExprKind::Pack{fields}=>for (_,e) in fields{collect_anonymous_inventory(e,bound,out,calls,reads)?;},
        ExprKind::Call{args,..}=>{calls.insert(expr.id);for arg in args{collect_anonymous_inventory(&arg.expr,bound,out,calls,reads)?;}},
        ExprKind::FunctionBlock{statements,result}=>{let mut nested=bound.clone();anonymous_block_capture_names(&statements.stmts,&mut nested,out,calls,reads)?;collect_anonymous_inventory(result,&nested,out,calls,reads)?;},
        ExprKind::Eval{thunk,..}=>{if let ExprKind::Thunk(body)=&thunk.kind{anonymous_block_capture_names(&body.stmts,&mut bound.clone(),out,calls,reads)?;}else{collect_anonymous_inventory(thunk,bound,out,calls,reads)?;}},
        ExprKind::SeedLiteral{param,body}=>{let mut nested=bound.clone();nested.extend(param.split(',').map(str::trim).filter(|n|!n.is_empty()).map(str::to_owned));collect_anonymous_inventory(body,&nested,out,calls,reads)?;},
        _=>return Err("E_FUNCTION_CAPTURE_EXPRESSION_UNSUPPORTED: 이 함수 본문의 자유변수·효과 연결은 아직 지원하지 않습니다".into()),
    }Ok(())
}
fn anonymous_block_capture_names(stmts:&[Stmt],bound:&mut std::collections::HashSet<String>,out:&mut std::collections::HashSet<String>,calls:&mut BTreeSet<ddonirang_lang::NodeId>,reads:&mut BTreeMap<ddonirang_lang::NodeId,String>)->Result<(),String>{
    for stmt in stmts{match stmt{
        Stmt::Definition{name,value,reaction:None,..}=>{collect_anonymous_inventory(value,bound,out,calls,reads)?;bound.insert(name.clone());},
        Stmt::DeclBlock{items,..}=>for item in items{if let Some(value)=&item.value{collect_anonymous_inventory(value,bound,out,calls,reads)?;}bound.insert(item.name.clone());},
        Stmt::Mutate{target,value,deferred:false,..}=>{
            let ExprKind::Var(name)=&target.kind else{return Err("E_FUNCTION_BODY_EFFECT: 지역 이름 밖의 수정은 명시 효과·권한 공급이 필요합니다".into())};
            if !bound.contains(name){return Err("E_FUNCTION_CAPTURE_MUTATION: 갈무리한 바깥 값은 수정할 수 없습니다".into());}
            collect_anonymous_inventory(value,bound,out,calls,reads)?;
        },
        Stmt::Expr{expr,..}=>collect_anonymous_inventory(expr,bound,out,calls,reads)?,
        Stmt::If{condition,then_body,else_body,..}=>{
            collect_anonymous_inventory(condition,bound,out,calls,reads)?;
            anonymous_block_capture_names(&then_body.stmts,&mut bound.clone(),out,calls,reads)?;
            if let Some(body)=else_body{anonymous_block_capture_names(&body.stmts,&mut bound.clone(),out,calls,reads)?;}
        },
        Stmt::While{condition,body,..}=>{collect_anonymous_inventory(condition,bound,out,calls,reads)?;anonymous_block_capture_names(&body.stmts,&mut bound.clone(),out,calls,reads)?;},
        Stmt::Repeat{body,..}=>anonymous_block_capture_names(&body.stmts,&mut bound.clone(),out,calls,reads)?,
        Stmt::ForEach{item,iterable,body,..}=>{collect_anonymous_inventory(iterable,bound,out,calls,reads)?;let mut nested=bound.clone();nested.insert(item.clone());anonymous_block_capture_names(&body.stmts,&mut nested,out,calls,reads)?;},
        Stmt::Break{..}|Stmt::ContinueLoop{..}=>{},
        _=>return Err("E_FUNCTION_BODY_EFFECT: 이 함수 본문 문장의 효과·호출 공급은 아직 지원하지 않습니다".into()),
    }}Ok(())
}
