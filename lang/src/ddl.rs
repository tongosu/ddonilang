//! DDL Wave 1 frontdoor.
//!
//! This module accepts only the Owner-bound two-language, immutable-value
//! projection.  It is intentionally not a second general DDN parser and it
//! never creates an executable body.

use crate::ast::{
    DdlDocument, DdlExtension, ImmutablePublicValueProjection, LanguageSection, OmittedPayload,
    Span,
};
use crate::lexer::{Lexer, TokenKind};

const KOREAN: DdlRegistryEntry = DdlRegistryEntry {
    header: "#!한국어",
    profile_identity: "language-profile:ko",
    localized_name: "원주율",
    localized_type: "셈수",
    definition_identity: "definition:math.pi",
    type_identity: "type:number",
};

const JAPANESE: DdlRegistryEntry = DdlRegistryEntry {
    header: "#!日本語",
    profile_identity: "language-profile:ja",
    localized_name: "円周率",
    localized_type: "数値",
    definition_identity: "definition:math.pi",
    type_identity: "type:number",
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DdlRegistryEntry {
    pub header: &'static str,
    pub profile_identity: &'static str,
    pub localized_name: &'static str,
    pub localized_type: &'static str,
    pub definition_identity: &'static str,
    pub type_identity: &'static str,
}

const REGISTRY: [DdlRegistryEntry; 2] = [KOREAN, JAPANESE];

/// Internal, version-bound registry context for the DDL Wave 1 parser.
///
/// This is deliberately a Rust-side product context, not a public DDL wire
/// or dynamic language-profile registry.  It lets the parser validate that
/// translated entries preserve their common definition and type identities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DdlWave1Context {
    entries: Vec<DdlRegistryEntry>,
}

impl DdlWave1Context {
    pub fn standard() -> Self {
        Self {
            entries: REGISTRY.to_vec(),
        }
    }

    pub fn from_entries(entries: Vec<DdlRegistryEntry>) -> Self {
        Self { entries }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DdlError {
    pub code: &'static str,
    pub span: Span,
    pub message: String,
}

impl std::fmt::Display for DdlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for DdlError {}

/// Parses the exact Wave 1 DDL surface.  `file_path` has extension-admission
/// authority only; it is never stored as a semantic identity.
pub fn parse_ddl_wave1(source: &str, file_path: &str) -> Result<DdlDocument, DdlError> {
    parse_ddl_wave1_with_context(source, file_path, &DdlWave1Context::standard())
}

/// Parses a Wave 1 document against an explicit internal registry context.
/// Callers must supply the fixed versioned context; source text never creates
/// profile or semantic identities dynamically.
pub fn parse_ddl_wave1_with_context(
    source: &str,
    file_path: &str,
    context: &DdlWave1Context,
) -> Result<DdlDocument, DdlError> {
    let extension = parse_extension(file_path)?;
    let headers = scan_headers(source)?;
    if headers.len() != 2 {
        return Err(error(
            "E_DDL_SECTION_COUNT",
            Span::new(0, source.len()),
            "DDL Wave 1은 정확히 두 LanguageSection이 필요합니다",
        ));
    }

    let mut sections = Vec::with_capacity(2);
    for (index, header) in headers.iter().enumerate() {
        let content_end = headers
            .get(index + 1)
            .map_or(source.len(), |next| next.start);
        let entry = registry_entry(&context.entries, header.text).ok_or_else(|| {
            error(
                if matches!(header.text, "#!qu" | "#!Quechua") {
                    "E_DDL_GENERIC_QUECHUA_FORBIDDEN"
                } else {
                    "E_DDL_PROFILE_UNKNOWN"
                },
                Span::new(header.start, header.end),
                "등록되지 않았거나 일반화된 LanguageProfile header입니다",
            )
        })?;
        if sections.iter().any(|section: &LanguageSection| {
            section.language_profile_identity == entry.profile_identity
        }) {
            return Err(error(
                "E_DDL_PROFILE_DUPLICATE",
                Span::new(header.start, header.end),
                "같은 LanguageProfileIdentity section을 두 번 쓸 수 없습니다",
            ));
        }
        let projection = parse_projection(&source[header.end..content_end], header.end, entry)?;
        sections.push(LanguageSection {
            language_profile_identity: entry.profile_identity.to_owned(),
            header_span: Span::new(header.start, header.end),
            content_span: Span::new(header.end, content_end),
            projection,
        });
    }

    let first = &sections[0].projection;
    let second = &sections[1].projection;
    if first.definition_identity != second.definition_identity {
        return Err(error(
            "E_DDL_DEFINITION_IDENTITY_MISMATCH",
            second.declaration_span,
            "두 localized projection은 같은 DefinitionIdentity를 가리켜야 합니다",
        ));
    }
    if first.type_identity != second.type_identity {
        return Err(error(
            "E_DDL_TYPE_IDENTITY_MISMATCH",
            second.declaration_span,
            "두 localized projection은 같은 TypeIdentity를 가리켜야 합니다",
        ));
    }

    Ok(DdlDocument {
        extension,
        source_span: Span::new(0, source.len()),
        sections,
    })
}

fn parse_extension(file_path: &str) -> Result<DdlExtension, DdlError> {
    if file_path.ends_with(".ddl") {
        Ok(DdlExtension::Ddl)
    } else if file_path.ends_with(".옮김") {
        Ok(DdlExtension::Olgim)
    } else {
        Err(error(
            "E_DDL_EXTENSION_UNSUPPORTED",
            Span::new(0, 0),
            "DDL Wave 1은 .ddl 또는 .옮김 extension만 허용합니다",
        ))
    }
}

fn registry_entry<'a>(
    entries: &'a [DdlRegistryEntry],
    header: &str,
) -> Option<&'a DdlRegistryEntry> {
    entries.iter().find(|entry| entry.header == header)
}

