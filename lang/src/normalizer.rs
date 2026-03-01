// ddonirang-lang/src/normalizer.rs
// Phase 1: N1 레벨 정본화 (표준 띄어쓰기)
//
// 정규화 레벨:
// - N0: 표면 유지 (편집 중)
// - N1: 띄어쓰기/조사 분리 (저장/포맷)
// - N2: 설탕 해소 (리팩터/리뷰)
// - N3: 완전 정본 (빌드/증명)

use crate::ast::*;

/// 정규화 레벨
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizationLevel {
    /// 표면 유지 (원본 그대로)
    N0,
    /// 띄어쓰기 정리
    N1,
    /// 설탕 해소
    N2,
    /// 완전 정본
    N3,
}

/// 정본화기
pub struct Normalizer {
    _level: NormalizationLevel,
    indent: usize,
    output: String,
}

impl Normalizer {
    pub fn new(level: NormalizationLevel) -> Self {
        Self {
            _level: level,
            indent: 0,
            output: String::new(),
        }
    }

    /// 프로그램 정본화
    pub fn normalize_program(&mut self, program: &CanonProgram) -> String {
        for item in &program.items {
            self.normalize_top_level_item(item);
            self.write("\n\n");
        }

        self.output.trim_end().to_string()
    }

    fn normalize_top_level_item(&mut self, item: &TopLevelItem) {
        match item {
            TopLevelItem::SeedDef(seed) => self.normalize_seed_def(seed),
        }
    }

    /// 씨앗 정의 정본화
    /// 표준 형식: (params) name:kind = { body }
    fn normalize_seed_def(&mut self, seed: &SeedDef) {
        // 매개변수
        if !seed.params.is_empty() {
            self.write("(");
            for (i, param) in seed.params.iter().enumerate() {
                if i > 0 {
                    self.write(", ");
                }
                self.normalize_param(param);
            }
            self.write(") ");
        }

        // 이름
        self.write(&seed.canonical_name);

        // 콜론 (N1: 앞뒤 공백 없음)
        self.write(":");

        // 씨앗 종류
        self.normalize_seed_kind(&seed.seed_kind);

        // 등호 (N1: 앞뒤 공백 있음)
        self.write(" = ");

        // 본문
        if let Some(body) = &seed.body {
            if seed.params.is_empty() {
                if let Some(Stmt::Return { value, .. }) = body.stmts.first() {
                    if body.stmts.len() == 1 {
                        self.normalize_expr(value);
                        return;
                    }
                }
            }
            self.normalize_body(body);
        }
    }

    fn normalize_param(&mut self, param: &ParamPin) {
        self.write(&param.pin_name);
        self.write(":");
        self.normalize_type(&param.type_ref);
        for josa in &param.josa_list {
            self.write("~");
            self.write(josa);
        }
        if param.optional {
            self.write("?");
        }
        if let Some(default_value) = &param.default_value {
            self.write(" = ");
            self.normalize_expr(default_value);
        }
    }

    fn normalize_seed_kind(&mut self, kind: &SeedKind) {
        let s = match kind {
            SeedKind::Imeumssi => "이름씨",
            SeedKind::Umjikssi => "움직씨",
            SeedKind::ValueFunc => "셈씨",
            SeedKind::Gallaessi => "갈래씨",
            SeedKind::Relationssi => "관계씨",
            SeedKind::Sam => "샘",
            SeedKind::Heureumssi => "흐름씨",
            SeedKind::Ieumssi => "이음씨",
            SeedKind::Semssi => "셈씨",
            SeedKind::Named(name) => name,
        };
        self.write(s);
    }

    fn normalize_type(&mut self, type_ref: &TypeRef) {
        match type_ref {
            TypeRef::Named(name) => self.write(name),
            TypeRef::Applied { name, args } => {
                self.write("(");
                for (idx, arg) in args.iter().enumerate() {
                    if idx > 0 {
                        self.write(" ");
                    }
                    self.normalize_type(arg);
                }
                self.write(") ");
                self.write(name);
            }
            TypeRef::Infer => self.write("_"),
        }
    }

    fn normalize_body(&mut self, body: &Body) {
        self.write("{\n");
        self.indent += 1;

        for stmt in &body.stmts {
            self.write_indent();
            self.normalize_stmt(stmt);
            self.write("\n");
        }

        self.indent -= 1;
        self.write_indent();
        self.write("}");
    }

