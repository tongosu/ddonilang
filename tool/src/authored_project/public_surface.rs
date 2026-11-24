//! Source-owned public declaration facts, never registration or DDL admission.
//! Every exported target must resolve to one actual declaration in the module
//! closure. Types/effects/Definition and Pin issuance remain with their owners.
use super::{ModuleId, ProjectArtifact};
use ddonirang_lang::{CanonProgram, Lexer, ModuleDirectiveContext, ParseMode, Parser, SeedDef, Stmt, TopLevelItem, TokenKind};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
pub(crate) enum DeclarationNode {
    // Any NodeId in these parser nodes is source-local, never a persistent
    // DefinitionIdentity/ParamPinIdentity or a successful type/effect proof.
    Callable(SeedDef),
    Data(Stmt),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authored_project::{SourceFile,SourceSnapshot};
    const F:&str="(왼값:나눔수~에, 오른값:나눔수~을) 더하다:나눔수:셈씨 := { 더하다 <- 왼값 + 오른값. }.\n";
    const G:&str="(값:나눔수~을) 두배하다:나눔수:셈씨 := { 두배하다 <- 값 + 값. }.\n";
    fn project(public_g:bool,split:bool)->ProjectArtifact {
        let exports=if public_g {"(첫째=더하다, 둘째=두배하다) 드러냄.\n"} else {"(첫째=더하다) 드러냄.\n"};
        let mut files=vec![
            SourceFile{path:"root.ddn".into(),text:format!("(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"main.ddn\",원문=\"main.ddn\"),(이름=모음,머리=\"모음.ddn\",원문={})], 시작=(모듈=앱,정의=실행)) 얽음.\n",if split {"차림[\"모음.ddn\",\"추가.ddn\"]"}else{"\"모음.ddn\""})},
            SourceFile{path:"main.ddn".into(),text:"실행:움직씨 := { 0 보여주기. }.\n".into()},
            SourceFile{path:"모음.ddn".into(),text:format!("#!한국어\n// 日本語 원문 위치\n{exports}{F}{}",if split{""}else{G})},
        ];
        if split {files.push(SourceFile{path:"추가.ddn".into(),text:G.into()});}
        let created=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot{entry_path:"main.ddn".into(),root_path:Some("root.ddn".into()),files},String::new()).unwrap();
        crate::authored_project::open(created.save().unwrap().as_bytes()).unwrap()
    }
    fn facts(project:&ProjectArtifact)->SourceOwnedPublicFacts {
        let owner=project.authored_identity.modules.iter().find(|m|m.name=="모음").unwrap();
        project.source_owned_public_facts(owner.id.as_str()).unwrap()
    }
    #[test]
    fn r20_public_denominator_uses_all_real_exports_and_declared_pins() {
        let project=project(true,false);let facts=facts(&project);
        assert_eq!(facts.declarations.len(),2);
        assert!(facts.check_export_name_coverage(&["첫째"]).is_err());
        assert!(facts.check_export_name_coverage(&["첫째","둘째"]).is_ok());
        assert!(facts.check_export_name_coverage(&["첫째","둘째","둘째"]).is_err());
        assert_eq!(facts.project_content_sha256,project.content_sha256);
        for (name,count) in [("더하다",2),("두배하다",1)] {
            let row=facts.declarations.iter().find(|r|r.internal_name==name).unwrap();
            let DeclarationNode::Callable(seed)=&row.node else {panic!("실제 callable 원문이어야 합니다")};
            assert_eq!(seed.params.len(),count);assert!(seed.result_type.is_some());
            let text=&project.source_snapshot.files.iter().find(|f|f.path==row.source_path).unwrap().text;
            let start=row.declaration_offset+seed.span.start;let end=row.declaration_offset+seed.span.end;
            assert!(text.get(start..end).unwrap().contains(name));
            assert_eq!(row.source_sha256,hex::encode(<sha2::Sha256 as sha2::Digest>::digest(text.as_bytes())));
        }
    }
    #[test]
    fn r20_private_definition_is_not_public_and_module_fragments_are_not_dropped() {
        let private=project(false,false);let private_facts=facts(&private);
        assert_eq!(private_facts.public_targets(),["더하다".to_string()].into_iter().collect());
        assert!(private_facts.check_export_name_coverage(&["첫째"]).is_ok());
        let split=project(true,true);let split_facts=facts(&split);
        assert_eq!(split_facts.source_paths.len(),2);assert_eq!(split_facts.declarations.len(),2);
        assert_eq!(split_facts.declarations.iter().find(|r|r.internal_name=="두배하다").unwrap().source_path,"추가.ddn");
        let owner=split.authored_identity.modules.iter().find(|m|m.name=="모음").unwrap();
        assert_eq!(split_facts.module,owner.id);
    }
    #[test]
    fn r20_stale_foreign_owner_and_missing_declaration_are_rejected() {
        let mut stale=project(true,false);let owner=stale.authored_identity.modules.iter().find(|m|m.name=="모음").unwrap().id.clone();
        let checked=facts(&stale);checked.recheck(&stale).unwrap();
        stale.content_sha256="00".repeat(32);
        assert!(stale.source_owned_public_facts(owner.as_str()).is_err());
        assert!(checked.recheck(&stale).is_err());
        let good=project(true,false);
        assert!(good.source_owned_public_facts(&"ff".repeat(32)).unwrap_err().contains("MODULE_OWNER_REQUIRED"));
        let mut snapshot=good.source_snapshot.clone();
        let head=snapshot.files.iter_mut().find(|f|f.path=="모음.ddn").unwrap();
        head.text=head.text.replace("둘째=두배하다","둘째=없는정의");
        let created=ProjectArtifact::create("v1-core-v25".into(),snapshot,String::new()).unwrap();
        let changed=crate::authored_project::open(created.save().unwrap().as_bytes()).unwrap();
        let owner=changed.authored_identity.modules.iter().find(|m|m.name=="모음").unwrap();
        let error=changed.source_owned_public_facts(owner.id.as_str()).unwrap_err();
        assert!(error.contains("E_RD457_EXPORT_TARGET_PRIVATE_OR_UNKNOWN"),"{error}");
    }
    #[test]
    fn r20_rootless_existing_first_callable_is_preserved_without_synthetic_root() {
        let raw="(총액합하다=총액합하다) 드러냄.\n(금액:나눔수~에, 세액:나눔수~을~를) 총액합하다{총액합하기}:나눔수:셈씨 := { 총액합하다 <- 금액 + 세액. }.\n";
        let created=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot{entry_path:"세금.ddn".into(),root_path:None,files:vec![SourceFile{path:"세금.ddn".into(),text:raw.into()}]},"세금".into()).unwrap();
        let original=crate::authored_project::open(created.save().unwrap().as_bytes()).unwrap();
        let owner=&original.authored_identity.modules[0];
        assert!(original.source_owned_public_facts(owner.id.as_str()).is_err());
        let ddl="#!日本語\n(金額:有理数~に, 税額:有理数~を) 合計する:有理数:関数 :={...}.\n";
        let r=ddonirang_lang::ddl::FIRST_CALLABLE_JA_REGISTRATION;
        original.first_callable_registration_context(owner.id.as_str(),"세금.ddn",ddl,"税.ddl",r.language_profile_identity,r.registration_revision).unwrap();
    }
    #[test]
    fn r20_existing_closed_consumer_uses_actual_rooted_public_supply() {
        let original=project(false,false);let mut snapshot=original.source_snapshot.clone();
        snapshot.files.iter_mut().find(|f|f.path=="모음.ddn").unwrap().text=
            "(총액합하다=총액합하다) 드러냄.\n(금액:나눔수~에, 세액:나눔수~을~를) 총액합하다{총액합하기}:나눔수:셈씨 := { 총액합하다 <- 금액 + 세액. }.\n".into();
        let created=ProjectArtifact::create("v1-core-v25".into(),snapshot,String::new()).unwrap();
        let original=crate::authored_project::open(created.save().unwrap().as_bytes()).unwrap();
        let owner=original.authored_identity.modules.iter().find(|m|m.name=="모음").unwrap();
        let ddl="#!日本語\n(金額:有理数~に, 税額:有理数~を) 合計する:有理数:関数 :={...}.\n";
        let r=ddonirang_lang::ddl::FIRST_CALLABLE_JA_REGISTRATION;
        original.first_callable_registration_context(owner.id.as_str(),"모음.ddn",ddl,"税.ddl",r.language_profile_identity,r.registration_revision).unwrap();
    }
    fn rational_body_facts(expression:&str,extra:&str)->SourceOwnedPublicFacts {
        let mut snapshot=project(true,false).source_snapshot;
        let file=snapshot.files.iter_mut().find(|file|file.path=="모음.ddn").unwrap();
        file.text=file.text.replace("더하다 <- 왼값 + 오른값.",
            &format!("더하다 <- {expression}."));
        file.text.push_str(extra);
        let project=ProjectArtifact::create("v1-core-v25".into(),snapshot,String::new()).unwrap();
        let verified=crate::authored_project::open(project.save().unwrap().as_bytes()).unwrap();
        facts(&verified)
    }

    #[test]
    fn r697_public_rational_multiply_and_literal_body_use_real_ast() {
        for (expression,expected) in [
            ("왼값 * 오른값","mul(pin:왼값,pin:오른값)"),
            ("왼값 + 오른값 + (1,1) 나눔수","add(add(pin:왼값,pin:오른값),rational(1,1))"),
            ("왼값 * (2,4) 나눔수","mul(pin:왼값,rational(1,2))"),
            ("왼값 * (-2,4) 나눔수","mul(pin:왼값,rational(-1,2))"),
        ] {
            let facts=rational_body_facts(expression,"");
            let declaration=facts.declarations.iter().find(|row|row.internal_name=="더하다").unwrap();
            let (_,body)=check_ddl_callable_shape(declaration).unwrap_or_else(|error|panic!("{expression}: {error}"));
            assert_eq!(body,expected);
        }
    }

    #[test]
    fn r697_public_rational_body_rejects_other_calls_and_invalid_constants() {
        for expression in [
            "왼값 / 오른값",
            "왼값 + 1",
            "왼값 + (1,0) 나눔수",
            "왼값 + (0.5,1) 나눔수",
            "왼값 + (1.0000000000001,1) 나눔수",
            "왼값 + (왼값,1) 나눔수",
            "(왼값) 두배하다",
        ] {
            let facts=rational_body_facts(expression,"");
            let declaration=facts.declarations.iter().find(|row|row.internal_name=="더하다").unwrap();
            assert!(check_ddl_callable_shape(declaration).is_err(),"{expression}");
        }
        let facts=rational_body_facts("왼값 + (1,1) 나눔수",
            "\n(p:수~에,q:수~을) 나눔수:나눔수:셈씨 := { 나눔수 <- (p,q) 유리수. }.\n");
        let declaration=facts.declarations.iter().find(|row|row.internal_name=="더하다").unwrap();
        assert!(check_ddl_callable_shape(declaration).is_err(),"same-spelling user constructor");
    }

}