#[derive(Debug, Clone, Copy)]
struct Header<'a> {
    start: usize,
    end: usize,
    text: &'a str,
}

/// Finds only raw top-level line-start headers.  Quoted, line-commented,
/// block-commented, and nested `#!` bytes are ordinary section content.
fn scan_headers(source: &str) -> Result<Vec<Header<'_>>, DdlError> {
    let mut headers = Vec::new();
    let mut offset = 0;
    let mut depth = 0_i32;
    let mut in_string = false;
    let mut escape = false;
    let mut in_block_comment = false;

    for line_with_end in source.split_inclusive('\n') {
        let line = line_with_end.strip_suffix('\n').unwrap_or(line_with_end);
        let line = line.strip_suffix('\r').unwrap_or(line);
        let eligible = depth == 0 && !in_string && !in_block_comment;
        if eligible && line.starts_with("#!") {
            if offset != 0 && headers.is_empty() {
                return Err(error(
                    "E_DDL_FIRST_HEADER_NOT_BYTE_ZERO",
                    Span::new(offset, offset + line.len()),
                    "첫 DDL header는 raw byte zero에서 시작해야 합니다",
                ));
            }
            // Exact registry matching also rejects whitespace and trailing text.
            headers.push(Header {
                start: offset,
                end: offset + line.len(),
                text: line,
            });
        }
        advance_lexical_depth(
            line,
            &mut depth,
            &mut in_string,
            &mut escape,
            &mut in_block_comment,
        );
        offset += line_with_end.len();
    }

    if !source.is_empty() && !source.ends_with('\n') {
        // `split_inclusive` already included the final unterminated line.
    }
    if in_string || in_block_comment || depth != 0 {
        return Err(error(
            "E_DDL_SOURCE_UNBALANCED",
            Span::new(0, source.len()),
            "DDL section boundary를 판정할 수 없는 미닫힌 문자열·주석·블록이 있습니다",
        ));
    }
    if headers.first().map(|header| header.start) != Some(0) {
        return Err(error(
            "E_DDL_FIRST_HEADER_REQUIRED",
            Span::new(0, source.len().min(1)),
            "DDL 문서는 byte zero의 exact LanguageProfile header로 시작해야 합니다",
        ));
    }
    Ok(headers)
}