    fn normalize_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::DeclBlock { items, .. } => {
                self.write("채비");
                self.write(": ");
                self.write("{\n");
                self.indent += 1;
                for item in items {
                    self.write_indent();
                    self.write(&item.name);
                    self.write(":");
                    self.normalize_type(&item.type_ref);
                    if let Some(value) = &item.value {
                        match item.kind {
                            DeclKind::Gureut => self.write(" <- "),
                            DeclKind::Butbak => self.write(" = "),
                        }
                        self.normalize_expr(value);
                    }
                    self.write(".");
                    self.write("\n");
                }
                self.indent -= 1;
                self.write_indent();
                self.write("}");
                self.write_stmt_terminator(stmt);
            }
            Stmt::Mutate { target, value, .. } => {
                self.normalize_expr(target);
                self.write(" <- ");
                self.normalize_expr(value);
                self.write_stmt_terminator(stmt);
            }
            Stmt::Expr { expr, .. } => {
                self.normalize_expr(expr);
                self.write_stmt_terminator(stmt);
            }
            Stmt::MetaBlock { kind, entries, .. } => {
                if matches!(kind, MetaBlockKind::BogeaMadang) {
                    // opaque 블록: { } 원본 텍스트를 그대로 재출력
                    self.write("보개마당 {");
                    if let Some(raw) = entries.first() {
                        self.write(raw);
                    }
                    self.write("}");
                    self.write_stmt_terminator(stmt);
                } else {
                    let name = match kind {
                        MetaBlockKind::Setting => "설정",
                        MetaBlockKind::Bogae => "보개",
                        MetaBlockKind::Seulgi => "슬기",
                        MetaBlockKind::BogeaMadang => unreachable!(),
                    };
                    self.write(name);
                    self.write(": ");
                    self.write("{\n");
                    self.indent += 1;
                    for entry in entries {
                        self.write_indent();
                        self.write(entry);
                        self.write(".\n");
                    }
                    self.indent -= 1;
                    self.write_indent();
                    self.write("}");
                    self.write_stmt_terminator(stmt);
                }
            }
            Stmt::Pragma { name, args, .. } => {
                self.write("#");
                self.write(name);
                if !args.is_empty() {
                    self.write(" ");
                    self.write(args);
                }
            }
            Stmt::Return { value, .. } => {
                self.normalize_expr(value);
                self.write(" 돌려줘");
                self.write_stmt_terminator(stmt);
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.normalize_expr(condition);
                self.write(" 일때 ");
                self.normalize_body(then_body);
                if let Some(body) = else_body {
                    self.write(" 아니면 ");
                    self.normalize_body(body);
                }
            }
            Stmt::Try { action, body, .. } => {
                self.normalize_expr(action);
                self.write(" ???: ");
                self.normalize_body(body);
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                self.write("???:\n");
                self.indent += 1;
                for branch in branches {
                    self.write_indent();
                    self.normalize_expr(&branch.condition);
                    self.write(": ");
                    self.normalize_body(&branch.body);
                    self.write("\n");
                }
                self.write_indent();
                self.write("???: ");
                self.normalize_body(else_body);
                self.indent -= 1;
            }
            Stmt::Repeat { body, .. } => {
                self.write("반복: ");
                self.normalize_body(body);
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.normalize_expr(condition);
                self.write(" 동안: ");
                self.normalize_body(body);
            }
            Stmt::ForEach {
                item,
                item_type,
                iterable,
                body,
                ..
            } => {
                self.write("(");
                self.write(item);
                if let Some(type_ref) = item_type.as_ref() {
                    self.write(":");
                    self.normalize_type(type_ref);
                }
                self.write(") ");
                self.normalize_expr(iterable);
                self.write("에 대해: ");
                self.normalize_body(body);
            }
            Stmt::Break { .. } => {
                self.write("멈추기");
                self.write_stmt_terminator(stmt);
            }
            Stmt::Contract {
                kind,
                mode,
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.normalize_expr(condition);
                match kind {
                    ContractKind::Pre => self.write(" 바탕으로"),
                    ContractKind::Post => self.write(" 다짐하고"),
                }
                if matches!(mode, ContractMode::Alert) {
                    self.write("(알림)");
                }
                self.write("\n");
                self.indent += 1;
                self.write_indent();
                self.write("아니면 ");
                self.normalize_body(else_body);
                if let Some(body) = then_body {
                    self.write("\n");
                    self.write_indent();
                    self.write("맞으면 ");
                    self.normalize_body(body);
                }
                self.indent -= 1;
            }
            Stmt::Guard {
                condition, body, ..
            } => {
                self.normalize_expr(condition);
                self.write(" 늘지켜보고 ");
                self.normalize_body(body);
            }
        }
    }

    fn normalize_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Literal(lit) => self.normalize_literal(lit),
            ExprKind::Var(name) => self.write(name),
            ExprKind::FieldAccess { target, field } => {
                self.normalize_expr(target);
                self.write(".");
                self.write(field);
            }
            ExprKind::SeedLiteral { param, body } => {
                self.write("{");
                self.write(param);
                self.write(" | ");
                self.normalize_expr(body);
                self.write("}");
            }
            ExprKind::Call { args, func } => {
                if self.normalize_transform_call(args, func) {
                    return;
                }
                // ???? ??: (args) func
                self.write("(");
                let mut first = true;
                for arg in args.iter() {
                    if matches!(arg.binding_reason, BindingReason::FlowInjected) {
                        continue;
                    }
                    if !first {
                        self.write(", ");
                    }
                    first = false;
                    if matches!(arg.binding_reason, BindingReason::UserFixed) {
                        if let Some(pin) = &arg.resolved_pin {
                            self.write(pin);
                            self.write("=");
                        }
                        self.normalize_expr(&arg.expr);
                    } else {
                        self.normalize_expr(&arg.expr);
                    }
                    if let Some(josa) = &arg.josa {
                        self.write("~");
                        self.write(josa);
                    }
                }
                self.write(") ");
                self.write(func);
            }
            ExprKind::Infix { left, op, right } => {
                self.normalize_expr(left);
                self.write(" ");
                self.write(op);
                self.write(" ");
                self.normalize_expr(right);
            }
            ExprKind::Suffix { value, at } => {
                self.normalize_expr(value);
                match at {
                    AtSuffix::Unit(unit) => {
                        self.write("@");
                        self.write(unit);
                    }
                    AtSuffix::Asset(path) => {
                        self.write("@\"");
                        self.write(path);
                        self.write("\"");
                    }
                }
            }
            ExprKind::Thunk(body) => self.normalize_body(body),
            ExprKind::Eval { thunk, mode } => {
                self.normalize_expr(thunk);
                let suffix = match mode {
                    ThunkEvalMode::Value => "한것",
                    ThunkEvalMode::Bool => "인것",
                    ThunkEvalMode::Not => "아닌것",
                    ThunkEvalMode::Do => "하고",
                    ThunkEvalMode::Pipe => "해서",
                };
                self.write(suffix);
            }
            ExprKind::Pipe { stages } => {
                for (i, stage) in stages.iter().enumerate() {
                    if i > 0 {
                        self.write(" 해서 ");
                    }
                    self.normalize_expr(stage);
                }
            }
            ExprKind::FlowValue => self.write("흐름값"),
            ExprKind::Pack { fields } => {
                self.write("(");
                let mut first = true;
                for (name, value) in fields {
                    if !first {
                        self.write(", ");
                    }
                    first = false;
                    self.write(name);
                    self.write(": ");
                    self.normalize_expr(value);
                }
                self.write(")");
            }
            ExprKind::Formula(formula) => {
                self.normalize_formula_literal(formula);
            }
            ExprKind::Template(template) => {
                self.normalize_template_literal(template);
            }
            ExprKind::TemplateRender { template, inject } => {
                self.normalize_injection_fields(inject);
                self.write(" ");
                self.normalize_template_literal(template);
            }
            ExprKind::FormulaEval { formula, inject } => {
                self.normalize_injection_fields(inject);
                self.write(" ");
                self.normalize_formula_literal(formula);
            }
            ExprKind::Nuance { level, expr } => {
                self.write("$");
                self.write(level);
                self.write(" ");
                self.normalize_expr(expr);
            }
        }
    }

    fn normalize_literal(&mut self, lit: &Literal) {
        match lit {
            Literal::Int(n) => self.write(&n.to_string()),
            Literal::Fixed64(f) => self.write(&f.to_string()),
            Literal::Bool(b) => self.write(if *b { "참" } else { "거짓" }),
            Literal::Atom(a) => {
                self.write("#");
                self.write(a);
            }
            Literal::Regex(regex) => {
                self.write("정규식{");
                self.write("\"");
                self.write(&regex.pattern);
                self.write("\"");
                if !regex.flags.is_empty() {
                    self.write(", ");
                    self.write("\"");
                    self.write(&regex.flags);
                    self.write("\"");
                }
                self.write("}");
            }
            Literal::String(s) => {
                self.write("\"");
                self.write(s);
                self.write("\"");
            }
            Literal::Resource(path) => {
                self.write("@\"");
                self.write(path);
                self.write("\"");
            }
            Literal::None => self.write("없음"),
        }
    }

    fn normalize_formula_literal(&mut self, formula: &Formula) {
        if formula.explicit_tag || !matches!(formula.dialect, FormulaDialect::Ascii) {
            self.write("(#");
            match &formula.dialect {
                FormulaDialect::Ascii => self.write("ascii"),
                FormulaDialect::Ascii1 => self.write("ascii1"),
                FormulaDialect::Latex => self.write("latex"),
                FormulaDialect::Other(tag) => self.write(tag),
            }
            self.write(") ");
        }
        self.write("\u{C218}\u{C2DD}{");
        self.write(&formula.raw);
        self.write("}");
    }

    fn normalize_template_literal(&mut self, template: &Template) {
        if let Some(tag) = &template.tag {
            self.write("(#");
            self.write(tag);
            self.write(") ");
        }
        self.write("글무늬{");
        self.write(&template.raw);
        self.write("}");
    }

    fn normalize_injection_fields(&mut self, fields: &[(String, Expr)]) {
        self.write("(");
        let mut first = true;
        for (name, value) in fields {
            if !first {
                self.write(", ");
            }
            first = false;
            self.write(name);
            self.write("=");
            self.normalize_expr(value);
        }
        self.write(")");
    }

    fn normalize_transform_call(&mut self, args: &[ArgBinding], func: &str) -> bool {
        if !matches!(func, "채우기" | "풀기") {
            return false;
        }
        if args.len() != 2 {
            return false;
        }
        let value_expr = &args[0].expr;
        let pack_expr = &args[1].expr;
        let ExprKind::Pack { fields } = &pack_expr.kind else {
            return false;
        };
        if func == "채우기" {
            if let ExprKind::Template(template) = &value_expr.kind {
                self.normalize_injection_fields(fields);
                self.write(" ");
                self.normalize_template_literal(template);
                return true;
            }
            self.normalize_injection_fields(fields);
            self.write("인 ");
            self.normalize_expr(value_expr);
            self.write(" ");
            self.write(func);
            return true;
        }
        if let ExprKind::Formula(formula) = &value_expr.kind {
            self.normalize_injection_fields(fields);
            self.write(" ");
            self.normalize_formula_literal(formula);
            return true;
        }
        self.normalize_injection_fields(fields);
        self.write("인 ");
        self.normalize_expr(value_expr);
        self.write(" ");
        self.write(func);
        true
    }

    // ========== 유틸리티 ==========

    fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
    }

    fn write_stmt_terminator(&mut self, stmt: &Stmt) {
        let mood = match stmt {
            Stmt::DeclBlock { mood, .. } => mood,
            Stmt::Mutate { mood, .. } => mood,
            Stmt::Expr { mood, .. } => mood,
            Stmt::MetaBlock { mood, .. } => mood,
            Stmt::Pragma { .. } => return,
            Stmt::Return { mood, .. } => mood,
            Stmt::If { mood, .. } => mood,
            Stmt::Try { mood, .. } => mood,
            Stmt::Choose { mood, .. } => mood,
            Stmt::Repeat { mood, .. } => mood,
            Stmt::While { mood, .. } => mood,
            Stmt::ForEach { mood, .. } => mood,
            Stmt::Break { mood, .. } => mood,
            Stmt::Contract { mood, .. } => mood,
            Stmt::Guard { mood, .. } => mood,
        };
        match mood {
            Mood::Interrogative => self.write("?"),
            Mood::Exclamative => self.write("!"),
            _ => self.write("."),
        }
    }
}