#[derive(Debug, Clone)]
pub(crate) struct PublicDeclarationFacts {
    external_name: String,
    internal_name: String,
    source_path: String,
    /// AST spans are relative to this verbatim declaration tail, not a new identity.
    declaration_offset: usize,
    source_sha256: String,
    node: DeclarationNode,
    rational_constructor_unshadowed: bool,
}

impl PublicDeclarationFacts {
    pub(crate) fn external_name(&self) -> &str { &self.external_name }
    pub(crate) fn internal_name(&self) -> &str { &self.internal_name }
    pub(crate) fn source_path(&self) -> &str { &self.source_path }
    pub(crate) fn declaration_offset(&self) -> usize { self.declaration_offset }
    pub(crate) fn source_sha256(&self) -> &str { &self.source_sha256 }
    pub(crate) fn node(&self) -> &DeclarationNode { &self.node }
}

#[derive(Debug, Clone)]
pub(crate) struct SourceOwnedPublicFacts {
    module: ModuleId,
    project_content_sha256: String,
    source_paths: Vec<String>,
    declarations: Vec<PublicDeclarationFacts>,
}

impl SourceOwnedPublicFacts {
    pub(crate) fn module(&self) -> &ModuleId { &self.module }
    pub(crate) fn source_paths(&self) -> &[String] { &self.source_paths }
    pub(crate) fn declarations(&self) -> &[PublicDeclarationFacts] { &self.declarations }
    pub(crate) fn public_targets(&self) -> BTreeSet<String> {
        self.declarations.iter().map(|row| row.internal_name.clone()).collect()
    }
    /// Checks only names, not types/effects/profile/coverage admission.
    pub(crate) fn check_export_name_coverage(&self, names: &[&str]) -> Result<(), String> {
        let expected=self.declarations.iter().map(|row|row.external_name.as_str()).collect::<BTreeSet<_>>();
        let actual=names.iter().copied().collect::<BTreeSet<_>>();
        if actual.len()!=names.len() || actual!=expected {
            return Err("E_PUBLIC_SURFACE_INCOMPLETE: 실제 공개 선언 전체의 대응이 필요합니다".into());
        }
        Ok(())
    }
    pub(crate) fn recheck(&self, project: &ProjectArtifact) -> Result<(),String> {
        let current=from_project(project,self.module.as_str())?;
        if current.project_content_sha256!=self.project_content_sha256 || current.source_paths!=self.source_paths {
            return Err("E_PUBLIC_SURFACE_STALE: 실제 공개 원본 판본이 달라졌습니다".into());
        }
        Ok(())
    }
}

