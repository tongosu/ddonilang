//! Explicit B1 DDL compilation on the common parser/AST, never execution admission.
use super::*;
use crate::stdlib::logic_callable_registration_r1::{CheckedLogicDefinition,LogicPinRole as P};
use std::sync::Arc;

#[derive(Debug,Clone)]
pub struct RegisteredLogicCallableContext{
    profile:crate::ddl::RegisteredSuccessorProfile,
    definitions:Vec<CheckedLogicDefinition>,
    projections:Vec<SeedDef>,ddl_source:String,ddl_path:String,
    activation:Option<crate::ModuleDirectiveEntry>,
}
#[derive(Debug,Clone)]
pub(crate) struct LogicCallBinding{
    context:Arc<RegisteredLogicCallableContext>,definition:usize,
    pins:Vec<[u8;32]>,arguments:Vec<(NodeId,Span,NodeId,Span)>,
    expression_span:Span,name_span:Span,qualified_name:String,
}
/// Compilation provenance only. Tool source/type/effect owners must admit it
/// against the same actual project before a runtime may consume the binding.
pub struct ResolvedLogicCall<'a>{binding:&'a LogicCallBinding}
impl ResolvedLogicCall<'_>{
    pub fn descriptor(&self)->&CheckedLogicDefinition{&self.binding.context.definitions[self.binding.definition]}
    pub fn pin_identities(&self)->&[[u8;32]]{&self.binding.pins}
    pub fn name_span(&self)->Span{self.binding.name_span}
    pub fn profile(&self)->&crate::ddl::RegisteredSuccessorProfile{&self.binding.context.profile}
}
impl CanonProgram{
    pub fn logic_call_sites(&self)->Vec<(NodeId,Span)>{
        let mut sites=self.origin.logic_callable_calls.iter().map(|(id,b)|(*id,b.expression_span)).collect::<Vec<_>>();
        sites.sort_by_key(|(id,_)|*id);sites
    }
    /// Rebuild compilation-local B1 call bindings after linking. The caller
    /// supplies the verified original module and its existing source map.
    /// This does not supply the runtime's separate source/type proof.
    pub fn carry_logic_module_calls(
        &mut self,
        original: &CanonProgram,
        segment: Span,
        original_span: impl Fn(Span) -> Option<Span>,
    ) -> Result<(), ParseError> {
        let existing = &original.origin.logic_callable_calls;
        let Some(first) = existing.values().next() else { return Ok(()) };
        let linked = collect_logic_calls(self, first.context.clone(), Some(segment))?;
        let mismatch = || failure(segment, "E_LOGIC_LINKED_SOURCE_SITE_MISMATCH");
        let mut by_site = HashMap::new();
        for binding in existing.values() {
            if by_site.insert(binding.expression_span.start, binding).is_some() {
                return Err(mismatch());
            }
        }
        if linked.len() != by_site.len() { return Err(mismatch()); }
        for (node, binding) in &linked {
            let site = original_span(binding.expression_span).ok_or_else(&mismatch)?;
            let source = by_site.remove(&site.start).ok_or_else(&mismatch)?;
            if site != source.expression_span
                || original_span(binding.name_span) != Some(source.name_span)
                || binding.qualified_name != source.qualified_name
                || !Arc::ptr_eq(&source.context, &binding.context)
                || binding.definition != source.definition
                || binding.pins != source.pins
                || binding.arguments.len() != source.arguments.len()
                || binding.arguments.iter().zip(&source.arguments).any(|(a,b)|
                    original_span(a.1) != Some(b.1) || original_span(a.3) != Some(b.3))
                || self.origin.logic_callable_calls.contains_key(node)
            {
                return Err(mismatch());
            }
        }
        self.origin.logic_callable_calls.extend(linked);
        Ok(())
    }

