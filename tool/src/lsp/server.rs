// 또니랑 teul-ide LSP v0.1
// SSOT TOOLCHAIN v17.0.6 §T3 준수

use ddonirang_core::is_known_unit as core_is_known_unit;
use ddonirang_lang::*; // lang 크레이트에서 AST 등을 가져옴
/// use ddonirang_core::Fixed64; // core 크레이트에서 Fixed64를 가져옴
use std::collections::{HashMap, HashSet};

type PinId = String;

// ============================================================================
// LSP 기본 구조
// ============================================================================

pub struct LspServer {
    pub documents: HashMap<String, Document>,
    pub symbol_table: SymbolTable,
}

pub struct Document {
    pub uri: String,
    pub version: i32,
    pub content: String,
    pub ast: Option<CanonProgram>,
    pub diagnostics: Vec<Diagnostic>,
}

pub struct SymbolTable {
    pub components: HashMap<String, ComponentInfo>,
    pub functions: HashMap<String, FunctionInfo>,
    pub enums: HashMap<String, EnumInfo>,
}

#[derive(Debug, Clone)]
pub struct ComponentInfo {
    pub name: String,
    pub fields: Vec<FieldInfo>,
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub type_ref: TypeRef,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub params: Vec<ParamPin>,
    pub return_type: TypeRef,
}

#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub name: String,
    pub variants: Vec<String>,
}

// ============================================================================
// 조사 핀 시각화 (Semantic Pinning) - §T3.1
// ============================================================================

#[derive(Debug, Clone)]
pub struct PinVisualization {
    pub span: Span,
    pub selected_pin: Option<PinId>,
    pub reason: BindingReason,
    pub candidates: Vec<PinCandidate>,
    pub display_style: DisplayStyle,
}

#[derive(Debug, Clone)]
pub struct PinCandidate {
    pub pin_id: PinId,
    pub confidence: f32,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub enum DisplayStyle {
    InlineBadge {
        text: String,
        color: BadgeColor,
    },
    GhostText {
        text: String,
    },
    Underline {
        color: BadgeColor,
        hover_text: String,
    },
}

#[derive(Debug, Clone)]
pub enum BadgeColor {
    Confirmed,
    Ambiguous,
    UserFixed,
    Error,
}

impl LspServer {
    pub fn get_pin_visualizations(&self, uri: &str) -> Vec<PinVisualization> {
        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return vec![],
        };

        let ast = match &doc.ast {
            Some(a) => a,
            None => return vec![],
        };

        let mut visualizations = Vec::new();
        for item in &ast.items {
            self.collect_visualizations_from_item(item, &mut visualizations);
        }
        visualizations
    }

    fn collect_visualizations_from_item(
        &self,
        item: &TopLevelItem,
        visualizations: &mut Vec<PinVisualization>,
    ) {
        match item {
            TopLevelItem::SeedDef(seed) => {
                if let Some(body) = &seed.body {
                    self.collect_from_body(body, visualizations);
                }
            }
        }
    }