fn declaration_nodes(program: &CanonProgram) -> Vec<(String, DeclarationNode)> {
    let mut nodes=Vec::new();
    fn data(nodes: &mut Vec<(String,DeclarationNode)>, statement: &Stmt) {
        match statement {
            Stmt::Definition{name,..}|Stmt::DerivedBinding{name,..} => nodes.push((name.clone(),DeclarationNode::Data(statement.clone()))),
            Stmt::DeclBlock{items,..} => for item in items {nodes.push((item.name.clone(),DeclarationNode::Data(statement.clone())));},
            _=>{},
        }
    }
    for item in &program.items {
        let TopLevelItem::SeedDef(seed)=item;
        nodes.push((seed.canonical_name.clone(),DeclarationNode::Callable(seed.clone())));
        // The same top-level declaration rule as the existing module owner;
        // callable-local declarations never become public module targets.
        if let Some(body)=&seed.body {
            for statement in &body.stmts {
                if let Stmt::DeclBlock{span,..}=statement {
                    if span.start<=seed.span.start {data(&mut nodes,statement);}
                }
            }
        }
    }
    for statement in &program.root_stmts {data(&mut nodes,statement);}
    nodes
}

pub(super) fn from_project(project: &ProjectArtifact, module_identity: &str) -> Result<SourceOwnedPublicFacts,String> {
    from_project_diagnostic(project, module_identity, None)
}