fn advance_lexical_depth(
    line: &str,
    depth: &mut i32,
    in_string: &mut bool,
    escape: &mut bool,
    in_block_comment: &mut bool,
) {
    let bytes = line.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if *in_block_comment {
            if bytes[index..].starts_with(b"*/") {
                *in_block_comment = false;
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }
        if *in_string {
            if *escape {
                *escape = false;
            } else if bytes[index] == b'\\' {
                *escape = true;
            } else if bytes[index] == b'"' {
                *in_string = false;
            }
            index += 1;
            continue;
        }
        if bytes[index..].starts_with(b"//") {
            break;
        }
        if bytes[index..].starts_with(b"/*") {
            *in_block_comment = true;
            index += 2;
            continue;
        }
        match bytes[index] {
            b'"' => *in_string = true,
            b'{' => *depth += 1,
            b'}' => *depth -= 1,
            _ => {}
        }
        index += 1;
    }
}

fn parse_projection(
    content: &str,
    content_start: usize,
    entry: &DdlRegistryEntry,
) -> Result<ImmutablePublicValueProjection, DdlError> {
    let declaration = content.trim();
    if declaration.is_empty() {
        return Err(error(
            "E_DDL_PROJECTION_MISSING",
            Span::new(content_start, content_start + content.len()),
            "각 LanguageSection에는 immutable public value projection 하나가 필요합니다",
        ));
    }
    if content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count()
        != 1
    {
        return Err(error(
            "E_DDL_PROJECTION_COUNT",
            Span::new(content_start, content_start + content.len()),
            "Wave 1 section에는 projection 하나만 허용합니다",
        ));
    }
    let leading = content.find(declaration).unwrap_or(0);
    let declaration_span = Span::new(
        content_start + leading,
        content_start + leading + declaration.len(),
    );
    let exact = format!(
        "{}:{} :={{...}}.",
        entry.localized_name, entry.localized_type
    );
    if declaration != exact {
        return Err(error(
            "E_DDL_OMITTED_VALUE_SHAPE",
            declaration_span,
            "DDL-only immutable value 표기는 exact `이름:타입 :={...}.`이어야 합니다",
        ));
    }
    validate_declaration_tokens(declaration, declaration_span, entry)?;
    Ok(ImmutablePublicValueProjection {
        definition_identity: entry.definition_identity.to_owned(),
        type_identity: entry.type_identity.to_owned(),
        localized_name: entry.localized_name.to_owned(),
        localized_type: entry.localized_type.to_owned(),
        declaration_span,
        omitted_payload: OmittedPayload::ValueInitializer,
    })
}

/// Reuses the shared lexer for the Korean declaration shell.  The existing
/// lexer has no Japanese identifier admission, so the Owner-bound Japanese
/// Wave 1 surface is accepted only through its exact registry literal; this
/// is deliberately not generalized parsing or a fallback language mode.
fn validate_declaration_tokens(
    source: &str,
    span: Span,
    entry: &DdlRegistryEntry,
) -> Result<(), DdlError> {
    if entry.profile_identity == JAPANESE.profile_identity {
        return Ok(());
    }
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|lex| error("E_DDL_DECLARATION_LEX", span, lex.message))?;
    let expected = [
        matches!(&tokens[0].kind, TokenKind::Ident(_)),
        matches!(tokens[1].kind, TokenKind::Colon),
        matches!(&tokens[2].kind, TokenKind::Ident(_)),
        matches!(tokens[3].kind, TokenKind::Define),
        matches!(tokens[4].kind, TokenKind::LBrace),
        matches!(tokens[5].kind, TokenKind::DotDot),
        matches!(tokens[6].kind, TokenKind::Dot),
        matches!(tokens[7].kind, TokenKind::RBrace),
        matches!(tokens[8].kind, TokenKind::Dot),
        matches!(tokens[9].kind, TokenKind::Eof),
    ];
    if tokens.len() != expected.len() || expected.iter().any(|valid| !valid) {
        return Err(error(
            "E_DDL_OMITTED_VALUE_TOKENS",
            span,
            "DDL omission은 exact ASCII `:={...}` token sequence여야 합니다",
        ));
    }
    Ok(())
}