    fn collect_from_body(&self, body: &Body, visualizations: &mut Vec<PinVisualization>) {
        for stmt in &body.stmts {
            match stmt {
                Stmt::DeclBlock { items, .. } => {
                    for item in items {
                        if let Some(value) = &item.value {
                            self.collect_from_expr(value, visualizations);
                        }
                    }
                }
                Stmt::Expr { expr, .. } => self.collect_from_expr(expr, visualizations),
                Stmt::Mutate { target, value, .. } => {
                    self.collect_from_expr(target, visualizations);
                    self.collect_from_expr(value, visualizations);
                }
                Stmt::Return { value, .. } => self.collect_from_expr(value, visualizations),
                Stmt::If {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                    self.collect_from_expr(condition, visualizations);
                    self.collect_from_body(then_body, visualizations);
                    if let Some(body) = else_body {
                        self.collect_from_body(body, visualizations);
                    }
                }
                Stmt::Try { action, body, .. } => {
                    self.collect_from_expr(action, visualizations);
                    self.collect_from_body(body, visualizations);
                }
                Stmt::Choose {
                    branches,
                    else_body,
                    ..
                } => {
                    for branch in branches {
                        self.collect_from_expr(&branch.condition, visualizations);
                        self.collect_from_body(&branch.body, visualizations);
                    }
                    self.collect_from_body(else_body, visualizations);
                }
                Stmt::Repeat { body, .. } => {
                    self.collect_from_body(body, visualizations);
                }
                Stmt::While {
                    condition, body, ..
                } => {
                    self.collect_from_expr(condition, visualizations);
                    self.collect_from_body(body, visualizations);
                }
                Stmt::ForEach { iterable, body, .. } => {
                    self.collect_from_expr(iterable, visualizations);
                    self.collect_from_body(body, visualizations);
                }
                Stmt::Break { .. } => {}
                Stmt::Contract {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                    self.collect_from_expr(condition, visualizations);
                    if let Some(body) = then_body {
                        self.collect_from_body(body, visualizations);
                    }
                    self.collect_from_body(else_body, visualizations);
                }
                Stmt::Guard {
                    condition, body, ..
                } => {
                    self.collect_from_expr(condition, visualizations);
                    self.collect_from_body(body, visualizations);
                }
                Stmt::Pragma { .. } | Stmt::MetaBlock { .. } => {}
            }
        }
    }

    fn collect_from_expr(&self, expr: &Expr, visualizations: &mut Vec<PinVisualization>) {
        match &expr.kind {
            ExprKind::Call { args, func } => {
                for arg in args {
                    let viz = self.create_pin_visualization(arg, func);
                    visualizations.push(viz);
                    self.collect_from_expr(&arg.expr, visualizations);
                }
            }
            ExprKind::FieldAccess { target, .. } => self.collect_from_expr(target, visualizations),
            ExprKind::Infix { left, right, .. } => {
                self.collect_from_expr(left, visualizations);
                self.collect_from_expr(right, visualizations);
            }
            ExprKind::Suffix { value, .. } => self.collect_from_expr(value, visualizations),
            ExprKind::Thunk(body) => self.collect_from_body(body, visualizations),
            ExprKind::Eval { thunk, .. } => self.collect_from_expr(thunk, visualizations),
            ExprKind::Pipe { stages } => {
                for stage in stages {
                    self.collect_from_expr(stage, visualizations);
                }
            }
            ExprKind::FlowValue => {}
            ExprKind::TemplateRender { inject, .. } => {
                for (_, value) in inject {
                    self.collect_from_expr(value, visualizations);
                }
            }
            ExprKind::FormulaEval { inject, .. } => {
                for (_, value) in inject {
                    self.collect_from_expr(value, visualizations);
                }
            }
            _ => {}
        }
    }

    fn create_pin_visualization(&self, arg: &ArgBinding, _func: &str) -> PinVisualization {
        let display_style = match (&arg.resolved_pin, &arg.binding_reason) {
            (Some(pin), BindingReason::UserFixed) => DisplayStyle::InlineBadge {
                text: format!("[{}]", pin),
                color: BadgeColor::UserFixed,
            },
            (Some(pin), BindingReason::Dictionary) => DisplayStyle::InlineBadge {
                text: format!("[{}]", pin),
                color: BadgeColor::Confirmed,
            },
            (Some(pin), BindingReason::FlowInjected) => DisplayStyle::InlineBadge {
                text: format!("[{}]", pin),
                color: BadgeColor::Confirmed,
            },
            (None, BindingReason::Ambiguous { candidates }) => DisplayStyle::InlineBadge {
                text: format!("[?×{}]", candidates.len()),
                color: BadgeColor::Ambiguous,
            },
            _ => DisplayStyle::GhostText {
                text: format!(
                    "/* {} */",
                    arg.resolved_pin.as_ref().unwrap_or(&"?".to_string())
                ),
            },
        };

        let candidates = match &arg.binding_reason {
            BindingReason::Ambiguous { candidates } => {
                candidates
                    .iter()
                    .map(|c: &String| PinCandidate {
                        // 타입 명시 추가
                        pin_id: c.clone(),
                        confidence: 0.5,
                        reason: format!(
                            "조사 '{}'와 호환",
                            arg.josa.as_ref().unwrap_or(&"".to_string())
                        ),
                    })
                    .collect()
            }
            _ => vec![],
        };

        PinVisualization {
            span: arg.span,
            selected_pin: arg.resolved_pin.clone(),
            reason: arg.binding_reason.clone(),
            candidates,
            display_style,
        }
    }
}