pub(super) fn from_project_diagnostic(
    project: &ProjectArtifact,
    module_identity: &str,
    diagnostic_out: Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>,
) -> Result<SourceOwnedPublicFacts,String> {
    project.validate()?;
    if project.content_sha256!=project.checksum()? {
        return Err("E_MULTILANG_ORIGINAL_CHECKSUM_STALE: 검증된 원본 판본이 필요합니다".into());
    }
    let module=project.authored_identity.modules.iter().find(|m|m.id.as_str()==module_identity)
        .ok_or("E_MULTILANG_ORIGINAL_MODULE_OWNER_REQUIRED")?;
    let j1=if project.source_profile==ddonirang_lang::ddl::J1_SOURCE_PROFILE{
        Some(ddonirang_lang::ddl::CheckedJ1Profile::resolve(&project.source_profile).map_err(|error|error.to_string())?)
    }else{None};
    let source=super::snapshot_source_selected(&project.source_snapshot,j1)?;
    let directives=match j1{
        Some(profile)=>crate::local_project::resolve_work_project_module_directives_with_j1_profile(&source,&module.head,profile),
        None=>crate::local_project::resolve_work_project_module_directives(&source,&module.head),
    }.map_err(|error| {
        if let Some(out) = diagnostic_out {
            if let Some(diagnostic) = error.cause.as_deref().filter(|cause| cause.phase == "lang_parse") {
                *out = Some(diagnostic.clone());
            }
        }
        format!("{}: {}", error.code, error.message)
    })?;
    if !directives.typed_current {return Err("E_PUBLIC_SURFACE_CURRENT_DECLARATIONS_REQUIRED".into());}
    // Targets come from the real owner's validated exports, not an arbitrary
    // token allowlist or a first-function spelling supplied by the consumer.
    let targets=directives.exports.iter().map(|row|row.internal_name.clone()).collect::<BTreeSet<_>>();
    let context=ModuleDirectiveContext::default().with_public_definitions(targets)
        .with_known_imports(directives.imports.iter().map(|row|row.source.clone()));
    let mut definitions:BTreeMap<String,Vec<PublicDeclarationFacts>>=BTreeMap::new();
    for path in &module.sources {
        let file=project.source_snapshot.files.iter().find(|file|&file.path==path)
            .ok_or("E_MULTILANG_ORIGINAL_SOURCE_REQUIRED")?;
        let (program,offset)=if let Some(profile)=j1{
            let prefix=ddonirang_lang::module_directive::parse_module_directive_prefix_with_j1_profile(&file.text,path,&context,profile).map_err(|error|error.to_string())?;
            if path!=&module.head && (prefix.imports.is_some()||prefix.exports.is_some()){
                return Err("E_PUBLIC_SURFACE_FRAGMENT_DIRECTIVE_UNSUPPORTED: 조각의 별도 공개 결속 공급이 필요합니다".into());
            }
            // The checked J1 parser retains the entire original source. Its
            // node spans are already original coordinates, including the prefix.
            (Parser::parse_unlinked_j1_module(&file.text,path,profile).map_err(|error|error.to_string())?,0)
        }else{
        let (mut source_lexer,_)=Lexer::with_korean_source_header(&file.text).map_err(|error|error.message)?;
        let source_tokens=source_lexer.tokenize().map_err(|error|error.message)?;
        // Module records start with a tuple. Other declarations, including
        // ordinary data in fragments, stay with the same unlinked parser.
        // Keep the existing retired brace directive rejection as well.
        let metadata_prefix=matches!(source_tokens.first().map(|t|&t.kind),Some(TokenKind::LParen))
            || matches!((source_tokens.first().map(|t|&t.kind),source_tokens.get(1).map(|t|&t.kind)),
                (Some(TokenKind::Ident(name)|TokenKind::Josa(name)),Some(TokenKind::LBrace)) if matches!(name.as_str(),"쓰임"|"드러냄"|"공개"));
        let (declaration_source,offset)=if metadata_prefix {
            let prefix=ddonirang_lang::parse_module_directive_prefix(&file.text,path,&context).map_err(|error|error.to_string())?;
            if path!=&module.head && (prefix.imports.is_some()||prefix.exports.is_some()) {
                return Err("E_PUBLIC_SURFACE_FRAGMENT_DIRECTIVE_UNSUPPORTED: 조각의 별도 공개 결속 공급이 필요합니다".into());
            }
            let offset=file.text.len().checked_sub(prefix.declaration_source.len())
                .filter(|offset|file.text.get(*offset..)==Some(prefix.declaration_source.as_str()))
                .ok_or("E_MULTILANG_ORIGINAL_SPAN_SUPPLY_REQUIRED")?;
            (prefix.declaration_source,offset)
        }else {(file.text.clone(),0)};
        let (mut declaration_lexer,_)=Lexer::with_korean_source_header(&declaration_source).map_err(|error|error.message)?;
        let tokens=declaration_lexer.tokenize().map_err(|error|error.message)?;
        let program=Parser::new_v25_with_mode(tokens,ParseMode::Strict)
            .parse_unlinked_module(declaration_source,path.clone()).map_err(|error|error.to_string())?;
            (program,offset)
        };
        let fingerprint=hex::encode(<sha2::Sha256 as sha2::Digest>::digest(file.text.as_bytes()));
        for (name,node) in declaration_nodes(&program) {
            definitions.entry(name.clone()).or_default().push(PublicDeclarationFacts{
                external_name:String::new(),internal_name:name,source_path:path.clone(),
                declaration_offset:offset,source_sha256:fingerprint.clone(),node,
                rational_constructor_unshadowed:false,
            });
        }
    }
    let rational_constructor_unshadowed=!definitions.keys().any(|name|
        ddonirang_lang::stdlib::canonicalize_stdlib_alias(name)=="나눔수" || name=="有理数");
    let mut declarations=Vec::new();
    for export in directives.exports {
        let candidates=definitions.get(&export.internal_name)
            .ok_or("E_PUBLIC_SURFACE_DECLARATION_SUPPLY_REQUIRED: 공개 선언의 실제 원본 공급이 없습니다")?;
        if candidates.len()!=1 {return Err("E_PUBLIC_SURFACE_DECLARATION_AMBIGUOUS: 공개 선언의 실제 소유가 유일하지 않습니다".into());}
        let mut row=candidates[0].clone();
        row.external_name=export.external_name;
        row.rational_constructor_unshadowed=rational_constructor_unshadowed;
        declarations.push(row);
    }
    Ok(SourceOwnedPublicFacts{module:module.id.clone(),project_content_sha256:project.content_sha256.clone(),
        source_paths:module.sources.clone(),declarations})
}

