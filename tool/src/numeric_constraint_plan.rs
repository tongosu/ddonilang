//! Compilation-local numeric constraints. These keys are not stored identities.
//! Projection and consumers must use source owners, never runtime Value shapes.
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use std::sync::Arc;
use ddonirang_core::platform::RecordFieldType;
use ddonirang_lang::{CanonProgram, Expr, ExprKind, Literal, NodeId, SeedDef, Stmt, TopLevelItem, TypeRef};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NumericKind { Integer, Approximate, Rational, BigInteger, Factor }

impl NumericKind {
    fn bit(self) -> u8 { 1 << self as u8 }

    fn all() -> [Self; 5] { [Self::Integer, Self::Approximate, Self::Rational, Self::BigInteger, Self::Factor] }
    fn from_type(kind: &TypeRef) -> Option<Self> {
        let TypeRef::Named(name) = kind else { return None; };
        Some(match ddonirang_lang::stdlib::canonicalize_type_alias(name) {
            "바른수" => Self::Integer, "셈수" => Self::Approximate,
            "나눔수" => Self::Rational, "큰바른수" => Self::BigInteger, "곱수" => Self::Factor,
            // A coarse 수 label is deliberately not a canonical 셈수 supplier.
            _ => return None,
        })
    }
    pub(crate) fn type_ref(self) -> TypeRef {
        TypeRef::Named(match self {
            Self::Integer => "바른수", Self::Approximate => "셈수",
            Self::Rational => "나눔수", Self::BigInteger => "큰바른수", Self::Factor => "곱수",
        }.into())
    }
}

#[derive(Clone, Debug)]
enum Shape {
    // Private projection of the admitted input-event producer, not a public
    // record type, saved tag, or inference from a runtime Pack's field names.
    InputEvent,
    Numeric(Variable),
    Unit(ddonirang_core::UnitDim),
    // Aliases share immutable source structure. Copying a binding must not
    // recursively duplicate every nested candidate before execution starts.
    Tuple(Arc<[Shape]>),
    Collection(Arc<[Shape]>),
    CollectionSlice(Arc<[Shape]>),
    SelectionCandidates(Arc<[Shape]>),
    Declared(TypeRef),
    ConditionalScalar {site:NodeId,args:Vec<Shape>},
    Bag(BagVariable),
    BagDraw(BagVariable),
    BagElement(BagVariable),
    BagPreview(BagVariable),
    // Binding-local source flow stays live across branches, loops and cached
    // reads. A collection's first initializer is not its complete supplier.
    CollectionFlow(BagVariable),
    // A range's homogeneous source element, distinct from a random bag draw.
    // Its actual source variables remain in the compile-wide constraint graph.
    CollectionRangeElement(BagVariable),
    DeferredSelection { target: Arc<Shape>, index: Option<usize> },
    BagOperation(usize),
    Unknown(NodeId),
}

#[derive(Clone,Copy,Debug)]
struct BagVariable(usize);
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
enum StaticNumericProjection { Position(usize,usize), CollectionElement(usize), Operation(usize) }
#[derive(Debug)]
struct BagNode {parent:usize,rank:u8,seeds:Vec<Shape>,unknown:bool,collection_signature:bool,source_collection:bool}
#[derive(Default,Debug)]
struct BagConstraints {nodes:Vec<BagNode>,operations:Vec<BagOperation>,comparisons:Vec<BagOperation>,static_numeric:BTreeMap<StaticNumericProjection,Variable>}
#[derive(Clone,Debug)]
struct BagOperation {left:Shape,right:Shape,op:String,site:NodeId,literals:[bool;2]}
impl BagConstraints {
    fn variable(&mut self,seed:Option<Shape>)->BagVariable {
        let id=self.nodes.len();self.nodes.push(BagNode{parent:id,rank:0,seeds:seed.into_iter().collect(),unknown:false,collection_signature:false,source_collection:false});BagVariable(id)
    }
    fn root(&self,id:BagVariable)->usize {let mut root=id.0;while self.nodes[root].parent!=root {root=self.nodes[root].parent;}root}
    fn join(&mut self,a:BagVariable,b:BagVariable) {
        let (mut a,mut b)=(self.root(a),self.root(b));if a==b {return;}
        if self.nodes[a].rank<self.nodes[b].rank {std::mem::swap(&mut a,&mut b);}
        self.nodes[b].parent=a;
        if self.nodes[a].rank==self.nodes[b].rank {self.nodes[a].rank+=1;}
        let seeds=std::mem::take(&mut self.nodes[b].seeds);self.nodes[a].seeds.extend(seeds);
        self.nodes[a].unknown|=self.nodes[b].unknown;
        self.nodes[a].collection_signature|=self.nodes[b].collection_signature;
        self.nodes[a].source_collection&=self.nodes[b].source_collection;
    }
    fn reject(&mut self,id:BagVariable) {let root=self.root(id);self.nodes[root].unknown=true;}
}

type Scope = BTreeMap<String, (BindingOwner, Shape)>;

pub(crate) struct ResolvedSourceBuiltinRoles {
    pub(crate) arg_for_role:BTreeMap<&'static str,usize>,
    pub(crate) canonical_order:Vec<usize>,
}

/// Shared source admission and execution binding for the six R1.2 builtins.
/// This is the actual role producer, not a numeric consumer's ordinal guess.
pub(crate) fn resolve_source_builtin_roles(program:&CanonProgram,func:&str,args:&[ddonirang_lang::ArgBinding],shadowed:bool)->Result<Option<ResolvedSourceBuiltinRoles>,String> {
    let canonical=ddonirang_lang::stdlib::canonicalize_stdlib_alias(func);
    if shadowed || !matches!(canonical,"추가"|"토막내기"|"표준장면.원"|"표준장면.다각형"|"표준장면.글"|"표준장면.평면")
        || program.items.iter().any(|TopLevelItem::SeedDef(seed)|seed.callable_surface_names().iter().any(|name|name==func || name==canonical))
        || !crate::work_cost::builtin_work_cost_rule(canonical).is_ok_and(|rule|rule.is_some()) {return Ok(None);}
    let signature=ddonirang_lang::stdlib::minimal_stdlib_sigs().into_iter().find(|signature|signature.name==canonical)
        .ok_or_else(||format!("E_CALL_SIGNATURE_REQUIRED: {canonical}의 기존 서명이 없습니다"))?;
    let mut slots=vec![None;signature.params.len()];
    for (input,arg) in args.iter().enumerate() {
        let slot=if let Some(role)=arg.resolved_pin.as_deref() {
            // Existing parser normalization emits 처음 for the accepted 시작
            // source role (also preserved by lang's fixed-pin regression).
            // This does not add a source alias or rename the VC17 START pin.
            let role=if canonical=="토막내기" && role=="처음" {"시작"}else{role};
            signature.params.iter().position(|name|*name==role).ok_or_else(||format!("E_CALL_PIN_UNKNOWN: {canonical}의 `{role}` 받는자리가 없습니다"))?
        }else{slots.iter().position(Option::is_none).ok_or_else(||format!("E_CALL_ARGUMENT_COUNT: {canonical}의 인자가 너무 많습니다"))?};
        if slots[slot].replace(input).is_some() {return Err(format!("E_CALL_PIN_DUPLICATE: {canonical}의 {} 받는자리가 겹칩니다",signature.params[slot]));}
    }
    if slots.iter().any(Option::is_none) {return Err(format!("E_CALL_REQUIRED_PIN_MISSING: {canonical}의 받는자리가 부족합니다"));}
    let canonical_order=slots.into_iter().map(Option::unwrap).collect::<Vec<_>>();
    Ok(Some(ResolvedSourceBuiltinRoles{arg_for_role:signature.params.iter().copied().zip(canonical_order.iter().copied()).collect(),canonical_order}))
}

fn admitted_collection_signature_kind(kind:&TypeRef)->bool {
    let mut pending=vec![kind];
    while let Some(kind)=pending.pop() {match kind {
        TypeRef::Named(name) if matches!(name.as_str(),"글"|"바른수"|"셈수"|"참거짓")=>{},
        TypeRef::Applied{name,args} if matches!(name.as_str(),"차림"|"있없") && args.len()==1=>pending.push(&args[0]),
        _=>return false,
    }}
    true
}

/// The existing sealed numeric constructor and its signature own the result.
/// A runtime Pack, spelling alone or an ordinary source declaration cannot.
pub(crate) fn sealed_numeric_constructor_result(spelling:&str)->Option<TypeRef> {
    let canonical=ddonirang_lang::stdlib::canonicalize_stdlib_alias(spelling);
    if !matches!(canonical,"큰바른수"|"나눔수"|"곱수")
        || !crate::work_cost::builtin_work_cost_rule(canonical).ok()?.is_some() {return None;}
    let mut result=None;
    for sig in ddonirang_lang::stdlib::minimal_stdlib_sigs() {
        if ddonirang_lang::stdlib::canonicalize_stdlib_alias(sig.name)!=canonical {continue;}
        let kind=TypeRef::Named(ddonirang_lang::stdlib::canonicalize_type_alias(sig.ret).into());
        NumericKind::from_type(&kind)?;
        if result.as_ref().is_some_and(|previous|previous!=&kind) {return None;}
        result=Some(kind);
    }
    result
}

pub(crate) fn numeric_operation_result_type(left:&TypeRef,right:&TypeRef,op:&str)->Option<TypeRef> {
    let (left,right)=(NumericKind::from_type(left)?,NumericKind::from_type(right)?);
    // This projection also serves the legacy exact-number bridge. Do not
    // apply V25's new default division owner to legacy Fixed64 arithmetic.
    if ![left,right].iter().any(|kind|matches!(kind,NumericKind::Rational|NumericKind::BigInteger|NumericKind::Factor)) {return None;}
    numeric_signature_result(left,right,op)
}
fn numeric_signature_result(left:NumericKind,right:NumericKind,op:&str)->Option<TypeRef> {
    let mut graph=NumericConstraints::default();
    let a=graph.variable(DefaultPreference::Integer);let b=graph.variable(DefaultPreference::Integer);
    graph.require(a,left,0).ok()?;graph.require(b,right,0).ok()?;
    let result=graph.operation(a,b,op,0);graph.finish().ok()?;
    (graph.components[graph.root(result)].allowed.count_ones()==1).then(||graph.resolved(result).type_ref())
}

fn numeric_comparison_conflict(left:&TypeRef,right:&TypeRef)->Option<(NumericKind,NumericKind)> {
    let mut pairs=vec![(left,right)];
    while let Some((left,right))=pairs.pop() {
        if let Some((a,b))=NumericKind::from_type(left).zip(NumericKind::from_type(right)) {
            if a!=b {return Some((a,b));}
        }else if let (TypeRef::Applied{name:a,args:left},TypeRef::Applied{name:b,args:right})=(left,right) {
            if a==b && left.len()==right.len() {pairs.extend(left.iter().zip(right));}
        }
    }
    None
}

fn precollectable_initializer(root:&Expr,scope:&Scope,callables:&BTreeMap<String,Vec<&SeedDef>>)->bool {
    let mut pending=vec![root];
    while let Some(expr)=pending.pop() {match &expr.kind {
        ExprKind::Literal(_)=>{},
        ExprKind::Var(name) if scope.contains_key(name) || matches!(name.as_str(),"참"|"거짓"|"없음")=>{},
        ExprKind::TupleConstructor{elements}|ExprKind::CollectionConstructor{elements}=>pending.extend(elements),
        ExprKind::Infix{left,right,..}=>{pending.push(left);pending.push(right);},
        ExprKind::Suffix{value,at:ddonirang_lang::AtSuffix::Unit(_)}=>pending.push(value),
        ExprKind::SelectionPlace{target,index}=>{pending.push(target);pending.push(index);},
        ExprKind::Call{func,args} if func=="표준.범위"=>pending.extend(args.iter().map(|arg|&arg.expr)),
        ExprKind::Call{func,args} if !scope.contains_key(func)
            && crate::work_cost::builtin_work_cost_rule(ddonirang_lang::stdlib::canonicalize_stdlib_alias(func)).is_ok_and(|rule|rule.is_none())
            && callables.get(func).is_some_and(|seeds|seeds.len()==1 && seeds[0].seed_kind==ddonirang_lang::SeedKind::Semssi)
            =>pending.extend(args.iter().map(|arg|&arg.expr)),
        ExprKind::MultiAxisSelection(selection)=>{
            pending.push(&selection.target);
            for selector in &selection.selectors {if let ddonirang_lang::TensorAxisSelector::Index(index)=selector {pending.push(index);}}
        }
        _=>return false,
    }}
    true
}

fn has_concrete_numeric_element_owner(mut kind: &TypeRef) -> bool {
    loop {
        if NumericKind::from_type(kind).is_some() { return true; }
        match kind {
            TypeRef::Applied { name, args }
                if ddonirang_lang::stdlib::canonicalize_type_alias(name) == "차림" && args.len() == 1 => kind = &args[0],
            _ => return false,
        }
    }
}

pub(crate) fn literal_arithmetic_tree(expression: &Expr) -> bool {
    let mut pending = vec![expression];
    while let Some(expression) = pending.pop() {
        if let Some(operand) = expression.source_negated_operand() { pending.push(operand); continue; }
        match &expression.kind {
            ExprKind::Literal(_) if expression.numeric_literal_source.is_some() => {},
            ExprKind::Infix { left, op, right } if matches!(op.as_str(), "+" | "-" | "*" | "/") => {
                pending.push(right); pending.push(left);
            }
            _ => return false,
        }
    }
    true
}

/// A producer owns this plan for exactly one admitted compilation. Neither
/// NodeIds nor the private BindingOwner may escape into saved TypeIdentity.
#[derive(Default, Debug)]
pub(crate) struct NumericConstraintPlan {
    pub(crate) subject_reference_targets:BTreeMap<NodeId,NodeId>,
    pub(crate) expressions: BTreeMap<NodeId, TypeRef>,
    pub(crate) literals: BTreeMap<NodeId, NumericKind>,
    pub(crate) bindings: BTreeMap<BindingOwner, TypeRef>,
    pub(crate) position_types: BTreeMap<NodeId, RecordFieldType>,
}

struct Projection<'a> {
    program: &'a CanonProgram,
    graph: NumericConstraints,
    bags:BagConstraints,
    shapes: BTreeMap<NodeId, Shape>,
    literal_sites: BTreeMap<NodeId, Variable>,
    exact_integer_literals:BTreeMap<NodeId,bool>,
    bindings: BTreeMap<BindingOwner, Shape>,
    callables: BTreeMap<String, Vec<&'a SeedDef>>,
    literal_trees: BTreeSet<NodeId>,
    records: ddonirang_lang::runtime::RecordSchemas,
    payload_fields:BTreeMap<String,Option<BTreeMap<String,TypeRef>>>,
    nominal_fields:BTreeMap<String,BTreeMap<String,TypeRef>>,
    sums:BTreeMap<String,Vec<ddonirang_lang::SumBranchDecl>>,
    builtin_results: BTreeMap<String, Vec<TypeRef>>,
    callable_globals: Scope,
    analyzed_callables: BTreeSet<NodeId>,
    active_callables: BTreeSet<NodeId>,
    callable_returns: BTreeMap<NodeId, Shape>,
    source_sites:BTreeMap<NodeId,ddonirang_lang::Span>,
    lexical_locals:BTreeSet<BindingOwner>,
    pattern_binding_types:BTreeMap<BindingOwner,TypeRef>,
    inferred_absences:BTreeSet<BindingOwner>,
    issued_numeric_variables:BTreeSet<usize>,
    consumed_conditionals:Vec<(NodeId,Shape,Variable)>,
    selection_equalities:Vec<(NodeId,Shape,Shape)>,
    scene_proofs:Vec<(NodeId,Shape,bool)>,
    root_world_hook:bool,
}

impl NumericConstraintPlan {
    pub(crate) fn compile(program: &CanonProgram, world_decls: &HashSet<String>) -> Result<Self, String> {
        Self::compile_with_host(program,world_decls,None)
    }