// ============================================================================
// 진단 (Diagnostics)
// ============================================================================

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub span: Span,
    pub severity: DiagnosticSeverity,
    pub code: DiagnosticCode,
    pub message: String,
    pub fixes: Vec<CodeFix>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticCode {
    AmbiguousPin,
    SpacingError,
    TypeMismatch,
    UndefinedSymbol,
    NoMutatePermission,
    DeterminismViolation,
    InvalidSuffix,
    CallTailMissingAfterArgs,
    CallTailMissingStmt,
}

#[derive(Debug, Clone)]
pub struct CodeFix {
    pub title: String,
    pub kind: CodeFixKind,
    pub edits: Vec<TextEdit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeFixKind {
    QuickFix,
    Refactor,
    RebindPin,
    Canonicalize,
}

#[derive(Debug, Clone)]
pub struct TextEdit {
    pub span: Span,
    pub new_text: String,
}

impl LspServer {
    pub fn diagnose(&self, uri: &str) -> Vec<Diagnostic> {
        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return vec![],
        };
        let ast = match &doc.ast {
            Some(a) => a,
            None => return vec![],
        };

        let mut diagnostics = Vec::new();
        for item in &ast.items {
            self.check_pin_ambiguity(item, &mut diagnostics);
        }
        self.check_spacing(&doc.content, &mut diagnostics);
        self.check_suffix_misuse(&doc.content, &mut diagnostics);
        self.check_call_tail_missing(ast, &mut diagnostics);
        self.check_determinism(ast, &mut diagnostics);
        diagnostics
    }

    fn check_pin_ambiguity(&self, _item: &TopLevelItem, _diagnostics: &mut Vec<Diagnostic>) {}
    fn check_spacing(&self, _content: &str, _diagnostics: &mut Vec<Diagnostic>) {}
    fn check_suffix_misuse(&self, content: &str, diagnostics: &mut Vec<Diagnostic>) {
        let mut in_string = false;
        let mut escape = false;
        let chars: Vec<(usize, char)> = content.char_indices().collect();
        let mut idx = 0;
        while idx < chars.len() {
            let (pos, ch) = chars[idx];
            if in_string {
                if escape {
                    escape = false;
                } else if ch == '\\' {
                    escape = true;
                } else if ch == '"' {
                    in_string = false;
                }
                idx += 1;
                continue;
            }
            if ch == '"' {
                in_string = true;
                idx += 1;
                continue;
            }
            if ch != '@' {
                idx += 1;
                continue;
            }
            let Some((_, next)) = chars.get(idx + 1) else {
                break;
            };
            if *next == '"' {
                idx += 1;
                continue;
            }
            if !is_ident_char(*next) {
                idx += 1;
                continue;
            }
            let start = chars[idx + 1].0;
            let mut end = content.len();
            let mut j = idx + 1;
            while j < chars.len() {
                let (cpos, cch) = chars[j];
                if !is_ident_char(cch) {
                    end = cpos;
                    break;
                }
                j += 1;
            }
            let ident = content[start..end].to_string();
            if core_is_known_unit(&ident) {
                idx = j;
                continue;
            }
            diagnostics.push(Diagnostic {
                span: Span::new(pos, end),
                severity: DiagnosticSeverity::Error,
                code: DiagnosticCode::InvalidSuffix,
                message: format!(
                    "'@{}'는 단위/자원 접미만 허용됩니다. 핀 고정은 '핀=값'을 사용하세요",
                    ident
                ),
                fixes: Vec::new(),
            });
            idx = j;
        }
    }
    fn check_determinism(&self, _ast: &CanonProgram, _diagnostics: &mut Vec<Diagnostic>) {}