    pub fn resolved_logic_call(&self,call:&Expr)->Result<Option<ResolvedLogicCall<'_>>,ParseError>{
        let Some(binding)=self.origin.logic_callable_calls.get(&call.id)else{return Ok(None)};
        let ExprKind::Call{func,args}=&call.kind else{return Err(failure(call.span,"E_LOGIC_CALL_PROVENANCE"))};
        if call.span!=binding.expression_span||func!=&binding.qualified_name||args.len()!=binding.arguments.len()
            ||args.iter().zip(&binding.arguments).any(|(arg,expected)|(arg.id,arg.span,arg.expr.id,arg.expr.span)!=*expected){
            return Err(failure(call.span,"E_LOGIC_CALL_PROVENANCE"));
        }
        let descriptor=&binding.context.definitions[binding.definition];
        if args.iter().zip(&binding.pins).any(|(arg,id)|{
            let name=[(P::Left,"左"),(P::Right,"右"),(P::Value,"値")].into_iter()
                .find_map(|(role,name)|(descriptor.pin(role)==Some(id)).then_some(name));
            arg.binding_reason!=BindingReason::UserFixed||arg.josa.is_some()||arg.resolved_pin.as_deref()!=name
        }){return Err(failure(call.span,"E_LOGIC_CALL_PROVENANCE"));}
        Ok(Some(ResolvedLogicCall{binding}))
    }
}
fn failure(span:Span,message:&str)->ParseError{ParseError{span,message:message.into()}}
impl RegisteredLogicCallableContext{
    /// Lexical/Definition/Pin projection only. No raw/JSON constructor, source
    /// translation, fabricated SeedDef body, Bool ID or registration issuance.
    pub fn from_exported_ddl(profile:crate::ddl::RegisteredSuccessorProfile,
        definitions:Vec<CheckedLogicDefinition>,source:&str,path:&str,syntax:crate::ddl::CheckedJ1Profile,
    )->Result<Self,ParseError>{
        let whole=Span::new(0,source.len());
        if !profile.is_logic_builtin()||profile.version()!="1"||definitions.len()!=4||profile.surfaces().len()!=4{
            return Err(failure(whole,"E_LOGIC_DDL_PROFILE_KIND_OR_DENOMINATOR"));
        }
        let projections=Parser::parse_j1_ddl_declarations(source,path,syntax)?;
        if projections.len()!=4{return Err(failure(whole,"E_LOGIC_DDL_PUBLIC_DENOMINATOR"));}
        let mut seen=HashSet::new();let mut ids=HashSet::new();let mut all_pins=HashSet::new();
        let owner=definitions[0].owner();
        for definition in &definitions{
            if definition.owner()!=owner||definition.version()!=1||!ids.insert(*definition.definition()){
                return Err(failure(whole,"E_LOGIC_DDL_DEFINITION_OWNER"));
            }
            let id=definition.definition().iter().map(|b|format!("{b:02x}")).collect::<String>();
            let rows=profile.surfaces().iter().filter(|row|row.identity==id&&row.surface==definition.surface()).collect::<Vec<_>>();
            let declared=projections.iter().filter(|seed|seed.canonical_name==definition.surface()).collect::<Vec<_>>();
            if rows.len()!=1||declared.len()!=1||!seen.insert(definition.name()){
                return Err(failure(whole,"E_LOGIC_DDL_PUBLIC_DENOMINATOR"));
            }
            let row=rows[0];let seed=declared[0];
            if seed.seed_kind!=SeedKind::Semssi||seed.body.is_some()||!seed.alternate_forms.is_empty()
                ||!seed.type_params.is_empty()||seed.proof_header.is_some()||!seed.modifiers.is_empty()
                ||seed.result_type!=Some(TypeRef::Named("真偽".into()))||seed.params.len()!=row.pins.len(){
                return Err(failure(seed.span,"E_LOGIC_DDL_DECLARATION_SCOPE"));
            }
            let expected=[(P::Left,"左"),(P::Right,"右"),(P::Value,"値")].into_iter()
                .filter_map(|(role,name)|definition.pin(role).map(|id|(id,name))).collect::<Vec<_>>();
            if row.pins.len()!=expected.len(){return Err(failure(seed.span,"E_LOGIC_DDL_PIN_DENOMINATOR"));}
            for (identity,name) in expected{
                let hex=identity.iter().map(|b|format!("{b:02x}")).collect::<String>();
                let registered=row.pins.iter().filter(|pin|pin.identity==hex&&pin.surface==name&&pin.particle.is_empty()).count();
                let pins=seed.params.iter().filter(|pin|pin.pin_name==name).collect::<Vec<_>>();
                if registered!=1||pins.len()!=1||!all_pins.insert(*identity){return Err(failure(seed.span,"E_LOGIC_DDL_PIN_OWNER"));}
                let pin=pins[0];
                if pin.type_ref!=TypeRef::Named("真偽".into())||pin.optional||pin.default_value.is_some()||!pin.josa_list.is_empty(){
                    return Err(failure(pin.span,"E_LOGIC_DDL_PIN_TYPE_OR_SCOPE"));
                }
            }
        }
        if all_pins.len()!=6||seen!=["그리고","또는","아님","아니다"].into_iter().collect(){return Err(failure(whole,"E_LOGIC_DDL_PUBLIC_DENOMINATOR"));}
        Ok(Self{profile,definitions,projections,ddl_source:source.into(),ddl_path:path.into(),activation:None})
    }
    pub fn with_module_activation(mut self,binding:crate::ModuleDirectiveEntry)->Result<Self,ParseError>{
        if binding.local_name.is_empty()||binding.target.is_empty(){return Err(failure(Span::new(0,0),"E_DDL_ACTIVATION_BINDING_REQUIRED"));}
        self.activation=Some(binding);Ok(self)
    }
    pub fn profile(&self)->&crate::ddl::RegisteredSuccessorProfile{&self.profile}
    pub fn activation(&self)->Option<&crate::ModuleDirectiveEntry>{self.activation.as_ref()}
    pub fn builtin_owner(&self)->&[u8;32]{self.definitions[0].owner()}
    pub fn ddl_source(&self)->&str{&self.ddl_source}
    pub fn ddl_path(&self)->&str{&self.ddl_path}
}
impl Parser{
    /// Resolve exports against actual local callable declarations before admitting B1.
    pub fn logic_callable_module_prefix(source:&str,path:&str,syntax:crate::ddl::CheckedJ1Profile)
        ->Result<crate::ModuleDirectiveDocument,ParseError>{
        Self::logic_callable_module(source,path,syntax,1,false).map(|(document,_)|document)
    }

