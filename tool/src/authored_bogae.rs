//! Explicit v2 authoring metadata and immutable persistent activation admission.
//! Geometry and recipes always come from the actual typed product execution.
// F5-GA[정의표]: 관계를 push하기 전에 같은 이름을 definitions에 넣는다. content의
// 관계/보기법/보기/도움 정의도 먼저 definitions에 넣고, 이후 두 표의 열쇠는 제거하지 않는다.
// supplied/bindings는 이 definitions 전체를 성공적으로 순회한 뒤에만 관계·내용에서 읽는다.
// 겹의 basis는 유일한 으뜸 이름인지 검사하고 recipe/view 이름도 definitions에 명시 등록한다.
// F5-GA[모듈]: original_owner의 순번은 같은 project.modules를 enumerate한 값이다.
// register의 copy는 그 project의 clone이고 중첩 bindings/compositions만 바꾸므로 순번이 유지된다.
// primaries/owners의 [0]은 각각 len==1 확인 뒤, module_index는 같은 Vec의 position 성공 뒤다.
// F5-GA[등록]: Candidate는 registration이 Some임을 확인한 composition의 clone만 담는다.
// Activation/Candidate 필드는 비공개이고 내부 변경 가능 셀·역직렬화 입구가 없어 apply 중 불변이다.
// F5-GA[JSON]: serde_json의 읽기 Index는 잘못된 모양에서 Null을 돌려주지만 변경 Index는 패닉 가능하다.
// ready 문자열은 projection 객체를, label/kind 문자열 일치는 candidate/관계 객체를 보장한다.
// artifact는 get_mut 관계 차림 검사 뒤에만 쓰고, recipe도 고정 필드 일치 뒤에만 변경한다.
// 공개 JSON의 artifact 모양 거부는 별도 F5 교정이다. 메모리·재귀·전체 제품 안전성 증명은 아니다.
use crate::authored_project::{
    self, AuthoredComposition, AuthoredModule, BindingId, CompositionId, Entry, ProjectArtifact,
};
use ddonirang_lang::{CanonProgram, Expr, ExprKind, Literal, Span, Stmt, TopLevelItem};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