fn error(code: &'static str, span: Span, message: impl Into<String>) -> DdlError {
    DdlError {
        code,
        span,
        message: message.into(),
    }
}

/// Owner-approved first-function vocabulary, registration revision 1.
///
/// This is a finite declaration-side registry, not an executable source
/// profile, a Unicode policy, or an extension of immutable-value Wave 1.
/// In particular it does not admit a Japanese source header to the runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FirstCallableJaRegistration {
    pub language_profile_identity: &'static str,
    pub registration_revision: u32,
    pub callable_name: &'static str,
    pub pin_names: [&'static str; 2],
    pub pin_particles: [&'static str; 2],
    pub rational_type_name: &'static str,
    pub callable_kind_name: &'static str,
}

pub const FIRST_CALLABLE_JA_REGISTRATION: FirstCallableJaRegistration = FirstCallableJaRegistration {
    language_profile_identity: JAPANESE.profile_identity,
    registration_revision: 1,
    callable_name: "合計する",
    pin_names: ["金額", "税額"],
    pin_particles: ["に", "を"],
    rational_type_name: "有理数",
    callable_kind_name: "関数",
};

/// Exact input manifest for the approved limited surface. These names are
/// compilation-local; neither this descriptor nor Wave1's spelling issues a
/// durable LanguageProfileIdentity/version or exported-module DDL authority.
#[derive(Debug,Clone,Copy)]
pub struct FirstCallableProfileManifest {
    pub canonical_name: &'static str,
    pub canonical_header: &'static str,
    pub explicit_aliases: &'static [&'static str],
    pub registration_revision: u32,
    pub persistent_identity: Option<&'static str>,
    pub exact_version: Option<&'static str>,
    pub content: &'static str,
}
pub const FIRST_CALLABLE_PROFILE_MANIFEST:FirstCallableProfileManifest=FirstCallableProfileManifest{
    canonical_name:"日本語",canonical_header:"#!日本語",explicit_aliases:&[],registration_revision:1,
    persistent_identity:None,exact_version:None,
    content:"関数/有理数/合計する/金額/税額/に/を;required-pins=2;UTF8-exact;no-fallback;ASCII-syntax-numbers;closed-public-surface",
};