/// 편리 함수
pub fn normalize(program: &CanonProgram, level: NormalizationLevel) -> String {
    let mut normalizer = Normalizer::new(level);
    normalizer.normalize_program(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn parse_and_normalize(source: &str, level: NormalizationLevel) -> String {
        let tokens = Lexer::new(source).tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser
            .parse_program(source.to_string(), "test.ddoni".to_string())
            .unwrap();
        normalize(&program, level)
    }

    #[test]
    fn test_n1_spacing_correction() {
        let irregular = "나이:수=10";
        let normalized = parse_and_normalize(irregular, NormalizationLevel::N1);

        assert_eq!(normalized.trim(), "나이:수 = 10");
    }

    #[test]
    fn test_n1_extra_spaces() {
        let irregular = "나이  :  수  =  10";
        let normalized = parse_and_normalize(irregular, NormalizationLevel::N1);

        assert_eq!(normalized.trim(), "나이:수 = 10");
    }

    #[test]
    fn test_n1_function_formatting() {
        let source = "(x:수)증가:셈씨={x+1돌려줘.}";
        let normalized = parse_and_normalize(source, NormalizationLevel::N1);

        let expected = r#"(x:수) 증가:셈씨 = {
    x + 1 돌려줘.
}"#;
        assert_eq!(normalized.trim(), expected);
    }

    #[test]
    fn test_roundtrip() {
        let original = "(x:수) 증가:셈씨 = { x + 1 돌려줘. }";
        let normalized = parse_and_normalize(original, NormalizationLevel::N1);

        // 두 번째 정본화는 동일해야 함 (왕복성)
        let normalized2 = parse_and_normalize(&normalized, NormalizationLevel::N1);

        assert_eq!(normalized.trim(), normalized2.trim());
    }

    #[test]
    fn test_param_default_value_normalization() {
        let source = r#"
(속도:수=10) 이동:셈씨 = {
    속도 돌려줘.
}
"#;
        let normalized = parse_and_normalize(source, NormalizationLevel::N1);

        let expected = r#"(속도:수 = 10) 이동:셈씨 = {
    속도 돌려줘.
}"#;
        assert_eq!(normalized.trim(), expected);
    }

    #[test]
    fn test_josa_inserts_at() {
        let source = r#"
(대상:수~을~를) 이동:셈씨 = {
    (대상을) 이동.
}
"#;
        let normalized = parse_and_normalize(source, NormalizationLevel::N1);

        let expected = r#"(대상:수~을~를) 이동:셈씨 = {
    (대상~을) 이동.
}"#;
        assert_eq!(normalized.trim(), expected);
    }

    #[test]
    fn test_question_mood_normalization() {
        let source = r#"
(값:수) 묻:움직씨 = {
    1.
}

질문:셈씨 = {
    (값) 묻기?
}
"#;
        let normalized = parse_and_normalize(source, NormalizationLevel::N1);

        let expected = r#"(값:수) 묻:움직씨 = {
    1.
}

