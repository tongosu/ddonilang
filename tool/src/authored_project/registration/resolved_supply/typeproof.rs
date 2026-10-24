//! R3.1 source-bound static type producer and private proof projection.
//! No serde/public raw-byte constructor. A recorded field is accepted only
//! by comparison with a proof rebuilt from the actual source/type owners.
use super::*;
use ddonirang_core::platform::RecordFieldType;
use std::collections::HashSet;

pub(super) const DOMAIN:&[u8]=b"DDN.R3.TYPEPROOF.1\0";
const LIMIT:usize=1_000_000;

#[derive(Clone,Copy,PartialEq,Eq)]
enum Scalar {Integer,Approximate,Rational,Bool,Text}
#[derive(Clone,Copy)]
enum SymbolKind {Sum(DefinitionId),Nominal(u8,[u8;32]),StructuralProduct(usize)}
struct Symbol {name:String,scope:Span,site:Site,kind:SymbolKind}
enum Node {
    Leaf{bytes:Vec<u8>,scalar:Option<Scalar>},
    Sum{template:[u8;32],args:Vec<usize>},
    Tuple(Vec<usize>),Container(u8,Vec<usize>),Function{params:Vec<usize>,result:usize},
    Tensor{element:usize,shape:Vec<u64>},
    Record(Vec<(String,usize)>),
}

/// Flat arena: verification/encoding/drop never recurse through a type tree.
pub(super) struct StaticTypeProof {nodes:Vec<Node>,root:usize}