// Closed product registry entry assigned once by its registry owner. Its
// opaque instance value is not derived from source/header/path/content hash.
// Wave1's compilation-local labels remain a separate non-durable API.
struct FirstCallableProfileRecord {
    identity:&'static str,version:&'static str,content:&'static str,sha256:&'static str,
}
const FIRST_ISSUED_PROFILE:FirstCallableProfileRecord=FirstCallableProfileRecord {
    identity:"language-profile:02ac4b11c9124a118777eb0a07dd16e4",version:"1",
    content:"{\"registration_revision\":1,\"canonical_name\":\"日本語\",\"canonical_header\":\"#!日本語\",\"explicit_aliases\":[],\"callable\":\"合計する\",\"pins\":[\"金額\",\"税額\"],\"particles\":[\"に\",\"を\"],\"type\":\"有理数\",\"kind\":\"関数\",\"required_pins\":2,\"unicode\":\"UTF8-exact\",\"syntax\":\"ASCII\",\"fallback\":false}",sha256:"60e45a1121b6dab1bfc6b44c0a391a3fbf4986d382aa454cba48a33a40012239",
};
/// Only the registry owner creates this capability; no Deserialize/flag or
/// caller-selected string can manufacture successful registry admission.
#[derive(Debug,Clone)]
pub struct CheckedFirstCallableProfile { row:&'static FirstCallableProfileRecord }
impl std::fmt::Debug for FirstCallableProfileRecord {
 fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.debug_struct("FirstCallableProfileRecord").field("identity",&self.identity).field("version",&self.version).finish()}
}
impl CheckedFirstCallableProfile {
 pub fn identity(&self)->&'static str{self.row.identity}
 pub fn version(&self)->&'static str{self.row.version}
 pub fn content(&self)->&'static [u8]{self.row.content.as_bytes()}
 pub fn content_sha256(&self)->&'static str{self.row.sha256}
 pub fn registration_revision(&self)->u32{FIRST_CALLABLE_JA_REGISTRATION.registration_revision}
 pub fn canonical_name(&self)->&'static str{FIRST_CALLABLE_PROFILE_MANIFEST.canonical_name}
 pub fn canonical_header(&self)->&'static str{FIRST_CALLABLE_PROFILE_MANIFEST.canonical_header}
 pub fn check_name(&self,name:&str)->bool{name==self.canonical_name()||FIRST_CALLABLE_PROFILE_MANIFEST.explicit_aliases.contains(&name)}
}
struct FirstCallableRegistryOwner;
impl FirstCallableRegistryOwner {
 fn issue()->Result<CheckedFirstCallableProfile,DdlError>{
  let record=&FIRST_ISSUED_PROFILE;let r=FIRST_CALLABLE_JA_REGISTRATION;
  let expected=serde_json::json!({"registration_revision":r.registration_revision,"canonical_name":FIRST_CALLABLE_PROFILE_MANIFEST.canonical_name,
   "canonical_header":FIRST_CALLABLE_PROFILE_MANIFEST.canonical_header,"explicit_aliases":FIRST_CALLABLE_PROFILE_MANIFEST.explicit_aliases,
   "callable":r.callable_name,"pins":r.pin_names,"particles":r.pin_particles,"type":r.rational_type_name,"kind":r.callable_kind_name,
   "required_pins":2,"unicode":"UTF8-exact","syntax":"ASCII","fallback":false});
  let actual:serde_json::Value=serde_json::from_str(record.content).map_err(|_|error("E_DDL_PROFILE_REGISTRY_RECORD",Span::new(0,0),"말씨 등록 자료를 읽을 수 없습니다"))?;
  if actual!=expected || record.identity==JAPANESE.profile_identity || record.version.is_empty() || record.sha256.len()!=64 {
   return Err(error("E_DDL_PROFILE_REGISTRY_RECORD",Span::new(0,0),"닫힌 등록 임자의 정확한 표면·판본과 다릅니다"));
  }
  Ok(CheckedFirstCallableProfile{row:record})
 }
}
pub fn issued_first_callable_profile()->Result<CheckedFirstCallableProfile,DdlError>{FirstCallableRegistryOwner::issue()}
pub fn lookup_first_callable_profile(identity:&str,version:&str,content_sha256:&str,revision:u32)->Result<CheckedFirstCallableProfile,DdlError>{
 let profile=FirstCallableRegistryOwner::issue()?;
 if identity!=profile.identity() || version!=profile.version() || content_sha256!=profile.content_sha256() || revision!=profile.registration_revision(){
  return Err(error("E_DDL_PROFILE_EXACT_REQUIRED",Span::new(0,0),"등록된 말씨 정체·정확 판본·내용·등록 revision이 필요합니다"));
 }
 Ok(profile)
}
pub fn resolve_first_callable_profile_name(name:&str,version:&str)->Result<CheckedFirstCallableProfile,DdlError>{
 let profile=FirstCallableRegistryOwner::issue()?;
 if !profile.check_name(name)||version!=profile.version(){return Err(error("E_DDL_PROFILE_NAME_OR_VERSION",Span::new(0,0),"미등록 말씨 이름 또는 정확 판본입니다"));}
 Ok(profile)
}

/// A declaration binding into one actual common-parser result.
///
/// The references retain that result's definition/pin/type ownership. No
/// persistent DefinitionIdentity/ParamPinIdentity/TypeIdentity is issued here.
/// A tool consumer must still obtain admission, module ownership and durable
/// identity from their existing producers before activating or executing DDL.
/// This type cannot be serialized as a success receipt or used as one.
#[derive(Debug)]
pub struct FirstCallableJaDeclarationBinding<'a> {
    original_program: &'a crate::ast::CanonProgram,
    definition: &'a crate::ast::SeedDef,
}