질문:셈씨 = {
    (값) 묻기?
}"#;
        assert_eq!(normalized.trim(), expected);
    }

    #[test]
    fn test_fixed_pin_normalization() {
        let source = r#"
(대상:수~을, 도구:수~을) 이동:셈씨 = {
    대상 돌려줘.
}

테스트:셈씨 = {
    (1:도구, 2) 이동.
}
"#;
        let normalized = parse_and_normalize(source, NormalizationLevel::N1);
        assert!(normalized.contains("(2, 도구=1) 이동"));
    }

    #[test]
    fn test_template_injection_normalization() {
        let source = r#"
Test:셈씨 = {
    (id=1) 글무늬{"ID={id}"}.
}
"#;
        let normalized = parse_and_normalize(source, NormalizationLevel::N1);
        let expected = r#"Test:셈씨 = {
    (id=1) 글무늬{ID={id}}.
}"#;
        assert_eq!(normalized.trim(), expected);
    }

    #[test]
    fn test_formula_injection_normalization() {
        let source = r#"
Test:셈씨 = {
    (x=6) (#ascii) 수식{ y = 2*x + 3/2 }.
}
"#;
        let normalized = parse_and_normalize(source, NormalizationLevel::N1);
        let expected = r#"Test:셈씨 = {
    (x=6) (#ascii) 수식{ y = 2*x + 3/2 }.
}"#;
        assert_eq!(normalized.trim(), expected);
    }

    #[test]
    fn test_type_ref_applied_normalization() {
        let source = r#"
(목록:(글)차림) 이동:셈씨 = {
    목록 돌려줘.
}
"#;
        let normalized = parse_and_normalize(source, NormalizationLevel::N1);
        let expected = r#"(목록:(글) 차림) 이동:셈씨 = {
    목록 돌려줘.
}"#;
        assert_eq!(normalized.trim(), expected);
    }
}