#[cfg(test)]
mod r31_j1_public_owner_tests{
    use super::*;
    const SNAPSHOT:&str="{\"root_path\":\"root.ddn\",\"entry_path\":\"main.ddn\",\"files\":[{\"path\":\"root.ddn\",\"text\":\"#!日本語\\n(版=\\\"v25\\\", モジュール群=列[\\n  (名前=アプリ, 見出し=\\\"main.ddn\\\", 原文=\\\"main.ddn\\\"),\\n  (名前=計算, 見出し=\\\"calc.ddn\\\", 原文=\\\"calc.ddn\\\")\\n], 開始=(モジュール=アプリ, 定義=実行)) 構成.\\n\"},{\"path\":\"calc.ddn\",\"text\":\"#!日本語\\n(F=合計, G=倍) 公開.\\n(金額:有理数~に, 税額:有理数~を) 合計:有理数:関数 := {\\n  合計 <- 金額 + 税額.\\n}.\\n(金額:有理数~を) 倍:有理数:関数 := {\\n  倍 <- 金額 + 金額.\\n}.\\n(金額:有理数~を) 内部:有理数:関数 := {\\n  内部 <- 金額 + 金額.\\n}.\\n\"},{\"path\":\"main.ddn\",\"text\":\"#!日本語\\n(計算=計算) 使用.\\n実行:手続き := {\\n  ((税額=(3,1) 有理数, 金額=(12,1) 有理数) 計算.F) 表示.\\n  ((金額=(4,1) 有理数) 計算.G) 表示.\\n}.\\n\"}]}";
    fn project()->ProjectArtifact{
        let snapshot=serde_json::from_str(SNAPSHOT).unwrap();
        let project=ProjectArtifact::create(ddonirang_lang::ddl::J1_SOURCE_PROFILE.into(),snapshot,String::new()).unwrap();
        crate::authored_project::open(project.save().unwrap().as_bytes()).unwrap()
    }
    #[test]fn r31_j1_actual_public_owner_denominator_and_original_spans(){
        let project=project();let module=project.authored_identity.modules.iter().find(|module|module.name=="計算").unwrap();
        let facts=from_project(&project,module.id.as_str()).unwrap();
        assert_eq!(facts.module,module.id);assert_eq!(facts.declarations.len(),2);
        facts.check_export_name_coverage(&["F","G"]).unwrap();assert!(facts.check_export_name_coverage(&["F"]).is_err());
        assert!(!facts.declarations.iter().any(|row|row.internal_name=="内部"));
        for row in &facts.declarations{
            assert_eq!(row.declaration_offset,0);let file=project.source_snapshot.files.iter().find(|file|file.path==row.source_path).unwrap();
            let DeclarationNode::Callable(seed)=&row.node else{panic!("callable owner expected")};
            assert!(file.text[seed.span.start..seed.span.end].contains(&row.internal_name));
            assert_eq!(seed.params[0].type_ref,ddonirang_lang::TypeRef::Named("나눔수".into()));
        }
    }
    #[test]fn r31_j1_same_spelling_other_owner_and_stale_original_refused(){
        let first=project();let second=project();let one=first.authored_identity.modules.iter().find(|m|m.name=="計算").unwrap();
        let two=second.authored_identity.modules.iter().find(|m|m.name=="計算").unwrap();assert_ne!(one.id,two.id);
        assert!(from_project(&first,two.id.as_str()).is_err());let mut stale=first.clone();
        stale.source_snapshot.files.iter_mut().find(|file|file.path=="calc.ddn").unwrap().text.push(' ');
        assert!(from_project(&stale,one.id.as_str()).unwrap_err().contains("CHECKSUM_STALE"));
    }
}