impl<'a> FirstCallableJaDeclarationBinding<'a> {
    pub fn original_program(&self) -> &'a crate::ast::CanonProgram {
        self.original_program
    }

    /// Lookup stays within the explicitly selected original module context.
    /// No Korean/English spelling, trimming or Unicode normalization fallback.
    pub fn callable(&self, surface: &str) -> Option<&'a crate::ast::SeedDef> {
        (surface == FIRST_CALLABLE_JA_REGISTRATION.callable_name).then_some(self.definition)
    }

    pub fn pin(&self, surface: &str) -> Option<&'a crate::ast::ParamPin> {
        FIRST_CALLABLE_JA_REGISTRATION.pin_names.iter()
            .position(|registered| *registered == surface)
            .map(|index| &self.definition.params[index])
    }

    /// This exposes the original declared TypeRef, not a resolved TypeIdentity.
    pub fn result_type(&self, surface: &str) -> Option<&'a crate::ast::TypeRef> {
        if surface != FIRST_CALLABLE_JA_REGISTRATION.rational_type_name {
            return None;
        }
        self.definition.result_type.as_ref()
    }

    /// A declaration of 셈씨 is not by itself a proof of effect admission.
    pub fn callable_kind(&self, surface: &str) -> Option<&'a crate::ast::SeedKind> {
        (surface == FIRST_CALLABLE_JA_REGISTRATION.callable_kind_name)
            .then_some(&self.definition.seed_kind)
    }
}

/// Bind the approved Japanese declaration vocabulary to the complete public
/// declaration of the selected Korean first-function module. Parsing is owned
/// by the existing module/common parser; this function parses no source text,
/// copies no body, and creates no name-based semantic identity.
///
/// The registry revision is intentionally distinct from a source-profile
/// version. Runtime admission must not infer one from the other.
pub fn bind_first_callable_ja_declaration<'a>(
    public: &'a crate::frontdoor::ModuleDirectiveFrontdoorResult,
    language_profile_identity: &str,
    registration_revision: u32,
) -> Result<FirstCallableJaDeclarationBinding<'a>, DdlError> {
    use crate::ast::{SeedKind, SourceEpoch, TopLevelItem, TypeRef};

    let span = Span::new(0, public.directives.declaration_source.len());
    let registry = FIRST_CALLABLE_JA_REGISTRATION;
    if language_profile_identity != registry.language_profile_identity
        || registration_revision != registry.registration_revision
    {
        return Err(error("E_DDL_FIRST_REGISTRATION_UNSUPPORTED", span,
            "등록된 일본어 첫 함수의 말씨·등록 판본이 필요합니다"));
    }
    let program = public.declaration_program.as_ref().ok_or_else(||
        error("E_DDL_FIRST_ORIGINAL_REQUIRED", span, "공통 파서의 실제 원본 선언이 필요합니다"))?;
    if program.source_epoch != SourceEpoch::V25
        || program.origin.source != public.directives.declaration_source
    {
        return Err(error("E_DDL_FIRST_ORIGINAL_CONTEXT_STALE", span,
            "공개 선언과 분석 원문 문맥이 일치하지 않습니다"));
    }
    let exports = public.directives.exports.as_ref().ok_or_else(||
        error("E_DDL_FIRST_PUBLIC_SURFACE_REQUIRED", span, "원본의 명시적인 공개 선언이 필요합니다"))?;
    if exports.entries.len() != 1
        || exports.entries[0].local_name != "총액합하다"
        || exports.entries[0].target != "총액합하다"
        || public.directives.imports.is_some()
        || program.items.len() != 1
        || !program.root_stmts.is_empty()
    {
        return Err(error("E_DDL_FIRST_PUBLIC_SURFACE_MISMATCH", span,
            "첫 함수 원본의 전체 공개 선언·의존 범위를 확인해야 합니다"));
    }
    let TopLevelItem::SeedDef(definition) = &program.items[0];
    if definition.canonical_name != exports.entries[0].target
        || definition.seed_kind != SeedKind::Semssi
        || definition.body.is_none()
        || !definition.type_params.is_empty()
        || definition.proof_header.is_some()
        || !definition.modifiers.is_empty()
        || definition.alternate_forms.as_slice() != ["총액합하기"]
        || definition.result_type != Some(TypeRef::Named("나눔수".into()))
    {
        return Err(error("E_DDL_FIRST_CALLABLE_DECLARATION_MISMATCH", definition.span,
            "본문이 있는 원본 셈씨·결과 나눔수·등록 호출 표면이 필요합니다"));
    }
    if definition.params.len() != 2 {
        return Err(error("E_DDL_FIRST_PIN_COUNT", definition.span,
            "첫 함수는 기본값 없는 필수 인수 두 개가 필요합니다"));
    }
    for (index, original_name) in ["금액", "세액"].iter().enumerate() {
        let pin = &definition.params[index];
        let roles: std::collections::BTreeSet<_> = pin.josa_list.iter().map(String::as_str).collect();
        let expected_roles: std::collections::BTreeSet<_> = if index == 0 {
            ["에"].into_iter().collect()
        } else {
            ["을", "를"].into_iter().collect()
        };
        if pin.pin_name != *original_name || pin.type_ref != TypeRef::Named("나눔수".into())
            || pin.optional || pin.default_value.is_some() || roles != expected_roles
            || roles.len() != pin.josa_list.len()
        {
            return Err(error("E_DDL_FIRST_PIN_DECLARATION_MISMATCH", pin.span,
                "원본 핀의 의미 순서·타입·필수 여부·조사 등록이 일치하지 않습니다"));
        }
    }
    Ok(FirstCallableJaDeclarationBinding { original_program: program, definition })
}