    pub(crate) fn compile_with_host(program:&CanonProgram,world_decls:&HashSet<String>,host:Option<&crate::runtime_surface::BogaeHostNumericSupply>)->Result<Self,String> {
        let mut projection = Projection {
            program, graph: NumericConstraints::default(), bags:BagConstraints::default(), shapes: BTreeMap::new(),
            literal_sites: BTreeMap::new(), exact_integer_literals:BTreeMap::new(), bindings: BTreeMap::new(), callables: BTreeMap::new(),
            literal_trees: BTreeSet::new(), records: BTreeMap::new(), payload_fields:BTreeMap::new(),nominal_fields:BTreeMap::new(),
            sums:program.root_stmts.iter().filter_map(|stmt|match stmt {
                Stmt::SumDefinition{name,branches,..}=>Some((name.clone(),branches.clone())),_=>None,
            }).collect(),builtin_results:BTreeMap::new(),
            callable_globals:Scope::new(),analyzed_callables:BTreeSet::new(),active_callables:BTreeSet::new(),callable_returns:BTreeMap::new(),
            source_sites:BTreeMap::new(),
            lexical_locals:BTreeSet::new(),pattern_binding_types:BTreeMap::new(),inferred_absences:BTreeSet::new(),issued_numeric_variables:BTreeSet::new(),
            consumed_conditionals:Vec::new(),selection_equalities:Vec::new(),scene_proofs:Vec::new(),root_world_hook:false,
        };
        for signature in ddonirang_lang::stdlib::minimal_stdlib_sigs() {
            let name=ddonirang_lang::stdlib::canonicalize_stdlib_alias(signature.name);
            let result=ddonirang_lang::stdlib::canonicalize_type_alias(signature.ret);
            let numeric=NumericKind::from_type(&TypeRef::Named(result.into())).is_some();
            if numeric || matches!(result,"글"|"참거짓"|"없음"|"보기값"|"장면값"|"장면요소값"|"보개짜임값"|"으뜸보개관계값"|"겹보개관계값"|"곁보개관계값"|"돋보개관계값"|"보개시간값") {
                let rows=projection.builtin_results.entry(name.into()).or_default();
                let result=TypeRef::Named(result.into());
                if !rows.contains(&result) {rows.push(result);}
            }
        }
        for TopLevelItem::SeedDef(seed) in &program.items {
            for name in seed.callable_surface_names() { projection.callables.entry(name).or_default().push(seed); }
            if seed.seed_kind == ddonirang_lang::SeedKind::Product {
                projection.records.insert(seed.canonical_name.clone(), seed.params.iter().map(|pin| (pin.pin_name.clone(), pin.type_ref.clone())).collect());
            }
            if matches!(seed.seed_kind,ddonirang_lang::SeedKind::Schema(_)){
                projection.nominal_fields.insert(seed.canonical_name.clone(),seed.params.iter().map(|pin|(pin.pin_name.clone(),pin.type_ref.clone())).collect());
            }
            if matches!(&seed.seed_kind,ddonirang_lang::SeedKind::Named(kind) if kind=="알림씨") {
                let fields=seed.params.iter().map(|pin|(pin.pin_name.clone(),pin.type_ref.clone())).collect();
                projection.payload_fields.entry(seed.canonical_name.clone()).and_modify(|value|*value=None).or_insert(Some(fields));
            }
        }
        // The existing source admission supplies these names from authored
        // top-level 채비, not from runtime resources or a caller's locals.
        let mut globals = Scope::new();
        for TopLevelItem::SeedDef(seed) in &program.items {
            if let Some(body) = &seed.body {
                for statement in &body.stmts {
                    if let Stmt::DeclBlock {items,..}=statement {
                        for item in items.iter().filter(|item|world_decls.contains(&item.name)) {
                            projection.declaration_item(program.id,&mut globals,item)?;
                        }
                    }
                    if let Stmt::Definition {name,..}=statement {
                        if world_decls.contains(name) { projection.statements(program.id,&mut globals,std::slice::from_ref(statement),None)?; }
                    }
                }
            }
        }
        // Collect literal initializer owners before world-writing entry bodies
        // and authored observation expressions. This is constraint collection,
        // not an execution-order change or a second evaluation of initializers.
        for statement in &program.root_stmts {
            projection.callable_globals.extend(globals.clone());
            if let Stmt::DeclBlock{items,..}=statement {
                for item in items {projection.declaration_item(program.id,&mut globals,item)?;}
                continue;
            }
            let value=match statement {
                Stmt::Definition{value,..}=>Some(value),
                Stmt::Mutate{target,value,..} if matches!(target.kind,ExprKind::Var(_))=>Some(value),
                _=>None,
            };
            if value.is_some_and(|value|precollectable_initializer(value,&globals,&projection.callables)) {
                projection.statements(program.id,&mut globals,std::slice::from_ref(statement),None)?;
            }
        }
        projection.callable_globals.extend(globals.clone());
        // Root lifecycle hooks own world writes too. Collect their source
        // owners before root observation definitions, without running them.
        if host.is_some() && !globals.contains_key("프레임수") {
            let shape=projection.declared(&NumericKind::Integer.type_ref(),program.id)?;
            let owner=BindingOwner{scope:program.id,declaration:program.id};
            globals.insert("프레임수".into(),(owner,shape.clone()));
            projection.callable_globals.insert("프레임수".into(),(owner,shape));
        }
        for start in [true,false] {
            for statement in &program.root_stmts {
                if matches!(statement,Stmt::Hook{kind,..} if (*kind==ddonirang_lang::HookKind::Start)==start) {
                    projection.statements(program.id,&mut globals,std::slice::from_ref(statement),None)?;
                }
            }
        }
        globals.extend(projection.callable_globals.clone());
        // Parameterless 움직씨 can initialize an implicit world slot which
        // another named 움직씨 reads. Collect literal-only writes across all
        // such declarations before analyzing bodies: source declaration order
        // must not decide whether that slot has a static numeric owner.
        // Local definitions/declared locals and result places are excluded.
        let no_scope=Scope::new();let no_calls=BTreeMap::new();
        for TopLevelItem::SeedDef(seed) in &program.items {
            if seed.seed_kind!=ddonirang_lang::SeedKind::Umjikssi || !seed.params.is_empty() {continue;}
            let Some(body)=&seed.body else {continue;};
            let mut local_names=BTreeSet::new();
            for statement in &body.stmts {match statement {
                Stmt::Definition{name,..}=>{local_names.insert(name.clone());},
                Stmt::DeclBlock{items,..}=>local_names.extend(items.iter().map(|item|item.name.clone())),
                _=>{},
            }}
            let mut scope=globals.clone();
            for statement in &body.stmts {
                if let Stmt::Mutate{target,value,..}=statement {
                    if let ExprKind::Var(name)=&target.kind {
                        if name!=&seed.canonical_name && !local_names.contains(name) && precollectable_initializer(value,&no_scope,&no_calls) {
                            projection.statements(seed.id,&mut scope,std::slice::from_ref(statement),Some(seed))?;
                        }
                    }
                }
            }
            globals.extend(projection.callable_globals.clone());
        }
        for TopLevelItem::SeedDef(seed) in &program.items {
            if seed.seed_kind==ddonirang_lang::SeedKind::Umjikssi && seed.params.is_empty() {
                projection.analyze_callable(seed,None)?;
            }
        }
        globals.extend(projection.callable_globals.clone());
        let mut root = globals.clone();
        projection.statements(program.id, &mut root, &program.root_stmts, None)?;
        for statement in &program.root_stmts {
            if let Stmt::Definition{name,..}=statement {
                if let Some(binding)=root.get(name) {globals.insert(name.clone(),binding.clone());}
            }
            if let Stmt::Mutate {target,..}=statement {
                if let ExprKind::Var(name)=&target.kind {
                    if let Some(binding)=root.get(name) {
                        if matches!(&binding.1,Shape::Numeric(_)|Shape::Tuple(_)|Shape::Collection(_)|Shape::CollectionFlow(_)|Shape::DeferredSelection{..}) {globals.insert(name.clone(),binding.clone());}
                    }
                }
            }
        }
        projection.callable_globals=globals.clone();
        // Callable locals are not cloned from the caller or root locals. Only
        // explicit static pin/result owners are supplied at this boundary.
        for TopLevelItem::SeedDef(seed) in &program.items {
            // Coarse collection pins require an actual source call supplier.
            // Do not execute a body with a fabricated homogeneous element type.
            if !seed.params.iter().any(|pin|matches!(&pin.type_ref,TypeRef::Named(name) if matches!(name.as_str(),"차림"|"무작위가방"))) {
                projection.analyze_callable(seed,None)?;
            }
        }
        // Static positions are scalar source paths, not homogeneous T issued
        // by a result consumer. Build their existing numeric equations before
        // solving: seeded state updates (RK4 etc.) may refer to earlier reads
        // of the same position. Every incoming source still participates.
        let scalar_uses=projection.consumed_conditionals.clone();
        for (_,shape,result) in scalar_uses {
            let mut anchors=0;
            if projection.static_numeric_sources(&shape,&mut BTreeSet::new(),&mut anchors) && anchors>0 {
                let source=projection.project_static_numeric(&shape)?;
                projection.graph.same(source,result)?;
            }
        }
        // Statically supplied range elements are part of the same arithmetic
        // constraint component even when the result is printed directly.
        // Project the whole supported operation before independent defaulting;
        // unknown/random-bag evidence still cannot acquire a consumer's type.
        for index in 0..projection.bags.operations.len() {
            let operation=Shape::BagOperation(index);
            let mut anchors=0;
            if projection.static_numeric_sources(&operation,&mut BTreeSet::new(),&mut anchors) && anchors>0 {
                projection.project_static_numeric(&operation)?;
            }
        }
        for operation in projection.bags.operations.clone() {
            for shape in [operation.left,operation.right] {
                let mut anchors=0;
                if projection.static_numeric_sources(&shape,&mut BTreeSet::new(),&mut anchors) && anchors>0 {projection.project_static_numeric(&shape)?;}
            }
        }
        let mut independent_bag_owners=Vec::new();
        if !projection.consumed_conditionals.is_empty() || !projection.selection_equalities.is_empty() || !projection.bags.operations.is_empty() || !projection.bags.comparisons.is_empty() || !projection.scene_proofs.is_empty() {
            // Resolve the inputs without the conditional builtin's output
            // requirement. A self-assignment must not certify its own pins.
            let mut independent=projection.graph.clone();independent.finish()?;
            for (site,shape,pair) in &projection.scene_proofs {
                let resolved=Projection::finish_shape_in(shape,&independent,&projection.bags);
                let scalar=if *pair {
                    match resolved.as_ref() {Some(TypeRef::Applied{name,args}) if name=="차림" && args.len()==1=>Some(&args[0]),_=>None}
                }else{resolved.as_ref()};
                if !matches!(scalar.and_then(NumericKind::from_type),Some(NumericKind::Integer|NumericKind::Approximate)) {
                    return Err(projection.failure(*site,"장면은 독립적으로 발급된 바른수/셈수 좌표만 받습니다; 선택·연산 결과를 암묵 변환하지 않습니다"));
                }
            }
            for id in 0..projection.bags.nodes.len() {
                independent_bag_owners.push((BagVariable(id),Projection::finish_shape_in(&Shape::BagElement(BagVariable(id)),&independent,&projection.bags)));
            }
            for (index,operation) in projection.bags.operations.iter().enumerate() {
                if Projection::finish_shape_in(&Shape::BagOperation(index),&independent,&projection.bags).is_none() {
                    return Err(projection.failure(operation.site,"가방 연산에는 독립 동일 후보씨와 지원된 연산 서명이 필요합니다"));
                }
            }
            for operation in projection.bags.operations.iter().chain(&projection.bags.comparisons) {
                let operands=[&operation.left,&operation.right];
                let kinds=operands.map(|shape|Projection::finish_shape_in(shape,&independent,&projection.bags));
                for index in 0..2 {
                    if operation.literals[index] {
                        if let (Shape::Numeric(variable),Some(kind))=(operands[index],kinds[1-index].as_ref().and_then(NumericKind::from_type)) {
                            projection.graph.require(*variable,kind,operation.site)?;
                        }
                    }
                }
            }
            for operation in &projection.bags.comparisons {
                let mut active=BTreeSet::new();
                let (left,right)=Projection::bag_operation_operands(operation,&independent,&projection.bags,&mut active);
                if let Some((a,b))=left.as_ref().zip(right.as_ref()).and_then(|(left,right)|numeric_comparison_conflict(left,right)) {
                    return Err(format!("E_NUMERIC_COMPARISON_TYPE_MISMATCH: {} 비교의 가방 원소 씨는 {:?}/{:?}입니다. 자동 변환하지 않습니다",operation.op,a.type_ref(),b.type_ref()));
                }
                if left.is_none() || right.is_none() {return Err(projection.failure(operation.site,"가방 비교에 독립적으로 동일한 원 후보씨가 필요합니다"));}
            }
            for (site,left_shape,right_shape) in &projection.selection_equalities {
                let left=Projection::finish_shape_in(left_shape,&independent,&projection.bags);
                let right=Projection::finish_shape_in(right_shape,&independent,&projection.bags);
                if left.is_none() || left!=right {
                    let label=|shape:&Shape,kind:&Option<TypeRef>|match kind {
                        Some(TypeRef::Named(name))=>format!("씨 {name}"),
                        Some(TypeRef::Applied{name,args})=>format!("적용씨 {name}/{}",args.len()),
                        Some(_)=>"구조씨".into(),
                        None=>match shape {Shape::Unknown(site)=>format!("미공급 노드 {site}"),Shape::SelectionCandidates(items)=>format!("미해소 원후보 {}개",items.len()),_=>"미해소".into()},
                    };
                    return Err(projection.failure(*site,&format!("선택 결과의 원후보 씨가 독립적으로 같아야 합니다 ({}/{}). 차림 선택을 수 값으로 간주하거나 소비자의 결과씨로 원후보를 바꾸지 않습니다",label(left_shape,&left),label(right_shape,&right))));
                }
            }
            for (site,shape,_) in &projection.consumed_conditionals {
                if Projection::finish_shape_in(shape,&independent,&projection.bags).as_ref().and_then(NumericKind::from_type).is_none() {
                    let detail=match shape {
                        Shape::BagElement(id)=>{let root=projection.bags.root(*id);let node=&projection.bags.nodes[root];format!("가방 원후보 owner={root}, 공급 {}개, 미해소={}",node.seeds.len(),node.unknown)},
                        Shape::SelectionCandidates(items)=>format!("선택 원후보 {}개",items.len()),
                        Shape::ConditionalScalar{site,..}=>format!("조건부 수학 호출 노드 {site}"),
                        _=>"파생 수 연산".into(),
                    };
                    return Err(projection.failure(*site,&format!("수 결과씨에는 독립적으로 해소된 원문 입력 owner가 필요합니다 ({detail}). 선택 후보가 모두 같은 씨이거나 atan2의 셈수 인자 두 개여야 하며 자동 변환은 없습니다")));
                }
            }
            for (site,shape,result) in &projection.consumed_conditionals {
                let kind=Projection::finish_shape_in(shape,&independent,&projection.bags).as_ref().and_then(NumericKind::from_type).expect("checked independent owner");
                projection.graph.require(*result,kind,*site)?;
            }
        }
        projection.graph.finish()?;
        for (owner,independent) in independent_bag_owners {
            if projection.finish_shape(&Shape::BagElement(owner))!=independent {
                return Err("E_TYPE_MISMATCH: 가방 결과의 소비자 제약으로 원 후보씨를 역발급할 수 없습니다".into());
            }
        }
        for (site,shape,result) in &projection.consumed_conditionals {
            if projection.finish_shape(shape)!=Some(projection.graph.resolved(*result).type_ref()) {return Err(projection.failure(*site,"독립 입력 owner가 결과 해소 뒤 변경됐습니다"));}
        }
        let mut result = Self::default();
        for (site, shape) in &projection.shapes {
            if let Some(kind) = projection.finish_shape(shape) { result.expressions.insert(*site, kind); }
            if let Some(kind) = projection.finish_descriptor(shape) { result.position_types.insert(*site, kind); }
        }
        for site in program.origin.subject_reference_calls.keys(){
            let Some(TypeRef::Applied{name,args})=result.expressions.get(site) else {continue};
            if !matches!(name.as_str(),"가리킴"|"맺음"){return Err(projection.failure(*site,"참조 동작의 결과 씨가 다릅니다"));}
            let Some(TypeRef::Named(owner))=args.first() else {return Err(projection.failure(*site,"임자 씨가 해소되지 않았습니다"));};
            let seed=program.items.iter().map(|TopLevelItem::SeedDef(seed)|seed).find(|seed|
                seed.canonical_name==*owner&&seed.seed_kind==ddonirang_lang::SeedKind::Schema(ddonirang_lang::SchemaCategory::Subject))
                .ok_or_else(||projection.failure(*site,"임자 선언 owner가 없습니다"))?;
            result.subject_reference_targets.insert(*site,seed.id);
        }
        for (site, variable) in &projection.literal_sites {
            let kind=projection.graph.resolved(*variable);
            if kind==NumericKind::Integer && projection.exact_integer_literals.get(site)==Some(&false) {
                return Err(projection.failure(*site,"바른수 상수가 기존 Q32 운반에 정확히 표현되지 않습니다; 포화하지 않습니다"));
            }
            result.literals.insert(*site,kind);
        }
        for (owner, shape) in &projection.bindings {
            if let Some(kind) = projection.finish_shape(shape) { result.bindings.insert(*owner, kind); }
        }
        Ok(result)
    }
}