const REGISTRATION: &str = "ddn.authored_bogae_registration.v2";
const PROJECTION: &str = "ddn.authored_bogae_projection.v2";
fn fail(message: &str) -> String {
    format!("E_AUTHORED_BOGAE: {message}")
}
fn hash(bytes: &[u8]) -> String {
    crate::runtime_surface::sha256_hex(bytes)
}
fn source_hash(source: &str) -> String {
    format!("sha256:{}", hash(source.as_bytes()))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BindingDefinition {
    pub id: BindingId,
    pub definition: String,
    pub source_path: String,
    pub kind: String,
    pub declaration: [usize; 2],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lock {
    pub schema: String,
    pub source_content: String,
    pub adapter: String,
    pub time: String,
    pub diagnostic: String,
}
impl Lock {
    fn for_source(source: &str) -> Self {
        Self {
            schema: PROJECTION.into(),
            source_content: source_hash(source),
            adapter: "typed_view.v1".into(),
            time: "common.v1".into(),
            diagnostic: "bogae.v1".into(),
        }
    }
    fn versions_valid(&self) -> bool {
        self.schema == PROJECTION
            && self.adapter == "typed_view.v1"
            && self.time == "common.v1"
            && self.diagnostic == "bogae.v1"
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Registration {
    pub schema: String,
    pub source_path: String,
    pub lock: Lock,
    pub clone_origin: Option<CompositionId>,
}
#[derive(Clone, Debug)]
struct Relation {
    definition: String,
    kind: String,
    location: [u32; 4],
    order: Option<i64>,
    basis: Option<String>,
    placement: Option<String>,
    view_definition: Option<String>,
    recipe: Option<String>,
    recipe_seal: Option<String>,
    recipe_parameter_type: Option<String>,
    recipe_declaration: Option<[usize; 2]>,
}
struct Parsed {
    relations: Vec<Relation>,
    definitions: BTreeMap<String, (String, Span)>,
    content: BTreeMap<String, DefinitionContent>,
    composition_span: Span,
}
#[derive(Clone)]
struct DefinitionContent {
    semantic: Value,
    references: BTreeSet<String>,
}
struct ContentScope<'a> {
    globals: &'a BTreeSet<String>,
    locals: BTreeMap<String, usize>,
    references: BTreeSet<String>,
}
impl ContentScope<'_> {
    fn bind(&mut self, name: &str) -> usize {
        let next = self.locals.len();
        *self.locals.entry(name.into()).or_insert(next)
    }
    fn reference(&mut self, name: &str) -> Value {
        if self.globals.contains(name) {
            self.references.insert(name.into());
            json!({"authored_ref":name})
        } else {
            json!({"external_ref":name})
        }
    }
    fn expr(&mut self, expr: &Expr) -> Result<Value, String> {
        Ok(match &expr.kind {
            ExprKind::Literal(value)=>json!({"literal":format!("{value:?}")}),
            ExprKind::Var(name)=>if let Some(local)=self.locals.get(name) {json!({"local":local})} else {self.reference(name)},
            ExprKind::Call{func,args}=>{
                let callable=self.reference(func);let mut values=Vec::new();
                for (index,arg) in args.iter().enumerate() {values.push(json!({"position":index,"pin":arg.resolved_pin,"value":self.expr(&arg.expr)?}));}
                json!({"call":callable,"args":values})
            },
            ExprKind::FieldAccess{target,field,..}=>json!({"field":field,"target":self.expr(target)?}),
            ExprKind::SelectionPlace{target,index}=>json!({"selection":self.expr(target)?,"index":self.expr(index)?}),
            ExprKind::MultiAxisSelection(selection)=>{
                // RD-388's one authored bracket group is not a chain of
                // SelectionPlace nodes. Preserve selector order and role in
                // this existing private authored-content projection.
                let target=self.expr(&selection.target)?;let mut selectors=Vec::new();
                for selector in &selection.selectors {selectors.push(match selector {
                    ddonirang_lang::TensorAxisSelector::Index(index)=>json!({"index":self.expr(index)?}),
                    ddonirang_lang::TensorAxisSelector::Range{start,end}=>json!({"range":[start,end]}),
                    ddonirang_lang::TensorAxisSelector::Full=>json!({"full":true}),
                });}
                json!({"multi_axis_selection":{"target":target,"selectors":selectors,"writable":selection.writable}})
            },
            ExprKind::CollectionConstructor{elements}=>json!({"collection":elements.iter().map(|e|self.expr(e)).collect::<Result<Vec<_>,_>>()?}),
            ExprKind::Pack{fields}=>json!({"pack":fields.iter().map(|(name,value)|Ok(json!({"field":name,"value":self.expr(value)?}))).collect::<Result<Vec<_>,String>>()?}),
            ExprKind::Infix{left,op,right}=>json!({"op":op,"left":self.expr(left)?,"right":self.expr(right)?}),
            ExprKind::Suffix{value,at}=>json!({"suffix":format!("{at:?}"),"value":self.expr(value)?}),
            ExprKind::Nuance{level,expr}=>json!({"nuance":level,"value":self.expr(expr)?}),
            ExprKind::Pipe{stages}=>json!({"pipe":stages.iter().map(|e|self.expr(e)).collect::<Result<Vec<_>,_>>()?}),
            ExprKind::FlowValue=>json!({"flow":true}),
            ExprKind::Thunk(body)=>json!({"thunk":self.body(body)?}),
            ExprKind::Eval{thunk,mode}=>json!({"eval":format!("{mode:?}"),"thunk":self.expr(thunk)?}),
            ExprKind::SeedLiteral{param,body}=>{
                let previous=self.locals.clone();let parameter=self.bind(param);let body=self.expr(body)?;
                self.locals=previous;json!({"lambda_parameter":parameter,"body":body})
            },
            _=>return Err("E_AUTHORED_BOGAE_CONTENT_UNSUPPORTED: 작성 내용의 이 AST 표현은 아직 정체로 결속하지 않습니다".into()),
        })
    }
    fn body(&mut self, body: &ddonirang_lang::Body) -> Result<Value, String> {
        let mut statements = Vec::new();
        for stmt in &body.stmts {
            statements.push(match stmt {
            Stmt::Definition{name,type_ref,value,..}=>{let value=self.expr(value)?;let local=self.bind(name);json!({"define":local,"type":format!("{type_ref:?}"),"value":value})},
            Stmt::Mutate{target,value,deferred,..}=>{
                let value=self.expr(value)?;
                let target=if let ExprKind::Var(name)=&target.kind {json!({"local":self.bind(name)})} else {self.expr(target)?};
                json!({"mutate":target,"value":value,"deferred":deferred})
            },
            Stmt::Return{value,..}=>json!({"return":self.expr(value)?}),
            Stmt::Expr{expr,..}=>json!({"expression":self.expr(expr)?}),
            Stmt::Show{expr,..}=>json!({"show":self.expr(expr)?}),
            Stmt::Inspect{expr,..}=>json!({"inspect":self.expr(expr)?}),
            Stmt::If{condition,then_body,else_body,..}=>{
                let condition=self.expr(condition)?;let previous=self.locals.clone();let then=self.body(then_body)?;
                let then_locals=self.locals.clone();self.locals=previous.clone();
                let otherwise=else_body.as_ref().map(|b|self.body(b)).transpose()?;
                self.locals.retain(|name,id|previous.contains_key(name)||then_locals.get(name)==Some(id));
                json!({"if":condition,"then":then,"else":otherwise})
            },
            Stmt::While{condition,body,..}=>{
                let condition=self.expr(condition)?;let previous=self.locals.clone();let body=self.body(body)?;
                self.locals=previous;json!({"while":condition,"body":body})
            },
            Stmt::Repeat{body,..}=>json!({"repeat":self.body(body)?}),
            Stmt::ForEach{item,item_type,iterable,body,..}=>{
                let iterable=self.expr(iterable)?;let previous=self.locals.clone();let local=self.bind(item);
                let body=self.body(body)?;self.locals=previous;
                json!({"foreach":local,"type":format!("{item_type:?}"),"iterable":iterable,"body":body})
            },
            Stmt::Break{..}=>json!({"break":true}),Stmt::ContinueLoop{..}=>json!({"continue":true}),
            Stmt::DeclBlock{items,..}=>{
                let mut values=Vec::new();for item in items {let value=item.value.as_ref().map(|e|self.expr(e)).transpose()?;
                    values.push(json!({"declare":self.bind(&item.name),"kind":format!("{:?}",item.kind),"type":format!("{:?}",item.type_ref),"value":value}));}
                json!({"declarations":values})
            },
            _=>return Err("E_AUTHORED_BOGAE_CONTENT_UNSUPPORTED: 작성 내용의 이 AST 문장은 아직 정체로 결속하지 않습니다".into()),
        });
        }
        Ok(json!(statements))
    }
}
fn expression_content(
    expr: &Expr,
    globals: &BTreeSet<String>,
) -> Result<DefinitionContent, String> {
    let mut scope = ContentScope {
        globals,
        locals: BTreeMap::new(),
        references: BTreeSet::new(),
    };
    let semantic = scope.expr(expr)?;
    Ok(DefinitionContent {
        semantic,
        references: scope.references,
    })
}
fn callable_content(
    seed: &ddonirang_lang::SeedDef,
    globals: &BTreeSet<String>,
) -> Result<DefinitionContent, String> {
    let mut scope = ContentScope {
        globals,
        locals: BTreeMap::new(),
        references: BTreeSet::new(),
    };
    let mut params = Vec::new();
    for pin in &seed.params {
        scope.bind(&pin.pin_name);
        params.push(
            json!({"pin":pin.pin_name,"type":format!("{:?}",pin.type_ref),"optional":pin.optional,
        "default":pin.default_value.as_ref().map(|e|scope.expr(e)).transpose()?}),
        );
    }
    scope.bind(&seed.canonical_name);
    let semantic = json!({"callable_kind":format!("{:?}",seed.seed_kind),"params":params,"result":format!("{:?}",seed.result_type),
        "modifiers":format!("{:?}",seed.modifiers),"body":seed.body.as_ref().map(|b|scope.body(b)).transpose()?});
    Ok(DefinitionContent {
        semantic,
        references: scope.references,
    })
}
fn bind_semantic_refs(
    value: &mut Value,
    bindings: &BTreeMap<String, BindingDefinition>,
) -> Result<(), String> {
    match value {
        Value::Object(object) => {
            if let Some(name) = object.get("authored_ref").and_then(Value::as_str) {
                let id = bindings
                    .get(name)
                    .ok_or_else(|| fail("AST 참조에 명시 결속이 없습니다"))?
                    .id
                    .as_str()
                    .to_string();
                object.remove("authored_ref");
                object.insert("binding_ref".into(), json!(id));
            } else {
                for value in object.values_mut() {
                    bind_semantic_refs(value, bindings)?;
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                bind_semantic_refs(value, bindings)?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn location(source: &str, span: Span) -> [u32; 4] {
    fn point(source: &str, offset: usize) -> (u32, u32) {
        let mut line = 1;
        let mut col = 0;
        for (byte, ch) in source.char_indices() {
            if byte >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += ch.len_utf16() as u32;
            }
        }
        (line, col)
    }
    let (a, b) = point(source, span.start);
    let (c, d) = point(source, span.end);
    [a, b, c, d]
}
fn pin<'a>(
    args: &'a [ddonirang_lang::ArgBinding],
    name: &str,
    index: usize,
) -> Result<&'a Expr, String> {
    args.iter()
        .find(|a| a.resolved_pin.as_deref() == Some(name))
        .or_else(|| args.get(index))
        .map(|a| &a.expr)
        .ok_or_else(|| fail("생성자의 필수 받는자리가 없습니다"))
}
fn named(expr: &Expr) -> Result<String, String> {
    if let ExprKind::Var(name) = &expr.kind {
        Ok(name.clone())
    } else {
        Err(fail("명시 이름 정의 참조만 등록할 수 있습니다"))
    }
}
fn order(expr: &Expr) -> Result<i64, String> {
    match &expr.kind {
        ExprKind::Literal(Literal::Int(n)) if *n >= 0 => Ok(*n),
        ExprKind::Literal(Literal::Fixed64(n))
            if n.raw_i64() >= 0 && n.raw_i64() % (1i64 << 32) == 0 =>
        {
            Ok(n.raw_i64() >> 32)
        }
        _ => Err(fail("겹 순서는 명시 바른수 literal이어야 합니다")),
    }
}
fn parse(source: &str, path: &str, composition: &str) -> Result<Parsed, String> {
    let mut program = ddonirang_lang::parse_v25(source, path)
        .map_err(|e| fail(&format!("원문 파싱 실패: {}", e.message)))?;
    // Seal the same admitted definition that the product runner executes.
    // Source bytes/spans remain original; M-A aliases are not a second ABI.
    ddonirang_lang::canonicalize(&mut program)
        .map_err(|e|fail(&format!("원문 정본화 실패: {}",e.message)))?;
    parse_program(&program, source, composition)
}
fn parse_program(
    program: &CanonProgram,
    source: &str,
    composition: &str,
) -> Result<Parsed, String> {
    let mut roots = BTreeMap::new();
    for stmt in &program.root_stmts {
        if let Stmt::Definition { name, value, .. } = stmt {
            if roots.insert(name.clone(), value).is_some() {
                return Err(fail("루트 작성 정의가 중복됩니다"));
            }
        }
    }
    let expr = roots
        .get(composition)
        .ok_or_else(|| fail("등록할 루트 짜임 정의가 없습니다"))?;
    let ExprKind::Call { func, args } = &expr.kind else {
        return Err(fail("짜임 정의는 직접 typed 생성자여야 합니다"));
    };
    if func != "표준보개.짜임" || args.len() != 1 {
        return Err(fail("짜임 정의는 표준보개.짜임 직접 호출이어야 합니다"));
    }
    let list = pin(args, "관계들", 0)?;
    let elements = match &list.kind {
        ExprKind::CollectionConstructor { elements } => elements.iter().collect::<Vec<_>>(),
        ExprKind::Call { func, args } if func == "차림" => args.iter().map(|a| &a.expr).collect(),
        _ => return Err(fail("관계들은 직접 평평한 authored 차림이어야 합니다")),
    };
    if elements.is_empty() {
        return Err(fail("짜임 관계 차림이 비었습니다"));
    }
    let mut functions = HashMap::new();
    for TopLevelItem::SeedDef(seed) in &program.items {
        if functions
            .insert(seed.canonical_name.clone(), seed.clone())
            .is_some()
        {
            return Err(fail("중복 또는 과적재된 작성 셈씨는 등록하지 않습니다"));
        }
    }
    let globals = roots
        .keys()
        .chain(functions.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut definitions = BTreeMap::new();
    let mut relations = Vec::new();
    let mut content = BTreeMap::new();
    for element in elements {
        let definition = named(element)?;
        let expr = roots
            .get(&definition)
            .ok_or_else(|| fail("관계는 같은 원문의 명시 루트 정의여야 합니다"))?;
        let ExprKind::Call { func, args } = &expr.kind else {
            return Err(fail("간접 관계 정의는 등록하지 않습니다"));
        };
        let (kind, count) = match func.as_str() {
            "표준보개.으뜸" => ("primary", 1),
            "표준보개.겹" => ("overlay", 3),
            "표준보개.곁" => ("side", 2),
            "표준보개.돋" => ("magnifier", 2),
            _ => return Err(fail("실제 관계 생성자 정의가 아닙니다")),
        };
        if args.len() != count {
            return Err(fail("관계 생성자의 받는자리 수가 다릅니다"));
        }
        let mut relation = Relation {
            definition: definition.clone(),
            kind: kind.into(),
            location: location(source, expr.span),
            order: None,
            basis: None,
            placement: None,
            view_definition: None,
            recipe: None,
            recipe_seal: None,
            recipe_parameter_type: None,
            recipe_declaration: None,
        };
        definitions.insert(definition, (kind.into(), expr.span));
        if kind == "overlay" {
            relation.order = Some(order(pin(args, "순서", 2)?)?);
            relation.basis = Some(named(pin(args, "기준", 1)?)?);
        }
        if kind == "side" {
            relation.placement = Some(match &pin(args, "자리", 1)?.kind {
                ExprKind::Literal(Literal::Atom(s) | Literal::String(s)) => {
                    s.trim_start_matches('#').to_string()
                }
                _ => return Err(fail("곁 자리는 명시 placement literal이어야 합니다")),
            });
            if !matches!(
                relation.placement.as_deref(),
                Some("왼쪽" | "오른쪽" | "위" | "아래" | "자동")
            ) {
                return Err(fail("알 수 없는 곁 자리입니다"));
            }
        }
        if kind == "magnifier" {
            let recipe = named(pin(args, "보기법", 1)?)?;
            let seed = functions
                .get(&recipe)
                .ok_or_else(|| fail("보기법의 작성 정의가 없습니다"))?;
            let sealed =
                crate::bogae_recipe::seal(seed, &source_hash(source), &functions, &HashSet::new())?;
            relation.recipe_seal = Some(sealed.seal);
            relation.recipe_parameter_type = Some(sealed.parameter_type);
            relation.recipe_declaration = Some(sealed.definition_span);
            definitions.insert(recipe.clone(), ("recipe".into(), seed.span));
            relation.recipe = Some(recipe);
        } else {
            let view = pin(args, "보기", 0)?;
            let ExprKind::Call { func, .. } = &view.kind else {
                return Err(fail(
                    "보기는 직접 typed 생성자 또는 명시 셈씨 호출이어야 합니다",
                ));
            };
            if let Some(seed) = functions.get(func) {
                if seed.result_type != Some(ddonirang_lang::TypeRef::Named("보기값".into())) {
                    return Err(fail("보기 정의의 반환 씨가 보기값이 아닙니다"));
                }
                definitions.insert(func.clone(), ("view".into(), seed.span));
                relation.view_definition = Some(func.clone());
            } else if !matches!(func.as_str(), "표준보기.장면" | "표준보기.결과표") {
                return Err(fail("해소할 수 없는 typed 보기 생성자입니다"));
            }
            content.insert(
                relation.definition.clone(),
                expression_content(view, &globals)?,
            );
        }
        relations.push(relation);
    }
    let primaries = relations
        .iter()
        .filter(|r| r.kind == "primary")
        .collect::<Vec<_>>();
    if primaries.len() != 1 {
        return Err(fail("짜임에는 으뜸이 정확히 하나 있어야 합니다"));
    }
    let primary = primaries[0].definition.clone();
    let mut orders = BTreeSet::new();
    for r in &relations {
        if r.kind == "overlay" && (r.basis.as_deref() != Some(&primary) || !orders.insert(r.order))
        {
            return Err(fail("겹 기준 또는 겹 순서가 올바르지 않습니다"));
        }
    }
    // Include every real callable referenced by these authored view/recipe
    // declarations, including transitive helpers. This is source provenance,
    // not a computed drawing captured from execution.
    let mut pending = definitions
        .keys()
        .filter(|name| functions.contains_key(*name))
        .cloned()
        .collect::<Vec<_>>();
    pending.extend(content.values().flat_map(|c| c.references.iter().cloned()));
    let mut visited = BTreeSet::new();
    while let Some(name) = pending.pop() {
        if !visited.insert(name.clone()) {
            continue;
        }
        let authored = if let Some(seed) = functions.get(&name) {
            definitions
                .entry(name.clone())
                .or_insert(("helper".into(), seed.span));
            callable_content(seed, &globals)?
        } else if let Some(expr) = roots.get(&name) {
            definitions
                .entry(name.clone())
                .or_insert(("value".into(), expr.span));
            expression_content(expr, &globals)?
        } else {
            continue;
        };
        pending.extend(authored.references.iter().cloned());
        content.insert(name, authored);
    }
    Ok(Parsed {
        relations,
        definitions,
        content,
        composition_span: expr.span,
    })
}

fn original_owner(
    project: &ProjectArtifact,
    prepared: Option<&crate::local_project::AuthoredPreparation>,
    span: Span,
    fallback_path: &str,
) -> Result<(usize, String, Span), String> {
    let (path, original) = if let Some(prepared) = prepared {
        prepared.original_span(span)?
    } else {
        (fallback_path.to_string(), span)
    };
    let owners = project
        .authored_identity
        .modules
        .iter()
        .enumerate()
        .filter(|(_, module)| module.sources.contains(&path))
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    if owners.len() != 1 {
        return Err(
            "E_AUTHORED_BOGAE_SOURCE_OWNER: 원래 정의의 모듈 소유가 유일하지 않습니다".into(),
        );
    }
    Ok((owners[0], path, original))
}

fn validate_linked_references(
    project: &ProjectArtifact,
    prepared: Option<&crate::local_project::AuthoredPreparation>,
    parsed: &Parsed,
    path: &str,
) -> Result<(), String> {
    let Some(prepared) = prepared else {
        return Ok(());
    };
    for (name, content) in &parsed.content {
        let caller = original_owner(project, Some(prepared), parsed.definitions[name].1, path)?.0;
        for dependency in &content.references {
            if let Some((_, span)) = parsed.definitions.get(dependency) {
                let owner = original_owner(project, Some(prepared), *span, path)?.0;
                if caller != owner
                    && !prepared.permits_import(
                        &project.authored_identity.modules[caller].head,
                        &project.authored_identity.modules[owner].head,
                        dependency,
                    )
                {
                    return Err(format!("E_AUTHORED_BOGAE_IMPORT_UNRESOLVED: 다른 모듈 정의의 명시 쓰임·드러냄 해소가 없습니다: {dependency}"));
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn verify_explicit_rename(
    project: &ProjectArtifact,
    module_id: &str,
    path: &str,
    old_name: &str,
    new_name: &str,
) -> Result<(), String> {
    let prepared = project.authored_preparation()?;
    let source = prepared
        .as_ref()
        .map(|p| p.source.as_str())
        .unwrap_or(file(project, path)?);
    let program = ddonirang_lang::parse_v25(source, path).map_err(|error| fail(&error.message))?;
    let mut definitions = Vec::new();
    for TopLevelItem::SeedDef(seed) in &program.items {
        definitions.push((seed.canonical_name.as_str(), seed.span));
    }
    for statement in &program.root_stmts {
        if let Stmt::Definition { name, value, .. } = statement {
            definitions.push((name.as_str(), value.span));
        }
    }
    let mut actual_new = 0;
    for (name, span) in definitions {
        if name != new_name && name != old_name {
            continue;
        }
        let (owner, actual_path, _) = original_owner(project, prepared.as_ref(), span, path)?;
        if project.authored_identity.modules[owner].id.as_str() == module_id {
            if name == old_name {
                return Err(
                    "E_AUTHORED_PROJECT_RENAME_MAPPING: 원래 소유 모듈에 옛 정의가 남아 있습니다"
                        .into(),
                );
            }
            if actual_path != path {
                return Err("E_AUTHORED_PROJECT_RENAME_MAPPING: 이름 변경으로 정의 소유 원문을 이동할 수 없습니다".into());
            }
            actual_new += 1;
        }
    }
    if actual_new != 1 {
        return Err("E_AUTHORED_PROJECT_RENAME_MAPPING: 완성 원문에 새 이름 정의가 정확히 하나 있어야 합니다".into());
    }
    Ok(())
}
fn file<'a>(project: &'a ProjectArtifact, path: &str) -> Result<&'a str, String> {
    project
        .source_snapshot
        .files
        .iter()
        .find(|f| f.path == path)
        .map(|f| f.text.as_str())
        .ok_or_else(|| fail("소유 원문이 없습니다"))
}
pub(crate) fn validate_module_storage(
    project: &ProjectArtifact,
    module: &AuthoredModule,
) -> Result<(), String> {
    if project.schema == authored_project::SCHEMA {
        if !module.bindings.is_empty()
            || module.compositions.iter().any(|c| c.registration.is_some())
        {
            return Err(fail("v1에 successor 필드를 넣을 수 없습니다"));
        }
        return Ok(());
    }
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for binding in &module.bindings {
        if binding.id.as_str().len() != 64
            || !binding
                .id
                .as_str()
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || !ids.insert(&binding.id)
            || !names.insert((&binding.source_path, &binding.definition))
            || !module.sources.contains(&binding.source_path)
            || binding.definition.is_empty()
            || !matches!(
                binding.kind.as_str(),
                "primary"
                    | "overlay"
                    | "side"
                    | "magnifier"
                    | "view"
                    | "recipe"
                    | "helper"
                    | "value"
            )
        {
            return Err(fail("작성 정의 정체 또는 소유가 잘못되었습니다"));
        }
    }
    for composition in &module.compositions {
        let registration = composition
            .registration
            .as_ref()
            .ok_or_else(|| fail("v2 짜임에는 명시 등록 자료가 필요합니다"))?;
        if registration.schema != REGISTRATION
            || !registration.lock.versions_valid()
            || !module.sources.contains(&registration.source_path)
            || registration.lock.source_content
                != source_hash(file(project, &registration.source_path)?)
            || composition.binding_ids.iter().any(|id| !ids.contains(id))
        {
            return Err(fail("등록 버전·원문 lock·관계 결속 소유가 다릅니다"));
        }
    }
    Ok(())
}
pub(crate) fn register(
    project: &ProjectArtifact,
    module_name: &str,
    path: &str,
    definition: &str,
    make_entry: bool,
) -> Result<ProjectArtifact, String> {
    let prepared = project.authored_preparation()?;
    let mut copy = project.clone();
    let source = prepared
        .as_ref()
        .map(|p| p.source.as_str())
        .unwrap_or(file(project, path)?);
    let parsed = parse(source, path, definition)?;
    validate_linked_references(project, prepared.as_ref(), &parsed, path)?;
    let module_index = copy
        .authored_identity
        .modules
        .iter()
        .position(|m| m.name == module_name)
        .ok_or_else(|| fail("명시 모듈을 해소할 수 없습니다"))?;
    if !copy.authored_identity.modules[module_index]
        .sources
        .contains(&path.to_string())
    {
        return Err(fail("모듈이 원문을 소유하지 않습니다"));
    }
    let (composition_owner, composition_path, _) =
        original_owner(project, prepared.as_ref(), parsed.composition_span, path)?;
    if composition_owner != module_index || composition_path != path {
        return Err(
            "E_AUTHORED_BOGAE_SOURCE_OWNER: 짜임 정의의 실제 원문 소유가 등록 요청과 다릅니다"
                .into(),
        );
    }
    if copy
        .authored_identity
        .modules
        .iter()
        .any(|m| m.compositions.iter().any(|c| c.registration.is_none()))
    {
        return Err(fail("기존 미등록 v1 짜임을 조용히 승격할 수 없습니다"));
    }
    let mut supplied = BTreeMap::new();
    for (name, (kind, span)) in &parsed.definitions {
        let (owner, original_path, original_span) =
            original_owner(project, prepared.as_ref(), *span, path)?;
        if matches!(kind.as_str(), "primary" | "overlay" | "side" | "magnifier")
            && owner != module_index
        {
            return Err("E_AUTHORED_BOGAE_RELATION_OWNER: 다른 모듈의 관계를 직접 등록하는 경로는 지원하지 않습니다".into());
        }
        let module = &mut copy.authored_identity.modules[owner];
        if let Some(binding) = module
            .bindings
            .iter_mut()
            .find(|b| b.source_path == original_path && b.definition == *name)
        {
            if binding.kind != *kind {
                return Err(fail("기존 결속의 역할이 바뀌었습니다"));
            }
            binding.declaration = [original_span.start, original_span.end];
            supplied.insert(name.clone(), binding.id.clone());
        } else {
            let id = BindingId::fresh()?;
            module.bindings.push(BindingDefinition {
                id: id.clone(),
                definition: name.clone(),
                source_path: original_path,
                kind: kind.clone(),
                declaration: [original_span.start, original_span.end],
            });
            supplied.insert(name.clone(), id);
        }
    }
    let binding_ids = parsed
        .relations
        .iter()
        .map(|r| supplied[&r.definition].clone())
        .collect();
    let registration = Registration {
        schema: REGISTRATION.into(),
        source_path: path.into(),
        lock: Lock::for_source(file(project, path)?),
        clone_origin: None,
    };
    let module = &mut copy.authored_identity.modules[module_index];
    let id = if let Some(c) = module
        .compositions
        .iter_mut()
        .find(|c| c.definition == definition)
    {
        if c.registration
            .as_ref()
            .is_some_and(|r| r.source_path != path)
        {
            return Err(fail("짜임의 소유 원문이 다릅니다"));
        }
        c.binding_ids = binding_ids;
        c.registration = Some(registration);
        c.id.clone()
    } else {
        let id = CompositionId::fresh()?;
        module.compositions.push(AuthoredComposition {
            id: id.clone(),
            definition: definition.into(),
            binding_ids,
            registration: Some(registration),
        });
        id
    };
    if make_entry {
        copy.authored_identity.entry = Some(Entry {
            module: module.id.clone(),
            composition: id,
        });
    }
    copy.schema = authored_project::SCHEMA_V2.into();
    copy.authored_identity.schema = "ddn.authored_identity.v2".into();
    copy.content_sha256.clear();
    // Reuse the full common codec admission, not a parallel writer-side checker.
    authored_project::open(copy.save()?.as_bytes())
}
pub(crate) fn refresh_source_locks(project: &mut ProjectArtifact) -> Result<(), String> {
    if project.schema != authored_project::SCHEMA_V2 {
        return Ok(());
    }
    let prepared = project.authored_preparation()?;
    let mut remapped = BTreeMap::new();
    for module in &project.authored_identity.modules {
        for composition in &module.compositions {
            if let Some(registration) = &composition.registration {
                let source = prepared
                    .as_ref()
                    .map(|p| p.source.as_str())
                    .unwrap_or(file(project, &registration.source_path)?);
                if let Ok(parsed) =
                    parse(source, &registration.source_path, &composition.definition)
                {
                    for (name, (_, span)) in parsed.definitions {
                        let (_, path, original) = original_owner(
                            project,
                            prepared.as_ref(),
                            span,
                            &registration.source_path,
                        )?;
                        remapped.insert((path, name), [original.start, original.end]);
                    }
                }
            }
        }
    }
    for module in &mut project.authored_identity.modules {
        for binding in &mut module.bindings {
            if let Some(span) =
                remapped.get(&(binding.source_path.clone(), binding.definition.clone()))
            {
                binding.declaration = *span;
            }
        }
        for c in &mut module.compositions {
            if let Some(registration) = &mut c.registration {
                let source = project
                    .source_snapshot
                    .files
                    .iter()
                    .find(|f| f.path == registration.source_path)
                    .ok_or_else(|| fail("등록 원문이 사라졌습니다"))?;
                registration.lock.source_content = source_hash(&source.text);
                // Saving an edit preserves identities. Invalid/uncertain source is
                // retained; the activation admission below rejects it atomically.
            }
        }
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct Candidate {
    module_id: String,
    composition: AuthoredComposition,
    relations: Vec<Relation>,
    bindings: BTreeMap<String, BindingDefinition>,
    definition_content: Value,
}
#[derive(Clone, Debug)]
pub struct Activation {
    project_id: String,
    source_content: String,
    entry: Option<String>,
    candidates: Vec<Candidate>,
    clone_origin: Option<String>,
}
impl Activation {
    pub fn from_artifact(
        artifact_json: &str,
        source: &str,
        _source_path: &str,
    ) -> Result<Self, String> {
        let project = authored_project::open(artifact_json.as_bytes())?;
        if project.schema != authored_project::SCHEMA_V2 {
            return Err(fail("영구 보개에는 명시 v2 등록이 필요합니다"));
        }
        let prepared = project.authored_preparation()?;
        if let Some(prepared) = &prepared {
            if prepared.source != source {
                return Err("E_AUTHORED_BOGAE_LINK_SOURCE_MISMATCH: 저장 원문을 실제 연결한 bytes와 실행 원문이 다릅니다".into());
            }
        }
        let mut candidates = Vec::new();
        for (module_index, module) in project.authored_identity.modules.iter().enumerate() {
            for c in &module.compositions {
                let r = c
                    .registration
                    .as_ref()
                    .ok_or_else(|| fail("짜임 등록이 없습니다"))?;
                if prepared.is_none() && file(&project, &r.source_path)? != source {
                    return Err(fail("실행 원문과 등록 원문 bytes가 다릅니다"));
                }
                let parsed = parse(source, &r.source_path, &c.definition)?;
                validate_linked_references(&project, prepared.as_ref(), &parsed, &r.source_path)?;
                let (owner, path, _) = original_owner(
                    &project,
                    prepared.as_ref(),
                    parsed.composition_span,
                    &r.source_path,
                )?;
                if owner != module_index || path != r.source_path {
                    return Err("E_AUTHORED_BOGAE_SOURCE_OWNER: 연결 짜임의 원래 소유가 저장 자료와 다릅니다".into());
                }
                let mut bindings = BTreeMap::new();
                for (name, (kind, span)) in &parsed.definitions {
                    let (owner, path, original) =
                        original_owner(&project, prepared.as_ref(), *span, &r.source_path)?;
                    if matches!(kind.as_str(), "primary" | "overlay" | "side" | "magnifier")
                        && owner != module_index
                    {
                        return Err("E_AUTHORED_BOGAE_RELATION_OWNER: 연결 관계의 원래 모듈 소유가 다릅니다".into());
                    }
                    let binding = project.authored_identity.modules[owner]
                        .bindings
                        .iter()
                        .find(|b| b.source_path == path && b.definition == *name)
                        .ok_or_else(|| fail("원문 정의에 대응하는 명시 작성 결속이 없습니다"))?;
                    if binding.kind != *kind
                        || binding.declaration != [original.start, original.end]
                    {
                        return Err(fail("원문 정의 locator 또는 역할이 등록과 다릅니다"));
                    }
                    bindings.insert(name.clone(), binding.clone());
                }
                let ids = parsed
                    .relations
                    .iter()
                    .map(|r| bindings[&r.definition].id.clone())
                    .collect::<Vec<_>>();
                if ids != c.binding_ids {
                    return Err(fail("원문 정본 관계 차림과 저장 결속 순서가 다릅니다"));
                }
                let mut content = BTreeMap::new();
                for (name, authored) in &parsed.content {
                    let id = bindings[name].id.as_str();
                    let mut semantic = authored.semantic.clone();
                    bind_semantic_refs(&mut semantic, &bindings)?;
                    content.insert(id.to_string(), semantic);
                }
                candidates.push(Candidate {
                    module_id: module.id.as_str().into(),
                    composition: c.clone(),
                    relations: parsed.relations,
                    bindings,
                    definition_content: json!(content),
                });
            }
        }
        if candidates.is_empty() {
            return Err(fail("등록된 짜임이 없습니다"));
        }
        let entry = project
            .authored_identity
            .entry
            .as_ref()
            .map(|entry| {
                project
                    .authored_identity
                    .modules
                    .iter()
                    .find(|module| module.id == entry.module)
                    .ok_or_else(|| fail("명시 시작 짜임 모듈이 없습니다"))?
                    .compositions
                    .iter()
                    .find(|c| c.id == entry.composition)
                    .map(|c| c.definition.clone())
                    .ok_or_else(|| fail("명시 시작 짜임을 해소할 수 없습니다"))
            })
            .transpose()?;
        Ok(Self {
            project_id: project
                .authored_identity
                .logical_dependency_id
                .as_str()
                .into(),
            source_content: source_hash(source),
            entry,
            candidates,
            clone_origin: project
                .authored_identity
                .clone_origin
                .map(|id| id.as_str().to_string()),
        })
    }
    pub fn entry_definition(&self) -> Option<&str> {
        self.entry.as_deref()
    }
    pub fn apply(&self, projection: Option<Value>) -> Result<Option<Value>, String> {
        let mut projection =
            projection.ok_or_else(|| fail("등록 짜임의 실제 typed projection이 없습니다"))?;
        crate::bogae_projection::admit_activation(Some(&projection))?;
        let candidates = projection["candidates"]
            .as_array_mut()
            .ok_or_else(|| fail("typed 후보 차림이 없습니다"))?;
        if candidates.len() != 1 {
            return Err(fail("실제 후보가 명시 해소되지 않았습니다"));
        }
        let candidate = &mut candidates[0];
        let label = candidate["label"]
            .as_str()
            .ok_or_else(|| fail("실제 정의 참조가 없습니다"))?;
        if self.entry.as_ref().is_some_and(|entry| entry != label) {
            return Err(fail("명시 시작 정의가 다릅니다"));
        }
        let registered = self
            .candidates
            .iter()
            .find(|c| c.composition.definition == label)
            .ok_or_else(|| fail("실제 후보가 등록된 짜임이 아닙니다"))?;
        // 공개 JSON 입구에서는 artifact가 scalar일 수도 있다.
        // 변경 인덱스의 패닉 대신 기존 typed 관계 오류로 거부한다.
        let relations = candidate.get_mut("artifact")
            .and_then(|artifact| artifact.get_mut("relations"))
            .and_then(Value::as_array_mut)
            .ok_or_else(|| fail("실제 typed 관계들이 없습니다"))?;
        if relations.len() != registered.relations.len() {
            return Err(fail("실제 관계 수가 원문 차림과 다릅니다"));
        }
        let mut structural = Vec::new();
        for (occurrence, (projected, authored)) in
            relations.iter_mut().zip(&registered.relations).enumerate()
        {
            if projected["kind"] != authored.kind
                || projected["identity"]["authored_binding_location"] != json!(authored.location)
                || projected["identity"]["canonical_occurrence"] != json!(occurrence)
                || projected["order"] != json!(authored.order)
                || projected["placement"] != json!(authored.placement)
                || projected["time_policy"] != json!({"kind":"common"})
            {
                return Err(fail(
                    "실제 관계 종류·위치·순서·자리·시간이 작성 정의와 다릅니다",
                ));
            }
            let binding = &registered.bindings[&authored.definition];
            let basis = authored
                .basis
                .as_ref()
                .map(|name| registered.bindings[name].id.as_str());
            if let Some(name) = &authored.basis {
                let expected = registered
                    .relations
                    .iter()
                    .find(|r| r.definition == *name && r.kind == "primary")
                    .ok_or_else(|| fail("으뜸 기준 결속이 없습니다"))?;
                if projected["basis_location"] != json!(expected.location) {
                    return Err(fail("실제 겹 기준이 작성 으뜸과 다릅니다"));
                }
            } else if !projected["basis_location"].is_null() {
                return Err(fail("기준 없는 관계에 실제 겹 기준이 있습니다"));
            }
            let recipe = authored
                .recipe
                .as_ref()
                .map(|name| registered.bindings[name].id.as_str());
            if let Some(name) = &authored.recipe {
                if projected["recipe"]["kind"] != "sealed_named_callable"
                    || projected["recipe"]["source"] != self.source_content
                    || projected["recipe"]["definition"] != *name
                    || projected["recipe"]["declaration"] != json!(authored.recipe_declaration)
                    || projected["recipe"]["seal"] != json!(authored.recipe_seal)
                    || projected["recipe"]["parameter_type"]
                        != json!(authored.recipe_parameter_type)
                    || projected["recipe"]["result_type"] != "보기값"
                {
                    return Err(fail("실제 봉인 보기법이 등록 정의와 다릅니다"));
                }
                projected["recipe"]["definition_id"] = json!(recipe);
            }
            let identity = json!({"module":registered.module_id,"composition":registered.composition.id.as_str(),
                "authored_binding":binding.id.as_str(),"canonical_occurrence":occurrence});
            projected["relation_id"] = json!(hash(
                format!("ddn.bogae_relation.v2\0{identity}").as_bytes()
            ));
            projected["identity"] = identity;
            projected["basis_binding_id"] = json!(basis);
            let view = authored
                .view_definition
                .as_ref()
                .map(|name| registered.bindings[name].id.as_str())
                .unwrap_or(binding.id.as_str());
            structural.push(json!({"binding":binding.id.as_str(),"occurrence":occurrence,"kind":authored.kind,
                "view_definition":view,"order":authored.order,"basis":basis,"placement":authored.placement,
                "recipe_definition":recipe,"time":{"kind":"common"}}));
        }
        let identity = json!({"scope":"persistent_authored","project":self.project_id,"module":registered.module_id,
            "composition":registered.composition.id.as_str()});
        let structural = json!({"schema":PROJECTION,"identity":identity,"relations":structural,
            "definition_content":registered.definition_content});
        candidate["authored_digest"] = json!(hash(
            format!("ddn.authored_bogae_structural.v2\0{structural}").as_bytes()
        ));
        candidate["artifact"]["schema"] = json!(PROJECTION);
        candidate["artifact"]["identity"] = identity;
        candidate["artifact"]["authored_structure"] = structural;
        candidate["artifact"]["lock"] =
            serde_json::to_value(&registered.composition.registration.as_ref().unwrap().lock)
                .map_err(|_| fail("lock 직렬화 실패"))?;
        candidate["artifact"]["provenance"] = json!({"project_clone_origin":self.clone_origin,
            "composition_clone_origin":registered.composition.registration.as_ref().unwrap().clone_origin,
            "module_source_content":self.source_content});
        projection["persistent_identity"] = json!(true);
        Ok(Some(projection))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn malformed_public_projection_artifact_returns_existing_error_without_panic() {
        let source = "기본 := (보기=(() 표준보기.결과표)) 표준보개.으뜸. 짜임 := (관계들=차림[기본]) 표준보개.짜임.";
        let registered = project(source).register_bogae("앱", "main.ddn", "짜임", true).unwrap();
        let artifact = registered.save().unwrap();
        let activation = Activation::from_artifact(&artifact, source, "main.ddn").unwrap();
        for malformed in [json!(1), json!(true), json!("글"), json!([]), json!(null), json!({"relations":1})] {
            let result = activation.apply(Some(json!({"status":"ready", "candidates":[{
                "label":"짜임", "artifact":malformed
            }]})));
            assert!(matches!(result, Err(ref error) if error == "E_AUTHORED_BOGAE: 실제 typed 관계들이 없습니다"));
        }
        let temporary = run(source, None).bogae_projection.unwrap();
        let expected = run(source, Some(&artifact)).bogae_projection.unwrap();
        assert_eq!(activation.apply(Some(temporary)).unwrap().unwrap(), expected);
    }

    #[test]
    fn beta1_common_drawing_labs_register_original_multiaxis_source_content() {
        for (root,entry,composition,root_text,text) in [
            ("alpha3_double_pendulum_lab.project.ddn","alpha3_double_pendulum_lab.ddn","진자실험짜임",
                include_str!("../../solutions/seamgrim_ui_mvp/samples/alpha3_double_pendulum_lab.project.ddn"),
                include_str!("../../solutions/seamgrim_ui_mvp/samples/alpha3_double_pendulum_lab.ddn")),
            ("alpha4_lrl_lab.project.ddn","alpha4_lrl_lab.ddn","LRL실험짜임",
                include_str!("../../solutions/seamgrim_ui_mvp/samples/alpha4_lrl_lab.project.ddn"),
                include_str!("../../solutions/seamgrim_ui_mvp/samples/alpha4_lrl_lab.ddn")),
        ] {
            let request=json!({"source_profile":"v1-core-v25","module_name":"","source_snapshot":{
                "root_path":root,"entry_path":entry,"files":[{"path":root,"text":root_text},{"path":entry,"text":text},
                {"path":"alpha4_drawing_common.ddn","text":include_str!("../../solutions/seamgrim_ui_mvp/samples/alpha4_drawing_common.ddn")}]}});
            let original=authored_project::operate("create",&request.to_string()).unwrap();
            let project=authored_project::open(original.as_bytes()).unwrap();
            let registered=project.register_bogae("앱",entry,composition,true).unwrap();
            let artifact=registered.save().unwrap();let reopened=authored_project::open(artifact.as_bytes()).unwrap();
            assert_eq!(reopened.save().unwrap(),artifact);
            assert!(artifact.contains("registration"));
            assert_eq!(reopened.source_snapshot.files.iter().find(|file|file.path==entry).unwrap().text,text);
            let execution=crate::runtime_surface::run_authored_general_input_prefix(&artifact,1,7,None,None).unwrap();
            assert_eq!(execution.summary()["bogae_projection"]["persistent_identity"],true);
            let relations=execution.summary()["bogae_projection"]["candidates"][0]["artifact"]["relations"].as_array().unwrap();
            assert_eq!(relations.iter().map(|row|row["kind"].as_str().unwrap()).collect::<Vec<_>>(),vec!["primary","overlay","side","side"]);
        }
    }

    #[test]
    fn beta1_authored_content_multiaxis_keeps_order_bounds_and_writable_role() {
        let make=|reverse:bool,writable:bool,end:u64| {
            let mut selectors=vec![ddonirang_lang::TensorAxisSelector::Range{start:0,end},
                ddonirang_lang::TensorAxisSelector::Full,
                ddonirang_lang::TensorAxisSelector::Index(Box::new(Expr::new(2,Span::new(0,0),ExprKind::Var("순서".into()))))];
            if reverse {selectors.reverse();}
            let expression=Expr::new(3,Span::new(0,0),ExprKind::MultiAxisSelection(ddonirang_lang::MultiAxisSelection{
                target:Box::new(Expr::new(1,Span::new(0,0),ExprKind::Var("자료".into()))),selectors,writable}));
            expression_content(&expression,&BTreeSet::from(["자료".into(),"순서".into()])).unwrap()
        };
        let original=make(false,false,2);
        assert_eq!(original.references,BTreeSet::from(["자료".into(),"순서".into()]));
        for changed in [make(true,false,2),make(false,true,2),make(false,false,3)] {assert_ne!(original.semantic,changed.semantic);}
    }

    fn rename_request(
        project: &ProjectArtifact,
        kind: &str,
        id: &str,
        module: &str,
        name: &str,
        snapshot: SourceSnapshot,
    ) -> Result<ProjectArtifact, String> {
        let request = json!({"original_json":project.save()?,"target_kind":kind,"target_id":id,
            "owner_module_id":module,"new_name":name,"source_snapshot":snapshot});
        authored_project::open(
            authored_project::operate("rename", &request.to_string())?.as_bytes(),
        )
    }
    fn relocate_request(
        project: &ProjectArtifact,
        old: &str,
        new: &str,
        snapshot: SourceSnapshot,
    ) -> Result<ProjectArtifact, String> {
        let request = json!({"original_json":project.save()?,"old_path":old,"new_path":new,"source_snapshot":snapshot});
        authored_project::open(
            authored_project::operate("relocate", &request.to_string())?.as_bytes(),
        )
    }
    #[test]
    fn explicit_codec_rename_and_relocate_preserve_ids_digest_and_linked_execution() {
        let original = linked_project()
            .register_bogae("앱", "app.ddn", "짜임", true)
            .unwrap();
        let before = run_linked(&original, true).bogae_projection.unwrap();
        let sealed = original.save().unwrap();
        let lib = original
            .authored_identity
            .modules
            .iter()
            .find(|m| m.name == "도움")
            .unwrap();
        let helper = lib
            .bindings
            .iter()
            .find(|b| b.definition == "자료그림")
            .unwrap();
        let mut snapshot = original.source_snapshot.clone();
        snapshot.files[1].text = snapshot.files[1].text.replace("자료그림", "새자료그림");
        let renamed = rename_request(
            &original,
            "binding",
            helper.id.as_str(),
            lib.id.as_str(),
            "새자료그림",
            snapshot,
        )
        .unwrap();
        let renamed_helper = renamed
            .authored_identity
            .modules
            .iter()
            .find(|m| m.name == "도움")
            .unwrap()
            .bindings
            .iter()
            .find(|b| b.definition == "새자료그림")
            .unwrap();
        assert_eq!(helper.id, renamed_helper.id);
        assert_eq!(
            before["candidates"][0]["authored_digest"],
            run_linked(&renamed, true).bogae_projection.unwrap()["candidates"][0]
                ["authored_digest"]
        );
        let app = renamed
            .authored_identity
            .modules
            .iter()
            .find(|m| m.name == "앱")
            .unwrap();
        let composition = &app.compositions[0];
        let mut snapshot = renamed.source_snapshot.clone();
        snapshot.files[2].text = snapshot.files[2].text.replace("짜임 :=", "새짜임 :=");
        let renamed = rename_request(
            &renamed,
            "composition",
            composition.id.as_str(),
            app.id.as_str(),
            "새짜임",
            snapshot,
        )
        .unwrap();
        assert_eq!(
            renamed.authored_identity.entry,
            original.authored_identity.entry
        );
        let projected = run_linked(&renamed, true).bogae_projection.unwrap();
        assert_eq!(projected["candidates"][0]["label"], "새짜임");
        assert_eq!(
            before["candidates"][0]["authored_digest"],
            projected["candidates"][0]["authored_digest"]
        );
        let mut snapshot = renamed.source_snapshot.clone();
        snapshot.files[0].text = snapshot.files[0].text.replace("app.ddn", "moved/app.ddn");
        snapshot.files[2].path = "moved/app.ddn".into();
        snapshot.entry_path = "moved/app.ddn".into();
        let moved = relocate_request(&renamed, "app.ddn", "moved/app.ddn", snapshot).unwrap();
        let moved_app = moved
            .authored_identity
            .modules
            .iter()
            .find(|m| m.name == "앱")
            .unwrap();
        assert_eq!(moved_app.id, app.id);
        assert_eq!(
            moved.authored_identity.entry,
            original.authored_identity.entry
        );
        let moved_projection = run_linked(&moved, true).bogae_projection.unwrap();
        assert_eq!(
            before["candidates"][0]["authored_digest"],
            moved_projection["candidates"][0]["authored_digest"]
        );
        assert_eq!(
            before["candidates"][0]["artifact"]["view"],
            moved_projection["candidates"][0]["artifact"]["view"]
        );
        assert!(moved_app
            .bindings
            .iter()
            .all(|b| b.source_path == "moved/app.ddn"));
        assert_eq!(original.save().unwrap(), sealed);
        assert_eq!(
            authored_project::open(moved.save().unwrap().as_bytes()).unwrap(),
            moved
        );
    }
    #[test]
    fn explicit_edit_rejects_tampered_unknown_collision_wrong_mapping_and_owner_change() {
        let original = linked_project()
            .register_bogae("앱", "app.ddn", "짜임", true)
            .unwrap();
        let app = original
            .authored_identity
            .modules
            .iter()
            .find(|m| m.name == "앱")
            .unwrap();
        let composition = &app.compositions[0];
        let sealed = original.save().unwrap();
        let unchanged = original.source_snapshot.clone();
        assert!(rename_request(
            &original,
            "composition",
            composition.id.as_str(),
            app.id.as_str(),
            "새짜임",
            unchanged.clone()
        )
        .is_err());
        assert!(rename_request(
            &original,
            "composition",
            &"0".repeat(64),
            app.id.as_str(),
            "새짜임",
            unchanged.clone()
        )
        .is_err());
        assert!(rename_request(
            &original,
            "composition",
            composition.id.as_str(),
            &"0".repeat(64),
            "새짜임",
            unchanged.clone()
        )
        .is_err());
        let mut collision = unchanged.clone();
        collision.files[2].text = collision.files[2].text.replace("짜임 :=", "기본 :=");
        assert!(rename_request(
            &original,
            "composition",
            composition.id.as_str(),
            app.id.as_str(),
            "기본",
            collision
        )
        .is_err());
        assert!(relocate_request(&original, "app.ddn", "lib.ddn", unchanged.clone()).is_err());
        assert!(
            relocate_request(&original, "app.ddn", "moved/app.ddn", unchanged.clone()).is_err()
        );
        let mut ownership = unchanged.clone();
        ownership.files[0].text = ownership.files[0].text.replace("이름=앱", "이름=새앱");
        ownership.files[2].text = ownership.files[2].text.replace("짜임 :=", "새짜임 :=");
        assert!(rename_request(
            &original,
            "composition",
            composition.id.as_str(),
            app.id.as_str(),
            "새짜임",
            ownership
        )
        .is_err());
        let request = json!({"original_json":sealed.replace("대상=2","대상=3"),"old_path":"app.ddn","new_path":"moved/app.ddn","source_snapshot":unchanged});
        assert!(authored_project::operate("relocate", &request.to_string()).is_err());
        assert_eq!(original.save().unwrap(), sealed);
    }
    #[test]
    fn one_file_lifecycle_keeps_v2_ids_and_rejects_implicit_rename_activation() {
        let source="기본 := (보기=(() 표준보기.결과표)) 표준보개.으뜸. 짜임 := (관계들=차림[기본]) 표준보개.짜임.";
        let original = project(source)
            .register_bogae("앱", "main.ddn", "짜임", true)
            .unwrap();
        let module = &original.authored_identity.modules[0];
        let composition = &module.compositions[0];
        let mut snapshot = original.source_snapshot.clone();
        snapshot.files[0].text = source.replace("짜임 :=", "새짜임 :=");
        let update = json!({"original_json":original.save().unwrap(),"source_snapshot":snapshot});
        let implicit = authored_project::open(
            authored_project::operate("update", &update.to_string())
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
        assert_eq!(
            implicit.authored_identity.modules[0].compositions[0].definition,
            "짜임"
        );
        assert!(Activation::from_artifact(
            &implicit.save().unwrap(),
            &snapshot.files[0].text,
            "main.ddn"
        )
        .is_err());
        let renamed = rename_request(
            &original,
            "composition",
            composition.id.as_str(),
            module.id.as_str(),
            "새짜임",
            snapshot,
        )
        .unwrap();
        let mut snapshot = renamed.source_snapshot.clone();
        snapshot.files[0].path = "new/main.ddn".into();
        snapshot.entry_path = "new/main.ddn".into();
        let moved = relocate_request(&renamed, "main.ddn", "new/main.ddn", snapshot).unwrap();
        assert_eq!(
            moved.authored_identity.entry,
            original.authored_identity.entry
        );
        let old = run(source, Some(&original.save().unwrap()))
            .bogae_projection
            .unwrap();
        let current = run(
            &moved.source_snapshot.files[0].text,
            Some(&moved.save().unwrap()),
        )
        .bogae_projection
        .unwrap();
        assert_eq!(
            old["candidates"][0]["authored_digest"],
            current["candidates"][0]["authored_digest"]
        );
    }
    #[test]
    fn scoped_callable_parameter_and_packfield_are_not_foreign_dependencies() {
        let mut p = linked_project();
        p.source_snapshot.files[1].text.push_str("\n지름 := 9.");
        p.source_snapshot.files[2].text = p.source_snapshot.files[2]
            .text
            .replace("(대상:수)", "(지름:수)")
            .replace("반지름=대상", "반지름=지름")
            .replace(
                "확대:보기값:셈씨 := {",
                "확대:보기값:셈씨 := { 자료 := (지름:지름).",
            );
        let registered = p.register_bogae("앱", "app.ddn", "짜임", true).unwrap();
        assert!(!registered
            .authored_identity
            .modules
            .iter()
            .flat_map(|m| &m.bindings)
            .any(|b| b.definition == "지름"));
        let before = run_linked(&registered, true).bogae_projection.unwrap();
        let mut after = registered.clone();
        after.source_snapshot.files[1].text = after.source_snapshot.files[1]
            .text
            .replace("지름 := 9", "지름 := 10");
        refresh_source_locks(&mut after).unwrap();
        let after = run_linked(&after, true).bogae_projection.unwrap();
        assert_eq!(
            before["candidates"][0]["authored_digest"],
            after["candidates"][0]["authored_digest"]
        );
    }
    fn linked_project() -> ProjectArtifact {
        ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot{
            root_path:Some("work.ddn".into()),entry_path:"app.ddn".into(),files:vec![
                SourceFile{path:"work.ddn".into(),text:"(글판=\"v25\",모듬=차림[(이름=앱,머리=\"app.ddn\",원문=\"app.ddn\"),(이름=도움,머리=\"lib.ddn\",원문=\"lib.ddn\")],시작=(모듈=앱,정의=실행)) 얽음.".into()},
                SourceFile{path:"lib.ddn".into(),text:"(그림=자료그림) 드러냄.\n자료그림:보기값:셈씨 := { 자료그림 <- () 표준보기.결과표. }.".into()},
                SourceFile{path:"app.ddn".into(),text:"(도움=도움) 쓰임.\n(대상:수) 확대:보기값:셈씨 := { 원 := (중심=차림[0,0],반지름=대상) 표준장면.원. 장면 := (요소들=차림[원],가로범위=차림[-5,5],세로범위=차림[-5,5]) 표준장면.평면. 확대 <- (장면=장면) 표준보기.장면. }.\n기본 := (보기=(() 도움.그림)) 표준보개.으뜸.\n자세히 := (대상=2,보기법=확대) 표준보개.돋.\n짜임 := (관계들=차림[기본,자세히]) 표준보개.짜임.\n실행:움직씨 := { 값 <- 7. 값 보여주기. }.".into()},
            ]},String::new()).unwrap()
    }
    fn run_linked(project: &ProjectArtifact, persistent: bool) -> crate::ddn_runtime::DdnRunOutput {
        let (source, path, start) = project.execution_source().unwrap();
        let start = start.unwrap();
        let program = DdnProgram::from_v25_source(&source, &path).unwrap();
        let mut runner = DdnRunner::new(program, &start).with_named_entry_after_definitions();
        runner.configure_project_entry(&start).unwrap();
        if persistent {
            runner
                .configure_authored_bogae(&project.save().unwrap())
                .unwrap();
        }
        runner
            .run_update(
                &NuriWorld::new(),
                &InputSnapshot {
                    tick_id: 0,
                    dt: Fixed64::ONE,
                    keys_pressed: 0,
                    last_key_name: String::new(),
                    pointer_x_i32: 0,
                    pointer_y_i32: 0,
                    ai_injections: vec![],
                    net_events: vec![],
                    frame_source: InputSource::Person,
                    rng_seed: 0,
                },
                &HashMap::new(),
            )
            .unwrap()
    }
    #[test]
    fn linked_root_registers_original_owners_and_exact_execution_provenance() {
        let original = linked_project();
        let (source, path, start) = original.execution_source().unwrap();
        assert_eq!(path, "app.ddn");
        assert_eq!(start.as_deref(), Some("실행"));
        assert!(!source.contains("쓰임"));
        assert!(!source.contains("드러냄"));
        assert!(!source.contains("도움.그림"));
        assert!(source.contains("자료그림"));
        let registered = original
            .register_bogae("앱", "app.ddn", "짜임", true)
            .unwrap();
        let app = registered
            .authored_identity
            .modules
            .iter()
            .find(|m| m.name == "앱")
            .unwrap();
        let lib = registered
            .authored_identity
            .modules
            .iter()
            .find(|m| m.name == "도움")
            .unwrap();
        assert!(app.bindings.iter().all(|b| b.source_path == "app.ddn"));
        assert!(lib
            .bindings
            .iter()
            .any(|b| b.source_path == "lib.ddn" && b.definition == "자료그림"));
        let raw = file(&registered, "app.ddn").unwrap();
        let parsed = ddonirang_lang::parse_v25(&source, &path).unwrap();
        let preparation = registered.authored_preparation().unwrap().unwrap();
        for binding in &app.bindings {
            assert!(binding.declaration[1] <= raw.len());
            assert!(!raw[binding.declaration[0]..binding.declaration[1]].is_empty());
        }
        for TopLevelItem::SeedDef(seed) in &parsed.items {
            if seed.canonical_name == "확대" {
                let (origin, span) = preparation.original_span(seed.span).unwrap();
                assert_eq!(origin, "app.ddn");
                let b = app
                    .bindings
                    .iter()
                    .find(|b| b.definition == "확대")
                    .unwrap();
                assert_eq!(b.declaration, [span.start, span.end]);
                assert_ne!(b.declaration, [seed.span.start, seed.span.end]);
            }
        }
        let sealed = registered.save().unwrap();
        let activation = Activation::from_artifact(&sealed, &source, "<wasm>").unwrap();
        assert_eq!(activation.entry_definition(), Some("짜임"));
        let temporary = run_linked(&registered, false);
        let persistent = run_linked(&registered, true);
        assert_eq!(temporary.patch, persistent.patch);
        assert_eq!(temporary.resources, persistent.resources);
        assert_eq!(
            temporary.public_observations,
            persistent.public_observations
        );
        let temp = temporary.bogae_projection.unwrap();
        let projected = persistent.bogae_projection.unwrap();
        assert_eq!(
            activation.apply(Some(temp.clone())).unwrap().unwrap(),
            projected
        );
        assert_eq!(
            projected["candidates"][0]["artifact"]["identity"]["module"],
            app.id.as_str()
        );
        let recipe = &projected["candidates"][0]["artifact"]["relations"][1]["recipe"];
        assert_eq!(recipe["source"], source_hash(&source));
        assert_eq!(
            recipe["seal"],
            temp["candidates"][0]["artifact"]["relations"][1]["recipe"]["seal"]
        );
        assert!(
            Activation::from_artifact(&sealed, &format!("{source} "), "<wasm>")
                .unwrap_err()
                .contains("LINK_SOURCE_MISMATCH")
        );
        assert!(original
            .register_bogae("도움", "lib.ddn", "짜임", true)
            .unwrap_err()
            .contains("SOURCE_OWNER"));
        let cloned = registered.duplicate().unwrap();
        let cloned_projection = run_linked(&cloned, true).bogae_projection.unwrap();
        assert_ne!(
            projected["candidates"][0]["artifact"]["identity"],
            cloned_projection["candidates"][0]["artifact"]["identity"]
        );
        assert_eq!(
            projected["candidates"][0]["artifact"]["view"],
            cloned_projection["candidates"][0]["artifact"]["view"]
        );
    }
    #[test]
    fn linked_update_keeps_ids_and_rejects_bad_import_or_original_binding_locator() {
        let registered = linked_project()
            .register_bogae("앱", "app.ddn", "짜임", true)
            .unwrap();
        let mut edited = registered.source_snapshot.clone();
        edited.files[1].text = format!("// source locator moved\n{}", edited.files[1].text);
        let request = json!({"original_json":registered.save().unwrap(),"source_snapshot":edited});
        let updated = authored_project::open(
            authored_project::operate("update", &request.to_string())
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
        assert_eq!(
            registered
                .authored_identity
                .modules
                .iter()
                .flat_map(|m| m.bindings.iter().map(|b| &b.id))
                .collect::<Vec<_>>(),
            updated
                .authored_identity
                .modules
                .iter()
                .flat_map(|m| m.bindings.iter().map(|b| &b.id))
                .collect::<Vec<_>>()
        );
        let before = run_linked(&registered, true).bogae_projection.unwrap();
        let after = run_linked(&updated, true).bogae_projection.unwrap();
        assert_eq!(
            before["candidates"][0]["authored_digest"],
            after["candidates"][0]["authored_digest"]
        );
        let mut wrong = registered.clone();
        wrong
            .authored_identity
            .modules
            .iter_mut()
            .find(|m| m.name == "앱")
            .unwrap()
            .bindings[0]
            .declaration = [0, 1];
        let (source, path, _) = wrong.execution_source().unwrap();
        assert!(Activation::from_artifact(&wrong.save().unwrap(), &source, &path).is_err());
        let mut bad = linked_project();
        bad.source_snapshot.files[2].text = bad.source_snapshot.files[2]
            .text
            .replace("도움.그림", "도움.없음");
        assert!(bad
            .execution_source()
            .unwrap_err()
            .contains("E_SYMBOL_NOT_EXPORTED"));
        let mut implicit = linked_project();
        implicit.source_snapshot.files[2].text = implicit.source_snapshot.files[2]
            .text
            .replace("도움.그림", "자료그림");
        assert!(implicit
            .register_bogae("앱", "app.ddn", "짜임", true)
            .unwrap_err()
            .contains("E_AUTHORED_BOGAE_IMPORT_UNRESOLVED"));
    }
    #[test]
    fn alpha_normalization_never_substitutes_pin_labels_or_fields() {
        let content = |source: &str| {
            let program = ddonirang_lang::parse_v25(source, "scope.ddn").unwrap();
            let globals = BTreeSet::new();
            let mut scope = ContentScope {
                globals: &globals,
                locals: BTreeMap::new(),
                references: BTreeSet::new(),
            };
            scope
                .body(&ddonirang_lang::Body {
                    id: 0,
                    span: Span::new(0, source.len()),
                    stmts: program.root_stmts,
                })
                .unwrap()
        };
        let a = content("x <- 1. y <- 2. (x=x,y=y) 함수.");
        let b = content("x <- 1. y <- 2. (y=x,x=y) 함수.");
        assert_ne!(a, b);
        // RD-203: field access needs a named record target. This identity
        // test varies real fields, not invalid field access on a number.
        let a = content("x <- (x:1,y:2). x.x.");
        let b = content("x <- (x:1,y:2). x.y.");
        assert_ne!(a, b);
        let a = content("x <- 1. y <- 2. (x:x,y:y).");
        let b = content("x <- 1. y <- 2. (y:x,x:y).");
        assert_ne!(a, b);
    }
    use super::*;
    use crate::authored_project::{SourceFile, SourceSnapshot};
    use crate::ddn_runtime::{DdnProgram, DdnRunner};
    use ddonirang_core::platform::NuriWorld;
    use ddonirang_core::{Fixed64, InputSnapshot, InputSource};

    const SOURCE:&str="기본 := (보기=(() 표준보기.결과표)) 표준보개.으뜸. 곁 := (보기=(() 표준보기.결과표),자리=#오른쪽) 표준보개.곁. 첫째 := (관계들=차림[기본,곁,곁]) 표준보개.짜임. 둘째 := (관계들=차림[기본,곁]) 표준보개.짜임.";
    fn project(source: &str) -> ProjectArtifact {
        ProjectArtifact::create(
            "v1-core-v25".into(),
            SourceSnapshot {
                root_path: None,
                entry_path: "main.ddn".into(),
                files: vec![SourceFile {
                    path: "main.ddn".into(),
                    text: source.into(),
                }],
            },
            "앱".into(),
        )
        .unwrap()
    }
    fn run(source: &str, artifact: Option<&str>) -> crate::ddn_runtime::DdnRunOutput {
        let program = DdnProgram::from_v25_source(source, "C:/alias/main.ddn").unwrap();
        let mut runner = DdnRunner::new(program, "매마디");
        if let Some(artifact) = artifact {
            runner.configure_authored_bogae(artifact).unwrap();
        }
        runner
            .run_update(
                &NuriWorld::new(),
                &InputSnapshot {
                    tick_id: 0,
                    dt: Fixed64::ONE,
                    keys_pressed: 0,
                    last_key_name: String::new(),
                    pointer_x_i32: 0,
                    pointer_y_i32: 0,
                    ai_injections: vec![],
                    net_events: vec![],
                    frame_source: InputSource::Person,
                    rng_seed: 0,
                },
                &HashMap::new(),
            )
            .unwrap()
    }
    #[test]
    fn registration_save_reopen_and_clone_share_named_binding_ids() {
        let original = project(SOURCE);
        let bytes = original.save().unwrap();
        assert!(!bytes.contains("registration"));
        assert!(!bytes.contains("bindings"));
        assert_eq!(
            authored_project::open(bytes.as_bytes())
                .unwrap()
                .save()
                .unwrap(),
            bytes
        );
        let one = original
            .register_bogae("앱", "main.ddn", "첫째", false)
            .unwrap();
        let two = one.register_bogae("앱", "main.ddn", "둘째", true).unwrap();
        let module = &two.authored_identity.modules[0];
        let first = &module.compositions[0];
        let second = &module.compositions[1];
        assert_eq!(first.binding_ids[1], first.binding_ids[2]);
        assert_eq!(first.binding_ids[..2], second.binding_ids);
        assert_eq!(module.bindings.len(), 2);
        let repeated = two.register_bogae("앱", "main.ddn", "둘째", true).unwrap();
        assert_eq!(repeated.authored_identity, two.authored_identity);
        let saved = two.save().unwrap();
        assert_eq!(
            authored_project::open(saved.as_bytes())
                .unwrap()
                .save()
                .unwrap(),
            saved
        );
        let cloned = two.duplicate().unwrap();
        let cloned_module = &cloned.authored_identity.modules[0];
        assert_ne!(cloned_module.id, module.id);
        assert_eq!(
            cloned_module.compositions[0].binding_ids[1],
            cloned_module.compositions[0].binding_ids[2]
        );
        assert_eq!(
            cloned_module.compositions[0].binding_ids[..2],
            cloned_module.compositions[1].binding_ids
        );
        assert_ne!(
            cloned_module.compositions[0].binding_ids[0],
            first.binding_ids[0]
        );
        assert_eq!(
            cloned_module.compositions[0]
                .registration
                .as_ref()
                .unwrap()
                .clone_origin,
            Some(first.id.clone())
        );
        assert_eq!(
            Activation::from_artifact(&saved, SOURCE, "<wasm>")
                .unwrap()
                .entry_definition(),
            Some("둘째")
        );
        assert_eq!(
            Activation::from_artifact(&one.save().unwrap(), SOURCE, "<wasm>")
                .unwrap()
                .entry_definition(),
            None
        );
    }
    #[test]
    fn original_seal_and_source_lock_tampering_are_rejected() {
        let registered = project(SOURCE)
            .register_bogae("앱", "main.ddn", "둘째", true)
            .unwrap();
        let mut tampered = registered.clone();
        tampered.authored_identity.modules[0].compositions[0]
            .registration
            .as_mut()
            .unwrap()
            .lock
            .source_content = "sha256:bad".into();
        assert!(tampered.save().is_err());
        let sealed = registered.save().unwrap();
        assert!(Activation::from_artifact(&sealed, &format!("{SOURCE} "), "<wasm>").is_err());
        assert!(
            Activation::from_artifact(&project(SOURCE).save().unwrap(), SOURCE, "main.ddn")
                .is_err()
        );
        let request = json!({"original_json":sealed.replace("둘째","변조"),"module_name":"앱","source_path":"main.ddn","composition_definition":"둘째","make_entry":true});
        assert!(authored_project::operate("register_bogae", &request.to_string()).is_err());
    }
    #[test]
    fn indirect_inline_and_wrong_relation_authorship_are_rejected() {
        for source in ["짜임 := (관계들=차림[(() 표준보기.결과표)]) 표준보개.짜임.",
            "기본 := (보기=(() 표준보기.결과표)) 표준보개.으뜸. 관계들 := 차림[기본]. 짜임 := (관계들=관계들) 표준보개.짜임.",
            "기본 := (() 표준보기.결과표). 짜임 := (관계들=차림[기본]) 표준보개.짜임."] {
            assert!(project(source).register_bogae("앱","main.ddn","짜임",true).is_err());
        }
    }
    #[test]
    fn edits_keep_ids_and_digest_but_refresh_exact_source_lock() {
        let source="기본 := (보기=(() 표준보기.결과표)) 표준보개.으뜸. 짜임 := (관계들=차림[기본]) 표준보개.짜임.";
        let registered = project(source)
            .register_bogae("앱", "main.ddn", "짜임", true)
            .unwrap();
        let before = run(source, Some(&registered.save().unwrap()));
        let mut snapshot = registered.source_snapshot.clone();
        snapshot.files[0].text = format!("// locator move\n{source}");
        let request =
            json!({"original_json":registered.save().unwrap(),"source_snapshot":snapshot});
        let updated = authored_project::open(
            authored_project::operate("update", &request.to_string())
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
        assert_eq!(
            registered.authored_identity.modules[0].compositions[0].id,
            updated.authored_identity.modules[0].compositions[0].id
        );
        assert_eq!(
            registered.authored_identity.modules[0].bindings[0].id,
            updated.authored_identity.modules[0].bindings[0].id
        );
        let after = run(
            &updated.source_snapshot.files[0].text,
            Some(&updated.save().unwrap()),
        );
        let a = &before.bogae_projection.unwrap()["candidates"][0];
        let b = &after.bogae_projection.unwrap()["candidates"][0];
        assert_eq!(a["authored_digest"], b["authored_digest"]);
        assert_eq!(
            a["artifact"]["relations"][0]["relation_id"],
            b["artifact"]["relations"][0]["relation_id"]
        );
        assert_ne!(
            a["artifact"]["lock"]["source_content"],
            b["artifact"]["lock"]["source_content"]
        );
    }
    #[test]
    fn authored_geometry_edit_changes_digest_and_explicit_definition_rename_does_not() {
        let source="그림:보기값:셈씨 := { 원 := (중심=차림[0,0],반지름=2) 표준장면.원. 장면 := (요소들=차림[원],가로범위=차림[-5,5],세로범위=차림[-5,5]) 표준장면.평면. 그림 <- (장면=장면) 표준보기.장면. }. 기본 := (보기=(() 그림)) 표준보개.으뜸. 짜임 := (관계들=차림[기본]) 표준보개.짜임.";
        let registered = project(source)
            .register_bogae("앱", "main.ddn", "짜임", true)
            .unwrap();
        let digest = |source: &str, project: &ProjectArtifact| {
            run(source, Some(&project.save().unwrap()))
                .bogae_projection
                .unwrap()["candidates"][0]["authored_digest"]
                .clone()
        };
        let original = digest(source, &registered);
        let mut edited = registered.clone();
        edited.source_snapshot.files[0].text = source.replace("반지름=2", "반지름=3");
        refresh_source_locks(&mut edited).unwrap();
        assert_ne!(
            original,
            digest(&edited.source_snapshot.files[0].text, &edited)
        );
        assert_eq!(
            registered.authored_identity.modules[0]
                .bindings
                .iter()
                .map(|b| &b.id)
                .collect::<Vec<_>>(),
            edited.authored_identity.modules[0]
                .bindings
                .iter()
                .map(|b| &b.id)
                .collect::<Vec<_>>()
        );
        // Explicit metadata rename, not inference from a text diff. The codec
        // supports keeping the same authored identity through that edit.
        let mut renamed = registered.clone();
        renamed.source_snapshot.files[0].text = source.replace("그림", "새그림");
        renamed.authored_identity.modules[0]
            .bindings
            .iter_mut()
            .find(|b| b.definition == "그림")
            .unwrap()
            .definition = "새그림".into();
        refresh_source_locks(&mut renamed).unwrap();
        assert_eq!(
            original,
            digest(&renamed.source_snapshot.files[0].text, &renamed)
        );
    }
    #[test]
    fn actual_three_samples_register_and_apply_without_geometry_or_world_changes() {
        for (source, definition) in [
            (
                include_str!(
                    "../../solutions/seamgrim_ui_mvp/samples/alpha2_parabola_composition.ddn"
                ),
                "포물선짜임",
            ),
            (
                include_str!(
                    "../../solutions/seamgrim_ui_mvp/samples/alpha2_pendulum_composition.ddn"
                ),
                "진자짜임",
            ),
            (
                include_str!(
                    "../../solutions/seamgrim_ui_mvp/samples/alpha2_linear_composition.ddn"
                ),
                "일차함수짜임",
            ),
        ] {
            let registered = project(source)
                .register_bogae("앱", "main.ddn", definition, true)
                .unwrap();
            let sealed = registered.save().unwrap();
            let activation = Activation::from_artifact(&sealed, source, "<wasm>").unwrap();
            let temporary = run(source, None);
            let persistent = run(source, Some(&sealed));
            assert_eq!(temporary.patch, persistent.patch);
            assert_eq!(
                temporary.public_observations,
                persistent.public_observations
            );
            assert_eq!(temporary.resources, persistent.resources);
            let temp = temporary.bogae_projection.unwrap();
            let applied = activation.apply(Some(temp.clone())).unwrap().unwrap();
            assert_eq!(applied, persistent.bogae_projection.unwrap());
            assert_eq!(applied["persistent_identity"], true);
            let a = &temp["candidates"][0]["artifact"];
            let b = &applied["candidates"][0]["artifact"];
            assert_eq!(a["view"], b["view"]);
            for (old, new) in a["relations"]
                .as_array()
                .unwrap()
                .iter()
                .zip(b["relations"].as_array().unwrap())
            {
                assert_eq!(old["view"], new["view"]);
                assert_eq!(old["target"], new["target"]);
                assert_eq!(old["recipe"]["seal"], new["recipe"]["seal"]);
            }
            let mut wrong = temp;
            wrong["candidates"][0]["artifact"]["relations"][0]["identity"]
                ["authored_binding_location"] = json!([0, 0, 0, 0]);
            assert!(activation.apply(Some(wrong)).is_err());
        }
    }
}
