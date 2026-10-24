//! Existing RD-072/073/082 applied Result owner; not a new generic syntax,
//! public identity encoding, Optional branch protocol, or persistence carrier.
use crate::ast::{Span,SumBranchPayload,TypeRef};

pub struct ResolvedAppliedSumOwner {
    pub type_ref:TypeRef,
    /// Compilation-local lookup projection, not a public TypeIdentity wire.
    pub key:String,
    pub branches:Vec<ResolvedSumBranch>,
}

#[derive(Clone)]
pub struct ResolvedSumBranch {
    pub name:String,
    pub payload:Vec<SumBranchPayload>,
}

/// RD-442's existing closed builtin schema. This is a compiler lookup result,
/// not a registered identity or authority; the caller resolves source-visible
/// user declarations before consulting the builtin owner.
pub fn resolve_named_builtin_sum_owner(name:&str)->Option<ResolvedAppliedSumOwner> {
    if name!="가리킴안됨" {return None;}
    Some(ResolvedAppliedSumOwner{type_ref:TypeRef::Named(name.into()),key:name.into(),
        branches:["끊김","세대다름","누리다름","씨다름"].into_iter().map(|name|ResolvedSumBranch{name:name.into(),payload:Vec::new()}).collect()})
}

/// Borrow the actually declared arguments without recursively cloning them.
/// None is the same existing default error type (글), not a free type slot.
pub fn resolve_result_type_arguments<'a>(name:&str,args:&'a [TypeRef])->Result<(&'a TypeRef,Option<&'a TypeRef>),String> {
    if name!="맺음" {
        return Err(format!("E_SUM_OWNER_UNSUPPORTED: `{name}`의 적용 고름씨 소유자 결속이 없습니다"));
    }
    match args {
        [value]=>Ok((value,None)),
        [value,error]=>Ok((value,Some(error))),
        _=>Err("E_SUM_OWNER_TYPE_ARGUMENTS: 맺음에는 된씨와 안된씨가 필요합니다".into()),
    }
}

pub fn resolve_applied_sum_owner(kind:&TypeRef,span:Span)->Result<ResolvedAppliedSumOwner,String> {
    let TypeRef::Applied{name,args}=kind else {return Err("E_SUM_OWNER_UNKNOWN: 적용 고름씨가 필요합니다".into())};
    let (value,error)=resolve_result_type_arguments(name,args)?;
    let args=vec![value.clone(),error.cloned().unwrap_or_else(||TypeRef::Named("글".into()))];
    let branches=vec![
        ResolvedSumBranch{name:"잘됨".into(),payload:vec![SumBranchPayload{binder:"값".into(),binder_span:span,type_ref:args[0].clone()}]},
        ResolvedSumBranch{name:"안됨".into(),payload:vec![SumBranchPayload{binder:"이유".into(),binder_span:span,type_ref:args[1].clone()}]},
    ];
    let type_ref=TypeRef::Applied{name:"맺음".into(),args};
    let key=crate::runtime::format_type_ref(&type_ref);
    Ok(ResolvedAppliedSumOwner{type_ref,key,branches})
}