    fn check_call_tail_missing(&self, ast: &CanonProgram, diagnostics: &mut Vec<Diagnostic>) {
        let known_seeds = collect_known_seeds(ast);
        for item in &ast.items {
            self.check_call_tail_missing_item(item, &known_seeds, diagnostics);
        }
    }

    fn check_call_tail_missing_item(
        &self,
        item: &TopLevelItem,
        known_seeds: &HashSet<String>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match item {
            TopLevelItem::SeedDef(seed) => {
                if let Some(body) = &seed.body {
                    self.check_call_tail_missing_body(body, known_seeds, diagnostics);
                }
            }
        }
    }

    fn check_call_tail_missing_body(
        &self,
        body: &Body,
        known_seeds: &HashSet<String>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for stmt in &body.stmts {
            match stmt {
                Stmt::DeclBlock { items, .. } => {
                    for item in items {
                        if let Some(value) = &item.value {
                            self.check_call_tail_missing_expr(value, known_seeds, diagnostics);
                        }
                    }
                }
                Stmt::Expr { expr, .. } => {
                    if let ExprKind::Var(name) = &expr.kind {
                        if known_seeds.contains(name) {
                            diagnostics.push(self.build_call_tail_diag(
                                expr.span,
                                name,
                                DiagnosticCode::CallTailMissingStmt,
                            ));
                        }
                    }
                    self.check_call_tail_missing_expr(expr, known_seeds, diagnostics);
                }
                Stmt::Mutate { target, value, .. } => {
                    self.check_call_tail_missing_expr(target, known_seeds, diagnostics);
                    self.check_call_tail_missing_expr(value, known_seeds, diagnostics);
                }
                Stmt::Return { value, .. } => {
                    self.check_call_tail_missing_expr(value, known_seeds, diagnostics);
                }
                Stmt::If {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                    self.check_call_tail_missing_expr(condition, known_seeds, diagnostics);
                    self.check_call_tail_missing_body(then_body, known_seeds, diagnostics);
                    if let Some(body) = else_body {
                        self.check_call_tail_missing_body(body, known_seeds, diagnostics);
                    }
                }
                Stmt::Try { action, body, .. } => {
                    self.check_call_tail_missing_expr(action, known_seeds, diagnostics);
                    self.check_call_tail_missing_body(body, known_seeds, diagnostics);
                }
                Stmt::Choose {
                    branches,
                    else_body,
                    ..
                } => {
                    for branch in branches {
                        self.check_call_tail_missing_expr(
                            &branch.condition,
                            known_seeds,
                            diagnostics,
                        );
                        self.check_call_tail_missing_body(&branch.body, known_seeds, diagnostics);
                    }
                    self.check_call_tail_missing_body(else_body, known_seeds, diagnostics);
                }
                Stmt::Repeat { body, .. } => {
                    self.check_call_tail_missing_body(body, known_seeds, diagnostics);
                }
                Stmt::While {
                    condition, body, ..
                } => {
                    self.check_call_tail_missing_expr(condition, known_seeds, diagnostics);
                    self.check_call_tail_missing_body(body, known_seeds, diagnostics);
                }
                Stmt::ForEach { iterable, body, .. } => {
                    self.check_call_tail_missing_expr(iterable, known_seeds, diagnostics);
                    self.check_call_tail_missing_body(body, known_seeds, diagnostics);
                }
                Stmt::Contract {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                    self.check_call_tail_missing_expr(condition, known_seeds, diagnostics);
                    if let Some(body) = then_body {
                        self.check_call_tail_missing_body(body, known_seeds, diagnostics);
                    }
                    self.check_call_tail_missing_body(else_body, known_seeds, diagnostics);
                }
                Stmt::Guard {
                    condition, body, ..
                } => {
                    self.check_call_tail_missing_expr(condition, known_seeds, diagnostics);
                    self.check_call_tail_missing_body(body, known_seeds, diagnostics);
                }
                Stmt::Break { .. } => {}
                Stmt::Pragma { .. } | Stmt::MetaBlock { .. } => {}
            }
        }
    }