#[cfg(test)]
mod first_callable_registration_tests {
    use super::*;
    use crate::{ModuleDirectiveContext, ParseMode, TopLevelItem};

    const ORIGINAL: &str = "(총액합하다=총액합하다) 드러냄.\n\
        (금액:나눔수~에, 세액:나눔수~을~를)\n\
        총액합하다{총액합하기}:나눔수:셈씨 := { 총액합하다 <- 금액 + 세액. }.\n";

    fn parse(source: &str) -> crate::frontdoor::ModuleDirectiveFrontdoorResult {
        crate::frontdoor::parse_module_directive_frontdoor_with_mode(
            source, "세금.ddn", ParseMode::Strict,
            &ModuleDirectiveContext::default().with_public_definitions(["총액합하다", "다른함수"]),
        ).unwrap()
    }

    fn bind(public: &crate::frontdoor::ModuleDirectiveFrontdoorResult)
        -> Result<FirstCallableJaDeclarationBinding<'_>, DdlError>
    {
        bind_first_callable_ja_declaration(public, "language-profile:ja", 1)
    }

    #[test]
    fn first_callable_registration_keeps_actual_original_definition_pin_and_type_references() {
        let public = parse(ORIGINAL);
        let binding = bind(&public).unwrap();
        let program = public.declaration_program.as_ref().unwrap();
        let TopLevelItem::SeedDef(definition) = &program.items[0];
        assert!(std::ptr::eq(binding.original_program(), program));
        assert!(std::ptr::eq(binding.callable("合計する").unwrap(), definition));
        assert!(std::ptr::eq(binding.pin("金額").unwrap(), &definition.params[0]));
        assert!(std::ptr::eq(binding.pin("税額").unwrap(), &definition.params[1]));
        assert!(std::ptr::eq(binding.result_type("有理数").unwrap(), definition.result_type.as_ref().unwrap()));
        assert!(std::ptr::eq(binding.callable_kind("関数").unwrap(), &definition.seed_kind));
        assert_eq!(FIRST_CALLABLE_JA_REGISTRATION.pin_particles, ["に", "を"]);
    }

