// ddonirang-lang/src/normalizer.rs
// Phase 1: N1 레벨 정본화 (표준 띄어쓰기)
//
// 정규화 레벨:
// - N0: 표면 유지 (편집 중)
// - N1: 띄어쓰기/조사 분리 (저장/포맷)
// - N2: 설탕 해소 (리팩터/리뷰)
// - N3: 완전 정본 (빌드/증명)

use crate::ast::*;
use std::collections::{BTreeMap,BTreeSet,HashMap};

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
    call_signatures: HashMap<String, Vec<ParamPin>>,
    source_epoch: SourceEpoch,
    registered_sum_pins:Option<BTreeMap<NodeId,[u8;32]>>,
    registered_sum_seen:BTreeSet<NodeId>,
    registered_sum_failure:Option<(Span,&'static str)>,
}

impl Normalizer {
    pub fn new(level: NormalizationLevel) -> Self {
        Self {
            _level: level,
            indent: 0,
            output: String::new(),
            call_signatures: HashMap::new(),
            source_epoch: SourceEpoch::PreV25,
            registered_sum_pins:None,
            registered_sum_seen:BTreeSet::new(),
            registered_sum_failure:None,
        }
    }

    /// 기존 N1 타입 출력 경로를 읽기 전용 설명으로 제공한다.
    /// 원문/이름/타입 admission, 보임 성공, 저장 정체를 발급하지 않는다.
    /// 저장 소비자는 같은 원문 문맥의 검증된 타입 fact를 별도로 요구해야 한다.
    pub fn source_type_description(type_ref: &TypeRef, source_epoch: SourceEpoch) -> String {
        let mut normalizer = Self::new(NormalizationLevel::N1);
        normalizer.source_epoch = source_epoch;
        normalizer.normalize_type(type_ref);
        normalizer.output
    }