    fn check_call_tail_missing_expr(
        &self,
        expr: &Expr,
        known_seeds: &HashSet<String>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match &expr.kind {
            ExprKind::Call { args, func } => {
                if known_seeds.contains(func) && !has_call_tail(func) {
                    diagnostics.push(self.build_call_tail_diag(
                        expr.span,
                        func,
                        DiagnosticCode::CallTailMissingAfterArgs,
                    ));
                }
                for arg in args {
                    self.check_call_tail_missing_expr(&arg.expr, known_seeds, diagnostics);
                }
            }
            ExprKind::FieldAccess { target, .. } => {
                self.check_call_tail_missing_expr(target, known_seeds, diagnostics);
            }
            ExprKind::Infix { left, right, .. } => {
                self.check_call_tail_missing_expr(left, known_seeds, diagnostics);
                self.check_call_tail_missing_expr(right, known_seeds, diagnostics);
            }
            ExprKind::Suffix { value, .. } => {
                self.check_call_tail_missing_expr(value, known_seeds, diagnostics);
            }
            ExprKind::Thunk(body) => {
                self.check_call_tail_missing_body(body, known_seeds, diagnostics);
            }
            ExprKind::Eval { thunk, .. } => {
                self.check_call_tail_missing_expr(thunk, known_seeds, diagnostics);
            }
            ExprKind::Pipe { stages } => {
                for stage in stages {
                    self.check_call_tail_missing_expr(stage, known_seeds, diagnostics);
                }
            }
            ExprKind::TemplateRender { inject, .. } => {
                for (_, value) in inject {
                    self.check_call_tail_missing_expr(value, known_seeds, diagnostics);
                }
            }
            ExprKind::FormulaEval { inject, .. } => {
                for (_, value) in inject {
                    self.check_call_tail_missing_expr(value, known_seeds, diagnostics);
                }
            }
            ExprKind::Pack { fields } => {
                for (_, value) in fields {
                    self.check_call_tail_missing_expr(value, known_seeds, diagnostics);
                }
            }
            ExprKind::SeedLiteral { body, .. } => {
                self.check_call_tail_missing_expr(body, known_seeds, diagnostics);
            }
            ExprKind::Nuance { expr, .. } => {
                self.check_call_tail_missing_expr(expr, known_seeds, diagnostics);
            }
            _ => {}
        }
    }

    fn build_call_tail_diag(&self, span: Span, name: &str, code: DiagnosticCode) -> Diagnostic {
        let insert_span = Span::new(span.end, span.end);
        let fixes = vec![
            CodeFix {
                title: format!("'{}하기' 붙이기", name),
                kind: CodeFixKind::QuickFix,
                edits: vec![TextEdit {
                    span: insert_span,
                    new_text: "하기".to_string(),
                }],
            },
            CodeFix {
                title: format!("'{}기' 붙이기", name),
                kind: CodeFixKind::QuickFix,
                edits: vec![TextEdit {
                    span: insert_span,
                    new_text: "기".to_string(),
                }],
            },
        ];
        Diagnostic {
            span,
            severity: DiagnosticSeverity::Error,
            code,
            message: format!(
                "호출 꼬리가 필요합니다. 예: '{}하기.' 또는 '{}기.'",
                name, name
            ),
            fixes,
        }
    }
}

// ============================================================================
// 핀 재바인딩 (Code Action)
// ============================================================================