impl Projection<'_> {
    fn admitted_input_event(&self, shape: &Shape) -> bool {
        match shape {
            Shape::InputEvent => true,
            Shape::SelectionCandidates(items) => !items.is_empty() && items.iter().all(|item| self.admitted_input_event(item)),
            Shape::DeferredSelection { target, index } => self.source_selections(target, *index, &mut BTreeSet::new())
                .is_some_and(|items| !items.is_empty() && items.iter().all(|item| matches!(item, Shape::InputEvent))),
            _ => false,
        }
    }
    fn position_source(&self,shape:&Shape,index:usize)->Option<Shape> {
        match shape {
            Shape::Collection(items)=>items.get(index).cloned(),
            // Slice items carry possible element owners, not the runtime
            // slice's length or ordinal positions. One homogeneous owner is
            // valid at every admitted index; runtime owns the actual bounds.
            Shape::CollectionSlice(items)=>match items.as_ref() {
                [only]=>Some(only.clone()),
                []=>None,
                _=>Some(Shape::SelectionCandidates(items.clone())),
            },
            Shape::CollectionFlow(_)=>Some(Shape::DeferredSelection{target:Arc::new(shape.clone()),index:Some(index)}),
            Shape::Declared(TypeRef::Applied{name,args}) if name=="차림" && args.len()==1=>Some(Shape::Declared(args[0].clone())),
            _=>None,
        }
    }

    fn static_numeric_sources(&self,shape:&Shape,active:&mut BTreeSet<StaticNumericProjection>,anchors:&mut usize)->bool {
        match shape {
            Shape::Numeric(_)=>{*anchors+=1;true},
            Shape::Declared(kind) if NumericKind::from_type(kind).is_some()=>{*anchors+=1;true},
            Shape::SelectionCandidates(items)=>!items.is_empty()
                && items.iter().all(|item|self.static_numeric_sources(item,active,anchors)),
            Shape::CollectionRangeElement(owner)=>{
                let root=self.bags.root(*owner);
                // A bag preview can be aliased as a collection too. Its
                // independently issued candidate kind is not a source literal
                // constraint that a range consumer may change.
                if !self.bags.nodes[root].source_collection {return false;}
                let key=StaticNumericProjection::CollectionElement(root);
                if !active.insert(key) {return true;}
                let sources=self.source_selections(&Shape::CollectionFlow(*owner),None,&mut BTreeSet::new());
                let valid=sources.is_some_and(|values|!values.is_empty()
                    && values.iter().all(|value|self.static_numeric_sources(value,active,anchors)));
                active.remove(&key);valid
            },
            Shape::BagOperation(index)=>{
                let key=StaticNumericProjection::Operation(*index);
                if !active.insert(key) {return true;}
                let op=&self.bags.operations[*index];
                let valid=self.static_numeric_sources(&op.left,active,anchors) && self.static_numeric_sources(&op.right,active,anchors);
                active.remove(&key);valid
            },
            Shape::DeferredSelection{target,index:Some(index)}=>{
                let Shape::CollectionFlow(owner)=target.as_ref() else {return false;};
                let root=self.bags.root(*owner);let node=&self.bags.nodes[root];
                if node.unknown || node.collection_signature || node.seeds.is_empty() {return false;}
                let key=StaticNumericProjection::Position(root,*index);
                if !active.insert(key) {return true;}
                let valid=node.seeds.iter().all(|seed|self.position_source(seed,*index).is_some_and(|source|self.static_numeric_sources(&source,active,anchors)));
                active.remove(&key);valid
            },
            _=>false,
        }
    }

    fn project_static_numeric(&mut self,shape:&Shape)->Result<Variable,String> {
        match shape {
            Shape::Numeric(variable)=>Ok(*variable),
            Shape::Declared(kind)=>match self.declared(kind,self.program.id)? {Shape::Numeric(variable)=>Ok(variable),_=>unreachable!("preflight source")},
            Shape::SelectionCandidates(items)=>{
                let result=self.graph.variable(DefaultPreference::Integer);
                for item in items.iter() {
                    let actual=self.project_static_numeric(item)?;
                    self.graph.same(result,actual)?;
                }
                Ok(result)
            },
            Shape::CollectionRangeElement(owner)=>{
                let key=StaticNumericProjection::CollectionElement(self.bags.root(*owner));
                if let Some(variable)=self.bags.static_numeric.get(&key) {return Ok(*variable);}
                let result=self.graph.variable(DefaultPreference::Integer);
                self.bags.static_numeric.insert(key,result);
                let sources=self.source_selections(&Shape::CollectionFlow(*owner),None,&mut BTreeSet::new()).expect("preflight range element sources");
                for source in sources {
                    let actual=self.project_static_numeric(&source)?;
                    self.graph.same(result,actual)?;
                }
                Ok(result)
            },
            Shape::DeferredSelection{target,index:Some(index)}=>{
                let Shape::CollectionFlow(owner)=target.as_ref() else {unreachable!("preflight static target")};
                let root=self.bags.root(*owner);let key=StaticNumericProjection::Position(root,*index);
                if let Some(variable)=self.bags.static_numeric.get(&key) {return Ok(*variable);}
                let result=self.graph.variable(DefaultPreference::Integer);self.bags.static_numeric.insert(key,result);
                for seed in self.bags.nodes[root].seeds.clone() {
                    let source=self.position_source(&seed,*index).expect("preflight source position");
                    let actual=self.project_static_numeric(&source)?;self.graph.same(result,actual)?;
                }
                Ok(result)
            },
            Shape::BagOperation(index)=>{
                let key=StaticNumericProjection::Operation(*index);
                if let Some(variable)=self.bags.static_numeric.get(&key) {return Ok(*variable);}
                let result=self.graph.variable(DefaultPreference::Integer);self.bags.static_numeric.insert(key,result);
                let op=&self.bags.operations[*index];let (left,right,operator,site,literals)=(op.left.clone(),op.right.clone(),op.op.clone(),op.site,op.literals);
                let left=self.project_static_numeric(&left)?;let right=self.project_static_numeric(&right)?;
                if literals.iter().any(|literal|*literal) {self.graph.same(left,right)?;}
                let output=self.graph.operation(left,right,&operator,site);self.graph.same(result,output)?;Ok(result)
            },
            _=>unreachable!("preflight numeric source"),
        }
    }

    fn direct_numeric_position(&self,shape:&Shape)->bool {
        let Shape::DeferredSelection{target,index:Some(index)}=shape else {return false;};
        let Some(values)=self.source_selections(target,Some(*index),&mut BTreeSet::new()) else {return false;};
        let Some(Shape::Numeric(variable))=values.first() else {return false;};
        values.iter().all(|value|matches!(value,Shape::Numeric(other) if other.0==variable.0))
    }

    fn source_selections(&self,target:&Shape,index:Option<usize>,active:&mut BTreeSet<usize>)->Option<Vec<Shape>> {
        self.source_selections_for(target,index,active,false)
    }

    fn source_selections_for(&self,target:&Shape,index:Option<usize>,active:&mut BTreeSet<usize>,scene:bool)->Option<Vec<Shape>> {
        match target {
            Shape::CollectionSlice(items)=>(!items.is_empty()).then(||items.to_vec()),
            Shape::Collection(items)=>match index {
                Some(index)=>match items.get(index)? {
                    Shape::DeferredSelection{target,index}=>self.source_selections_for(target,*index,active,scene),
                    item=>Some(vec![item.clone()]),
                },
                None=>(!items.is_empty()).then(||items.to_vec()),
            },
            Shape::CollectionFlow(owner)|Shape::BagPreview(owner)=>{
                let root=self.bags.root(*owner);let node=&self.bags.nodes[root];
                if node.unknown || (!scene && node.collection_signature) || node.seeds.is_empty() || !active.insert(root) {return None;}
                let result=node.seeds.iter().map(|seed|self.source_selections_for(seed,index,active,scene)).collect::<Option<Vec<_>>>().map(|rows|rows.into_iter().flatten().collect());
                active.remove(&root);result
            },
            Shape::DeferredSelection{target, index:previous}=>{
                self.source_selections_for(target,*previous,active,scene)?.iter().map(|item|self.source_selections_for(item,index,active,scene)).collect::<Option<Vec<_>>>().map(|rows|rows.into_iter().flatten().collect())
            },
            Shape::Declared(TypeRef::Applied{name,args}) if name=="차림" && args.len()==1=>Some(vec![Shape::Declared(args[0].clone())]),
            _=>None,
        }
    }

    fn collection_binding(&mut self, owner:BindingOwner, actual:Shape)->Shape {
        if !matches!(actual,Shape::Collection(_)|Shape::CollectionSlice(_)|Shape::BagPreview(_)|Shape::CollectionFlow(_)) {return actual;}
        if let Some(Shape::CollectionFlow(previous))=self.bindings.get(&owner).cloned() {
            match &actual {
                Shape::CollectionFlow(next)|Shape::BagPreview(next)=>self.bags.join(previous,*next),
                _=>{let root=self.bags.root(previous);self.bags.nodes[root].seeds.push(actual);},
            }
            return Shape::CollectionFlow(previous);
        }
        let root=match actual {
            Shape::CollectionFlow(root)|Shape::BagPreview(root)=>root,
            actual=>{
                let root=self.bags.variable(Some(actual));
                self.bags.nodes[root.0].source_collection=true;
                root
            },
        };
        Shape::CollectionFlow(root)
    }

    fn scene_scalar(&mut self,shape:&Shape,site:NodeId)->Result<(),String> {
        match shape {
            Shape::BagElement(_)|Shape::BagOperation(_)|Shape::SelectionCandidates(_)=>{
                self.scene_proofs.push((site,shape.clone(),false));Ok(())
            },
            Shape::DeferredSelection{target,index}=>{
                if let Some(values)=self.source_selections_for(target,*index,&mut BTreeSet::new(),true) {
                    for value in values {self.scene_scalar(&value,site)?;}
                }else{self.scene_proofs.push((site,shape.clone(),false));} Ok(())
            },
            Shape::Numeric(variable) if self.issued_numeric_variables.contains(&variable.0)=>{
                if matches!(self.graph.explicit_kind(*variable),Some(NumericKind::Integer|NumericKind::Approximate)) {Ok(())}
                else {Err(self.failure(site,"장면은 이미 발급된 나눔수·큰바른수·곱수를 암묵 변환하지 않습니다"))}
            },
            Shape::Numeric(variable)=>self.graph.require(*variable,NumericKind::Approximate,site),
            Shape::Declared(kind) if matches!(NumericKind::from_type(kind),Some(NumericKind::Integer|NumericKind::Approximate))=>Ok(()),
            _=>Err(self.failure(site,"장면 scalar에는 독립 바른수/셈수 또는 미결 숫자 source가 필요합니다")),
        }
    }

    fn scene_pair(&mut self,shape:&Shape,site:NodeId)->Result<(),String> {
        match shape {
            Shape::CollectionFlow(owner)=>{
                let node=&self.bags.nodes[self.bags.root(*owner)];
                if node.unknown || node.seeds.is_empty() {return Err(self.failure(site,"장면 좌표 쌍의 모든 source가 필요합니다"));}
                let seeds=node.seeds.clone();for seed in seeds {self.scene_pair(&seed,site)?;} Ok(())
            },
            Shape::DeferredSelection{target,index}=>{
                if let Some(values)=self.source_selections_for(target,*index,&mut BTreeSet::new(),true) {
                    for value in values {self.scene_pair(&value,site)?;}
                }else{self.scene_proofs.push((site,shape.clone(),true));} Ok(())
            },
            Shape::BagPreview(_)|Shape::BagElement(_)|Shape::SelectionCandidates(_)=>{
                self.scene_proofs.push((site,shape.clone(),true));Ok(())
            },
            Shape::Collection(items)|Shape::CollectionSlice(items) if items.len()==2=>{
                for scalar in items.iter() {self.scene_scalar(scalar,site)?;} Ok(())
            },
            Shape::Declared(TypeRef::Applied{name,args}) if name=="차림" && args.len()==1=>self.scene_scalar(&Shape::Declared(args[0].clone()),site),
            _=>Err(self.failure(site,"장면 좌표 쌍의 원문 위치별 source owner가 없습니다")),
        }
    }

    fn range_selection(&mut self,scope_id:NodeId,scope:&mut Scope,target:&Shape,index:&Expr,site:NodeId)->Result<Option<Shape>,String> {
        let ExprKind::Call{func,args}=&index.kind else {return Ok(None);};
        if func!="표준.범위" || args.len()!=3 {return Ok(None);}
        self.expression(scope_id,scope,index)?;
        for arg in &args[..2] {
            let bound=self.expression(scope_id,scope,&arg.expr)?;
            let integer=self.declared(&NumericKind::Integer.type_ref(),arg.expr.id)?;
            self.same_shape(&integer,&bound,arg.expr.id)?;
        }
        Ok(Some(match target {
            // A range retains the source collection's homogeneous element
            // owner, not its unsliced positional indices. Keep the live flow
            // so later aliases/assignments participate before defaulting.
            Shape::CollectionFlow(owner)=>Shape::CollectionSlice(Arc::from([Shape::CollectionRangeElement(*owner)])),
            Shape::Collection(items)|Shape::CollectionSlice(items)=>Shape::CollectionSlice(items.clone()),
            shape @ Shape::Declared(TypeRef::Applied{name,..}) if ddonirang_lang::stdlib::canonicalize_type_alias(name)=="차림"=>shape.clone(),
            _=>Shape::Unknown(site),
        }))
    }
    fn sum_payload(&self,owner:&str,applied:&Option<TypeRef>,branch:&str,span:ddonirang_lang::Span)->Option<Vec<ddonirang_lang::SumBranchPayload>> {
        if let Some(kind)=applied {
            let resolved=ddonirang_lang::sum_owner::resolve_applied_sum_owner(kind,span).ok()?;
            if resolved.key!=owner {return None;}
            resolved.branches.into_iter().find(|decl|decl.name==branch).map(|decl|decl.payload)
        } else {self.sums.get(owner)?.iter().find(|decl|decl.name==branch).map(|decl|decl.payload.clone())}
    }

    fn select_one(&mut self,target:Shape,index:Option<usize>,site:NodeId)->Result<Shape,String> {
        match target {
            target @ (Shape::CollectionFlow(_)|Shape::DeferredSelection{..})=>Ok(Shape::DeferredSelection{target:Arc::new(target),index}),
            Shape::BagPreview(owner)=>Ok(Shape::BagElement(owner)),
            Shape::Tuple(items)=>match index {
                Some(index)=>items.get(index).cloned().ok_or_else(||self.failure(site,"위치형 위치가 범위를 벗어났습니다").replacen("E_TYPE_MISMATCH","E_SELECTION_BOUNDS",1)),
                None=>Err(self.failure(site,"위치형 위치는 기존 정적 상수식으로 해소해야 합니다").replacen("E_TYPE_MISMATCH","E_TUPLE_STATIC_SELECTION",1)),
            },
            Shape::Collection(items)=>Ok(match index {
                Some(index)=>items.get(index).cloned().unwrap_or(Shape::Unknown(site)),
                None=>Shape::SelectionCandidates(items),
            }),
            Shape::CollectionSlice(items)=>Ok(Shape::SelectionCandidates(items)),
            Shape::SelectionCandidates(items)=>{
                let selected=items.iter().cloned().map(|item|self.select_one(item,index,site)).collect::<Result<Vec<_>,_>>()?;
                Ok(Shape::SelectionCandidates(selected.into()))
            }
            Shape::Declared(TypeRef::Tensor{element,shape})=>{
                if shape.is_empty() {return Ok(Shape::Unknown(site));}
                if shape.len()==1 {self.declared(&element,site)}
                else {Ok(Shape::Declared(TypeRef::Tensor{element,shape:shape[1..].to_vec()}))}
            }
            Shape::Declared(TypeRef::Applied{name,args}) if args.len()==1 && ddonirang_lang::stdlib::canonicalize_type_alias(&name)=="차림"=>self.declared(&args[0],site),
            Shape::Declared(TypeRef::Applied{name,args}) if args.len()==2 && ddonirang_lang::stdlib::canonicalize_type_alias(&name)=="짝맞춤"=>self.declared(&args[1],site),
            Shape::Declared(TypeRef::Named(name)) if name=="글"=>Ok(Shape::Declared(TypeRef::Named(name))),
            _=>Ok(Shape::Unknown(site)),
        }
    }

    fn integer_selection_target(target:&Shape)->bool {
        match target {
            Shape::Tuple(_)|Shape::Collection(_)|Shape::CollectionSlice(_)|Shape::BagPreview(_)|Shape::CollectionFlow(_)|Shape::DeferredSelection{..}=>true,
            Shape::SelectionCandidates(items)=>!items.is_empty() && items.iter().all(Self::integer_selection_target),
            Shape::Declared(TypeRef::Tensor{..})=>true,
            Shape::Declared(TypeRef::Applied{name,..})=>ddonirang_lang::stdlib::canonicalize_type_alias(name)=="차림",
            Shape::Declared(TypeRef::Named(name))=>name=="글",
            _=>false,
        }
    }

    fn static_tuple_target(target:&Shape)->bool {
        match target {
            Shape::Tuple(_)=>true,
            Shape::SelectionCandidates(items)=>!items.is_empty() && items.iter().all(Self::static_tuple_target),
            _=>false,
        }
    }

    fn require_integer_index(&mut self,target:&Shape,actual:&Shape,index:&Expr)->Result<(),String> {
        let expected=self.declared(&NumericKind::Integer.type_ref(),index.id)?;
        self.same_shape(&expected,actual,index.id).map_err(|error| {
            // Earlier static rejection must retain the target owner's
            // registered diagnostic family, not become an unrelated generic
            // numeric failure. Neither branch converts the index value.
            let code=match target {
                Shape::Declared(TypeRef::Named(name)) if name=="글"=>"E_SELECTION_TEXT_INDEX",
                Shape::Declared(TypeRef::Tensor{..})=>"E_TENSOR_SELECTION",
                shape if Self::static_tuple_target(shape)=>"E_TUPLE_STATIC_SELECTION",
                _=>"E_TYPE_MISMATCH",
            };
            self.failure(index.id,&format!("선택 위치의 씨는 바른수여야 합니다: {error}")).replacen("E_TYPE_MISMATCH",code,1)
        })
    }

    fn replacement_failure(&self,target:&Expr,actual:&Shape,error:String)->String {
        if matches!(actual,Shape::Unknown(_)) {return error;}
        let range_key=|key:&Expr|matches!(&key.kind,ExprKind::Call{func,args} if func=="표준.범위" && args.len()==3);
        let (base,range)=match &target.kind {
            ExprKind::SelectionPlace{target,index}=>(target.as_ref(),range_key(index)),
            ExprKind::MultiAxisSelection(selection) if selection.selectors.len()==1=>{
                let range=match &selection.selectors[0] {
                    ddonirang_lang::TensorAxisSelector::Range{..}=>true,
                    ddonirang_lang::TensorAxisSelector::Index(index)=>range_key(index),
                    _=>false,
                };
                (selection.target.as_ref(),range)
            }
            _=>return error,
        };
        let Some(owner)=self.shapes.get(&base.id) else {return error;};
        let code=if Self::static_tuple_target(owner) {"E_TUPLE_REPLACEMENT"}
            else if range && matches!(owner,Shape::Collection(_)|Shape::CollectionSlice(_)|Shape::CollectionFlow(_)|Shape::Declared(TypeRef::Applied{..})) {"E_SELECTION_RANGE_REPLACEMENT"}
            else {return error;};
        self.failure(target.id,&error).replacen("E_TYPE_MISMATCH",code,1)
    }

    fn unit_result(&mut self,dim:ddonirang_core::UnitDim,site:NodeId)->Result<Shape,String> {
        if dim.is_dimensionless() {self.declared(&NumericKind::Approximate.type_ref(),site)}
        else {Ok(Shape::Unit(dim))}
    }

    fn unit_operation(&mut self,left:&Shape,right:&Shape,op:&str,site:NodeId)->Result<Option<Shape>,String> {
        use ddonirang_core::UnitDim;
        let (a,b)=match (left,right) {
            (Shape::Unit(a),Shape::Unit(b))=>(*a,*b),
            (Shape::Unit(a),Shape::Numeric(_))=>(*a,UnitDim::NONE),
            (Shape::Numeric(_),Shape::Unit(b))=>(UnitDim::NONE,*b),
            _=>return Ok(None),
        };
        let dim=if matches!(op,"+"|"-") {
            if a!=b {return Ok(Some(Shape::Unknown(site)));} a
        } else {
            let mut exponents=[0;UnitDim::AXIS_COUNT];
            for ((result,a),b) in exponents.iter_mut().zip(a.exponents()).zip(b.exponents()) {
                *result=if op=="*" {a.checked_add(b)} else {a.checked_sub(b)}
                    .ok_or_else(||self.failure(site,"기존 단위 차원 표현 범위를 벗어났습니다"))?;
            }
            UnitDim::from_exponents(exponents)
        };
        self.unit_result(dim,site).map(Some)
    }

    fn tensor_selection(&mut self,element:&TypeRef,shape:&[u64],selectors:&[ddonirang_lang::TensorAxisSelector],site:NodeId)->Result<Shape,String> {
        use ddonirang_lang::TensorAxisSelector as S;
        if selectors.len()>shape.len() {return Ok(Shape::Unknown(site));}
        let mut result=Vec::new();
        for (axis,&length) in shape.iter().enumerate() {match selectors.get(axis) {
            Some(S::Index(_))=>{},
            Some(S::Range{start,end}) if start<=end && *end<=length=>result.push(end-start),
            Some(S::Range{..})=>return Ok(Shape::Unknown(site)),
            Some(S::Full)|None=>result.push(length),
        }}
        if result.is_empty() {self.declared(element,site)}
        else {Ok(Shape::Declared(TypeRef::Tensor{element:Box::new(element.clone()),shape:result}))}
    }

    fn analyze_callable(&mut self,seed:&SeedDef,actuals:Option<&[(NodeId,Shape)]>) -> Result<Shape,String> {
        let mut scope=self.callable_globals.clone();
        for (index,pin) in seed.params.iter().enumerate() {
            let owner=BindingOwner{scope:seed.id,declaration:pin.id};
            let actual=actuals.and_then(|items|items.get(index));
            let shape=if let Some(shape)=self.bindings.get(&owner) {shape.clone()}
                else if matches!(&pin.type_ref,TypeRef::Named(name) if name=="차림") {
                    match actual.map(|(_,shape)|shape) {
                        Some(shape @ (Shape::Collection(_)|Shape::CollectionSlice(_)|Shape::CollectionFlow(_)|Shape::BagPreview(_)))=>self.collection_binding(owner,shape.clone()),
                        Some(Shape::Declared(kind @ TypeRef::Applied{..}))=>Shape::Declared(kind.clone()),
                        _=>self.declared(&pin.type_ref,pin.id)?,
                    }
                }else{self.declared(&pin.type_ref,pin.id)?};
            if let Some((site,actual))=actual {self.same_shape(&shape,actual,*site)?;}
            self.bindings.insert(owner,shape.clone());
            self.lexical_locals.insert(owner);
            scope.insert(pin.pin_name.clone(),(owner,shape));
        }
        if !self.analyzed_callables.contains(&seed.id) && self.active_callables.insert(seed.id) {
            if let Some(body)=&seed.body {
                let old=self.sums.clone();let result=self.statements(seed.id,&mut scope,&body.stmts,Some(seed));
                self.sums=old;result?;
            }
            self.active_callables.remove(&seed.id);
            self.analyzed_callables.insert(seed.id);
        }
        // An explicit result owner is the callable contract. A coarse body
        // supplier (for example a legacy collection operation) must not erase
        // it at the call site. The body still passes callable_return checks
        // and the existing runtime result gate; no runtime shape issues a type.
        if let Some(kind)=seed.result_type.as_ref().filter(|kind|has_concrete_numeric_element_owner(kind)) {
            return self.declared(kind,seed.id);
        }
        if let Some(shape)=self.callable_returns.get(&seed.id) {return Ok(shape.clone());}
        if let Some(kind)=&seed.result_type {self.declared(kind,seed.id)}else{Ok(Shape::Unknown(seed.id))}
    }

    fn callable_return(&mut self,seed:&SeedDef,actual:&Shape,site:NodeId) -> Result<(),String> {
        if let Some(kind)=&seed.result_type {
            let expected=self.declared(kind,seed.id)?;
            self.same_shape(&expected,actual,site)?;
        }
        if let Some(previous)=self.callable_returns.get(&seed.id).cloned() {self.same_shape(&previous,actual,site)?;}
        else {self.callable_returns.insert(seed.id,actual.clone());}
        Ok(())
    }
    fn declaration_item(&mut self, scope_id:NodeId, scope:&mut Scope, item:&ddonirang_lang::DeclItem) -> Result<(),String> {
        let actual=item.value.as_ref().map(|value|self.expression(scope_id,scope,value)).transpose()?;
        let mut shape=if matches!(item.type_ref,TypeRef::Infer) {actual.unwrap_or(Shape::Unknown(item.id))}
            else {
                let expected=self.declared(&item.type_ref,item.id)?;
                if let Some(actual)=actual {self.same_shape(&expected,&actual,item.id)?;}
                expected
            };
        let owner=scope.get(&item.name).filter(|(owner,_)|owner.declaration==item.id).map(|(owner,_)|*owner)
            .unwrap_or(BindingOwner{scope:scope_id,declaration:item.id});
        if matches!(item.type_ref,TypeRef::Infer) {shape=self.collection_binding(owner,shape);}
        if matches!(item.type_ref,TypeRef::Infer) && matches!(&shape,Shape::Declared(TypeRef::Named(name)) if name=="없음") {
            self.inferred_absences.insert(owner);
        }
        self.bindings.insert(owner,shape.clone());scope.insert(item.name.clone(),(owner,shape));
        Ok(())
    }
    fn failure(&self, site: NodeId, reason: &str) -> String {
        let span = self.source_sites.get(&site).or_else(||self.program.origin.node_spans.get(&site)).copied()
            .or_else(||self.program.items.iter().find_map(|TopLevelItem::SeedDef(seed)|(seed.id==site).then_some(seed.span)));
        let source=span.and_then(|span|self.program.origin.source.get(span.start..span.end)).unwrap_or("");
        format!("E_TYPE_MISMATCH: 수 종류의 정적 공급을 확정할 수 없습니다: {reason} ({}:{:?}; 노드={site}; 원문={source:?})",
            self.program.origin.file_path, span)
    }

    fn source_shape_label(&self,shape:&Shape)->String {
        match shape {
            Shape::Numeric(variable)=>self.graph.explicit_kind(*variable).map(|kind|format!("{:?}",kind.type_ref())).unwrap_or_else(||"미결 수 제약".into()),
            Shape::Declared(TypeRef::Named(name))=>format!("선언씨 {name}"),
            Shape::Declared(TypeRef::Applied{name,..})=>format!("적용씨 {name}"),
            Shape::Unknown(site)=>format!("미공급 원문 노드 {site}"),
            Shape::Tuple(_)=>"위치형".into(),Shape::Collection(_)|Shape::CollectionSlice(_)=>"차림".into(),
            _=>"복합 원문 공급".into(),
        }
    }

    fn declared(&mut self, kind: &TypeRef, site: NodeId) -> Result<Shape, String> {
        if let TypeRef::Tuple(items) = kind {
            return Ok(Shape::Tuple(items.iter().map(|item|self.declared(item,site)).collect::<Result<Vec<_>,_>>()?.into()));
        }
        if let Some(kind) = NumericKind::from_type(kind) {
            let variable = self.graph.variable(DefaultPreference::Integer);
            self.graph.require(variable, kind, site)?;
            self.issued_numeric_variables.insert(variable.0);
            Ok(Shape::Numeric(variable))
        } else if matches!(kind, TypeRef::Infer) { Ok(Shape::Unknown(site)) }
        else if matches!(kind,TypeRef::Named(name) if name=="무작위가방") {Ok(Shape::Bag(self.bags.variable(None)))}
        else { Ok(Shape::Declared(kind.clone())) }
    }

    fn consume_conditional(&mut self,shape:Shape,site:NodeId)->Result<Shape,String> {
        if matches!(shape,Shape::ConditionalScalar{..}|Shape::SelectionCandidates(_)|Shape::BagElement(_)|Shape::BagOperation(_)|Shape::DeferredSelection{..}) {
            // A statically selected position transports that source token's
            // existing constraints, not a guessed homogeneous collection T.
            // Keep the deferred proof too, so later incoming sources cannot
            // be hidden by this early/cached positional read.
            if let Shape::DeferredSelection{target,index:Some(index)}=&shape {
                if let Some(values)=self.source_selections(target,Some(*index),&mut BTreeSet::new()) {
                    if let Some(Shape::Numeric(variable))=values.first() {
                        if values.iter().all(|value|matches!(value,Shape::Numeric(other) if other.0==variable.0)) {
                            let result=*variable;self.consumed_conditionals.push((site,shape,result));return Ok(Shape::Numeric(result));
                        }
                    }
                }
            }
            let result=self.graph.variable(DefaultPreference::Integer);
            self.consumed_conditionals.push((site,shape,result));
            Ok(Shape::Numeric(result))
        }else{Ok(shape)}
    }

    fn same_shape(&mut self, left: &Shape, right: &Shape, site: NodeId) -> Result<(), String> {
        // A selection may produce a collection, not only a scalar number.
        // Compare independently resolved owners without letting a consumer
        // expectation change the possible source elements.
        if matches!(left,Shape::SelectionCandidates(_)|Shape::DeferredSelection{..}) || matches!(right,Shape::SelectionCandidates(_)|Shape::DeferredSelection{..}) {
            let other=if matches!(left,Shape::SelectionCandidates(_)|Shape::DeferredSelection{..}) {right}else{left};
            if !matches!(other,Shape::Numeric(_)|Shape::ConditionalScalar{..}|Shape::BagElement(_)|Shape::BagOperation(_)) {
                self.selection_equalities.push((site,left.clone(),right.clone()));return Ok(());
            }
        }
        // Bag evidence unifies possible source inputs only. It never connects
        // a result consumer's numeric variable back to the candidate literals.
        match (left,right) {
            (Shape::CollectionFlow(a),Shape::CollectionFlow(b)|Shape::BagPreview(b))|(Shape::BagPreview(a),Shape::CollectionFlow(b))=>{
                self.bags.join(*a,*b);return Ok(());
            }
            (Shape::CollectionFlow(a),actual)=>{
                let root=self.bags.root(*a);self.bags.nodes[root].seeds.push(actual.clone());return Ok(());
            }
            (Shape::Bag(a),Shape::Bag(b))|(Shape::BagDraw(a),Shape::BagDraw(b))|(Shape::BagPreview(a),Shape::BagPreview(b))=>{
                self.bags.join(*a,*b);return Ok(());
            }
            (Shape::BagPreview(a),actual @ (Shape::Collection(_)|Shape::CollectionSlice(_)|Shape::Declared(TypeRef::Applied{..})))=>{
                let root=self.bags.root(*a);self.bags.nodes[root].seeds.push(actual.clone());return Ok(());
            }
            (Shape::Bag(a)|Shape::BagDraw(a)|Shape::BagPreview(a),_)=>{self.bags.reject(*a);return Ok(());}
            _=>{},
        }
        if matches!((left,right),(Shape::BagElement(_)|Shape::BagOperation(_)|Shape::DeferredSelection{..},Shape::Numeric(_)|Shape::BagElement(_)|Shape::BagOperation(_)|Shape::DeferredSelection{..})|(Shape::Numeric(_),Shape::BagElement(_)|Shape::BagOperation(_)|Shape::DeferredSelection{..})) {
            let left=self.consume_conditional(left.clone(),site)?;let right=self.consume_conditional(right.clone(),site)?;
            return self.same_shape(&left,&right,site);
        }
        if matches!((left,right),(Shape::ConditionalScalar{..}|Shape::SelectionCandidates(_),Shape::Numeric(_)|Shape::ConditionalScalar{..}|Shape::SelectionCandidates(_))
            |(Shape::Numeric(_),Shape::ConditionalScalar{..}|Shape::SelectionCandidates(_))) {
            let left=self.consume_conditional(left.clone(),site)?;
            let right=self.consume_conditional(right.clone(),site)?;
            return self.same_shape(&left,&right,site);
        }
        match (left, right) {
            // Non-numeric category/pin failures retain the existing checker
            // and runtime's structured diagnostic/atomicity contract. A
            // successful declared value must still pass that owner's gate.
            (Shape::Numeric(_),Shape::Declared(_)|Shape::Unit(_))
            | (Shape::Declared(_)|Shape::Unit(_),Shape::Numeric(_))=>Ok(()),
            (Shape::Numeric(a), Shape::Numeric(b)) => self.graph.same(*a, *b),
            (Shape::Tuple(a), Shape::Tuple(b)) if a.len() == b.len() => {
                for (a, b) in a.iter().zip(b.iter()) { self.same_shape(a, b, site)?; } Ok(())
            }
            (Shape::Declared(a), Shape::Declared(b)) if a == b => Ok(()),
            (Shape::Declared(TypeRef::Applied{name,args}),Shape::Collection(items))
                if ddonirang_lang::stdlib::canonicalize_type_alias(name)=="차림" && args.len()==1 => {
                for item in items.iter() {
                    let expected=self.declared(&args[0],site)?;
                    self.same_shape(&expected,item,site)?;
                }
                Ok(())
            }
            (Shape::Collection(a), Shape::Collection(b)) => {
                for (a,b) in a.iter().zip(b.iter()) {self.same_shape(a,b,site)?;} Ok(())
            }
            (Shape::Numeric(_), Shape::Unknown(source)) | (Shape::Unknown(source), Shape::Numeric(_)) =>
                Err(self.failure(*source, &format!("수 피연산자 원문 노드 {source}의 owner가 없습니다"))),
            (Shape::Numeric(_), _) | (_, Shape::Numeric(_)) => Err(self.failure(site, "서로 다른 정적 수/값 씨입니다")),
            _ => Ok(()),
        }
    }

    fn finish_shape(&self, shape: &Shape) -> Option<TypeRef> {
        Self::finish_shape_in(shape,&self.graph,&self.bags)
    }
    fn finish_shape_in(shape:&Shape,graph:&NumericConstraints,bags:&BagConstraints)->Option<TypeRef> {
        Self::finish_shape_inner(shape,graph,bags,&mut BTreeSet::new())
    }
    fn finish_shape_inner(shape:&Shape,graph:&NumericConstraints,bags:&BagConstraints,active:&mut BTreeSet<usize>)->Option<TypeRef> {
        match shape {
            Shape::DeferredSelection{target,index}=>{
                if let Shape::CollectionFlow(owner)=target.as_ref() {
                    let root=bags.root(*owner);let node=&bags.nodes[root];
                    if let Some(variable)=index.and_then(|index|bags.static_numeric.get(&StaticNumericProjection::Position(root,index))) {return Some(graph.resolved(*variable).type_ref());}
                    if node.collection_signature {return Self::finish_shape_inner(&Shape::BagElement(*owner),graph,bags,active);}
                    if node.unknown || node.seeds.is_empty() || !active.insert(root) {return None;}
                    let mut resolved=None;let mut valid=true;
                    for seed in &node.seeds {
                        let candidate=Self::finish_selected(seed,*index,graph,bags,active);
                        match candidate {Some(kind) if resolved.as_ref().is_none_or(|previous|previous==&kind)=>resolved=Some(kind),_=>{valid=false;break;}}
                    }
                    active.remove(&root);return if valid {resolved}else{None};
                }
                Self::finish_selected(target,*index,graph,bags,active)
            }
            Shape::CollectionFlow(owner)=>Self::finish_shape_inner(&Shape::BagPreview(*owner),graph,bags,active),
            Shape::CollectionRangeElement(owner)=>{
                let key=StaticNumericProjection::CollectionElement(bags.root(*owner));
                match bags.static_numeric.get(&key) {
                    Some(variable)=>Some(graph.resolved(*variable).type_ref()),
                    None=>Self::finish_shape_inner(&Shape::BagElement(*owner),graph,bags,active),
                }
            },
            Shape::BagOperation(index)=>{
                if let Some(variable)=bags.static_numeric.get(&StaticNumericProjection::Operation(*index)) {return Some(graph.resolved(*variable).type_ref());}
                let operation=&bags.operations[*index];
                let (left,right)=Self::bag_operation_operands(operation,graph,bags,active);
                let (left,right)=(left?,right?);
                if operation.op=="+" && left==TypeRef::Named("글".into()) && left==right {return Some(left);}
                numeric_signature_result(NumericKind::from_type(&left)?,NumericKind::from_type(&right)?,&operation.op)
            }
            Shape::Numeric(variable) => Some(graph.resolved(*variable).type_ref()),
            Shape::Declared(kind) => Some(kind.clone()),
            Shape::ConditionalScalar{args,..} if args.len()==2 && args.iter().all(|arg|Self::finish_shape_inner(arg,graph,bags,active)==Some(NumericKind::Approximate.type_ref()))=>Some(NumericKind::Approximate.type_ref()),
            Shape::ConditionalScalar{..}=>None,
            Shape::SelectionCandidates(items)=>{
                let first=Self::finish_shape_inner(items.first()?,graph,bags,active)?;
                items.iter().skip(1).all(|item|Self::finish_shape_inner(item,graph,bags,active).as_ref()==Some(&first)).then_some(first)
            }
            Shape::Collection(items)|Shape::CollectionSlice(items)=>{
                let first=Self::finish_shape_inner(items.first()?,graph,bags,active)?;
                items.iter().skip(1).all(|item|Self::finish_shape_inner(item,graph,bags,active).as_ref()==Some(&first))
                    .then_some(TypeRef::Applied{name:"차림".into(),args:vec![first]})
            }
            // Tuple positional shape is retained privately until the existing
            // RecordFieldType owner composes it; do not invent a TypeRef tag.
            Shape::BagElement(id)|Shape::BagPreview(id)=>{
                let root=bags.root(*id);let node=&bags.nodes[root];
                if node.unknown || node.seeds.is_empty() || !active.insert(root) {return None;}
                let mut kind=None;let mut valid=true;
                for seed in &node.seeds {
                    let candidate=match seed {
                        Shape::Collection(items)|Shape::CollectionSlice(items)=>{
                            let mut values=items.iter();
                            values.next().and_then(|first|Self::finish_shape_inner(first,graph,bags,active)).filter(|first|
                                values.all(|value|Self::finish_shape_inner(value,graph,bags,active).as_ref()==Some(first)))
                        }
                        Shape::Declared(TypeRef::Applied{name,args}) if name=="차림" && args.len()==1=>Some(args[0].clone()),
                        Shape::CollectionFlow(_)|Shape::DeferredSelection{..}=>match Self::finish_shape_inner(seed,graph,bags,active) {
                            Some(TypeRef::Applied{name,args}) if name=="차림" && args.len()==1=>Some(args[0].clone()),
                            _=>None,
                        },
                        _=>None,
                    };
                    match candidate {Some(candidate) if kind.as_ref().is_none_or(|prior|prior==&candidate)=>kind=Some(candidate),_=>{valid=false;break;}}
                }
                active.remove(&root);
                if !valid {return None;}
                if node.collection_signature && !kind.as_ref().is_some_and(admitted_collection_signature_kind) {return None;}
                kind.map(|kind|if matches!(shape,Shape::BagPreview(_)){TypeRef::Applied{name:"차림".into(),args:vec![kind]}}else{kind})
            }
            Shape::Tuple(items) => Some(TypeRef::Tuple(items.iter().map(|item|Self::finish_shape_inner(item,graph,bags,active)).collect::<Option<_>>()?)),
            Shape::Unit(_) | Shape::Unknown(_) |Shape::Bag(_)|Shape::BagDraw(_) | Shape::InputEvent => None,
        }
    }

    fn finish_selected(target:&Shape,index:Option<usize>,graph:&NumericConstraints,bags:&BagConstraints,active:&mut BTreeSet<usize>)->Option<TypeRef> {
        match target {
            Shape::CollectionSlice(items)=>Self::finish_shape_inner(&Shape::SelectionCandidates(items.clone()),graph,bags,active),
            Shape::Collection(items)=>{
                if let Some(index)=index {return Self::finish_shape_inner(items.get(index)?,graph,bags,active);}
                Self::finish_shape_inner(&Shape::SelectionCandidates(items.clone()),graph,bags,active)
            }
            _=>match Self::finish_shape_inner(target,graph,bags,active)? {
                TypeRef::Applied{name,args} if name=="차림" && args.len()==1=>Some(args[0].clone()),
                TypeRef::Named(name) if name=="글"=>Some(TypeRef::Named(name)),
                _=>None,
            }
        }
    }

    fn bag_operation_operands(operation:&BagOperation,graph:&NumericConstraints,bags:&BagConstraints,active:&mut BTreeSet<usize>)->(Option<TypeRef>,Option<TypeRef>) {
        let mut left=Self::finish_shape_inner(&operation.left,graph,bags,active);
        let mut right=Self::finish_shape_inner(&operation.right,graph,bags,active);
        if operation.literals[0] && right.as_ref().and_then(NumericKind::from_type).is_some() {left=right.clone();}
        if operation.literals[1] && left.as_ref().and_then(NumericKind::from_type).is_some() {right=left.clone();}
        (left,right)
    }

    fn finish_descriptor(&self, shape: &Shape) -> Option<RecordFieldType> {
        match shape {
            Shape::Tuple(items) => Some(RecordFieldType::Tuple {
                items: items.iter().map(|item| self.finish_descriptor(item)).collect::<Option<_>>()?,
            }),
            Shape::Collection(items) => {
                let mut descriptors = items.iter().map(|item| self.finish_descriptor(item));
                let first = descriptors.next()??;
                if descriptors.any(|kind| kind.as_ref() != Some(&first)) { return None; }
                Some(RecordFieldType::Applied { name: "차림".into(), args: vec![first] })
            }
            Shape::Declared(TypeRef::Named(name)) if matches!(name.as_str(), "수" | "값" | "차림") => None,
            _ => ddonirang_lang::runtime::record_type_identity(&self.finish_shape(shape)?, &self.records).ok(),
        }
    }

    fn expression(&mut self, scope_id: NodeId, scope: &mut Scope, expression: &Expr) -> Result<Shape, String> {
        self.source_sites.insert(expression.id,expression.span);
        if let Some(shape)=self.shapes.get(&expression.id) {return Ok(shape.clone());}
        let shape = if let Some(operand) = expression.source_negated_operand() {
            self.expression(scope_id, scope, operand)?
        } else { match &expression.kind {
            ExprKind::Literal(_) if expression.numeric_literal_source.is_some() => {
                let source = expression.numeric_literal_source.as_ref().expect("source token");
                let preference = if source.is_decimal() { DefaultPreference::Decimal } else { DefaultPreference::Integer };
                let variable = self.graph.variable(preference);
                // A fractional token cannot satisfy an integer expectation
                // merely because the fixed-point parser rounded its bits.
                if source.raw().split_once('.').is_some_and(|(_, fraction)| fraction.chars().any(|digit| digit != '0')) {
                    self.graph.restrict(variable, NumericKind::Approximate.bit() | NumericKind::Rational.bit(), expression.id)?;
                }
                self.literal_sites.insert(expression.id, variable);
                let (whole,fraction)=source.raw().split_once('.').unwrap_or((source.raw(),""));
                self.exact_integer_literals.insert(expression.id,fraction.chars().all(|digit|digit=='0')
                    && whole.parse::<i64>().ok().and_then(|value|value.checked_mul(1i64<<32)).is_some());
                Shape::Numeric(variable)
            }
            ExprKind::Literal(Literal::Bool(_)) => Shape::Declared(TypeRef::Named("참거짓".into())),
            ExprKind::Literal(Literal::String(_)) => Shape::Declared(TypeRef::Named("글".into())),
            ExprKind::Literal(Literal::None) => Shape::Declared(TypeRef::Named("없음".into())),
            ExprKind::SumBranchValue{owner,applied_owner,branch,payload}=>{
                let schema=self.sum_payload(owner,applied_owner,branch,expression.span);
                for arg in payload {
                    let actual=self.expression(scope_id,scope,&arg.value)?;
                    if let Some(pin)=schema.as_ref().and_then(|pins|pins.iter().find(|pin|pin.binder==arg.pin)) {
                        let expected=self.declared(&pin.type_ref,arg.value.id)?;
                        self.same_shape(&expected,&actual,arg.value.id)?;
                    }
                }
                if schema.is_some() {Shape::Declared(applied_owner.clone().unwrap_or_else(||TypeRef::Named(owner.clone())))}
                else {Shape::Unknown(expression.id)}
            }
            ExprKind::Var(name) => scope.get(name).or_else(||self.callable_globals.get(name)).map(|(_, shape)| shape.clone()).unwrap_or_else(|| match name.as_str() {
                "참" | "거짓" => Shape::Declared(TypeRef::Named("참거짓".into())),
                "없음" => Shape::Declared(TypeRef::Named("없음".into())),
                _ => Shape::Unknown(expression.id),
            }),
            ExprKind::Infix { left, op, right } => {
                let child_ids = (left.id, right.id);
                let left = self.expression(scope_id, scope, left)?;
                let right = self.expression(scope_id, scope, right)?;
                if matches!(op.as_str(), "+" | "-" | "*" | "/") {
                    if matches!(left,Shape::BagElement(_)|Shape::BagOperation(_)) || matches!(right,Shape::BagElement(_)|Shape::BagOperation(_))
                        || matches!(left,Shape::DeferredSelection{..}) && !self.direct_numeric_position(&left)
                        || matches!(right,Shape::DeferredSelection{..}) && !self.direct_numeric_position(&right) {
                        let index=self.bags.operations.len();
                        self.bags.operations.push(BagOperation{left,right,op:op.clone(),site:expression.id,
                            literals:[self.literal_trees.contains(&child_ids.0),self.literal_trees.contains(&child_ids.1)]});
                        Shape::BagOperation(index)
                    }else{
                    let left=self.consume_conditional(left,expression.id)?;
                    let right=self.consume_conditional(right,expression.id)?;
                    if let Some(result)=self.unit_operation(&left,&right,op,expression.id)? {result}
                    else if let (Shape::Numeric(a), Shape::Numeric(b)) = (&left, &right) {
                        if self.literal_trees.contains(&child_ids.0) && self.literal_trees.contains(&child_ids.1) {
                            self.graph.same(*a, *b)?;
                            if op == "/" {
                                let division = self.graph.variable(DefaultPreference::IntegerDivision);
                                self.graph.same(*a, division)?;
                            }
                            left
                        } else {
                            // An issued operand's unique expectation is given
                            // to raw literal trees, not recovered from values.
                            // RD083's issued integer widening is not a reason
                            // to default these unissued literals to integers.
                            if self.literal_trees.contains(&child_ids.0) {
                                if let Some(kind) = self.graph.explicit_kind(*b) { self.graph.require(*a, kind, child_ids.0)?; }
                            }
                            if self.literal_trees.contains(&child_ids.1) {
                                if let Some(kind) = self.graph.explicit_kind(*a) { self.graph.require(*b, kind, child_ids.1)?; }
                            }
                            let result=self.graph.operation(*a,*b,op,expression.id);
                            if let Some(kind)=(self.issued_numeric_variables.contains(&a.0)||self.issued_numeric_variables.contains(&b.0))
                                .then(||self.graph.explicit_kind(*a).zip(self.graph.explicit_kind(*b))).flatten()
                                .and_then(|(a,b)|numeric_signature_result(a,b,op)).as_ref().and_then(NumericKind::from_type) {
                                self.graph.require(result,kind,expression.id)?;
                                self.issued_numeric_variables.insert(result.0);
                            }
                            Shape::Numeric(result)
                        }
                    } else if matches!(op.as_str(), "+") && matches!((&left, &right), (Shape::Declared(a), Shape::Declared(b)) if a==b && a==&TypeRef::Named("글".into())) { left }
                    else { return Err(self.failure(expression.id, &format!("연산의 유일한 수 서명/원소 owner가 없습니다 (왼쪽: {}, 오른쪽: {})",self.source_shape_label(&left),self.source_shape_label(&right)))); }
                    }
                } else if matches!(op.as_str(), "==" | "!=" | "<" | "<=" | ">" | ">=") {
                    if matches!(left,Shape::BagElement(_)|Shape::BagOperation(_)|Shape::BagPreview(_)|Shape::DeferredSelection{..}) || matches!(right,Shape::BagElement(_)|Shape::BagOperation(_)|Shape::BagPreview(_)|Shape::DeferredSelection{..}) {
                        self.bags.comparisons.push(BagOperation{left,right,op:op.clone(),site:expression.id,
                            literals:[self.literal_trees.contains(&child_ids.0),self.literal_trees.contains(&child_ids.1)]});
                    } else if !matches!(left,Shape::Bag(_)|Shape::BagDraw(_)) && !matches!(right,Shape::Bag(_)|Shape::BagDraw(_)) {
                    self.same_shape(&left, &right, expression.id).map_err(|error| {
                        let kinds=match (&left,&right) {
                            (Shape::Numeric(a),Shape::Numeric(b))=>self.graph.explicit_kind(*a).zip(self.graph.explicit_kind(*b)),
                            _=>None,
                        };
                        if let Some((a,b))=kinds.filter(|(a,b)|a!=b) {
                            format!("E_NUMERIC_COMPARISON_TYPE_MISMATCH: {op} 비교의 왼쪽은 {:?}, 오른쪽은 {:?}입니다. 서로 다른 수 종류는 자동 변환하지 않습니다. {error}",a.type_ref(),b.type_ref())
                        } else {error}
                    })?;
                    }
                    Shape::Declared(TypeRef::Named("참거짓".into()))
                } else { Shape::Unknown(expression.id) }
            }
            ExprKind::TupleConstructor { elements } | ExprKind::CollectionConstructor { elements } => {
                let values = elements.iter().map(|expr| self.expression(scope_id, scope, expr)).collect::<Result<Vec<_>, _>>()?;
                if matches!(expression.kind, ExprKind::TupleConstructor { .. }) { Shape::Tuple(values.into()) } else { Shape::Collection(values.into()) }
            }
            ExprKind::SelectionPlace { target, index } => {
                let target = self.expression(scope_id, scope, target)?;
                let selected = crate::source_analysis::static_tuple_index(index);
                let index_shape = self.expression(scope_id, scope, index)?;
                if let ExprKind::Call{func,args}=&index.kind {
                    if func=="표준.범위" && args.len()==3 {
                        for arg in &args[..2] {
                            let bound=self.expression(scope_id,scope,&arg.expr)?;
                            let integer=self.declared(&NumericKind::Integer.type_ref(),arg.expr.id)?;
                            self.same_shape(&integer,&bound,arg.expr.id)?;
                        }
                        match target {
                            Shape::CollectionFlow(owner)=>Shape::CollectionSlice(Arc::from([Shape::CollectionRangeElement(owner)])),
                            Shape::Collection(items)|Shape::CollectionSlice(items)=>Shape::CollectionSlice(items),
                            shape @ Shape::Declared(TypeRef::Applied{..})=>shape,
                            _=>Shape::Unknown(expression.id),
                        }
                    } else {self.select_one(target,selected,expression.id)?}
                } else {
                    // RD-352's tuple position is a compile-time key. Its
                    // existing closed constant-expression owner is distinct
                    // from the runtime 바른수 index of a collection.
                    if Self::integer_selection_target(&target) && !(selected.is_some() && Self::static_tuple_target(&target)) {
                        self.require_integer_index(&target,&index_shape,index)?;
                    }
                    self.select_one(target,selected,expression.id)?
                }
            }
            ExprKind::MultiAxisSelection(selection) => {
                let target = self.expression(scope_id, scope, &selection.target)?;
                let range=match selection.selectors.as_slice() {
                    [ddonirang_lang::TensorAxisSelector::Index(index)]=>self.range_selection(scope_id,scope,&target,index,expression.id)?,
                    _=>None,
                };
                if let Some(result)=range {result} else {
                for selector in &selection.selectors {
                    if let ddonirang_lang::TensorAxisSelector::Index(index) = selector {
                        let actual = self.expression(scope_id, scope, index)?;
                        if Self::integer_selection_target(&target) && !(Self::static_tuple_target(&target) && crate::source_analysis::static_tuple_index(index).is_some()) {
                            self.require_integer_index(&target,&actual,index)?;
                        }
                    }
                }
                if let Shape::Declared(TypeRef::Tensor{element,shape})=&target {
                    self.tensor_selection(element,shape,&selection.selectors,expression.id)?
                } else if selection.selectors.len() == 1 {
                    match (&target, &selection.selectors[0]) {
                        (_,ddonirang_lang::TensorAxisSelector::Index(index))=>self.select_one(target.clone(),crate::source_analysis::static_tuple_index(index),expression.id)?,
                        (Shape::Tuple(items), ddonirang_lang::TensorAxisSelector::Range { start, end }) =>
                            usize::try_from(*start).ok().zip(usize::try_from(*end).ok()).and_then(|(start,end)|items.get(start..end)).map(|items|Shape::Tuple(Arc::from(items))).unwrap_or(Shape::Unknown(expression.id)),
                        (Shape::Collection(items), ddonirang_lang::TensorAxisSelector::Range { start, end }) =>
                            usize::try_from(*start).ok().zip(usize::try_from(*end).ok()).and_then(|(start,end)|items.get(start..end)).map(|items|Shape::Collection(Arc::from(items))).unwrap_or(Shape::Unknown(expression.id)),
                        (Shape::CollectionFlow(owner), ddonirang_lang::TensorAxisSelector::Range { .. }) =>
                            Shape::CollectionSlice(Arc::from([Shape::CollectionRangeElement(*owner)])),
                        (Shape::Declared(TypeRef::Applied { name, args }), selector)
                            if ddonirang_lang::stdlib::canonicalize_type_alias(name) == "차림" && args.len() == 1 => {
                                match selector {
                                    ddonirang_lang::TensorAxisSelector::Index(_) => self.declared(&args[0], expression.id)?,
                                    ddonirang_lang::TensorAxisSelector::Range { .. } => target.clone(),
                                    ddonirang_lang::TensorAxisSelector::Full => Shape::Unknown(expression.id),
                                }
                            }
                        _ => Shape::Unknown(expression.id),
                    }
                } else { Shape::Unknown(expression.id) }
                }
            }
            ExprKind::ProductConstruction { owner, definition, fields, category } => {
                let schema = self.program.items.iter().find_map(|TopLevelItem::SeedDef(seed)|
                    (seed.id == *definition && seed.canonical_name == *owner && match category {
                        Some(category)=>seed.seed_kind==ddonirang_lang::SeedKind::Schema(*category),
                        None=>seed.seed_kind==ddonirang_lang::SeedKind::Product,
                    }).then_some(seed))
                    .ok_or_else(||self.failure(expression.id,"묶음 구성의 실제 선언 owner가 없습니다"))?;
                for field in fields {
                    let actual = self.expression(scope_id, scope, &field.expr)?;
                    let pin = field.resolved_pin.as_ref().and_then(|name|schema.params.iter().find(|pin|&pin.pin_name==name))
                        .ok_or_else(||self.failure(field.id,"묶음 필드의 실제 받는자리 owner가 없습니다"))?;
                    let expected = self.declared(&pin.type_ref,pin.id)?;
                    self.same_shape(&expected,&actual,field.id)?;
                }
                Shape::Declared(TypeRef::Named(owner.clone()))
            }
            ExprKind::Pack { fields } => {
                for (_, value) in fields { self.expression(scope_id,scope,value)?; }
                Shape::Declared(TypeRef::Named("묶음".into()))
            }
            ExprKind::Tensor(tensor) => {
                let actual=self.expression(scope_id,scope,tensor.input())?;
                match tensor {
                    ddonirang_lang::TensorProviderExpr::FromCollection{element_type,shape,..} => {
                        let expected=self.declared(element_type,expression.id)?;
                        let mut pending=vec![&actual];
                        while let Some(value)=pending.pop() {
                            if let Shape::Collection(items)=value {pending.extend(items.iter());}
                            else {self.same_shape(&expected,value,expression.id)?;}
                        }
                        Shape::Declared(TypeRef::Tensor{element:Box::new(element_type.clone()),shape:shape.clone()})
                    }
                    ddonirang_lang::TensorProviderExpr::ToCollection{..} => match actual {
                        Shape::Declared(TypeRef::Tensor{element,shape})=>{
                            let mut kind=*element;
                            for _ in shape {kind=TypeRef::Applied{name:"차림".into(),args:vec![kind]};}
                            self.declared(&kind,expression.id)?
                        }
                        _=>Shape::Unknown(expression.id),
                    },
                }
            }
            ExprKind::SeedLiteral { param, body } => {
                let mut nested = scope.clone();
                nested.insert(param.clone(),(BindingOwner{scope:expression.id,declaration:expression.id},Shape::Unknown(expression.id)));
                self.expression(expression.id,&mut nested,body)?;
                Shape::Unknown(expression.id)
            }
            ExprKind::Pipe { stages } => {
                for stage in stages { self.expression(scope_id,scope,stage)?; }
                Shape::Unknown(expression.id)
            }
            ExprKind::SumBranchValue { payload, owner, .. } => {
                for argument in payload { self.expression(scope_id,scope,&argument.value)?; }
                Shape::Declared(TypeRef::Named(owner.clone()))
            }
            ExprKind::TemplateRender { inject, .. } | ExprKind::FormulaEval { inject, .. } => {
                for (_, value) in inject { self.expression(scope_id,scope,value)?; }
                Shape::Unknown(expression.id)
            }
            ExprKind::Call { args, func } => {
                let actuals = args.iter().map(|arg| self.expression(scope_id, scope, &arg.expr)).collect::<Result<Vec<_>, _>>()?;
                if let Some(binding)=self.program.origin.manipulation_phase_calls.get(&expression.id){
                    if binding.definition!="B1.DEF.INPUT_PHASE.R1"||binding.ordinal_pin!="B1.PIN.INPUT_PHASE.ORDINAL.R1"
                        ||scope.contains_key(func)||actuals.len()!=1 {
                        return Err(self.failure(expression.id,"조작단계의 실제 정의·순서 받는자리 결속이 없습니다"));
                    }
                    let expected=self.declared(&TypeRef::Named("바른수".into()),expression.id)?;
                    self.same_shape(&expected,&actuals[0],args[0].expr.id)?;
                    let shape=Shape::Declared(TypeRef::Named("글".into()));
                    self.shapes.insert(expression.id,shape.clone());return Ok(shape);
                }
                if let Some(binding)=self.program.origin.subject_reference_calls.get(&expression.id){
                    let read=binding.definition=="VC04.DEF.ILGI.R1"&&binding.receiver_pin=="VC04.PIN.ILGI.REFERENCE.R1";
                    if (!read&&(binding.definition!="VC04.DEF.GARIKIGI.R1"||binding.receiver_pin!="VC04.PIN.GARIKIGI.TARGET.R1"))||scope.contains_key(func){
                        return Err(self.failure(expression.id,"가리키기의 정의·받는자리 정체가 다릅니다"));
                    }
                    let receiver=if read {match actuals.first(){Some(Shape::Declared(TypeRef::Applied{name,args})) if name=="가리킴"&&args.len()==1=>args.first(),_=>None}}
                        else {match actuals.first(){Some(Shape::Declared(kind))=>Some(kind),_=>None}};
                    let Some(TypeRef::Named(owner))=receiver else {
                        return Err(self.failure(expression.id,"가리키기는 정적으로 해소된 임자 값을 받습니다"));
                    };
                    if !self.program.items.iter().any(|TopLevelItem::SeedDef(seed)|seed.canonical_name==*owner&&seed.seed_kind==ddonirang_lang::SeedKind::Schema(ddonirang_lang::SchemaCategory::Subject)){
                        return Err(self.failure(expression.id,"가리키기의 실제 임자 선언이 없습니다"));
                    }
                    let shape=Shape::Declared(if read {TypeRef::Applied{name:"맺음".into(),args:vec![TypeRef::Named(owner.clone()),TypeRef::Named("가리킴안됨".into())]}}
                        else {TypeRef::Applied{name:"가리킴".into(),args:vec![TypeRef::Named(owner.clone())]}});
                    self.shapes.insert(expression.id,shape.clone());return Ok(shape);
                }
                for arg in args {self.source_sites.insert(arg.id,arg.expr.span);}
                let canonical = ddonirang_lang::stdlib::canonicalize_stdlib_alias(func);
                if canonical == "입력사상.사건들" && args.is_empty() && !scope.contains_key(func)
                    && !self.callables.contains_key(func)
                    && crate::work_cost::builtin_work_cost_rule(canonical).is_ok_and(|rule| rule.is_some()) {
                    let shape = Shape::CollectionSlice(Arc::from([Shape::InputEvent]));
                    self.shapes.insert(expression.id, shape.clone());
                    return Ok(shape);
                }
                use ddonirang_lang::ordinary_callable::{resolve_scalar_math_callable,ScalarMathCallableOwner};
                let math_owner=(!scope.contains_key(func)).then(||resolve_scalar_math_callable(self.program,func)).flatten();
                let ordinary = !scope.contains_key(func) && (matches!(math_owner,Some(ScalarMathCallableOwner::Declared(_)))
                    || crate::work_cost::builtin_work_cost_rule(canonical).is_ok_and(|rule| rule.is_none()));
                let candidates = ordinary.then(|| self.callables.get(func)).flatten();
                if let Some(candidates) = candidates {
                    if candidates.len() != 1 {
                        // Do not invent a winner or issue a numeric owner.
                        // Existing non-numeric overload dispatch still owns
                        // disjoint byte/text calls and ambiguity rejection.
                        let shape=Shape::Unknown(expression.id);
                        self.shapes.insert(expression.id,shape.clone());return Ok(shape);
                    }
                    let seed = candidates[0];
                    let mut supplied=BTreeMap::new();
                    for (ordinal, (arg, actual)) in args.iter().zip(&actuals).enumerate() {
                        let pin = if let Some(name) = &arg.resolved_pin { seed.params.iter().find(|pin| &pin.pin_name == name) }
                            else if arg.josa.is_none() { seed.params.get(ordinal) } else { None };
                        if pin.is_none() { return Err(self.failure(arg.id, "호출 인자의 유일 받는자리 owner가 없습니다")); }
                        if let Some(pin) = pin {
                            supplied.insert(pin.id,(arg.id,actual.clone()));
                            if has_concrete_numeric_element_owner(&pin.type_ref) {
                                let expected = self.declared(&pin.type_ref, pin.id)?;
                                if let (Shape::Numeric(_),Shape::Declared(actual_type))=(&expected,actual) {
                                    // Preserve the established argument-type diagnostic identity.
                                    // Parser field checks already use this code before execution;
                                    // a static rejection must not become a different teaching error.
                                    return Err(format!("E_RUNTIME_TYPE_MISMATCH: 핀={} 기대={:?} 실제={actual_type:?}; 실행 전 원문 {}:{}..{}",
                                        pin.pin_name,pin.type_ref,self.program.origin.file_path,arg.expr.span.start,arg.expr.span.end));
                                }
                                self.same_shape(&expected, actual, arg.id)?;
                            }
                        }
                    }
                    let ordered=seed.params.iter().map(|pin|supplied.get(&pin.id).cloned()
                        .unwrap_or((pin.id,Shape::Unknown(pin.id)))).collect::<Vec<_>>();
                    self.analyze_callable(seed,Some(&ordered))?
                } else if !scope.contains_key(func) && !self.callables.contains_key(func) && !self.callables.contains_key(canonical)
                    && matches!(canonical,"표준장면.원"|"표준장면.다각형"|"표준장면.글"|"표준장면.평면")
                    && crate::work_cost::builtin_work_cost_rule(canonical).is_ok_and(|rule|rule.is_some()) {
                    let owner=resolve_source_builtin_roles(self.program,func,args,false)?.ok_or_else(||self.failure(expression.id,"장면 builtin의 실제 source 역할 owner가 없습니다"))?;
                    let roles=owner.arg_for_role.iter().map(|(role,index)|(*role,&actuals[*index])).collect::<BTreeMap<_,_>>();
                    match canonical {
                        "표준장면.원"=>{
                            if let Some(center)=roles.get("중심") {self.scene_pair(center,expression.id)?;}
                            if let Some(radius)=roles.get("반지름") {self.scene_scalar(radius,expression.id)?;}
                        },
                        "표준장면.글"=>if let Some(position)=roles.get("자리") {self.scene_pair(position,expression.id)?;},
                        "표준장면.평면"=>for role in ["가로범위","세로범위"] {if let Some(bounds)=roles.get(role) {self.scene_pair(bounds,expression.id)?;}},
                        "표준장면.다각형"=>if let Some(vertices)=roles.get("꼭짓점들") {match vertices {
                            Shape::Collection(items)|Shape::CollectionSlice(items)=>for pair in items.iter() {self.scene_pair(pair,expression.id)?;},
                            Shape::CollectionFlow(_)|Shape::DeferredSelection{..}=>{
                                let pairs=self.source_selections(vertices,None,&mut BTreeSet::new()).ok_or_else(||self.failure(expression.id,"장면 꼭짓점의 모든 원문 공급이 필요합니다"))?;
                                for pair in pairs {self.scene_pair(&pair,expression.id)?;}
                            },
                            Shape::Declared(TypeRef::Applied{name,args}) if name=="차림" && args.len()==1=>self.scene_pair(&Shape::Declared(args[0].clone()),expression.id)?,
                            _=>return Err(self.failure(expression.id,"장면 꼭짓점의 독립 source 좌표씨가 없습니다")),
                        }},
                        _=>unreachable!(),
                    }
                    self.declared(&TypeRef::Named(if canonical=="표준장면.평면" {"장면값"}else{"장면요소값"}.into()),expression.id)?
                } else if !scope.contains_key(func) && !self.callables.contains_key(func) && !self.callables.contains_key(canonical)
                    && matches!(canonical,"토막내기"|"추가")
                    && crate::work_cost::builtin_work_cost_rule(canonical).is_ok_and(|rule|rule.is_some()) {
                    // R1.2 §5's explicit private bridge. The parser's actual
                    // admitted role mapping is required; never infer a role
                    // from ordinal or re-resolve a user callable by spelling.
                    let owner=resolve_source_builtin_roles(self.program,func,args,false)?.ok_or_else(||self.failure(expression.id,"collection builtin의 실제 source 역할 owner가 없습니다"))?;
                    let roles=owner.arg_for_role.iter().map(|(role,index)|(*role,&actuals[*index])).collect::<BTreeMap<_,_>>();
                    let expected=if canonical=="토막내기" {&["차림","시작","끝"][..]}else{&["차림","요소"][..]};
                    if roles.len()!=args.len() || args.len()!=expected.len() || expected.iter().any(|role|!roles.contains_key(role)) {Shape::Unknown(expression.id)}
                    else {
                        if canonical=="토막내기" {for role in ["시작","끝"] {
                            let expected=self.declared(&NumericKind::Integer.type_ref(),expression.id)?;
                            self.same_shape(&expected,roles[role],expression.id)?;
                        }}
                        let owner=match roles["차림"] {
                            Shape::BagPreview(owner)|Shape::CollectionFlow(owner)=>{
                                // 추가/토막내기 return a new immutable value.
                                // Only an actual binding write joins it back;
                                // computing and discarding a result must not
                                // add candidate elements to its input owner.
                                let source=&self.bags.nodes[self.bags.root(*owner)];
                                let seeds=source.seeds.clone();let unknown=source.unknown;
                                let fresh=self.bags.variable(None);let root=self.bags.root(fresh);
                                self.bags.nodes[root].seeds=seeds;self.bags.nodes[root].unknown=unknown;fresh
                            },
                            original=>self.bags.variable(Some(original.clone())),
                        };
                        let root=self.bags.root(owner);self.bags.nodes[root].collection_signature=true;
                        if canonical=="추가" {self.bags.nodes[root].seeds.push(Shape::Collection(vec![roles["요소"].clone()].into()));}
                        // The original source T survives a zero-length slice;
                        // no runtime element or length grants that T.
                        Shape::BagPreview(owner)
                    }
                } else if !scope.contains_key(func) && !self.callables.contains_key(func) && !self.callables.contains_key(canonical)
                    && matches!(canonical,"무작위가방.만들기"|"무작위가방.꺼내기"|"무작위가방.미리보기"|"무작위가방.남은것")
                    && crate::work_cost::builtin_work_cost_rule(canonical).is_ok_and(|rule|rule.is_some()) {
                    match (canonical,actuals.as_slice()) {
                        ("무작위가방.만들기",[_,candidates])=>Shape::Bag(self.bags.variable(Some(candidates.clone()))),
                        ("무작위가방.꺼내기",[Shape::Bag(owner)])=>Shape::BagDraw(*owner),
                        ("무작위가방.미리보기",[Shape::Bag(owner),_])|("무작위가방.남은것",[Shape::Bag(owner)])=>Shape::BagPreview(*owner),
                        _=>Shape::Unknown(expression.id),
                    }
                } else if !scope.contains_key(func) && matches!(canonical,"차림"|"목록")
                    && crate::work_cost::builtin_work_cost_rule(canonical).is_ok_and(|rule|rule.is_some()) {
                    Shape::Collection(actuals.into())
                } else if let Some(ScalarMathCallableOwner::Builtin(builtin))=math_owner {
                    if let [Shape::Unit(dim)]=actuals.as_slice() {
                        use ddonirang_lang::ordinary_callable::ScalarMathBuiltin as B;
                        // Existing UnitDim/registered unit owner, not the
                        // dimensionless scalar pin profile or runtime bits.
                        match builtin {
                            B::Sin|B::Cos if dim.is_dimensionless() || *dim==ddonirang_core::UnitDim::ANGLE=>self.declared(&NumericKind::Approximate.type_ref(),expression.id)?,
                            B::Sqrt=>match dim.sqrt() {Some(dim)=>self.unit_result(dim,expression.id)?,None=>Shape::Unknown(expression.id)},
                            _=>Shape::Unknown(expression.id),
                        }
                    } else if let Some(pin)=builtin.scalar_pin_type() {
                        if actuals.len()!=1 {return Err(self.failure(expression.id,"수학 함수 scalar overload는 받는자리 하나가 필요합니다"));}
                        let expected=self.declared(&pin,expression.id)?;
                        self.same_shape(&expected,&actuals[0],args[0].id)?;
                        expected
                    }else{
                        if actuals.len()!=2 {return Err(self.failure(expression.id,"atan2는 인자 두 개를 받습니다"));}
                        // A conditional result, never a constraint on the pins.
                        // Resolve source defaults first; numeric consumers must
                        // then prove both input owners independently canonical.
                        Shape::ConditionalScalar{site:expression.id,args:actuals}
                    }
                } else if !scope.contains_key(func) && crate::work_cost::builtin_work_cost_rule(canonical).is_ok_and(|rule|rule.is_some()) {
                    let result=self.builtin_results.get(canonical).filter(|results|results.len()==1).and_then(|results|results.first()).cloned();
                    if let Some(kind)=result {self.declared(&kind,expression.id)?}else{Shape::Unknown(expression.id)}
                } else { Shape::Unknown(expression.id) }
            }
            ExprKind::Thunk(body) => {
                self.statements(body.id, &mut scope.clone(), &body.stmts, None)?;
                Shape::Unknown(expression.id)
            }
            ExprKind::FieldAccess { target, field, .. } => {
                let target = self.expression(scope_id, scope, target)?;
                if self.admitted_input_event(&target) {
                    // The existing typed projection supplies these values:
                    // receive_ordinal is an integer; Number/Coordinate raw is
                    // Fixed64. Runtime still rejects a field absent on the
                    // selected event kind; a Pack with the same names is not
                    // this source owner.
                    let kind = match field.as_str() {
                        "순서" => Some(TypeRef::Named("바른수".into())),
                        "수값" => Some(TypeRef::Named("셈수".into())),
                        "좌표값" => Some(TypeRef::Applied {name:"차림".into(),args:vec![TypeRef::Named("셈수".into())]}),
                        "채널" | "종류" => Some(TypeRef::Named("글".into())),
                        _ => None,
                    };
                    if let Some(kind) = kind {
                        let shape = self.declared(&kind, expression.id)?;
                        self.shapes.insert(expression.id, shape.clone());
                        return Ok(shape);
                    }
                }
                if let Shape::BagDraw(owner)=target {
                    let shape=match field.as_str() {"값"=>Shape::BagElement(owner),"가방"=>Shape::Bag(owner),_=>Shape::Unknown(expression.id)};
                    self.shapes.insert(expression.id,shape.clone());return Ok(shape);
                }
                // The field's declaration supplies its type. Neither the
                // field spelling alone nor an evaluated Pack is an owner.
                let field_type = match target {
                    Shape::Declared(TypeRef::Named(owner)) => self.records.get(&owner)
                        .or_else(||self.nominal_fields.get(&owner))
                        .or_else(||self.payload_fields.get(&owner).and_then(Option::as_ref))
                        .and_then(|fields| fields.get(field)).cloned(),
                    _ => None,
                };
                if let Some(kind) = field_type { self.declared(&kind, expression.id)? }
                else { Shape::Unknown(expression.id) }
            }
            ExprKind::Suffix { value: target, at:ddonirang_lang::AtSuffix::Unit(symbol) }=>{
                self.expression(scope_id,scope,target)?;
                match ddonirang_core::units::unit_spec_from_symbol(symbol) {
                    Some(spec)=>Shape::Unit(spec.dim),None=>Shape::Unknown(expression.id),
                }
            }
            ExprKind::Suffix { value: target, .. }
            | ExprKind::Eval { thunk: target, .. } | ExprKind::Nuance { expr: target, .. } => {
                self.expression(scope_id, scope, target)?; Shape::Unknown(expression.id)
            }
            _ => Shape::Unknown(expression.id),
        }};
        let literal_tree = if expression.numeric_literal_source.is_some() { true }
            else if let Some(operand) = expression.source_negated_operand() { self.literal_trees.contains(&operand.id) }
            else if let ExprKind::Infix {left,op,right} = &expression.kind {
                matches!(op.as_str(), "+"|"-"|"*"|"/") && self.literal_trees.contains(&left.id) && self.literal_trees.contains(&right.id)
            } else { false };
        if literal_tree { self.literal_trees.insert(expression.id); }
        self.shapes.insert(expression.id, shape.clone());
        Ok(shape)
    }

    fn statements(&mut self, scope_id: NodeId, scope: &mut Scope, statements: &[Stmt], callable: Option<&SeedDef>) -> Result<(), String> {
        for statement in statements {
            match statement {
                Stmt::SumDefinition{name,branches,..}=>{self.sums.insert(name.clone(),branches.clone());}
                Stmt::Definition { id, name, type_ref, value, reaction, .. } => {
                    let actual = if let (Some(registration),ExprKind::Thunk(body))=(reaction,&value.kind) {
                        // A registered provider has its own declared payload
                        // pin. Analyze its body under that source scope, not
                        // the receiver's ambient locals or runtime Pack shape.
                        let pin=&registration.payload;
                        let shape=self.declared(&pin.type_ref,pin.id)?;
                        let owner=BindingOwner{scope:body.id,declaration:pin.id};
                        let mut nested=scope.clone();
                        self.lexical_locals.insert(owner);
                        nested.insert(pin.pin_name.clone(),(owner,shape));
                        self.statements(body.id,&mut nested,&body.stmts,None)?;
                        Shape::Unknown(value.id)
                    }else{self.expression(scope_id, scope, value)?};
                    let mut shape = if matches!(type_ref, TypeRef::Infer) { actual } else {
                        let expected = self.declared(type_ref, *id)?;
                        self.same_shape(&expected, &actual, *id)?; expected
                    };
                    let owner = scope.get(name).filter(|(owner,_)|owner.declaration==*id).map(|(owner,_)|*owner)
                        .unwrap_or(BindingOwner { scope: scope_id, declaration: *id });
                    if matches!(type_ref,TypeRef::Infer) {shape=self.collection_binding(owner,shape);}
                    if matches!(type_ref,TypeRef::Infer) && matches!(&shape,Shape::Declared(TypeRef::Named(name)) if name=="없음") {
                        self.inferred_absences.insert(owner);
                    }
                    self.bindings.insert(owner, shape.clone()); scope.insert(name.clone(), (owner, shape.clone()));
                    self.lexical_locals.insert(owner);
                    if scope_id==self.program.id {self.callable_globals.insert(name.clone(),(owner,shape));}
                }
                Stmt::DeclBlock { items, .. } => for item in items {
                    self.declaration_item(scope_id,scope,item)?;
                    if item.name=="프레임수" && item.value.is_some()
                        && (scope_id==self.program.id || callable.is_some_and(|seed|seed.seed_kind==ddonirang_lang::SeedKind::Umjikssi)) {
                        if let (Some((_,previous)),Some((_,current)))=(self.callable_globals.get(&item.name).cloned(),scope.get(&item.name).cloned()) {
                            self.same_shape(&previous,&current,item.id)?;
                        }
                    }
                    // Initialized 채비 already publishes a resource in the
                    // runtime's 움직씨 path. Supply that authored owner to
                    // callees too; do not derive it from a saved/default value.
                    if item.value.is_some() && callable.is_some_and(|seed|seed.seed_kind==ddonirang_lang::SeedKind::Umjikssi) {
                        if let Some(binding)=scope.get(&item.name) {
                            self.callable_globals.insert(item.name.clone(),binding.clone());
                        }
                    }
                },
                Stmt::Mutate { id, target, value, .. } => {
                    let actual = self.expression(scope_id, scope, value)?;
                    if let ExprKind::Var(name) = &target.kind {
                        if let Some(seed) = callable.filter(|seed| seed.canonical_name == *name) {
                            self.callable_return(seed,&actual,*id)?;
                        } else if let Some((owner, expected)) = scope.get(name).cloned() {
                            // An inferred absence does not issue a payload
                            // type. Adopt the first independently supplied
                            // source owner, as the existing runtime does;
                            // an explicit `:없음` declaration is not widened.
                            if self.inferred_absences.contains(&owner)
                                && !matches!(&actual,Shape::Unknown(_))
                                && !matches!(&actual,Shape::Declared(TypeRef::Named(name)) if name=="없음") {
                                self.bindings.insert(owner,actual.clone());
                                scope.insert(name.clone(),(owner,actual.clone()));
                                self.inferred_absences.remove(&owner);
                            } else {
                            self.same_shape(&expected,&actual,*id).map_err(|error| {
                                if let Some(kind)=self.pattern_binding_types.get(&owner) {
                                    format!("E_RUNTIME_TYPE_MISMATCH: 핀={name} 기대={kind:?}; 실행 전 {}",self.failure(value.id,&error))
                                }else{error}
                            })?;
                            }
                        }
                        else {
                            let owner = BindingOwner { scope: scope_id, declaration: *id };
                            let actual=self.collection_binding(owner,actual);
                            if matches!(&actual,Shape::Declared(TypeRef::Named(name)) if name=="없음") {self.inferred_absences.insert(owner);}
                            self.bindings.insert(owner, actual.clone()); scope.insert(name.clone(), (owner, actual));
                        }
                        if let Some((_,shape))=scope.get(name) {self.shapes.insert(target.id,shape.clone());}
                        if callable.is_some_and(|seed|seed.seed_kind==ddonirang_lang::SeedKind::Umjikssi && seed.canonical_name!=*name)
                            || self.root_world_hook && callable.is_none() {
                            if let Some((owner,shape))=scope.get(name).cloned().filter(|(owner,_)|!self.lexical_locals.contains(owner)) {
                                if let Some((_,previous))=self.callable_globals.get(name).cloned() {self.same_shape(&previous,&shape,*id)?;}
                                else {self.callable_globals.insert(name.clone(),(owner,shape));}
                            }
                        }
                    } else {
                        let expected = self.expression(scope_id, scope, target)?;
                        // The existing non-scalar place owner still checks
                        // writes. Unknown grants no numeric saved descriptor.
                        if !matches!(expected,Shape::Unknown(_)) {
                            self.same_shape(&expected,&actual,*id).map_err(|error|self.replacement_failure(target,&actual,error))?;
                        }
                    }
                }
                Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => { self.expression(scope_id, scope, expr)?; }
                Stmt::Return { value, .. } => {
                    let actual=self.expression(scope_id,scope,value)?;
                    if let Some(seed)=callable {self.callable_return(seed,&actual,value.id)?;}
                }
                Stmt::ResultFlow { id, producer, destination, .. } => {
                    let actual=self.expression(scope_id,scope,producer)?;
                    let expected=self.expression(scope_id,scope,destination)?;
                    self.same_shape(&expected,&actual,*id)?;
                }
                Stmt::If { condition, then_body, else_body, .. } => {
                    self.expression(scope_id, scope, condition)?;
                    self.statements(scope_id, &mut scope.clone(), &then_body.stmts, callable)?;
                    if let Some(body) = else_body { self.statements(scope_id, &mut scope.clone(), &body.stmts, callable)?; }
                }
                Stmt::Hook{body,..}=>{
                    let old=self.root_world_hook;
                    if callable.is_none() && scope_id==self.program.id {self.root_world_hook=true;}
                    let result=self.statements(scope_id,&mut scope.clone(),&body.stmts,callable);
                    self.root_world_hook=old;result?;
                }
                Stmt::Repeat { body, .. } | Stmt::BeatBlock { body, .. } => {
                    self.statements(scope_id, &mut scope.clone(), &body.stmts, callable)?;
                }
                Stmt::While { condition, body, .. } | Stmt::Guard { condition, body, .. }
                | Stmt::HookWhenBecomes { condition, body, .. } | Stmt::HookWhile { condition, body, .. } => {
                    self.expression(scope_id, scope, condition)?;
                    self.statements(scope_id, &mut scope.clone(), &body.stmts, callable)?;
                }
                Stmt::Choose {branches,else_body,..} => {
                    for branch in branches {
                        self.expression(scope_id,scope,&branch.condition)?;
                        self.statements(scope_id,&mut scope.clone(),&branch.body.stmts,callable)?;
                    }
                    self.statements(scope_id,&mut scope.clone(),&else_body.stmts,callable)?;
                }
                Stmt::ForEach {id,span,item,item_type,iterable,body,..} => {
                    self.source_sites.insert(*id,*span);
                    let iterable=self.expression(scope_id,scope,iterable)?;
                    // The parser represents an omitted element annotation
                    // with Some(Infer) too. It is not a declared type owner.
                    let shape=if let Some(kind)=item_type.as_ref().filter(|kind|!matches!(kind,TypeRef::Infer)) {
                        let expected=self.declared(kind,*id)?;
                        if let Shape::Collection(items)=&iterable {for actual in items.iter() {self.same_shape(&expected,actual,*id)?;}}
                        else if matches!(iterable,Shape::CollectionFlow(_)|Shape::DeferredSelection{..}) {
                            let actual=self.select_one(iterable.clone(),None,*id)?;
                            self.same_shape(&expected,&actual,*id)?;
                        }
                        expected
                    } else if let Shape::BagPreview(owner)=&iterable {Shape::BagElement(*owner)
                    } else if matches!(iterable,Shape::CollectionFlow(_)|Shape::DeferredSelection{..}) {
                        self.select_one(iterable.clone(),None,*id)?
                    } else if let Shape::Declared(TypeRef::Applied{name,args})=&iterable {
                        if ddonirang_lang::stdlib::canonicalize_type_alias(name)=="차림" && args.len()==1 {self.declared(&args[0],*id)?}
                        else {Shape::Unknown(*id)}
                    } else if let Shape::Collection(items)|Shape::CollectionSlice(items)=iterable {
                        Shape::SelectionCandidates(items)
                    } else {Shape::Unknown(*id)};
                    let mut nested=scope.clone();
                    let owner=BindingOwner{scope:body.id,declaration:*id};self.lexical_locals.insert(owner);
                    nested.insert(item.clone(),(owner,shape));
                    self.statements(body.id,&mut nested,&body.stmts,callable)?;
                }
                Stmt::Quantifier {id,variable,domain,body,..} => {
                    let shape=self.declared(domain,*id)?;
                    let mut nested=scope.clone();
                    nested.insert(variable.clone(),(BindingOwner{scope:body.id,declaration:*id},shape));
                    self.statements(body.id,&mut nested,&body.stmts,callable)?;
                }
                Stmt::Try {id,action,body,..} => {
                    let shape=self.expression(scope_id,scope,action)?;
                    let mut nested=scope.clone();
                    nested.insert("그것".into(),(BindingOwner{scope:body.id,declaration:*id},shape));
                    self.statements(body.id,&mut nested,&body.stmts,callable)?;
                }
                Stmt::Receive {id,sender,binding,payload_type,condition,body,..} => {
                    if let Some(sender)=sender {self.expression(scope_id,scope,sender)?;}
                    let mut nested=scope.clone();
                    if let Some(name)=binding {
                        let shape=if let Some(kind)=payload_type {self.declared(&TypeRef::Named(kind.clone()),*id)?}else{Shape::Unknown(*id)};
                        nested.insert(name.clone(),(BindingOwner{scope:body.id,declaration:*id},shape));
                    }
                    if let Some(condition)=condition {self.expression(body.id,&mut nested,condition)?;}
                    self.statements(body.id,&mut nested,&body.stmts,callable)?;
                }
                Stmt::Send {sender,payload,receiver,..} => {
                    if let Some(sender)=sender {self.expression(scope_id,scope,sender)?;}
                    self.expression(scope_id,scope,payload)?;self.expression(scope_id,scope,receiver)?;
                }
                Stmt::Contract {condition,then_body,else_body,..} => {
                    self.expression(scope_id,scope,condition)?;
                    if let Some(body)=then_body {self.statements(scope_id,&mut scope.clone(),&body.stmts,callable)?;}
                    self.statements(scope_id,&mut scope.clone(),&else_body.stmts,callable)?;
                }
                Stmt::SumMatch {subject,arms,default_body,..} => {
                    self.expression(scope_id,scope,subject)?;
                    for arm in arms {
                        let mut nested=scope.clone();
                        let schema=self.sum_payload(&arm.owner,&arm.applied_owner,&arm.branch,arm.span);
                        for binding in &arm.binder {
                            let kind=schema.as_ref().and_then(|pins|pins.iter().find(|pin|pin.binder==binding.pin)).map(|pin|pin.type_ref.clone());
                            let shape=match &kind {Some(kind)=>self.declared(kind,binding.id)?,None=>Shape::Unknown(binding.id)};
                            let owner=BindingOwner{scope:arm.id,declaration:binding.id};self.lexical_locals.insert(owner);
                            if let Some(kind)=kind {self.pattern_binding_types.insert(owner,kind);}
                            nested.insert(binding.pin.clone(),(owner,shape));
                        }
                        self.statements(arm.id,&mut nested,&arm.body.stmts,callable)?;
                    }
                    if let Some(body)=default_body {self.statements(scope_id,&mut scope.clone(),&body.stmts,callable)?;}
                }
                Stmt::PublicObservation { fields, .. } => for field in fields { self.expression(scope_id, scope, &field.value)?; },
                Stmt::MetaBlock { .. } | Stmt::Pragma { .. } | Stmt::Break { .. } | Stmt::ContinueLoop { .. } | Stmt::SumDefinition { .. } => {},
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct BindingOwner { pub scope: NodeId, pub declaration: NodeId }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Variable(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum DefaultPreference { Integer, IntegerDivision, Decimal }

impl DefaultPreference {
    fn kind(self) -> NumericKind {
        match self {
            Self::Integer => NumericKind::Integer,
            Self::IntegerDivision => NumericKind::Rational,
            Self::Decimal => NumericKind::Approximate,
        }
    }
}

#[derive(Clone, Debug)]
struct Component {
    parent: usize,
    size: usize,
    required: Option<(NumericKind, NodeId)>,
    preference: DefaultPreference,
    allowed: u8,
}

#[derive(Clone, Debug)]
struct Operation { left: Variable, right: Variable, result: Variable, op: String, site: NodeId }

/// Pending projection state; no evaluated values or process-wide registry.
#[derive(Default, Debug, Clone)]
pub(crate) struct NumericConstraints {
    components: Vec<Component>,
    bindings: BTreeMap<BindingOwner, Variable>,
    operations: Vec<Operation>,
}

impl NumericConstraints {
    pub(crate) fn variable(&mut self, preference: DefaultPreference) -> Variable {
        let index = self.components.len();
        self.components.push(Component { parent: index, size: 1, required: None, preference, allowed: 0b1_1111 });
        Variable(index)
    }

    pub(crate) fn binding(&mut self, owner: BindingOwner) -> Variable {
        if let Some(variable) = self.bindings.get(&owner) { return *variable; }
        let variable = self.variable(DefaultPreference::Integer);
        self.bindings.insert(owner, variable);
        variable
    }

    fn root(&self, variable: Variable) -> usize {
        let mut index = variable.0;
        while self.components[index].parent != index { index = self.components[index].parent; }
        index
    }

    fn explicit_kind(&self, variable: Variable) -> Option<NumericKind> {
        self.components[self.root(variable)].required.map(|(kind, _)| kind)
    }

    fn conflict(left: (NumericKind, NodeId), right: (NumericKind, NodeId)) -> String {
        format!("E_TYPE_MISMATCH: 하나의 수 결속에 서로 다른 씨가 요구됩니다: {:?} (원문 노드 {}) / {:?} (원문 노드 {})",
            left.0.type_ref(), left.1, right.0.type_ref(), right.1)
    }

    pub(crate) fn require(&mut self, variable: Variable, kind: NumericKind, site: NodeId) -> Result<(), String> {
        let root = self.root(variable);
        if let Some(previous) = self.components[root].required {
            if previous.0 != kind { return Err(Self::conflict(previous, (kind, site))); }
        }
        self.restrict(variable, kind.bit(), site)?;
        self.components[root].required.get_or_insert((kind, site));
        Ok(())
    }

    /// Equality of unresolved constraints, not a conversion of issued values.
    /// Check conflicts before publishing a union so an error leaves it intact.
    pub(crate) fn same(&mut self, left: Variable, right: Variable) -> Result<(), String> {
        let (left, right) = (self.root(left), self.root(right));
        if left == right { return Ok(()); }
        // Union by size bounds root walks without recursion or an unbounded
        // cycle/fixed-point search. Index is only the deterministic tie break.
        let left_size = self.components[left].size;
        let right_size = self.components[right].size;
        let (parent, child) = if left_size > right_size || left_size == right_size && left < right {
            (left, right)
        } else { (right, left) };
        let (a, b) = (&self.components[parent], &self.components[child]);
        if let (Some(a), Some(b)) = (a.required, b.required) {
            if a.0 != b.0 { return Err(Self::conflict(a, b)); }
        }
        let required = a.required.or(b.required);
        let preference = a.preference.max(b.preference);
        let allowed = a.allowed & b.allowed;
        if allowed == 0 { return Err("E_TYPE_MISMATCH: 같은 수 결속의 정적 제약이 충돌합니다".into()); }
        let size = a.size.checked_add(b.size).expect("disjoint component count fits allocated nodes");
        self.components[parent].required = required;
        self.components[parent].preference = preference;
        self.components[parent].size = size;
        self.components[parent].allowed = allowed;
        self.components[child].parent = parent;
        Ok(())
    }

    /// Called only after the complete compilation's constraints are collected.
    pub(crate) fn resolved(&self, variable: Variable) -> NumericKind {
        let component = &self.components[self.root(variable)];
        let preferred = component.required.map(|(kind, _)| kind).unwrap_or_else(|| component.preference.kind());
        if component.allowed & preferred.bit() != 0 { preferred }
        else { NumericKind::all().into_iter().find(|kind| component.allowed & kind.bit() != 0).expect("nonempty numeric domain") }
    }

    fn restrict(&mut self, variable: Variable, allowed: u8, site: NodeId) -> Result<bool, String> {
        let root = self.root(variable);
        let previous = self.components[root].allowed;
        let next = previous & allowed;
        if next == 0 { return Err(format!("E_TYPE_MISMATCH: 원문 노드 {site}의 수 연산 제약을 만족하는 씨가 없습니다")); }
        self.components[root].allowed = next;
        Ok(previous != next)
    }

    fn operation(&mut self, left: Variable, right: Variable, op: &str, site: NodeId) -> Variable {
        let preference = if op == "/" { DefaultPreference::IntegerDivision } else { DefaultPreference::Integer };
        let result = self.variable(preference);
        self.operations.push(Operation { left, right, result, op: op.into(), site });
        result
    }

    fn propagate(&mut self, incidence: &[Vec<usize>], initial: impl IntoIterator<Item=usize>) -> Result<(), String> {
        // Each domain shrinks at most five times. An operation is requeued
        // only when a participating component shrinks, not by full rescans.
        let mut pending = VecDeque::new();
        let mut queued = vec![false; self.operations.len()];
        for index in initial { if !queued[index] { pending.push_back(index); queued[index] = true; } }
        while let Some(index) = pending.pop_front() {
                queued[index] = false;
                let operation = self.operations[index].clone();
                use NumericKind::{Approximate as A, BigInteger as B, Factor as F, Integer as I, Rational as R};
                let mut signatures = vec![(I,I,if operation.op == "/" { R } else { I }), (A,A,A), (R,R,R), (I,R,R), (R,I,R)];
                match operation.op.as_str() {
                    "+" | "-" => signatures.extend([(B,B,B),(F,F,R)]),
                    "*" => signatures.extend([(B,B,B),(F,F,F)]),
                    "/" => signatures.push((B,B,R)),
                    _ => return Err("E_TYPE_MISMATCH: 정적 수 연산 서명이 없습니다".into()),
                }
                let domains = [operation.left, operation.right, operation.result].map(|variable| self.components[self.root(variable)].allowed);
                let mut supported = [0_u8; 3];
                for (left, right, result) in signatures {
                    let signature = [left, right, result].map(NumericKind::bit);
                    if signature.iter().zip(domains).all(|(kind, domain)| kind & domain != 0) {
                        for (supported, kind) in supported.iter_mut().zip(signature) { *supported |= kind; }
                    }
                }
                for (variable, allowed) in [operation.left, operation.right, operation.result].into_iter().zip(supported) {
                    if self.restrict(variable, allowed, operation.site)? {
                        for next in &incidence[self.root(variable)] {
                            if !queued[*next] { pending.push_back(*next); queued[*next] = true; }
                        }
                    }
                }
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<(), String> {
        let mut incidence = vec![Vec::new(); self.components.len()];
        for (index, operation) in self.operations.iter().enumerate() {
            for variable in [operation.left, operation.right, operation.result] { incidence[self.root(variable)].push(index); }
        }
        self.propagate(&incidence, 0..self.operations.len())?;
        let mut roots = (0..self.components.len()).filter(|index| self.components[*index].parent == *index).collect::<Vec<_>>();
        roots.sort_by_key(|index| (std::cmp::Reverse(self.components[*index].preference), *index));
        for root in roots {
            let variable = Variable(root);
            let kind = self.resolved(variable);
            if self.restrict(variable, kind.bit(), 0)? { self.propagate(&incidence, incidence[root].iter().copied())?; }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn projection(source: &str) -> Result<(CanonProgram, NumericConstraintPlan), String> {
        let mut program = ddonirang_lang::parse_v25(source, "numeric-plan.ddn").map_err(|error| format!("{error:?}"))?;
        ddonirang_lang::canonicalize(&mut program).map_err(|error| format!("{error:?}"))?;
        let plan = NumericConstraintPlan::compile(&program,&HashSet::new())?;
        Ok((program, plan))
    }

    #[test]
    fn projection_actual_linear_composition_keeps_loop_source_owners() {
        let source=include_str!("../../solutions/seamgrim_ui_mvp/samples/alpha2_linear_composition.ddn");
        let mut program=ddonirang_lang::parse_v25(source,"linear-source.ddn").unwrap();
        ddonirang_lang::canonicalize(&mut program).unwrap();
        NumericConstraintPlan::compile(&program,&HashSet::new()).unwrap();
    }

    #[test]
    fn projection_collects_future_assignment_without_retyping() {
        let (program, plan) = projection("시간 <- 0. 간격 <- 0.02. 시간 <- 시간+간격. 위치 <- 0.").unwrap();
        let first = match &program.root_stmts[0] { Stmt::Mutate { value, .. } => value.id, _ => panic!("initial assignment") };
        let index = match program.root_stmts.last().unwrap() { Stmt::Mutate { value, .. } => value.id, _ => panic!("index assignment") };
        assert_eq!(plan.literals[&first], NumericKind::Approximate);
        assert_eq!(plan.literals[&index], NumericKind::Integer);
    }

    #[test]
    fn projection_aliases_share_immutable_aggregate_structure() {
        let constructors:[fn(Arc<[Shape]>)->Shape;4]=[
            Shape::Tuple,Shape::Collection,Shape::CollectionSlice,Shape::SelectionCandidates,
        ];
        for constructor in constructors {
            let mut shape=Shape::Numeric(Variable(0));
            for _ in 0..64 {shape=constructor(Arc::from([shape.clone(),shape]));}
            let copy=shape.clone();
            let children=|shape:&Shape|match shape {
                Shape::Tuple(items)|Shape::Collection(items)|Shape::CollectionSlice(items)|Shape::SelectionCandidates(items)=>Some(items.clone()),
                _=>None,
            };
            assert!(Arc::ptr_eq(&children(&shape).unwrap(),&children(&copy).unwrap()));
            let mut cursor=shape;let mut count=0;
            while let Some(items)=children(&cursor) {
                if let (Some(a),Some(b))=(children(&items[0]),children(&items[1])) {assert!(Arc::ptr_eq(&a,&b));}
                count+=1;cursor=items[0].clone();
            }
            assert_eq!(count,64);
        }
        // This tests alias copying only. Descriptor materialization and
        // constraint traversal still need their own bounded product evidence.
    }

    #[test]
    fn projection_preserves_scoped_pin_owners_and_detects_conflict() {
        let (_, plan) = projection("(입력값:셈수) 근사값:셈수:셈씨 := { 근사값 <- 입력값+1. }. (입력값:나눔수) 정확값:나눔수:셈씨 := { 정확값 <- 입력값+1. }.").unwrap();
        assert!(plan.literals.values().any(|kind| *kind == NumericKind::Approximate));
        assert!(plan.literals.values().any(|kind| *kind == NumericKind::Rational));
        let failure = projection("(입력값:셈수) 근사값:셈수:셈씨 := { 근사값 <- 입력값. }. (입력값:나눔수) 정확값:나눔수:셈씨 := { 정확값 <- 입력값. }. 변수 <- 1. (변수) 근사값 보여주기. (변수) 정확값 보여주기.").unwrap_err();
        assert!(failure.contains("서로 다른 씨"), "{failure}");
    }

    #[test]
    fn projection_original_literal_expectation_is_one_plan() {
        let (_, plan) = projection("값:나눔수 := 0.1+0.2. (값 == (3/10)) 보여주기.").unwrap();
        assert_eq!(plan.literals.len(), 4);
        assert!(plan.literals.values().all(|kind| *kind == NumericKind::Rational));
    }

    #[test]
    fn projection_tuple_constant_keys_do_not_retype_runtime_indexes() {
        for key in ["2 / 2","1.5 - 0.5","0.5 * 2"] {
            projection(&format!("자료 <- (1,\"가\"). (자료[({key})]+\"나\") 보여주기.")).unwrap();
        }
        assert!(projection("위치:셈수 := 1. 자료 <- (1,\"가\"). 자료[위치] 보여주기.").is_err());
        assert!(projection("위치:셈수 := 1. 자료 <- 차림[1,2]. 자료[위치] 보여주기.").is_err());
        projection("계산:움직씨 := { 값 := 없음. 값 <- 1. }.").unwrap();
    }

    #[test]
    fn projection_integer_bindings_divide_to_rational_not_integer() {
        let (program, plan) = projection("분자값 <- 3. 분모값 <- 10. 몫값 <- 분자값/분모값.").unwrap();
        let id = match program.root_stmts.last().unwrap() { Stmt::Mutate { value, .. } => value.id, _ => panic!("division") };
        assert_eq!(plan.expressions[&id], NumericKind::Rational.type_ref());
        assert!(plan.literals.values().all(|kind| *kind == NumericKind::Integer));
    }

    #[test]
    fn tuple_position_descriptor_does_not_unify_distinct_children() {
        let (program, plan) = projection("묶음값 := (1, 0.5, 참).").unwrap();
        let id = match &program.root_stmts[0] { Stmt::Definition { value, .. } => value.id, _ => panic!("tuple") };
        assert_eq!(plan.position_types[&id], RecordFieldType::Tuple { items: vec![
            RecordFieldType::Named { name: "바른수".into() }, RecordFieldType::Named { name: "셈수".into() }, RecordFieldType::Named { name: "참거짓".into() },
        ] });
    }

    #[test]
    fn declared_record_field_supplies_its_own_numeric_owner() {
        let (_, plan) = projection("(가로:셈수, 세로:나눔수) 좌표:묶음씨 := {}. (점:좌표) 가로계산:셈수:셈씨 := { 가로계산 <- 점.가로 + 1. }. (점:좌표) 세로계산:나눔수:셈씨 := { 세로계산 <- 점.세로 + 1. }.").unwrap();
        assert_eq!(plan.literals.values().filter(|kind| **kind == NumericKind::Approximate).count(), 1);
        assert_eq!(plan.literals.values().filter(|kind| **kind == NumericKind::Rational).count(), 1);
        let failure = projection("(가로:셈수) 좌표:묶음씨 := {}. (점:좌표) 계산:셈수:셈씨 := { 계산 <- 점.없는필드 + 1. }.").unwrap_err();
        assert!(failure.contains("owner"), "{failure}");
    }

    #[test]
    fn declared_collection_element_not_index_supplies_numeric_owner() {
        let (_, plan) = projection("(값들:(셈수)차림, 번호:바른수) 계산:셈수:셈씨 := { 계산 <- 값들[번호] + 1. }.").unwrap();
        assert_eq!(plan.literals.values().filter(|kind| **kind == NumericKind::Approximate).count(), 1);
        let failure = projection("(값들:(나눔수)차림, 번호:셈수) 계산:나눔수:셈씨 := { 계산 <- 값들[번호] + 1. }.").unwrap_err();
        assert!(failure.contains("서로 다른 씨"), "{failure}");
    }

    #[test]
    fn declared_collection_pin_supplies_raw_children_before_default() {
        let (_, plan) = projection("(값들:(셈수)차림, 번호:바른수) 계산:셈수:셈씨 := { 계산 <- 값들[번호] + 1. }. (값들=차림[1,2],번호=0) 계산 보여주기.").unwrap();
        assert_eq!(plan.literals.values().filter(|kind| **kind == NumericKind::Approximate).count(), 3);
        assert_eq!(plan.literals.values().filter(|kind| **kind == NumericKind::Integer).count(), 1);
        let failure = projection("(값들:(셈수)차림) 계산:셈수:셈씨 := { 계산 <- 값들[0] + 1. }. 정수:바른수 := 1. (값들=차림[정수]) 계산 보여주기.").unwrap_err();
        assert!(failure.contains("서로 다른 씨"), "{failure}");
    }

    #[test]
    fn constraints_solve_before_defaults_and_ignore_collection_order() {
        for reversed in [false, true] {
            let mut graph = NumericConstraints::default();
            let initial = graph.variable(DefaultPreference::Integer);
            let step = graph.variable(DefaultPreference::Decimal);
            let binding = graph.binding(BindingOwner { scope: 1, declaration: 4 });
            let pairs = if reversed { [(binding, step), (initial, binding)] } else { [(initial, binding), (binding, step)] };
            for (a, b) in pairs { graph.same(a, b).unwrap(); }
            assert_eq!(graph.resolved(initial), NumericKind::Approximate);
            graph.require(binding, NumericKind::Rational, 99).unwrap();
            assert_eq!(graph.resolved(step), NumericKind::Rational);
        }
    }

    #[test]
    fn scoped_bindings_positions_and_conflicts_do_not_leak() {
        let mut graph = NumericConstraints::default();
        let outer = graph.binding(BindingOwner { scope: 1, declaration: 3 });
        let inner = graph.binding(BindingOwner { scope: 2, declaration: 3 });
        let division = graph.variable(DefaultPreference::IntegerDivision);
        graph.require(outer, NumericKind::Integer, 11).unwrap();
        graph.require(inner, NumericKind::Approximate, 12).unwrap();
        assert!(graph.same(outer, inner).unwrap_err().contains("서로 다른 씨"));
        assert_eq!(graph.resolved(outer), NumericKind::Integer);
        assert_eq!(graph.resolved(inner), NumericKind::Approximate);
        assert_eq!(graph.resolved(division), NumericKind::Rational);
        assert_eq!(graph.binding(BindingOwner { scope: 1, declaration: 3 }), outer);
    }

    #[test]
    fn reverse_alias_chain_has_bounded_root_walks() {
        let mut graph = NumericConstraints::default();
        let variables = (0..1024).map(|_| graph.variable(DefaultPreference::Integer)).collect::<Vec<_>>();
        for pair in variables.windows(2).rev() { graph.same(pair[1], pair[0]).unwrap(); }
        graph.require(variables[0], NumericKind::Rational, 11).unwrap();
        for variable in variables {
            let mut index = variable.0;
            let mut depth = 0;
            while graph.components[index].parent != index {
                depth += 1;
                index = graph.components[index].parent;
            }
            assert!(depth <= 10);
            assert_eq!(graph.resolved(variable), NumericKind::Rational);
        }
    }
}