/// This owner is built from the already admitted project closure and actual
/// linked declarations. Names locate candidates; ambiguity/shadowing is
/// resolved before a private typed owner is returned, never first-wins.
pub(super) struct StaticTypeOwner {program:CanonProgram,symbols:Vec<Symbol>}
impl StaticTypeOwner {
    pub(super) fn from_source(source:&Source,nominals:&super::super::RegistrationDraft,templates:&[Template])->Result<Self,String> {
        source.verify_nominals(nominals)?;
        // Run the actual shared static checker (including declared numeric
        // families, effects and pattern subject compatibility) without a VM,
        // world, registration issuer, input or execution capability.
        let program=source.checked_program()?;
        let mut symbols=Vec::new();
        for (stmt,scope) in sum_declarations(&program) {
            let Stmt::SumDefinition{name,span,..}=stmt else {unreachable!()};
            let site=original_site(source,*span)?;
            let mut rows=templates.iter().filter(|row|row.site==site);
            let row=rows.next().ok_or_else(||fail("고름씨 형식 소유자의 예약 정체가 없습니다"))?;
            if rows.next().is_some() {return Err(fail("고름씨 형식 소유자가 중복됩니다"));}
            symbols.push(Symbol{name:name.clone(),scope,site,kind:SymbolKind::Sum(row.id)});
        }
        let recorded=crate::registered_atom_contract::source_binding::Candidate::parse(nominals.descriptor())
            .map_err(super::super::contract_error)?;
        for (seed_index,TopLevelItem::SeedDef(seed)) in program.items.iter().enumerate() {
            if !matches!(seed.seed_kind,ddonirang_lang::SeedKind::Product|ddonirang_lang::SeedKind::Schema(_)) {continue;}
            let site=original_site(source,seed.span)?;
            let kind=if matches!(seed.seed_kind,ddonirang_lang::SeedKind::Product) {SymbolKind::StructuralProduct(seed_index)} else {
                let tail=recorded.nominal.as_ref().ok_or_else(||fail("명목 형식 소유자의 실제 등록 자료가 없습니다"))?;
                let mut matches=tail.declarations.iter().filter(|row|row.start==site.start&&row.end==site.end
                    &&recorded.sources[row.source].snapshot==site.capture.as_slice());
                let declaration=matches.next().ok_or_else(||fail("명목 형식의 원문 대응이 없습니다"))?;
                if matches.next().is_some() {return Err(fail("명목 형식의 원문 대응이 중복됩니다"));}
                let schema=&tail.schemas[declaration.schema];
                if schema.module!=site.module.as_slice() {return Err(fail("명목 형식의 소유 모듈이 다릅니다"));}
                SymbolKind::Nominal(schema.category,schema.name_type.try_into().map_err(|_|fail("명목 형식 정체 길이 오류"))?)
            };
            symbols.push(Symbol{name:seed.canonical_name.clone(),scope:Span::new(0,program.origin.source.len()),site,kind});
        }
        Ok(Self{program,symbols})
    }
    fn visible(&self,at:Span)->Vec<&Symbol> {
        self.symbols.iter().filter(|symbol|symbol.scope.start<=at.start&&at.end<=symbol.scope.end).collect()
    }
    fn lookup<'a>(&self,visible:&[&'a Symbol],name:&str)->Result<Option<&'a Symbol>,String> {
        let nearest=visible.iter().filter(|symbol|symbol.name==name).map(|symbol|symbol.scope.end-symbol.scope.start).min();
        let Some(nearest)=nearest else {return Ok(None)};
        let mut matches=visible.iter().copied().filter(|symbol|symbol.name==name&&symbol.scope.end-symbol.scope.start==nearest);
        let first=matches.next().unwrap();
        if matches.next().is_some() {return Err(fail("형식 이름의 실제 lexical 소유자가 애매합니다"));}
        Ok(Some(first))
    }
    pub(super) fn pin_proofs(&self,source:&Source)->Result<BTreeMap<(Site,String,String),StaticTypeProof>,String> {
        let mut result=BTreeMap::new();
        for (stmt,_) in sum_declarations(&self.program) {
            let Stmt::SumDefinition{span,branches,..}=stmt else {unreachable!()};
            let site=original_site(source,*span)?;
            for branch in branches {for pin in &branch.payload {
                let key=(site.clone(),branch.name.clone(),pin.binder.clone());
                let proof=self.resolve(&pin.type_ref,pin.binder_span)?;
                if result.insert(key,proof).is_some() {return Err(fail("형식 증명의 실제 받는자리 대응이 중복됩니다"));}
            }}
        }
        Ok(result)
    }
    pub(super) fn callable_proofs(&self,source:&Source,sites:&BTreeSet<Site>)
        ->Result<BTreeMap<Site,(BTreeMap<String,StaticTypeProof>,Option<StaticTypeProof>)>,String> {
        let mut output=BTreeMap::new();
        for TopLevelItem::SeedDef(seed) in &self.program.items {
            let site=original_site(source,seed.span)?;
            if !sites.contains(&site){continue;}
            let mut params=BTreeMap::new();
            for pin in &seed.params {if params.insert(pin.pin_name.clone(),self.resolve(&pin.type_ref,seed.span)?).is_some(){return Err(fail("해소된 callable 받는자리가 중복됩니다"));}}
            let result=seed.result_type.as_ref().map(|kind|self.resolve(kind,seed.span)).transpose()?;
            output.insert(site,(params,result));
        }
        if output.len()!=sites.len(){return Err(fail("실제 linked callable 형식 소유자 집합이 다릅니다"));}Ok(output)
    }
    pub(super) fn resolve(&self,root:&TypeRef,at:Span)->Result<StaticTypeProof,String> {
        enum Work<'a>{Type(&'a TypeRef),Text,Tuple(usize),Container(u8,usize),Result,Function(usize),Tensor(&'a [u64]),Record(usize,Vec<String>)}
        let visible=self.visible(at);
        let declared=visible.iter().map(|symbol|symbol.name.clone()).collect::<HashSet<_>>();
        let mut work=vec![Work::Type(root)];let mut nodes=Vec::new();let mut built=Vec::new();let mut ancestry=BTreeSet::new();let mut encoded_budget=DOMAIN.len();
        while let Some(job)=work.pop() {
            let node=match job {
                Work::Text=>leaf(Scalar::Text),
                Work::Type(TypeRef::Infer)=>return Err(fail("미해소 형식으로 증명을 발급할 수 없습니다")),
                Work::Type(TypeRef::Named(name))=>{
                    let name=ddonirang_lang::canonicalizer::resolve_declared_type_spelling(name,true,&declared,false);
                    if let Some(symbol)=self.lookup(&visible,&name)? {match symbol.kind {
                        SymbolKind::Sum(id)=>Node::Sum{template:id.0,args:vec![]},
                        SymbolKind::Nominal(category,name_type_identity)=>{
                            let mut bytes=Vec::new();RecordFieldType::ResolvedNominalV1{category,name_type_identity}.encode_canon(&mut bytes);
                            Node::Leaf{bytes,scalar:None}
                        },
                        SymbolKind::StructuralProduct(index)=>{
                            if !ancestry.insert(index) {return Err(format!("E_R3_STRUCTURAL_TYPE_CYCLE: `{}`의 무한 구조 형식은 유한 증명으로 만들 수 없습니다 @{}..{}",symbol.name,symbol.site.start,symbol.site.end));}
                            let TopLevelItem::SeedDef(seed)=&self.program.items[index];
                            if seed.seed_kind!=ddonirang_lang::SeedKind::Product {return Err(fail("실제 구조 묶음씨 소유자가 아닙니다"));}
                            let mut names=BTreeSet::new();let mut fields=Vec::new();let mut lower=9;
                            for pin in &seed.params {
                                if !names.insert(pin.pin_name.as_str()){return Err(fail("구조 묶음씨 필드가 중복됩니다"));}
                                lower=add(lower,add(16,pin.pin_name.len())?)?;fields.push(pin);
                            }
                            fields.sort_by(|a,b|a.pin_name.as_bytes().cmp(b.pin_name.as_bytes()));
                            work.push(Work::Record(index,fields.iter().map(|pin|pin.pin_name.clone()).collect()));
                            work.extend(fields.into_iter().rev().map(|pin|Work::Type(&pin.type_ref)));continue;
                        },
                    }} else if ddonirang_lang::sum_owner::resolve_named_builtin_sum_owner(&name).is_some() {
                        Node::Sum{template:builtin_reference_failure_ids()?[0],args:Vec::new()}
                    } else {leaf(match name.as_str() {
                        "바른수"=>Scalar::Integer,"셈수"=>Scalar::Approximate,"나눔수"=>Scalar::Rational,"참거짓"=>Scalar::Bool,"글"=>Scalar::Text,
                        _=>return Err(format!("E_R3_TYPE_OWNER_UNRESOLVED: `{name}`의 실제 닫힌 형식 소유자가 없습니다 @{}..{}",at.start,at.end)),
                    })}
                },
                Work::Type(TypeRef::Tuple(items))=>{work.push(Work::Tuple(items.len()));work.extend(items.iter().rev().map(Work::Type));continue;},
                Work::Type(TypeRef::Applied{name,args})=>{
                    let name=ddonirang_lang::canonicalizer::resolve_declared_type_spelling(name,true,&declared,true);
                    if self.lookup(&visible,&name)?.is_some() {return Err(fail("사용자 적용 형식은 기존 공통 resolver의 지원 결속이 필요합니다"));}
                    if name=="맺음" {
                        // Common borrowed resolver owns arity and the default.
                        // No fake TypeRef or recursively cloned arguments.
                        let (value,error)=ddonirang_lang::sum_owner::resolve_result_type_arguments(&name,args)?;
                        work.push(Work::Result);work.push(error.map_or(Work::Text,Work::Type));work.push(Work::Type(value));continue;
                    }
                    // R3.3: the checked source owns the closed child. None is
                    // a value, never an issuer of its missing element type.
                    // The lexical lookup above keeps user homonyms separate.
                    let (constructor,arity)=match name.as_str(){"차림"=>(1,1),"모음"=>(2,1),"짝맞춤"=>(3,2),"있없"=>(4,1),_=>return Err(fail("해소된 기존 적용 형식 소유자가 없습니다"))};
                    if args.len()!=arity {return Err(fail("기존 container 형식의 인자 개수가 다릅니다"));}
                    work.push(Work::Container(constructor,arity));work.extend(args.iter().rev().map(Work::Type));continue;
                },
                Work::Type(TypeRef::Function{params,result})=>{
                    work.push(Work::Function(params.len()));work.push(Work::Type(result));work.extend(params.iter().rev().map(Work::Type));continue;
                },
                Work::Type(TypeRef::Tensor{element,shape})=>{
                    if shape.len()>64 {return Err(fail("현재 Tensor 형식 축 프로필 한도를 넘었습니다"));}
                    work.push(Work::Tensor(shape));work.push(Work::Type(element));continue;
                },
                Work::Tuple(n)=>Node::Tuple(take(&mut built,n)?),
                Work::Container(kind,n)=>Node::Container(kind,take(&mut built,n)?),
                Work::Result=>Node::Sum{template:builtin_result_ids()?[0],args:take(&mut built,2)?},
                Work::Function(n)=>{let result=built.pop().ok_or_else(||fail("함수 결과 형식이 없습니다"))?;Node::Function{params:take(&mut built,n)?,result}},
                Work::Tensor(shape)=>{
                    let element=built.pop().ok_or_else(||fail("Tensor 원소 형식이 없습니다"))?;
                    if !matches!(nodes.get(element),Some(Node::Leaf{scalar:Some(Scalar::Approximate),..})) {return Err(fail("현재 Basic Tensor의 실제 원소 형식은 셈수여야 합니다"));}
                    Node::Tensor{element,shape:shape.to_vec()}
                },
                Work::Record(index,keys)=>{
                    if !ancestry.remove(&index){return Err(fail("구조 형식 전개 경계가 다릅니다"));}
                    let children=take(&mut built,keys.len())?;Node::Record(keys.into_iter().zip(children).collect())
                },
            };
            encoded_budget=add(encoded_budget,own_size(&node)?)?;
            if nodes.len()>LIMIT/2 {return Err(fail("형식 증명 자원 한도를 넘었습니다"));}
            built.push(nodes.len());nodes.push(node);
        }
        if built.len()!=1 {return Err(fail("닫힌 형식 증명 뿌리가 하나가 아닙니다"));}
        Ok(StaticTypeProof{nodes,root:built[0]})
    }
}