impl LspServer {
    pub fn get_rebind_actions(&self, uri: &str, position: usize) -> Vec<CodeFix> {
        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return vec![],
        };
        let ast = match &doc.ast {
            Some(a) => a,
            None => return vec![],
        };

        let arg_binding = self.find_arg_binding_at(ast, position);
        match arg_binding {
            Some(arg) if matches!(arg.binding_reason, BindingReason::Ambiguous { .. }) => {
                self.create_rebind_actions(arg)
            }
            _ => vec![],
        }
    }

    fn find_arg_binding_at(&self, _ast: &CanonProgram, _position: usize) -> Option<&ArgBinding> {
        None
    }

    fn create_rebind_actions(&self, arg: &ArgBinding) -> Vec<CodeFix> {
        let BindingReason::Ambiguous { candidates } = &arg.binding_reason else {
            return vec![];
        };

        candidates
            .iter()
            .map(|pin_id| {
                let mut new_text = self.extract_text(arg.span);
                new_text.push(':');
                new_text.push_str(pin_id);
                if let Some(josa) = &arg.josa {
                    new_text.push('~');
                    new_text.push_str(josa);
                }
                CodeFix {
                    title: format!("'{}' 핀으로 바인딩", pin_id),
                    kind: CodeFixKind::RebindPin,
                    edits: vec![TextEdit {
                        span: arg.span,
                        new_text,
                    }],
                }
            })
            .collect()
    }

    fn extract_text(&self, _span: Span) -> String {
        "".to_string()
    }
}

// ============================================================================
// LSP 프로토콜 어댑터 (타입 확장)
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub struct LspPosition {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct LspRange {
    pub start: LspPosition,
    pub end: LspPosition,
}

/// E0116 해결: ddonirang_lang::Span을 확장하기 위한 트레이트
pub trait SpanExt {
    fn to_lsp_range(&self, content: &str) -> LspRange;
}

impl SpanExt for Span {
    fn to_lsp_range(&self, content: &str) -> LspRange {
        let (start_line, start_char) = offset_to_position(content, self.start);
        let (end_line, end_char) = offset_to_position(content, self.end);

        LspRange {
            start: LspPosition {
                line: start_line,
                character: start_char,
            },
            end: LspPosition {
                line: end_line,
                character: end_char,
            },
        }
    }
}

/// 독립 함수로 분리하여 E0599 해결
fn offset_to_position(content: &str, offset: usize) -> (u32, u32) {
    let mut line = 0u32;
    let mut col = 0u32;
    for (i, ch) in content.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    (line, col)
}

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || matches!(ch, '가'..='힣' | 'ㄱ'..='ㅎ' | 'ㅏ'..='ㅣ')
}

fn has_call_tail(name: &str) -> bool {
    let tails = ["기", "하기", "고", "하고", "면", "하면"];
    tails.iter().any(|tail| name.ends_with(tail))
}

fn collect_known_seeds(ast: &CanonProgram) -> HashSet<String> {
    let mut known = HashSet::new();
    for item in &ast.items {
        let TopLevelItem::SeedDef(seed) = item;
        known.insert(seed.canonical_name.clone());
    }
    for sig in minimal_stdlib_sigs() {
        known.insert(sig.name.to_string());
    }
    known
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pin_visualization_creation() {
        let arg = ArgBinding {
            id: 1,
            span: Span::new(0, 5),
            expr: Expr::new(2, Span::new(0, 5), ExprKind::Var("철수".to_string())),
            josa: Some("를".to_string()),
            resolved_pin: None,
            binding_reason: BindingReason::Ambiguous {
                candidates: vec!["대상".to_string(), "도구".to_string()],
            },
        };

        let server = LspServer {
            documents: HashMap::new(),
            symbol_table: SymbolTable {
                components: HashMap::new(),
                functions: HashMap::new(),
                enums: HashMap::new(),
            },
        };

        let viz = server.create_pin_visualization(&arg, "먹다");
        assert_eq!(viz.candidates.len(), 2);
    }
}