    /// 프로그램 정본화
    pub fn normalize_program(&mut self, program: &CanonProgram) -> String {
        self.registered_sum_seen.clear();self.registered_sum_failure=None;
        self.source_epoch = program.source_epoch;
        self.call_signatures = collect_call_signatures(program);
        // Sum schemas are declarations, not executable statements (RD-349).
        // Emit root schemas before callables that construct their branches.
        if program.source_epoch == SourceEpoch::V25 {
            for statement in &program.root_stmts {
                if matches!(statement,Stmt::SumDefinition {..}) {
                    self.normalize_stmt(statement);
                    self.write("\n\n");
                }
            }
        }
        // A3-P3: parser relocation is execution plumbing, not an authored
        // change of scope. Preserve top-level setup in saved V25 source.
        if program.source_epoch == SourceEpoch::V25 {
            for item in &program.items {
                let TopLevelItem::SeedDef(seed) = item;
                if let Some(body) = &seed.body {
                    for statement in &body.stmts {
                        if seed.is_relocated_top_level_setup(statement) {
                            self.normalize_stmt(statement);
                            self.write("\n\n");
                        }
                    }
                }
            }
        }
        // Keep executable roots and source-scoped definitions interleaved with
        // callables. Grouping all callables first changes lexical visibility.
        let mut roots = program.root_stmts.iter().filter(|stmt|
            program.source_epoch != SourceEpoch::V25 || !matches!(stmt, Stmt::SumDefinition { .. })
        ).peekable();
        for item in &program.items {
            if program.source_epoch == SourceEpoch::V25 {
                let TopLevelItem::SeedDef(seed) = item;
                while roots.peek().is_some_and(|stmt| statement_source_start(stmt) < seed.span.start) {
                    self.normalize_stmt(roots.next().unwrap());
                    self.write("\n\n");
                }
            }
            if program.source_epoch == SourceEpoch::V25 {
                let TopLevelItem::SeedDef(seed) = item;
                if let Some(body) = &seed.body {
                    if body.stmts.iter().any(|stmt| seed.is_relocated_top_level_setup(stmt)) {
                        let synthetic = seed.span == body.span
                            && body.stmts.iter().all(|stmt| seed.is_relocated_top_level_setup(stmt));
                        if synthetic { continue; }
                        let mut authored_seed = seed.clone();
                        authored_seed.body.as_mut().unwrap().stmts
                            .retain(|stmt| !seed.is_relocated_top_level_setup(stmt));
                        self.normalize_seed_def(&authored_seed);
                        self.write("\n\n");
                        continue;
                    }
                }
            }
            let TopLevelItem::SeedDef(seed) = item;
            if let Some(authored) = program.origin.piecewise_definitions.get(&seed.id) {
                // RD706 does not assign a machine canon for mathematical bodies.
                // Preserve their original declaration rather than erase the domain.
                self.write(&authored.source_declaration); self.write("\n\n");
                continue;
            }
            self.normalize_top_level_item(item);
            self.write("\n\n");
        }
        for stmt in roots {
            self.normalize_stmt(stmt);
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
        if !seed.type_params.is_empty() {
            self.write("(");
            for (index, param) in seed.type_params.iter().enumerate() {
                if index > 0 { self.write(", "); }
                self.write(&param.name);
                self.write(":씨자리");
            }
            self.write(") 두고\n");
        }
        if let Some(header) = seed.proof_header.as_ref().filter(|header| !header.propositions.is_empty()) {
            self.write("(");
            for (index, (name, _)) in header.propositions.iter().enumerate() {
                if index > 0 { self.write(", "); }
                self.write(name);
                self.write(":세움");
            }
            self.write(") 두고\n");
        }
        if let Some(result_type) = seed
            .result_type
            .as_ref()
            .filter(|_| seed.seed_kind == SeedKind::ValueFunc)
        {
            self.write(&seed.canonical_name);
            self.write(" (");
            for (i, param) in seed.params.iter().enumerate() {
                if i > 0 {
                    self.write(", ");
                }
                self.normalize_param(param);
            }
            self.write(" --> ");
            self.normalize_type(result_type);
            self.write(")");
            match self.source_epoch {
                SourceEpoch::V25 => self.write(" := "),
                SourceEpoch::PreV25 => self.write(" = "),
            }
            if let Some(Body { stmts, .. }) = &seed.body {
                if let [Stmt::Return { value, .. }] = stmts.as_slice() {
                    self.normalize_expr(value);
                    self.write(".");
                    return;
                }
            }
            return;
        }
        if let SeedKind::Schema(category) = seed.seed_kind {
            self.write(&seed.canonical_name);
            self.write(match category { SchemaCategory::Subject => ":임자 := {", SchemaCategory::NominalValue => ":이름씨 := {" });
            for slot in &seed.params {
                self.write(" ");
                self.write(&slot.pin_name);
                self.write(":");
                self.normalize_type(&slot.type_ref);
                self.write(".");
            }
            self.write(" }.");
            return;
        }
        // 매개변수
        if !seed.params.is_empty() {
            self.write("(");
            for (i, param) in seed.params.iter().enumerate() {
                if i > 0 {
                    self.write(", ");
                }
                if let Some(binding) = seed.proof_header.as_ref().filter(|h| !h.propositions.is_empty()).and_then(|h| h.evidence_bindings.get(i)) {
                    self.write(&param.pin_name);
                    self.write("=");
                    let mut local = param.clone();
                    local.pin_name = binding.local_name.clone();
                    self.normalize_param(&local);
                } else {
                    self.normalize_param(param);
                }
            }
            self.write(") ");
        }

        // 이름
        self.write(&seed.canonical_name);
        if !seed.alternate_forms.is_empty() {
            self.write("{");
            self.write(&seed.alternate_forms.join("~"));
            self.write("}");
        }

        // 콜론 (N1: 앞뒤 공백 없음)
        self.write(":");

        if let Some(result_type) = &seed.result_type {
            self.normalize_type(result_type);
            self.write(":");
        }

        // 씨앗 종류
        self.normalize_seed_kind(&seed.seed_kind);

        match self.source_epoch {
            SourceEpoch::V25 => self.write(" := "),
            SourceEpoch::PreV25 => self.write(" = "),
        }

        // 본문
        if let Some(body) = &seed.body {
            if self.source_epoch == SourceEpoch::V25 && seed.result_type.is_none() {
                if let Some(Stmt::Return { value, .. }) = body.stmts.first() {
                    if body.stmts.len() == 1 {
                        self.normalize_expr(value);
                        return;
                    }
                }
            } else if seed.params.is_empty() {
                if let Some(Stmt::Return { value, .. }) = body.stmts.first() {
                    if body.stmts.len() == 1 {
                        self.normalize_expr(value);
                        return;
                    }
                }
            }
            self.normalize_body(body);
            if seed.seed_kind == SeedKind::Product && self.source_epoch == SourceEpoch::V25 {
                self.write(".");
            }
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
            SeedKind::Product => "묶음씨",
            SeedKind::Schema(SchemaCategory::Subject) => "임자",
            SeedKind::Schema(SchemaCategory::NominalValue) => "이름씨",
            SeedKind::Named(name) => name,
        };
        self.write(s);
    }

    fn normalize_type(&mut self, type_ref: &TypeRef) {
        match type_ref {
            TypeRef::AlgebraEquality { left,right } => self.write(&format!("{} =:= {}",left.source_text(),right.source_text())),
            TypeRef::StructureLaw { structure, member } => {
                self.write(structure); self.write("."); self.write(member);
            }
            TypeRef::PropositionConjunction(items) => {
                self.write("(");
                for (index, item) in items.iter().enumerate() {
                    if index > 0 { self.write(" 이고 "); }
                    self.normalize_type(item);
                }
                self.write(")");
            }
            TypeRef::Tuple(items) => {
                self.write("(");
                for (index, item) in items.iter().enumerate() {
                    if index > 0 { self.write(", "); }
                    self.normalize_type(item);
                }
                self.write(")");
            }
            TypeRef::Tensor { element, shape } => {
                self.write("(원소씨=");
                self.normalize_type(element);
                self.write(", 모양=(");
                for (index, axis) in shape.iter().enumerate() {
                    if index > 0 { self.write(", "); }
                    self.write(&axis.to_string());
                }
                if shape.len() == 1 { self.write(","); }
                self.write(")) 텐서씨");
            }
            TypeRef::Named(name) => self.write(name),
            TypeRef::Applied { name, args } => {
                self.write("(");
                for (idx, arg) in args.iter().enumerate() {
                    if idx > 0 {
                        self.write(", ");
                    }
                    self.normalize_type(arg);
                }
                self.write(") ");
                self.write(name);
            }
            TypeRef::Function { params, result } => {
                if params.len() == 1 {
                    if matches!(&params[0], TypeRef::Tuple(_)) { self.write("("); }
                    self.normalize_function_type_component(&params[0]);
                    if matches!(&params[0], TypeRef::Tuple(_)) { self.write(")"); }
                } else {
                    self.write("(");
                    for (idx, param) in params.iter().enumerate() {
                        if idx > 0 {
                            self.write(", ");
                        }
                        self.normalize_type(param);
                    }
                    self.write(")");
                }
                self.write(" --> ");
                self.normalize_function_type_component(result);
            }
            TypeRef::Infer => self.write("_"),
        }
    }

    fn normalize_function_type_component(&mut self, type_ref: &TypeRef) {
        if matches!(type_ref, TypeRef::Function { .. }) {
            self.write("(");
            self.normalize_type(type_ref);
            self.write(")");
        } else {
            self.normalize_type(type_ref);
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
            Stmt::SumDefinition { name, branches, .. } => {
                self.write(name);
                self.write(":고름씨 := {\n");
                self.indent += 1;
                for branch in branches {
                    self.write_indent();
                    self.write("$");
                    self.write(&branch.name);
                    if !branch.payload.is_empty() {
                        self.write("(");
                        for (index,payload) in branch.payload.iter().enumerate() {
                        if index>0 {self.write(", ");}
                        self.write(&payload.binder);
                        self.write(":");
                        self.normalize_type(&payload.type_ref);
                        }
                        self.write(")");
                    }
                    self.write(".\n");
                }
                self.indent -= 1;
                self.write_indent();
                self.write("}");
                self.write_stmt_terminator(stmt);
            }
            Stmt::SumMatch { subject, arms, default_body, .. } => {
                self.normalize_expr(subject);
                self.write("에 따라 {\n");
                self.indent += 1;
                for arm in arms {
                    self.write_indent();
                    if let Some(kind)=&arm.applied_owner {self.normalize_type(kind);} else {self.write(&arm.owner);}
                    self.write(".$");
                    self.write(&arm.branch);
                    if !arm.binder.is_empty() {
                        self.write("(");
                        // Fixed owner+branch pin identity is independent of
                        // schema/source ordinals and of local binder NodeId.
                        let mut binders:Vec<_>=arm.binder.iter().collect();
                        if let Some(order)=&self.registered_sum_pins {
                            let mut selected=BTreeSet::new();
                            for binder in &binders {
                                let Some(raw)=order.get(&binder.id) else {
                                    self.registered_sum_failure.get_or_insert((binder.span,"E_SUM_PIN_IDENTITY_REQUIRED"));return;
                                };
                                if *raw==[0;32]||!selected.insert(*raw) {
                                    self.registered_sum_failure.get_or_insert((binder.span,"E_SUM_PIN_IDENTITY_DUPLICATE"));return;
                                }
                                self.registered_sum_seen.insert(binder.id);
                            }
                            binders.sort_by_key(|binder|order[&binder.id]);
                        } else if binders.iter().any(|binder|binder.registered_pin_identity.is_some()) {
                            let mut seen=BTreeSet::new();
                            for binder in &binders {
                                let Some(raw)=binder.registered_pin_identity.filter(|raw|*raw!=[0;32]) else {
                                    self.registered_sum_failure.get_or_insert((binder.span,"E_SUM_PIN_IDENTITY_REQUIRED"));return;
                                };
                                if !seen.insert(raw) {self.registered_sum_failure.get_or_insert((binder.span,"E_SUM_PIN_IDENTITY_DUPLICATE"));return;}
                            }
                            binders.sort_by_key(|binder|binder.registered_pin_identity);
                        } else {binders.sort_by(|a,b|a.pin.cmp(&b.pin));}
                        for (index,binder) in binders.iter().enumerate() {
                            if index>0 {self.write(", ");}
                            self.write(&binder.pin);
                        }
                        self.write(")");
                    }
                    self.write("이면 ");
                    self.normalize_body(&arm.body);
                    self.write(".\n");
                }
                if let Some(body) = default_body {
                    self.write_indent();
                    self.write("아니면 ");
                    self.normalize_body(body);
                    self.write(".\n");
                }
                self.indent -= 1;
                self.write_indent();
                self.write("}");
                self.write_stmt_terminator(stmt);
            }
            Stmt::DeclBlock { items, .. } => {
                self.write("채비 ");
                self.write("{\n");
                self.indent += 1;
                for item in items {
                    self.write_indent();
                    self.write(&item.name);
                    self.write(":");
                    self.normalize_type(&item.type_ref);
                    if let Some(value) = &item.value {
                        match item.kind {
                            DeclKind::Gureut if self.source_epoch == SourceEpoch::V25 => {
                                self.write(" := ")
                            }
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
            Stmt::Definition {
                name,
                type_ref,
                value,
                reaction,
                ..
            } => {
                if let Some(registration)=reaction {
                    self.write("(");self.normalize_param(&registration.payload);self.write(") ");
                    self.write(name);self.write(":움직씨 {");self.write(&registration.alert_type);
                    self.write("를 받으면} := ");self.normalize_expr(value);
                    self.write_stmt_terminator(stmt);
                    return;
                }
                self.write(name);
                if !matches!(type_ref, TypeRef::Infer) {
                    self.write(":");
                    self.normalize_type(type_ref);
                }
                self.write(" := ");
                self.normalize_expr(value);
                self.write_stmt_terminator(stmt);
            }
            Stmt::DerivedBinding {name,value,..} => {
                self.write(name);self.write(" <<- ");self.normalize_expr(value);
                self.write_stmt_terminator(stmt);
            }
            Stmt::Mutate {
                target,
                value,
                deferred,
                ..
            } => {
                self.normalize_expr(target);
                self.write(" <- ");
                self.normalize_expr(value);
                if *deferred {
                    self.write(" 미루기");
                }
                self.write_stmt_terminator(stmt);
            }
            Stmt::ResultFlow {
                producer,
                destination,
                ..
            } => {
                self.normalize_expr(producer);
                self.write(" -> ");
                self.normalize_expr(destination);
                self.write_stmt_terminator(stmt);
            }
            Stmt::Expr { expr, .. } => {
                self.normalize_expr(expr);
                self.write_stmt_terminator(stmt);
            }
            Stmt::Show { expr, .. } => {
                self.normalize_expr(expr);
                self.write(" 보여주기");
                self.write_stmt_terminator(stmt);
            }
            Stmt::Inspect { expr, .. } => {
                self.normalize_expr(expr);
                self.write(" 톺아보기");
                self.write_stmt_terminator(stmt);
            }
            Stmt::PublicObservation { fields, .. } => {
                self.write("(");
                for (index, field) in fields.iter().enumerate() {
                    if index > 0 {
                        self.write(", ");
                    }
                    self.write(&field.name);
                    self.write(": ");
                    self.normalize_expr(&field.value);
                }
                self.write(") 내보임");
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
                } else if matches!(kind, MetaBlockKind::Jjaim) {
                    // AGE5 최소 수용: 짜임 블록 본문을 opaque로 정본 재출력
                    self.write("짜임 {");
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
                        MetaBlockKind::Jjaim => unreachable!(),
                    };
                    self.write(name);
                    self.write(" ");
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
                self.write(" 되돌림");
                self.write_stmt_terminator(stmt);
            }
            Stmt::Receive {
                sender,
                kind,
                binding,
                payload_type,
                condition,
                body,
                ..
            } => {
                if let Some(sender) = sender {
                    self.normalize_expr(sender);
                    self.write("에게서 ");
                }
                if let Some(payload_type) = payload_type {
                    self.write("(");
                    self.write(binding.as_deref().expect("typed receive binding"));
                    self.write(":");
                    self.write(payload_type);
                    self.write(")를 받으면 ");
                } else {
                    if let Some(binding) = binding {
                        self.write("(");
                        self.write(binding);
                        if let Some(condition) = condition {
                            self.write(" ");
                            self.normalize_expr(condition);
                        }
                        self.write(")인 ");
                    }
                    self.write(kind.as_deref().unwrap_or("알림"));
                    self.write("을 받으면 ");
                }
                self.normalize_body(body);
                self.write_stmt_terminator(stmt);
            }
            Stmt::Send {
                transfer,
                sender,
                payload,
                receiver,
                ..
            } => {
                if let Some(sender) = sender {
                    self.normalize_expr(sender);
                    self.write("의 ");
                }
                self.normalize_expr(payload);
                self.write(match transfer {
                    crate::delivery::MessageTransfer::Async(_) => " ~> ",
                    crate::delivery::MessageTransfer::Rendezvous(_) => " ~~> ",
                    crate::delivery::MessageTransfer::HistoricalAsync(_) => {
                        " ~[historical-async-migration-required]> "
                    }
                });
                self.normalize_expr(receiver);
                self.write_stmt_terminator(stmt);
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                if self.source_epoch == SourceEpoch::V25 {
                    if let ExprKind::Eval { thunk, mode: ThunkEvalMode::Bool } = &condition.kind {
                        self.normalize_expr(thunk);
                    } else {
                        self.write("{ ");
                        self.normalize_expr(condition);
                        self.write(" }");
                    }
                    self.write("이면 ");
                } else {
                    self.write("만약 ");
                    self.normalize_expr(condition);
                    self.write(" 이라면 ");
                }
                self.normalize_body(then_body);
                if let Some(body) = else_body {
                    self.write(" 아니면 ");
                    self.normalize_body(body);
                }
                if self.source_epoch == SourceEpoch::V25 {
                    self.write_stmt_terminator(stmt);
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
                self.write("되풀이 ");
                self.normalize_body(body);
            }
            Stmt::BeatBlock { body, .. } => {
                self.write("덩이 ");
                self.normalize_body(body);
                self.write_stmt_terminator(stmt);
            }
            Stmt::Hook { kind, body, .. } => {
                match kind {
                    HookKind::Start => self.write("(시작)할때 "),
                    HookKind::End => self.write("(끝)할때 "),
                    HookKind::EveryMadi => self.write("(매마디)마다 "),
                    HookKind::EveryNMadi(interval) => {
                        self.write("(");
                        self.write(&interval.to_string());
                        self.write("마디)마다 ");
                    }
                }
                self.normalize_body(body);
                self.write_stmt_terminator(stmt);
            }
            Stmt::HookWhenBecomes {
                condition, body, ..
            } => {
                self.write("(");
                self.normalize_expr(condition);
                self.write(")이 될때 ");
                self.normalize_body(body);
                self.write_stmt_terminator(stmt);
            }
            Stmt::HookWhile {
                condition, body, ..
            } => {
                self.write("(");
                self.normalize_expr(condition);
                self.write(")인 동안 ");
                self.normalize_body(body);
                self.write_stmt_terminator(stmt);
            }
            Stmt::While {
                condition, body, ..
            } => {
                if let ExprKind::Eval {
                    thunk,
                    mode: ThunkEvalMode::Bool,
                } = &condition.kind
                {
                    self.normalize_expr(thunk);
                } else {
                    self.normalize_expr(condition);
                }
                self.write("인 동안 ");
                self.normalize_body(body);
                self.write_stmt_terminator(stmt);
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
                self.write("에 대해 ");
                self.normalize_body(body);
            }
            Stmt::Quantifier {
                kind,
                variable,
                domain,
                body,
                ..
            } => {
                self.write(variable);
                self.write(" 이 ");
                self.normalize_type(domain);
                match kind {
                    QuantifierKind::ForAll => self.write(" 낱낱에 대해 "),
                    QuantifierKind::Exists => self.write(" 중 하나가 "),
                    QuantifierKind::ExistsUnique => self.write(" 중 딱 하나가 "),
                }
                self.normalize_body(body);
            }
            Stmt::Break { .. } => {
                self.write("멈추기");
                self.write_stmt_terminator(stmt);
            }
            Stmt::ContinueLoop { .. } => {
                self.write("건너뛰기");
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

    fn normalize_selection_target(&mut self, expr: &Expr) {
        if let ExprKind::Call { args, func } = &expr.kind {
            if func == "차림.값" && args.len() == 2 {
                self.normalize_selection_target(&args[0].expr);
                self.write("[");
                self.normalize_expr(&args[1].expr);
                self.write("]");
                return;
            }
        }
        self.normalize_expr(expr);
    }

    fn normalize_selection_index(&mut self, expr: &Expr) {
        if let ExprKind::Call { args, func } = &expr.kind {
            if func == "표준.범위" && args.len() == 3 {
                self.write("(");
                self.normalize_expr(&args[0].expr);
                self.write(")..(");
                self.normalize_expr(&args[1].expr);
                self.write(")");
                return;
            }
        }
        self.normalize_expr(expr);
    }

    fn normalize_expr(&mut self, expr: &Expr) {
        if let Some(operand) = expr.source_negated_operand() {
            self.write("(-(");
            self.normalize_expr(operand);
            self.write("))");
            return;
        }
        match &expr.kind {
            ExprKind::SumBranchValue { owner, applied_owner, branch, payload } => {
                if let Some(kind)=applied_owner {self.normalize_type(kind);} else {self.write(owner);}
                self.write(".$");
                self.write(branch);
                if !payload.is_empty() {
                    self.write("(");
                    // Keep source evaluation order of argument expressions.
                    for (index,payload) in payload.iter().enumerate() {
                    if index>0 {self.write(", ");}
                    self.write(&payload.pin);
                    self.write("=");
                    self.normalize_expr(&payload.value);
                    }
                    self.write(")");
                }
            }
            ExprKind::ProductConstruction { owner, category, fields, .. } => {
                self.write("(");
                for (index, field) in fields.iter().enumerate() {
                    if index > 0 { self.write(", "); }
                    self.write(field.resolved_pin.as_deref().expect("admitted product field"));
                    self.write("=");
                    self.normalize_expr(&field.expr);
                }
                self.write(if category.is_some(){")인 "}else{") "});
                self.write(owner);
            }
            ExprKind::MultiAxisSelection(value) => {
                let grouped_call = matches!(&value.target.kind, ExprKind::Call { .. });
                if grouped_call { self.write("("); }
                self.normalize_expr(&value.target);
                if grouped_call { self.write(")"); }
                self.write("[");
                for (i, selector) in value.selectors.iter().enumerate() {
                    if i > 0 { self.write(", "); }
                    match selector {
                        TensorAxisSelector::Index(e) => self.normalize_selection_index(e),
                        TensorAxisSelector::Range {start,end} => self.write(&format!("{start}..{end}")),
                        TensorAxisSelector::Full => self.write(":"),
                    }
                }
                self.write("]");
            }
            ExprKind::Tensor(value) => match value {
                TensorProviderExpr::FromCollection { input, element_type, shape } => {
                    self.write("(차림=");
                    self.normalize_expr(input);
                    self.write(", 원소씨=");
                    self.normalize_type(element_type);
                    self.write(", 모양=(");
                    for (index, axis) in shape.iter().enumerate() {
                        if index > 0 { self.write(", "); }
                        self.write(&axis.to_string());
                    }
                    if shape.len() == 1 { self.write(","); }
                    self.write(")) 텐서로");
                }
                TensorProviderExpr::ToCollection { input } => {
                    self.write("(텐서=");
                    self.normalize_expr(input);
                    self.write(") 차림으로");
                }
            },
            ExprKind::Literal(lit) => {
                if let Some(source) = expr.numeric_literal_source.as_ref().filter(|s| s.is_decimal()) {
                    // The exact decimal digits and the decimal category must
                    // survive canon/reparse; Fixed64 text cannot supply them.
                    self.write(source.raw());
                } else { self.normalize_literal(lit); }
            },
            ExprKind::Var(name) => self.write(name),
            ExprKind::FieldAccess { target, field, .. } => {
                self.normalize_expr(target);
                self.write(".");
                self.write(field);
            }
            ExprKind::SeedLiteral { param, body } => {
                self.write("{");
                if seed_literal_param_count(param) > 1 {
                    self.write("(");
                    self.write(param);
                    self.write(")");
                } else {
                    self.write(param);
                }
                self.write(" | ");
                self.normalize_expr(body);
                self.write("}");
            }
            ExprKind::Call { args, func } => {
                if self.normalize_transform_call(args, func) {
                    return;
                }
                if self.normalize_bound_call(args, func) {
                    return;
                }
                self.normalize_positional_call(args, func);
            }
            ExprKind::CollectionConstructor { elements } => {
                self.write("차림[");
                for (index, element) in elements.iter().enumerate() {
                    if index > 0 {
                        self.write(", ");
                    }
                    self.normalize_expr(element);
                }
                self.write("]");
            }
            ExprKind::SelectionPlace { target, index } => {
                self.normalize_selection_target(target);
                self.write("[");
                self.normalize_selection_index(index);
                self.write("]");
            }
            ExprKind::Infix { left, op, right } => {
                let left_pipe = matches!(left.kind, ExprKind::Pipe { .. });
                if left_pipe { self.write("("); }
                self.normalize_expr(left);
                if left_pipe { self.write(")"); }
                self.write(" ");
                self.write(op);
                self.write(" ");
                let right_pipe = matches!(right.kind, ExprKind::Pipe { .. });
                if right_pipe { self.write("("); }
                self.normalize_expr(right);
                if right_pipe { self.write(")"); }
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
            ExprKind::TupleConstructor { elements } => {
                self.write("(");
                for (index, element) in elements.iter().enumerate() {
                    if index > 0 { self.write(", "); }
                    self.normalize_expr(element);
                }
                if elements.len() == 1 { self.write(","); }
                self.write(")");
            }
            ExprKind::Assertion(assertion) => {
                self.write(&assertion.canon);
            }
            ExprKind::StateMachine(machine) => {
                self.normalize_state_machine_literal(machine);
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

    fn normalize_bound_call(&mut self, args: &[ArgBinding], func: &str) -> bool {
        let rendered = self.render_bound_call_args(args, func);
        if rendered.is_empty() {
            return false;
        }
        self.write(&rendered.join(" "));
        self.write(" ");
        self.write(func);
        true
    }

    fn normalize_positional_call(&mut self, args: &[ArgBinding], func: &str) {
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
            self.normalize_call_arg_fallback(arg, func);
        }
        self.write(") ");
        self.write(func);
    }

    fn normalize_call_arg_fallback(&mut self, arg: &ArgBinding, func: &str) {
        if let ExprKind::Call {
            args,
            func: nested_func,
        } = &arg.expr.kind
        {
            self.write("(");
            self.normalize_nested_positional_call(args, nested_func);
            self.write(")");
            if let Some(josa) = &arg.josa {
                self.write("~");
                self.write(josa);
            }
            return;
        }
        if let Some(text) = self.render_bound_arg_text(arg, func) {
            self.write(&text);
            return;
        }
        self.normalize_expr(&arg.expr);
        if let Some(josa) = &arg.josa {
            self.write("~");
            self.write(josa);
        }
    }

    fn normalize_nested_positional_call(&mut self, args: &[ArgBinding], func: &str) {
        self.write("(");
        let mut first = true;
        for arg in args {
            if matches!(arg.binding_reason, BindingReason::FlowInjected) {
                continue;
            }
            if !first {
                self.write(", ");
            }
            first = false;
            if let ExprKind::Call {
                args,
                func: nested_func,
            } = &arg.expr.kind
            {
                self.write("(");
                self.normalize_nested_positional_call(args, nested_func);
                self.write(")");
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

    fn render_bound_call_args(&self, args: &[ArgBinding], func: &str) -> Vec<String> {
        let mut rendered = Vec::new();
        for arg in args {
            if matches!(arg.binding_reason, BindingReason::FlowInjected) {
                continue;
            }
            let Some(text) = self.render_bound_arg_text(arg, func) else {
                return Vec::new();
            };
            rendered.push(text);
        }
        rendered
    }

    fn render_bound_arg_text(&self, arg: &ArgBinding, func: &str) -> Option<String> {
        let mut expr = self.render_expr_inline(&arg.expr)?;
        // `이름:핀`을 V25의 `이름:타입 := ...`와 혼동하지 않도록
        // 기존 괄호 식으로 값 경계를 보존한다. 핀 identity는 그대로다.
        let fixed_expr = if self.source_epoch == SourceEpoch::V25
            && matches!(arg.expr.kind, ExprKind::Var(_))
        {
            format!("({expr})")
        } else {
            expr.clone()
        };
        if matches!(arg.binding_reason, BindingReason::ContextItemInjected) {
            return arg.resolved_pin.as_ref().map(|pin| format!("{fixed_expr}:{pin}"));
        }
        if matches!(arg.binding_reason, BindingReason::UserFixed) {
            return arg.resolved_pin.as_ref().map(|pin| format!("{fixed_expr}:{pin}"));
        }
        if let Some(josa) = self.preferred_param_josa(func, arg) {
            if let Some(pin) = arg.resolved_pin.as_deref() {
                expr = self.strip_existing_param_josa(func, pin, &expr);
            }
            return Some(format!("{expr}~{josa}"));
        }
        if let Some(pin) = &arg.resolved_pin {
            return Some(format!("{fixed_expr}:{pin}"));
        }
        if let Some(josa) = &arg.josa {
            return Some(format!("{expr}~{josa}"));
        }
        None
    }

    fn preferred_param_josa(&self, func: &str, arg: &ArgBinding) -> Option<&str> {
        let pin = arg.resolved_pin.as_deref()?;
        let params = self.call_signatures.get(func)?;
        let param = params.iter().find(|param| param.pin_name == pin)?;
        param
            .josa_list
            .iter()
            .find(|candidate| self.is_unique_param_josa(params, pin, candidate))
            .map(|candidate| candidate.as_str())
    }

    fn strip_existing_param_josa(&self, func: &str, pin: &str, expr: &str) -> String {
        let Some(params) = self.call_signatures.get(func) else {
            return expr.to_string();
        };
        let Some(param) = params.iter().find(|param| param.pin_name == pin) else {
            return expr.to_string();
        };
        for candidate in &param.josa_list {
            if expr.ends_with(candidate) && expr.len() > candidate.len() {
                return expr[..expr.len() - candidate.len()].to_string();
            }
        }
        expr.to_string()
    }

    fn is_unique_param_josa(&self, params: &[ParamPin], pin: &str, josa: &str) -> bool {
        params.iter().all(|param| {
            param.pin_name == pin || !param.josa_list.iter().any(|candidate| candidate == josa)
        })
    }

    fn can_inline_bound_expr(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Literal(_) => true,
            ExprKind::Var(_) => true,
            ExprKind::FieldAccess { target, .. } => self.can_inline_bound_expr(target),
            ExprKind::Suffix { value, .. } => self.can_inline_bound_expr(value),
            ExprKind::Eval { thunk, .. } => self.can_inline_bound_expr(thunk),
            ExprKind::FlowValue => true,
            _ => false,
        }
    }

    fn render_expr_inline(&self, expr: &Expr) -> Option<String> {
        if !self.can_inline_bound_expr(expr) {
            return None;
        }
        let mut normalizer = Normalizer::new(self._level);
        normalizer.call_signatures = self.call_signatures.clone();
        normalizer.normalize_expr(expr);
        Some(normalizer.output)
    }

    fn normalize_literal(&mut self, lit: &Literal) {
        match lit {
            Literal::Int(n) => self.write(&n.to_string()),
            Literal::Fixed64(f) => self.write(&f.canonical_decimal_text()),
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
            Literal::Bytes(bytes) => self.write(&bytes.canonical_literal()),
            Literal::String(s) => {
                self.write("\"");
                self.write(&escape_string_literal(s));
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

    fn normalize_state_machine_literal(&mut self, machine: &StateMachine) {
        self.write("상태머신{\n");
        self.indent += 1;
        self.write_indent();
        self.write(&machine.states.join(", "));
        self.write(" 으로 이뤄짐.\n");
        self.write_indent();
        self.write(&machine.initial);
        self.write(" 으로 시작.\n");
        for transition in &machine.transitions {
            self.write_indent();
            self.write(&transition.from);
            self.write(" 에서 ");
            self.write(&transition.to);
            self.write(" 으로");
            if let Some(guard_name) = &transition.guard_name {
                self.write(" 걸러서 ");
                self.write(guard_name);
            }
            if let Some(action_name) = &transition.action_name {
                self.write(" 하고 ");
                self.write(action_name);
            }
            self.write(".\n");
        }
        for check in &machine.on_transition_checks {
            self.write_indent();
            self.write("바뀔때마다 ");
            self.write(check);
            self.write(" 살피기.\n");
        }
        self.indent -= 1;
        self.write_indent();
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
        // 흐름값은 파이프가 공급하는 내부 항이므로 원문 이름으로 내보내지 않는다.
        if func == "풀기" && matches!(value_expr.kind, ExprKind::FlowValue) {
            if let ExprKind::Pack { fields } = &pack_expr.kind {
                self.normalize_injection_fields(fields);
            } else {
                self.write("(주입=");
                self.normalize_expr(pack_expr);
                self.write(")");
            }
            self.write(" 풀기");
            return true;
        }
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
            Stmt::SumDefinition { mood, .. } | Stmt::SumMatch { mood, .. } => mood,
            Stmt::DeclBlock { mood, .. } => mood,
            Stmt::Definition { mood, .. } => mood,
            Stmt::Mutate { mood, .. } | Stmt::DerivedBinding {mood,..} => mood,
            Stmt::ResultFlow { mood, .. } => mood,
            Stmt::Expr { mood, .. } => mood,
            Stmt::Receive { mood, .. } => mood,
            Stmt::Send { mood, .. } => mood,
            Stmt::Show { mood, .. } | Stmt::Inspect { mood, .. } => mood,
            Stmt::PublicObservation { mood, .. } => mood,
            Stmt::MetaBlock { mood, .. } => mood,
            Stmt::Pragma { .. } => return,
            Stmt::Return { mood, .. } => mood,
            Stmt::If { mood, .. } => mood,
            Stmt::Try { mood, .. } => mood,
            Stmt::Choose { mood, .. } => mood,
            Stmt::Repeat { mood, .. } => mood,
            Stmt::BeatBlock { mood, .. } => mood,
            Stmt::Hook { mood, .. } => mood,
            Stmt::HookWhenBecomes { mood, .. } => mood,
            Stmt::HookWhile { mood, .. } => mood,
            Stmt::While { mood, .. } => mood,
            Stmt::ForEach { mood, .. } => mood,
            Stmt::Quantifier { mood, .. } => mood,
            Stmt::Break { mood, .. } => mood,
            Stmt::ContinueLoop { mood, .. } => mood,
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

fn escape_string_literal(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

/// 편리 함수
pub fn normalize(program: &CanonProgram, level: NormalizationLevel) -> String {
    let mut normalizer = Normalizer::new(level);
    normalizer.normalize_program(program)
}

/// Formatting projection only. The shared product registration owner must
/// verify source/module/branch/pin/binder correspondence before supplying
/// this compilation-local NodeId cache. This API issues no registration,
/// TypeIdentity, runtime capability, saved artifact or PESI authority.
/// Missing/duplicate/unused bindings discard the whole candidate string;
/// a registered path never falls back to spelling order.
pub fn normalize_with_registered_sum_pin_order(program:&CanonProgram,level:NormalizationLevel,
    order:&BTreeMap<NodeId,[u8;32]>)->Result<String,String> {
    let mut normalizer=Normalizer::new(level);
    normalizer.registered_sum_pins=Some(order.clone());
    let output=normalizer.normalize_program(program);
    if let Some((span,code))=normalizer.registered_sum_failure {
        return Err(format!("{code}: 등록 받는자리 정체와 pattern이 다릅니다 @{}..{}",span.start,span.end));
    }
    if normalizer.registered_sum_seen.len()!=order.len() {
        return Err("E_SUM_PIN_IDENTITY_UNUSED: 원문에 없는 받는자리 대응이 있습니다".into());
    }
    Ok(output)
}

/// Same formatter for a source-owned declaration fragment. The registration
/// owner supplies only the selected pins present in this fragment; this is a
/// projection, not a constructor for a verified catalog or effect capability.
pub enum RegisteredCanonFragment<'a> {Definition(&'a SeedDef),Statement(&'a Stmt),Expression(&'a Expr)}
pub fn normalize_registered_fragment(program:&CanonProgram,fragment:RegisteredCanonFragment<'_>,
    order:&BTreeMap<NodeId,[u8;32]>)->Result<String,String> {
    let mut formatter=Normalizer::new(NormalizationLevel::N1);
    formatter.source_epoch=program.source_epoch;
    formatter.call_signatures=collect_call_signatures(program);
    formatter.registered_sum_pins=Some(order.clone());
    match fragment {
        RegisteredCanonFragment::Definition(seed)=>formatter.normalize_seed_def(seed),
        RegisteredCanonFragment::Statement(statement)=>formatter.normalize_stmt(statement),
        RegisteredCanonFragment::Expression(expression)=>formatter.normalize_expr(expression),
    }
    if let Some((span,code))=formatter.registered_sum_failure {
        return Err(format!("{code}: 원문 조각의 등록 받는자리 대응이 다릅니다 @{}..{}",span.start,span.end));
    }
    if formatter.registered_sum_seen.len()!=order.len() {return Err("E_SUM_PIN_IDENTITY_UNUSED: 원문 조각 밖의 대응입니다".into());}
    Ok(formatter.output)
}

/// Stable source-order-free expression material for semantic identities.
/// Spans and NodeIds are deliberately absent from Normalizer output.
pub fn normalize_expression_identity(expr: &Expr) -> String {
    let mut normalizer = Normalizer::new(NormalizationLevel::N3);
    normalizer.normalize_expr(expr);
    normalizer.output
}

fn collect_call_signatures(program: &CanonProgram) -> HashMap<String, Vec<ParamPin>> {
    let mut out = HashMap::new();
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        for surface in seed.callable_surface_names() {
            out.insert(surface, seed.params.clone());
        }
    }
    out
}

fn seed_literal_param_count(param: &str) -> usize {
    param
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .count()
        .max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    #[test]
    fn vc04_tuple_type_preserves_function_parameter_arity() {
        let parse = |source: &str| {
            let tokens=Lexer::new(source).tokenize().unwrap();
            Parser::new_v25_with_mode(tokens,crate::ParseMode::Strict).parse_program(source.into(),"tuple.ddn".into()).unwrap()
        };
        for (source, expected) in [
            ("(값:((바른수, 글)) --> 참거짓) 틀:묶음씨 := {}.","((바른수, 글)) --> 참거짓"),
            ("(값:(바른수, 글) --> 참거짓) 틀:묶음씨 := {}.","(바른수, 글) --> 참거짓"),
            ("(값:(바른수, 글)) 틀:묶음씨 := {}.","(바른수, 글)"),
            ("(값:(바른수, 글)짝맞춤) 틀:묶음씨 := {}.","(바른수, 글) 짝맞춤"),
        ] {
            let before=parse(source);
            let canon=normalize(&before,NormalizationLevel::N1);
            assert!(canon.contains(expected),"{canon}");
            let after=parse(&canon);
            assert_eq!(normalize(&after,NormalizationLevel::N1),canon);
            // Compare declaration-owned types, not merely stable output text.
            let TopLevelItem::SeedDef(before)=&before.items[0];
            let TopLevelItem::SeedDef(after)=&after.items[0];
            assert_eq!(before.params.len(),1);
            assert_eq!(before.params.iter().map(|pin|&pin.type_ref).collect::<Vec<_>>(),
                after.params.iter().map(|pin|&pin.type_ref).collect::<Vec<_>>());
        }
        for source in ["(값:()) 틀:묶음씨 := {}.","(값:(글)) 틀:묶음씨 := {}.","(값:(글,)) 틀:묶음씨 := {}."] {
            let tokens=Lexer::new(source).tokenize().unwrap();
            assert!(Parser::new_v25_with_mode(tokens,crate::ParseMode::Strict).parse_program(source.into(),"bad-tuple.ddn".into()).is_err());
        }
    }

    #[test]
    fn vc13_tuple_constructor_canon_preserves_position_and_category() {
        for source in ["값 <- (1, \"글\", 참).", "값 <- (1,).", "값 <- ()."] {
            let canon=parse_and_normalize(source,NormalizationLevel::N1);
            assert_eq!(parse_and_normalize(&canon,NormalizationLevel::N1),canon);
            assert!(canon.contains('('));
            assert!(!canon.contains("차림["));
        }
    }

    #[test]
    fn fixed64_literal_canonical_decimal_preserves_bits_including_min() {
        use ddonirang_core::Fixed64;
        for raw in [i64::MIN, -1, 0, 1, 1i64 << 25, 3i64 << 25, i64::MAX] {
            let mut normalizer = Normalizer::new(NormalizationLevel::N1);
            normalizer.normalize_literal(&Literal::Fixed64(Fixed64::from_raw_i64(raw)));
            let text = normalizer.output;
            assert_eq!(Fixed64::parse_decimal_exact(&text).unwrap().raw_i64(), raw);
            assert_eq!(text, Fixed64::from_raw_i64(raw).canonical_decimal_text());
            let source = format!("x <- {text}.");
            let normalized = parse_and_normalize(&source, NormalizationLevel::N1);
            assert_eq!(parse_and_normalize(&normalized, NormalizationLevel::N1), normalized);
        }
    }

    fn parse_and_normalize(source: &str, level: NormalizationLevel) -> String {
        let tokens = Lexer::new(source).tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser
            .parse_program(source.to_string(), "test.ddoni".to_string())
            .unwrap();
        normalize(&program, level)
    }

    #[test]
    fn wo09_signed_min_ingress_suffix_and_range_roundtrip() {
        for expression in ["-2147483648", "-2147483648@m", "(-2147483648)",
            "(-2147483648@m)", "- -2147483648", "-2147483648 아님"] {
            let source = format!("x <- {expression}.");
            let canonical = parse_and_normalize(&source, NormalizationLevel::N1);
            assert_eq!(parse_and_normalize(&canonical, NormalizationLevel::N1), canonical,
                "{expression}");
        }
        for expression in ["2147483648", "2147483648@m", "-2147483649",
            "-2147483648.000000001", "(-2147483649)"] {
            let source = format!("x <- {expression}.");
            let tokens = Lexer::new(&source).tokenize().unwrap();
            let error = Parser::new(tokens).parse_program(source, "range.ddn".into()).unwrap_err();
            assert!(error.message.contains("E_FIXED64_LITERAL_RANGE"), "{expression}: {error:?}");
        }
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
    x + 1 되돌림.
}"#;
        assert_eq!(normalized.trim(), expected);
    }

    #[test]
    fn v25_single_return_callable_emits_current_expression_surface() {
        let source = "(x:수) 증가:셈씨 := x + 1";
        let normalized =
            crate::parse_v25_and_normalize(source, "v25_single_return.ddn", NormalizationLevel::N1)
                .expect("V25 source must normalize");

        // Owner M-A/R4: a V25 declaration's 수 is the canonical 셈수 owner.
        // Keep the authored input and roundtrip check, not the old alias output.
        assert_eq!(normalized.trim(), "(x:셈수) 증가:셈씨 := x + 1");
        assert!(!normalized.contains("되돌림"));
        assert!(!normalized.contains("돌려줘"));

        let roundtrip = crate::parse_v25_and_normalize(
            &normalized,
            "v25_single_return.ddn",
            NormalizationLevel::N1,
        )
        .expect("emitted V25 source must normalize");
        assert_eq!(normalized, roundtrip);
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
    fn string_literal_roundtrips_lexer_escapes_without_normalizing_text() {
        let original = r#"글:글 = "첫\줄둘\칸\따옴인용\따옴\역빗금값""#;
        let normalized = parse_and_normalize(original, NormalizationLevel::N1);
        assert!(normalized.contains(r#""첫\n둘\t\"인용\"\\값""#));

        let normalized2 = parse_and_normalize(&normalized, NormalizationLevel::N1);
        assert_eq!(normalized.trim(), normalized2.trim());
        assert!(normalized.contains("값"));
        assert!(!normalized.contains("값"));
    }

    #[test]
    fn applied_type_roundtrips_with_comma_separated_arguments() {
        for source in [
            "(값:(글)차림) 처리:셈씨 = { 값 되돌림. }",
            "(값:(글, 수)짝맞춤) 처리:셈씨 = { 값 되돌림. }",
            "(값:(글, (수)차림)짝맞춤) 처리:셈씨 = { 값 되돌림. }",
            "(값:((글, 수)짝맞춤)차림) 처리:셈씨 = { 값 되돌림. }",
        ] {
            let normalized = parse_and_normalize(source, NormalizationLevel::N1);
            let normalized_again = parse_and_normalize(&normalized, NormalizationLevel::N1);
            assert_eq!(normalized.trim(), normalized_again.trim(), "{source}");
            assert!(!normalized.contains("(글 수)"), "{normalized}");
        }
    }

    #[test]
    fn sender_filtered_typed_receive_has_exact_canonical_surface() {
        let source = r#"
관제탑:임자 = {
    철수에게서 (받은수:수)를 받으면 {
        받은수 보여주기.
    }.
}
"#;
        let normalized = parse_and_normalize(source, NormalizationLevel::N1);
        assert!(normalized.contains("철수에게서 (받은수:수)를 받으면"));
        let normalized2 = parse_and_normalize(&normalized, NormalizationLevel::N1);
        assert_eq!(normalized.trim(), normalized2.trim());
    }

    #[test]
    fn typed_receive_without_sender_is_any_sender_surface() {
        let source = r#"
관제탑:임자 = {
    (받은수:수)를 받으면 {
        받은수 보여주기.
    }.
}
"#;
        let normalized = parse_and_normalize(source, NormalizationLevel::N1);
        assert!(normalized.contains("(받은수:수)를 받으면"));
        assert!(!normalized.contains("에게서"));
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
    속도 되돌림.
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
    대상~을 이동.
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
    값:값 묻기?
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
        assert!(normalized.contains("2:대상 1:도구 이동"));
    }

    #[test]
    fn test_named_multi_call_keeps_fixed_pins_for_roundtrip() {
        let source = r#"
(위치:수~을~를, 속도:수~과~와, 시간:수~로~으로) 다음:셈씨 = {
    위치 + 속도 * 시간 돌려줘.
}

실행:셈씨 = {
    (위치=1, 속도=2, 시간=3) 다음.
}
"#;
        let normalized = parse_and_normalize(source, NormalizationLevel::N1);
        assert!(normalized.contains("1:위치 2:속도 3:시간 다음"));
        let reparsed = crate::parse(&normalized, "named_multi_roundtrip.ddn")
            .expect("fixed-pin canonical output must reparse");
        let renormalized = normalize(&reparsed, NormalizationLevel::N1);
        assert_eq!(normalized, renormalized);
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
    목록 되돌림.
}"#;
        assert_eq!(normalized.trim(), expected);
    }

    #[test]
    fn test_block_headers_normalize_without_colon() {
        let source = r#"
테스트:움직씨 = {
    채비: { 값:수 <- 0. }.
    설정: { 화면: "기본". }.
    반복: { 멈추기. }.
    { 값 < 1 }인 동안 { 멈추기. }.
    (x) 값목록에 대해: { x 보여주기. }.
}
"#;
        let normalized = parse_and_normalize(source, NormalizationLevel::N1);
        assert!(normalized.contains("채비 {"));
        assert!(normalized.contains("설정 {"));
        assert!(normalized.contains("되풀이 {"));
        assert!(normalized.contains("동안 {"));
        assert!(normalized.contains("(x) 값목록에 대해 {"));

        assert!(!normalized.contains("채비: {"));
        assert!(!normalized.contains("설정: {"));
        assert!(!normalized.contains("반복: {"));
        assert!(!normalized.contains("동안: {"));
        assert!(!normalized.contains("에 대해: {"));
    }
}

fn statement_source_start(stmt: &Stmt) -> usize {
        match stmt {
            Stmt::SumDefinition { span, .. }
            | Stmt::SumMatch { span, .. }
            | Stmt::DeclBlock { span, .. }
            | Stmt::Definition { span, .. }
            | Stmt::DerivedBinding { span, .. }
            | Stmt::Mutate { span, .. }
            | Stmt::ResultFlow { span, .. }
            | Stmt::Expr { span, .. }
            | Stmt::Receive { span, .. }
            | Stmt::Send { span, .. }
            | Stmt::Show { span, .. }
            | Stmt::Inspect { span, .. }
            | Stmt::PublicObservation { span, .. }
            | Stmt::Return { span, .. }
            | Stmt::If { span, .. }
            | Stmt::Try { span, .. }
            | Stmt::Choose { span, .. }
            | Stmt::Repeat { span, .. }
            | Stmt::While { span, .. }
            | Stmt::ForEach { span, .. }
            | Stmt::Quantifier { span, .. }
            | Stmt::Break { span, .. }
            | Stmt::ContinueLoop { span, .. }
            | Stmt::Contract { span, .. }
            | Stmt::Guard { span, .. }
            | Stmt::BeatBlock { span, .. }
            | Stmt::Hook { span, .. }
            | Stmt::HookWhenBecomes { span, .. }
            | Stmt::HookWhile { span, .. }
            | Stmt::MetaBlock { span, .. }
            | Stmt::Pragma { span, .. } => span.start,
        }
    }