fn take(built:&mut Vec<usize>,n:usize)->Result<Vec<usize>,String> {
    let at=built.len().checked_sub(n).ok_or_else(||fail("형식 증명의 자식 개수가 다릅니다"))?;Ok(built.split_off(at))
}
fn leaf(kind:Scalar)->Node {
    let name=match kind{Scalar::Integer=>"바른수",Scalar::Approximate=>"셈수",Scalar::Rational=>"나눔수",Scalar::Bool=>"참거짓",Scalar::Text=>"글"};
    let mut bytes=Vec::new();RecordFieldType::Named{name:name.into()}.encode_canon(&mut bytes);Node::Leaf{bytes,scalar:Some(kind)}
}

fn add(a:usize,b:usize)->Result<usize,String> {a.checked_add(b).filter(|n|*n<=LIMIT).ok_or_else(||fail("형식 증명 바이트 한도를 넘었습니다"))}
fn own_size(node:&Node)->Result<usize,String> {
    // Charge the forest while expanding, not only after building an
    // exponentially repeated Product DAG into a large arena.
    let repeated=|count:usize,size:usize|count.checked_mul(size).filter(|n|*n<=LIMIT).ok_or_else(||fail("형식 증명 바이트 한도를 넘었습니다"));
    match node {
        Node::Leaf{bytes,..}=>add(9,bytes.len()),
        Node::Sum{args,..}=>add(50,repeated(args.len(),8+DOMAIN.len())?),
        Node::Tuple(items)=>add(9,repeated(items.len(),8)?),
        Node::Container(_,items)=>add(10,repeated(items.len(),8)?),
        Node::Function{params,..}=>add(17,repeated(params.len(),8)?),
        Node::Tensor{shape,..}=>add(17,repeated(shape.len(),8)?),
        Node::Record(fields)=>fields.iter().try_fold(9,|total,(key,_)|add(total,add(16,key.len())?)),
    }
}
impl StaticTypeProof {
    pub(super) fn sum_owner_raw(&self)->Result<std::sync::Arc<[u8]>,String> {
        if !matches!(self.nodes.get(self.root),Some(Node::Sum{..})) {return Err(fail("고름씨 소유자의 닫힌 형식 증명이 아닙니다"));}
        let bytes=self.encode()?;Ok(std::sync::Arc::from(&bytes[DOMAIN.len()+9..]))
    }
    pub(super) fn encode(&self)->Result<Vec<u8>,String> {
        let mut lengths=Vec::<usize>::new();
        for (index,node) in self.nodes.iter().enumerate() {
            let child=|at:usize|->Result<usize,String>{if at>=index{return Err(fail("형식 증명 순환·외래 자식입니다"))}Ok(lengths[at])};
            let framed=|at|add(8,child(at)?);
            let size=match node {
                Node::Leaf{bytes,..}=>add(9,bytes.len())?,
                Node::Sum{args,..}=>{let mut n=50;for at in args {n=add(n,add(8,add(DOMAIN.len(),child(*at)?)?)?)?;}n},
                Node::Tuple(items)|Node::Container(_,items)=>{let mut n=if matches!(node,Node::Tuple(_)){9}else{10};for at in items {n=add(n,framed(*at)?)?;}n},
                Node::Function{params,result}=>{let mut n=9;for at in params {n=add(n,framed(*at)?)?;}add(n,framed(*result)?)?},
                Node::Tensor{element,shape}=>add(add(9,framed(*element)?)?,shape.len().checked_mul(8).ok_or_else(||fail("Tensor 모양 한도"))?)?,
                Node::Record(fields)=>{let mut n=9;for (key,at) in fields {n=add(n,add(add(8,key.len())?,framed(*at)?)?)?;}n},
            };
            lengths.push(size);
        }
        let total=add(DOMAIN.len(),*lengths.get(self.root).ok_or_else(||fail("형식 증명 뿌리 오류"))?)?;
        let mut output=Vec::new();output.try_reserve_exact(total).map_err(|_|fail("형식 증명 메모리 한도"))?;
        enum Emit {Node(usize),FramedNode(usize),Type(usize),FramedType(usize),Axes(Vec<u64>),RecordKey(usize,usize)}
        let mut emit=vec![Emit::Type(self.root)];
        while let Some(job)=emit.pop(){match job {
            Emit::Type(at)=>{output.extend_from_slice(DOMAIN);emit.push(Emit::Node(at));},
            Emit::FramedNode(at)=>{output.extend_from_slice(&(lengths[at] as u64).to_le_bytes());emit.push(Emit::Node(at));},
            Emit::FramedType(at)=>{output.extend_from_slice(&((DOMAIN.len()+lengths[at]) as u64).to_le_bytes());emit.push(Emit::Type(at));},
            Emit::Axes(shape)=>{output.extend_from_slice(&(shape.len() as u64).to_le_bytes());for axis in shape {output.extend_from_slice(&axis.to_le_bytes());}},
            Emit::RecordKey(at,index)=>{let Node::Record(fields)=&self.nodes[at] else {return Err(fail("구조 필드 증명의 전개 노드가 다릅니다"))};let key=fields[index].0.as_bytes();output.extend_from_slice(&(key.len() as u64).to_le_bytes());output.extend_from_slice(key);},
            Emit::Node(at)=>match &self.nodes[at] {
                Node::Leaf{bytes,..}=>{output.push(0);output.extend_from_slice(&(bytes.len() as u64).to_le_bytes());output.extend_from_slice(bytes);},
                Node::Sum{template,args}=>{
                    output.push(1);output.extend_from_slice(&((lengths[at]-9) as u64).to_le_bytes());output.push(0x21);output.extend_from_slice(template);
                    output.extend_from_slice(&(args.len() as u64).to_le_bytes());emit.extend(args.iter().rev().map(|at|Emit::FramedType(*at)));
                },
                Node::Tuple(items)=>{output.push(2);output.extend_from_slice(&(items.len() as u64).to_le_bytes());emit.extend(items.iter().rev().map(|at|Emit::FramedNode(*at)));},
                Node::Container(kind,args)=>{output.extend_from_slice(&[3,*kind]);output.extend_from_slice(&(args.len() as u64).to_le_bytes());emit.extend(args.iter().rev().map(|at|Emit::FramedNode(*at)));},
                Node::Function{params,result}=>{output.push(4);output.extend_from_slice(&(params.len() as u64).to_le_bytes());emit.push(Emit::FramedNode(*result));emit.extend(params.iter().rev().map(|at|Emit::FramedNode(*at)));},
                Node::Tensor{element,shape}=>{output.push(5);emit.push(Emit::Axes(shape.clone()));emit.push(Emit::FramedNode(*element));},
                Node::Record(fields)=>{output.push(6);output.extend_from_slice(&(fields.len() as u64).to_le_bytes());for (index,(_,child)) in fields.iter().enumerate().rev(){emit.push(Emit::FramedNode(*child));emit.push(Emit::RecordKey(at,index));}},
            },
        }}
        if output.len()!=total {return Err(fail("형식 증명 길이 검산 실패"));}Ok(output)
    }
    pub(super) fn verify_recorded(&self,raw:&[u8])->Result<(),String> {
        if raw.len()>LIMIT||self.encode()?.as_slice()!=raw {return Err(fail("기록된 형식 증명이 실제 해소 결과와 다릅니다"));}Ok(())
    }
}