// Source-only check shared by original inspection and actual registration.
pub(crate) struct PublicCallableScopeError {
    message: &'static str,
    source_span: Option<ddonirang_lang::Span>,
}

impl std::fmt::Display for PublicCallableScopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "E_RESOLVED_SUPPLY: E_DDL_PUBLIC_BODY_SCOPE: {}", self.message)
    }
}

impl PublicCallableScopeError {
    pub(crate) fn source_diagnostic(
        &self,
        declaration: &PublicDeclarationFacts,
        source_text: &str,
    ) -> Option<crate::preprocess::ExecutionDiagnostic> {
        let span = self.source_span?;
        let offset = declaration.declaration_offset();
        let start = offset.checked_add(span.start)?;
        let end = offset.checked_add(span.end)?;
        if start >= end || source_text.get(start..end).is_none() {
            return None;
        }
        Some(crate::preprocess::ExecutionDiagnostic {
            code: "E_DDL_PUBLIC_BODY_SCOPE",
            message: self.to_string(),
            source_path: declaration.source_path().into(),
            span: Some((start, end)),
            phase: "source_prepare",
            cause: None,
            offending_field: None,
            expected: None,
            actual: None,
        })
    }
}

fn public_callable_scope_error(message: &'static str) -> PublicCallableScopeError {
    PublicCallableScopeError { message, source_span: None }
}
pub(crate) fn check_ddl_callable_shape(declaration:&PublicDeclarationFacts)
    ->Result<(&ddonirang_lang::SeedDef,String),PublicCallableScopeError>{
    let DeclarationNode::Callable(seed)=declaration.node() else {
        return Err(public_callable_scope_error("공개 비함수의 실제 공급이 필요합니다"));
    };
    let rational = ddonirang_lang::TypeRef::Named("나눔수".into());
    let unsupported_pin = seed.params.iter().find(|pin|
        pin.optional || pin.default_value.is_some() || pin.type_ref != rational);
    if seed.seed_kind != ddonirang_lang::SeedKind::Semssi || seed.params.is_empty()
        || unsupported_pin.is_some() || seed.result_type.as_ref() != Some(&rational) {
        return Err(PublicCallableScopeError {
            message: "현재 전체 공개 연결은 실제 필수 나눔수 셈씨만 지원합니다",
            source_span: Some(unsupported_pin.map_or(seed.span, |pin| pin.span)),
        });
    }
    let body = seed.body.as_ref().ok_or(PublicCallableScopeError {
        message: "실제 본문이 없습니다",
        source_span: Some(seed.span),
    })?;
    let value=match body.stmts.as_slice() {
        [Stmt::Mutate{target,value,deferred:false,..}] if matches!(&target.kind,ddonirang_lang::ExprKind::Var(name) if name==&seed.canonical_name)=>value,
        [Stmt::Return{value,..}]=>value,
        _ => return Err(PublicCallableScopeError {
            message: "실제 단일 결과 본문의 effect 검증 공급이 필요합니다",
            source_span: Some(body.span),
        }),
    };
    // This bounded audit is evidence about actual AST dependencies,
    // never the claim that every Semssi or static type is PURE.
    fn rational_literal(args:&[ddonirang_lang::ArgBinding])->Option<String> {
        let [numerator,denominator]=args else {return None;};
        if args.iter().any(|arg|arg.josa.is_some() || arg.resolved_pin.is_some()
            || arg.binding_reason!=ddonirang_lang::BindingReason::Positional) {
            return None;
        }
        let integer = |expr: &ddonirang_lang::Expr| {
            use ddonirang_lang::{ExprKind, Literal};
            // The parser lowers a unary minus to a subtraction from a
            // synthetic zero. Authored subtraction has an authored lhs.
            let (literal, negative) = match &expr.kind {
                ExprKind::Infix { left, op, right }
                    if op == "-" && left.numeric_literal_source.is_none()
                        && matches!(&left.kind, ExprKind::Literal(Literal::Fixed64(zero))
                            if *zero == ddonirang_core::Fixed64::ZERO) =>
                    (right.as_ref(), true),
                _ => (expr, false),
            };
            let value = match &literal.kind {
                ExprKind::Literal(Literal::Int(value)) => *value,
                ExprKind::Literal(Literal::Fixed64(value)) => {
                    if *value == ddonirang_core::Fixed64::MIN && literal.numeric_literal_source.is_none() {
                        // The parser's signed minimum literal takes this exact path.
                        value.int_part()
                    } else {
                        let token = literal.numeric_literal_source.as_ref()?;
                        if token.is_decimal() || value.frac_part() != 0 {
                            return None;
                        }
                        let integer = token.raw().parse::<i64>().ok()?;
                        if integer != value.int_part() {
                            return None;
                        }
                        integer
                    }
                }
                _ => return None,
            };
            let signed = if negative { value.checked_neg()? } else { value };
            signed.checked_mul(ddonirang_core::Fixed64::ONE_RAW)?;
            Some(signed)
        };
        let n = integer(&numerator.expr)?;
        let d = integer(&denominator.expr)?;
        if d == 0 {
            return None;
        }
        let value = num_rational::BigRational::new(num_bigint::BigInt::from(n), num_bigint::BigInt::from(d));
        Some(format!("rational({},{})",value.numer(),value.denom()))
    }
    fn audit(expr:&ddonirang_lang::Expr,seed:&ddonirang_lang::SeedDef,
        rational_constructor_unshadowed:bool,depth:usize)->Result<String,PublicCallableScopeError> {
        if depth > 32 {
            return Err(PublicCallableScopeError {
                message: "본문 깊이 한도",
                source_span: Some(expr.span),
            });
        }
        match &expr.kind {
            ddonirang_lang::ExprKind::Var(name) if seed.params.iter().any(|pin|pin.pin_name==*name)=>Ok(format!("pin:{}",name)),
            ddonirang_lang::ExprKind::Infix{left,op,right} if op=="+" || op=="*"=>{
                let operation=if op=="+" {"add"} else {"mul"};
                Ok(format!("{}({},{})",operation,
                    audit(left,seed,rational_constructor_unshadowed,depth+1)?,
                    audit(right,seed,rational_constructor_unshadowed,depth+1)?))
            },
            ddonirang_lang::ExprKind::Call{func,args}
                if func=="나눔수" && rational_constructor_unshadowed
                    && !seed.params.iter().any(|pin|
                        ddonirang_lang::stdlib::canonicalize_stdlib_alias(&pin.pin_name)=="나눔수"
                            || pin.pin_name=="有理数") =>
                rational_literal(args).ok_or(PublicCallableScopeError {
                    message:"정확한 나눔수 상수에는 표현 가능한 정수 분자와 0이 아닌 정수 분모가 필요합니다",
                    source_span:Some(expr.span),
                }),
            _ => Err(PublicCallableScopeError {
                message: "미공급 호출·연산·세계 의존을 공개하지 않습니다",
                source_span: Some(expr.span),
            }),
        }
    }
    Ok((seed,audit(value,seed,declaration.rational_constructor_unshadowed,0)?))

}