    #[test]
    fn first_callable_registration_has_no_spelling_locale_or_unicode_fallback() {
        let public = parse(ORIGINAL);
        let binding = bind(&public).unwrap();
        for name in ["총액합하다", "total", "合計する ", " 合計する", "合計する\u{200b}"] {
            assert!(binding.callable(name).is_none(), "{name:?}");
        }
        for name in ["금액", "金額\u{fe0f}", "税額 ", "missing"] {
            assert!(binding.pin(name).is_none(), "{name:?}");
        }
        assert!(binding.result_type("나눔수").is_none());
        assert!(binding.callable_kind("셈씨").is_none());
    }

    #[test]
    fn first_callable_registration_same_spelling_never_merges_original_contexts() {
        let first = parse(ORIGINAL);
        let second = parse(ORIGINAL);
        let first = bind(&first).unwrap();
        let second = bind(&second).unwrap();
        assert!(!std::ptr::eq(first.original_program(), second.original_program()));
        assert!(!std::ptr::eq(first.callable("合計する").unwrap(), second.callable("合計する").unwrap()));
        assert!(!std::ptr::eq(first.pin("金額").unwrap(), second.pin("金額").unwrap()));
    }

    #[test]
    fn first_callable_registration_rejects_unknown_profile_and_revision() {
        let public = parse(ORIGINAL);
        for (profile, revision) in [("language-profile:ko", 1), ("language-profile:ja ", 1),
            ("language-profile:ja", 0), ("language-profile:ja", 2)] {
            assert_eq!(bind_first_callable_ja_declaration(&public, profile, revision).unwrap_err().code,
                "E_DDL_FIRST_REGISTRATION_UNSUPPORTED");
        }
    }

    #[test]
    fn first_callable_registration_rejects_missing_or_partial_public_surface() {
        let mut public = parse(ORIGINAL);
        public.directives.exports = None;
        assert_eq!(bind(&public).unwrap_err().code, "E_DDL_FIRST_PUBLIC_SURFACE_REQUIRED");
        let public = parse(&ORIGINAL.replace("(총액합하다=총액합하다)",
            "(총액합하다=총액합하다, 다른함수=총액합하다)"));
        assert_eq!(bind(&public).unwrap_err().code, "E_DDL_FIRST_PUBLIC_SURFACE_MISMATCH");
    }

    #[test]
    fn first_callable_registration_rejects_pin_count_type_and_meaning_order_drift() {
        for (source, code) in [
            (ORIGINAL.replace(", 세액:나눔수~을~를", ""), "E_DDL_FIRST_PIN_COUNT"),
            (ORIGINAL.replace("금액:나눔수", "금액:글"), "E_DDL_FIRST_PIN_DECLARATION_MISMATCH"),
            (ORIGINAL.replace("금액:나눔수~에, 세액:나눔수~을~를", "세액:나눔수~을~를, 금액:나눔수~에"),
                "E_DDL_FIRST_PIN_DECLARATION_MISMATCH"),
        ] {
            assert_eq!(bind(&parse(&source)).unwrap_err().code, code);
        }
        let mut public = parse(ORIGINAL);
        let TopLevelItem::SeedDef(definition) = &mut public.declaration_program.as_mut().unwrap().items[0];
        definition.params[0].optional = true;
        assert_eq!(bind(&public).unwrap_err().code, "E_DDL_FIRST_PIN_DECLARATION_MISMATCH");
    }

    #[test]
    fn first_callable_registration_rejects_stale_source_and_omitted_original_body() {
        let mut public = parse(ORIGINAL);
        public.directives.declaration_source.push(' ');
        assert_eq!(bind(&public).unwrap_err().code, "E_DDL_FIRST_ORIGINAL_CONTEXT_STALE");
        let mut public = parse(ORIGINAL);
        let TopLevelItem::SeedDef(definition) = &mut public.declaration_program.as_mut().unwrap().items[0];
        definition.body = None;
        assert_eq!(bind(&public).unwrap_err().code, "E_DDL_FIRST_CALLABLE_DECLARATION_MISMATCH");
    }
}