    fn logic_callable_module(source:&str,path:&str,syntax:crate::ddl::CheckedJ1Profile,next_node:NodeId,resolve_calls:bool)
        ->Result<(crate::ModuleDirectiveDocument,CanonProgram),ParseError>{
        if source.len()>1_048_576{return Err(failure(Span::new(0,0),"E_LOGIC_SOURCE_LIMIT"));}
        // The unlinked parse collects declaration facts only. This second
        // directive check admits neither unresolved imports nor re-exports.
        let (program,_)=Self::parse_j1_module_source(source,path,next_node,syntax,resolve_calls)?;
        let scope=crate::ModuleDirectiveContext::default().with_known_imports(["logic"])
            .with_public_definitions(Self::local_export_targets(&program));
        let document=crate::module_directive::parse_module_directive_prefix_with_j1_profile(source,path,&scope,syntax)?;
        Ok((document,program))
    }

    pub fn parse_with_logic_callable_context(source:&str,path:&str,syntax:crate::ddl::CheckedJ1Profile,
        context:Arc<RegisteredLogicCallableContext>,
    )->Result<CanonProgram,ParseError>{
        let activation=context.activation.as_ref().ok_or_else(||failure(Span::new(0,0),"E_DDL_ACTIVATION_REQUIRED"))?;
        let next_node=context.projections.iter().flat_map(|s|std::iter::once(s.id).chain(s.params.iter().map(|p|p.id))).max().unwrap_or(0)+1;
        // Executable B1 source uses the common call elaboration. Declaration
        // discovery above stays syntax-only; the B1 activation is checked below.
        let (document,mut program)=Self::logic_callable_module(source,path,syntax,next_node,true)?;
        let imports=document.imports.ok_or_else(||failure(Span::new(0,0),"E_DDL_ACTIVATION_REQUIRED"))?;
        let request=imports.translation.ok_or_else(||failure(imports.span,"E_DDL_ACTIVATION_REQUIRED"))?;
        if imports.entries.as_slice()!=std::slice::from_ref(activation)
            ||request.target_language!=context.profile.canonical_name()||request.target_version!=context.profile.version(){
            return Err(failure(imports.span,"E_DDL_ACTIVATION_OWNER_OR_PROFILE"));
        }
        program.origin.logic_callable_calls=collect_logic_calls(&program,context,None)?;
        Ok(program)
    }
}
fn collect_logic_calls(program:&CanonProgram,context:Arc<RegisteredLogicCallableContext>,segment:Option<Span>)
    ->Result<HashMap<NodeId,LogicCallBinding>,ParseError>{
    let in_segment=|span:Span|segment.is_none_or(|range|range.start<=span.start&&span.end<=range.end);
    let activation=context.activation.as_ref().ok_or_else(||failure(Span::new(0,0),"E_DDL_ACTIVATION_REQUIRED"))?;
    let alias=activation.local_name.clone();let prefix=format!("{alias}.");
    let mut bindings=HashMap::new();let mut budget=100_000usize;
    fn expression(expr:&Expr,bound:&HashSet<String>,context:&Arc<RegisteredLogicCallableContext>,alias:&str,prefix:&str,
        origin:&OriginMap,out:&mut HashMap<NodeId,LogicCallBinding>,budget:&mut usize,depth:usize,
    )->Result<(),ParseError>{
        if depth>128||*budget==0{return Err(failure(expr.span,"E_LOGIC_SOURCE_LIMIT"));}*budget-=1;
        match &expr.kind{
            ExprKind::Call{func,args}=>{
                if let Some(surface)=func.strip_prefix(prefix){
                    let name_span=*origin.authored_call_name_spans.get(&expr.id).ok_or_else(||failure(expr.span,"E_LOGIC_SOURCE_CALL_NAME_REQUIRED"))?;
                    if bound.contains(alias){return Err(failure(name_span,"E_LOGIC_ALIAS_SHADOWED"));}
                    let found=context.definitions.iter().enumerate().filter(|(_,d)|d.surface()==surface).collect::<Vec<_>>();
                    if found.len()!=1{return Err(failure(name_span,"E_LOGIC_CALL_UNREGISTERED"));}
                    let (index,descriptor)=found[0];
                    let row=context.profile.surfaces().iter().find(|row|row.surface==surface).ok_or_else(||failure(name_span,"E_LOGIC_CALL_UNREGISTERED"))?;
                    if args.len()!=row.pins.len(){return Err(failure(name_span,"E_LOGIC_CALL_ARITY"));}
                    let mut pins=Vec::new();
                    for arg in args{
                        let pin_span=origin.authored_pin_label_spans.get(&arg.id).copied().unwrap_or(arg.span);
                        if arg.binding_reason!=BindingReason::UserFixed||arg.josa.is_some(){return Err(failure(arg.span,"E_LOGIC_CALL_EXPLICIT_PIN_REQUIRED"));}
                        let pin=arg.resolved_pin.as_ref().ok_or_else(||failure(arg.span,"E_LOGIC_CALL_EXPLICIT_PIN_REQUIRED"))?;
                        let matched=row.pins.iter().filter(|row|&row.surface==pin).collect::<Vec<_>>();
                        if matched.len()!=1{return Err(failure(pin_span,"E_LOGIC_CALL_PIN_OWNER"));}
                        let id=matched[0].identity.as_bytes();let mut raw=[0;32];
                        if id.len()!=64{return Err(failure(pin_span,"E_LOGIC_CALL_PIN_OWNER"));}
                        for (i,pair) in id.chunks_exact(2).enumerate(){
                            let digit=|b:u8|if b<=b'9'{b-b'0'}else{b-b'a'+10};raw[i]=16*digit(pair[0])+digit(pair[1]);
                        }
                        if pins.contains(&raw){return Err(failure(pin_span,"E_LOGIC_CALL_PIN_OWNER"));}
                        pins.push(raw);
                    }
                    descriptor.argument_slots(descriptor.definition(),&pins).map_err(|_|failure(name_span,"E_LOGIC_CALL_PIN_OWNER"))?;
                    out.insert(expr.id,LogicCallBinding{context:context.clone(),definition:index,pins,
                        arguments:args.iter().map(|a|(a.id,a.span,a.expr.id,a.expr.span)).collect(),expression_span:expr.span,name_span,qualified_name:func.clone()});
                }else if context.definitions.iter().any(|d|d.surface()==func)&&!bound.contains(func){
                    return Err(failure(expr.span,"E_LOGIC_QUALIFIED_CALL_REQUIRED"));
                }
                for arg in args{expression(&arg.expr,bound,context,alias,prefix,origin,out,budget,depth+1)?;}
            },
            ExprKind::Infix{left,right,..}=>{expression(left,bound,context,alias,prefix,origin,out,budget,depth+1)?;expression(right,bound,context,alias,prefix,origin,out,budget,depth+1)?;},
            ExprKind::TupleConstructor{elements}|ExprKind::CollectionConstructor{elements}=>{
                for value in elements{expression(value,bound,context,alias,prefix,origin,out,budget,depth+1)?;}
            },
            ExprKind::Literal(_)|ExprKind::Var(_)=>{},
            _=>return Err(failure(expr.span,"E_LOGIC_EXPRESSION_SCOPE")),
        }Ok(())
    }
    let globals=program.items.iter().filter_map(|TopLevelItem::SeedDef(seed)|in_segment(seed.span).then(||seed.canonical_name.clone())).collect::<HashSet<_>>();
    let mut scan=|stmts:&[Stmt],mut bound:HashSet<String>|->Result<(),ParseError>{
        for stmt in stmts{
            match stmt{
                Stmt::Show{expr,..}|Stmt::Expr{expr,..}|Stmt::Return{value:expr,..}=>expression(expr,&bound,&context,&alias,&prefix,&program.origin,&mut bindings,&mut budget,0)?,
                Stmt::Definition{name,value,..}=>{expression(value,&bound,&context,&alias,&prefix,&program.origin,&mut bindings,&mut budget,0)?;bound.insert(name.clone());},
                Stmt::Mutate{target,value,..}=>{
                    expression(target,&bound,&context,&alias,&prefix,&program.origin,&mut bindings,&mut budget,0)?;
                    expression(value,&bound,&context,&alias,&prefix,&program.origin,&mut bindings,&mut budget,0)?;
                    if let ExprKind::Var(name)=&target.kind{bound.insert(name.clone());}
                },
                _=>return Err(failure(Parser::stmt_span(stmt),"E_LOGIC_STATEMENT_SCOPE")),
            }
        }Ok(())
    };
    for TopLevelItem::SeedDef(seed) in &program.items{
        if !in_segment(seed.span){continue;}
        let mut bound=globals.clone();bound.extend(seed.params.iter().map(|p|p.pin_name.clone()));
        if let Some(body)=&seed.body{scan(&body.stmts,bound)?;}
    }
    if segment.is_none(){scan(&program.root_stmts,globals)?;}
    Ok(bindings)
}
