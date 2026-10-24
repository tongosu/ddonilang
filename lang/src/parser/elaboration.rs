//! Existing name and unit validation plus argument elaboration; original Parser state and method bodies retained.
use super::*;

impl Parser {
    pub(super) fn validate_seed_name_conflicts(&self, program: &CanonProgram) -> Result<(), ParseError> {
        let mut names: HashMap<String, Span> = HashMap::new();
        let mut all_names = HashMap::new();
        for item in &program.items {
            let TopLevelItem::SeedDef(seed) = item;
            if let Some(previous_product) = all_names.insert(&seed.canonical_name, seed.seed_kind.is_schema()) {
                if previous_product || seed.seed_kind.is_schema() {
                    return Err(ParseError { span: seed.span, message: format!(
                        "E_PRODUCT_DEFINITION_DUPLICATE: 정의 `{}`의 소유자가 중복되었습니다", seed.canonical_name) });
                }
            }
            if !seed.seed_kind.is_schema() {
                names.insert(seed.canonical_name.clone(), seed.span);
            }
        }
        for item in &program.items {
            let TopLevelItem::SeedDef(seed) = item;
            if seed.seed_kind.is_schema() { continue; }
            let name = &seed.canonical_name;
            if let Some(base) = name.strip_suffix("하") {
                if !base.is_empty() && names.contains_key(base) {
                    return Err(ParseError {
                        span: seed.span,
                        message: format!(
                            "E_SEED_NAME_CONFLICT_HA: '{}'와 '{}'를 동시에 정의할 수 없습니다",
                            base, name
                        ),
                    });
                }
            } else {
                let ha_name = format!("{}하", name);
                if names.contains_key(&ha_name) {
                    return Err(ParseError {
                        span: seed.span,
                        message: format!(
                            "E_SEED_NAME_CONFLICT_HA: '{}'와 '{}'를 동시에 정의할 수 없습니다",
                            name, ha_name
                        ),
                    });
                }
            }
        }

        // An exact surface may be shared only when typed resolution can still
        // distinguish the definitions. Identical typed call shapes would make
        // declaration/source order an implicit winner, so reject them here.
        let mut exact_surface_owners: HashMap<(String, String), (String, Span)> = HashMap::new();
        for item in &program.items {
            let TopLevelItem::SeedDef(seed) = item;
            if seed.seed_kind.is_schema() { continue; }
            let typed_shape = format!(
                "{:?}",
                seed.params
                    .iter()
                    .map(|param| (&param.type_ref, &param.josa_list, param.optional))
                    .collect::<Vec<_>>()
            );
            for surface in seed.callable_surface_names() {
                let key = (surface.clone(), typed_shape.clone());
                if let Some((owner, _)) = exact_surface_owners.get(&key) {
                    if owner != &seed.canonical_name {
                        return Err(ParseError {
                            span: seed.span,
                            message: format!(
                                "E_CALLABLE_SURFACE_AMBIGUOUS: '{}' 표면이 '{}'와 '{}'에 같은 typed call shape로 등록되었습니다",
                                surface, owner, seed.canonical_name
                            ),
                        });
                    }
                } else {
                    exact_surface_owners.insert(key, (seed.canonical_name.clone(), seed.span));
                }
            }
        }
        Ok(())
    }

    pub(super) fn validate_seed_name_tail(&self, name: &str, span: Span) -> Result<(), ParseError> {
        let tails = ["기", "하기", "고", "하고", "면", "하면", "면서", "하면서"];
        if tails.iter().any(|tail| name.ends_with(tail)) {
            return Err(ParseError {
                span,
                message: format!("Gate0: 정의 이름에는 꼬리({})를 붙일 수 없습니다", name),
            });
        }
        Ok(())
    }

    pub(super) fn apply_default_args(&mut self, program: &mut CanonProgram) -> Result<(), ParseError> {
        let mut signatures = HashMap::new();
        for item in &program.items {
            let TopLevelItem::SeedDef(seed) = item;
            if seed.callable_surface_names().iter().any(|name|
                crate::bytes::BytesConversion::from_registered_surface(name).is_some()) {
                return Err(ParseError { span: seed.span,
                    message: "E_BYTES_ARGUMENT_BINDING: UTF-8 변환의 기존 정의 정체를 다시 선언할 수 없습니다".into() });
            }
            if seed.seed_kind == SeedKind::Product {
                self.product_definitions.insert(seed.canonical_name.clone(), (seed.id, seed.params.clone()));
                continue;
            }
            if matches!(seed.seed_kind, SeedKind::Schema(_)) {
                let SeedKind::Schema(category)=seed.seed_kind else {unreachable!()};
                self.schema_categories.insert(seed.canonical_name.clone(),category);
                self.product_definitions.insert(seed.canonical_name.clone(),(seed.id,seed.params.clone()));
                continue;
            }
            for surface in seed.callable_surface_names() {
                signatures.insert(surface, seed.params.clone());
            }
        }
        let mut known_seeds: HashSet<String> = signatures.keys().cloned().collect();
        for sig in minimal_stdlib_sigs() {
            known_seeds.insert(sig.name.to_string());
        }

        for item in &mut program.items {
            match item {
                TopLevelItem::SeedDef(seed) => {
                    if let Some(body) = &mut seed.body {
                        self.apply_defaults_in_body(body, &signatures, &known_seeds)?;
                    }
                }
            }
        }
        self.apply_defaults_in_stmts(&mut program.root_stmts, &signatures, &known_seeds)?;
        Ok(())
    }

    pub(super) fn validate_units(&self, program: &CanonProgram) -> Result<(), ParseError> {
        for item in &program.items {
            let TopLevelItem::SeedDef(seed) = item;
            if let Some(body) = &seed.body {
                self.validate_body_units(body)?;
            }
        }
        for stmt in &program.root_stmts {
            self.validate_stmt_units(stmt)?;
        }
        Ok(())
    }

    pub(super) fn validate_body_units(&self, body: &Body) -> Result<(), ParseError> {
        for stmt in &body.stmts {
            self.validate_stmt_units(stmt)?;
        }
        Ok(())
    }

    pub(super) fn validate_stmt_units(&self, stmt: &Stmt) -> Result<(), ParseError> {
        match stmt {
            Stmt::SumDefinition { .. } => Ok(()),
            Stmt::SumMatch { subject, arms, default_body, .. } => {
                self.infer_expr_dim(subject)?;
                for arm in arms { self.validate_body_units(&arm.body)?; }
                if let Some(body) = default_body { self.validate_body_units(body)?; }
                Ok(())
            }
            Stmt::DeclBlock { items, .. } => {
                for item in items {
                    if let Some(value) = &item.value {
                        self.infer_expr_dim(value)?;
                    }
                }
                Ok(())
            }
            Stmt::ResultFlow {
                producer,
                destination,
                ..
            } => {
                self.infer_expr_dim(producer)?;
                self.infer_expr_dim(destination)?;
                Ok(())
            }
            Stmt::Definition { value, .. } | Stmt::DerivedBinding {value,..} => {
                self.infer_expr_dim(value)?;
                Ok(())
            }
            Stmt::Mutate { target, value, .. } => {
                let target_dim = self.infer_expr_dim(target)?;
                let value_dim = self.infer_expr_dim(value)?;
                if let (DimState::Known(left), DimState::Known(right)) = (target_dim, value_dim) {
                    if left != right {
                        return Err(self.unit_mismatch_error(value.span, left, right));
                    }
                }
                Ok(())
            }
            Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
                self.infer_expr_dim(expr)?;
                Ok(())
            }
            Stmt::Receive {
                condition, body, ..
            } => {
                if let Some(condition) = condition {
                    self.infer_expr_dim(condition)?;
                }
                self.validate_body_units(body)
            }
            Stmt::Send {
                sender,
                payload,
                receiver,
                ..
            } => {
                if let Some(sender) = sender {
                    self.infer_expr_dim(sender)?;
                }
                self.infer_expr_dim(payload)?;
                self.infer_expr_dim(receiver)?;
                Ok(())
            }
            Stmt::Return { value, .. } => {
                self.infer_expr_dim(value)?;
                Ok(())
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.infer_expr_dim(condition)?;
                self.validate_body_units(then_body)?;
                if let Some(body) = else_body {
                    self.validate_body_units(body)?;
                }
                Ok(())
            }
            Stmt::Try { action, body, .. } => {
                self.infer_expr_dim(action)?;
                self.validate_body_units(body)?;
                Ok(())
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                for branch in branches {
                    self.infer_expr_dim(&branch.condition)?;
                    self.validate_body_units(&branch.body)?;
                }
                self.validate_body_units(else_body)?;
                Ok(())
            }
            Stmt::Repeat { body, .. } => {
                self.validate_body_units(body)?;
                Ok(())
            }
            Stmt::BeatBlock { body, .. } => {
                self.validate_body_units(body)?;
                Ok(())
            }
            Stmt::Hook { body, .. } => {
                self.validate_body_units(body)?;
                Ok(())
            }
            Stmt::HookWhenBecomes {
                condition, body, ..
            }
            | Stmt::HookWhile {
                condition, body, ..
            } => {
                self.infer_expr_dim(condition)?;
                self.validate_body_units(body)?;
                Ok(())
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.infer_expr_dim(condition)?;
                self.validate_body_units(body)?;
                Ok(())
            }
            Stmt::ForEach { iterable, body, .. } => {
                self.infer_expr_dim(iterable)?;
                self.validate_body_units(body)?;
                Ok(())
            }
            Stmt::Quantifier { body, .. } => {
                self.validate_body_units(body)?;
                Ok(())
            }
            Stmt::Break { .. } | Stmt::ContinueLoop { .. } => Ok(()),
            Stmt::Contract {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.infer_expr_dim(condition)?;
                if let Some(body) = then_body {
                    self.validate_body_units(body)?;
                }
                self.validate_body_units(else_body)?;
                Ok(())
            }
            Stmt::Guard {
                condition, body, ..
            } => {
                self.infer_expr_dim(condition)?;
                self.validate_body_units(body)?;
                Ok(())
            }
            Stmt::MetaBlock { .. } => Ok(()),
            Stmt::PublicObservation { fields, .. } => {
                for field in fields {
                    self.infer_expr_dim(&field.value)?;
                }
                Ok(())
            }
            Stmt::Pragma { .. } => Ok(()),
        }
    }

    pub(super) fn infer_expr_dim(&self, expr: &Expr) -> Result<DimState, ParseError> {
        match &expr.kind {
            ExprKind::SumBranchValue { payload, .. } => {
                for payload in payload { self.infer_expr_dim(&payload.value)?; }
                Ok(DimState::Unknown)
            }
            ExprKind::ProductConstruction { fields, .. } => {
                for field in fields { self.infer_expr_dim(&field.expr)?; }
                Ok(DimState::Unknown)
            }
            ExprKind::Tensor(value) => {
                self.infer_expr_dim(value.input())?;
                Ok(DimState::Unknown)
            }
            ExprKind::MultiAxisSelection(value) => {
                for child in value.children() { self.infer_expr_dim(child)?; }
                Ok(DimState::Unknown)
            }
            ExprKind::Literal(lit) => match lit {
                Literal::Int(_) | Literal::Fixed64(_) => Ok(DimState::Known(UnitDim::NONE)),
                _ => Ok(DimState::Unknown),
            },
            ExprKind::Var(_) => Ok(DimState::Unknown),
            ExprKind::FieldAccess { target, .. } => {
                self.infer_expr_dim(target)?;
                Ok(DimState::Unknown)
            }
            ExprKind::Call { args, .. } => {
                for arg in args {
                    self.infer_expr_dim(&arg.expr)?;
                }
                Ok(DimState::Unknown)
            }
            ExprKind::CollectionConstructor { elements } | ExprKind::TupleConstructor { elements } => {
                for element in elements {
                    self.infer_expr_dim(element)?;
                }
                Ok(DimState::Unknown)
            }
            ExprKind::SelectionPlace { target, index } => {
                self.infer_expr_dim(target)?;
                self.infer_expr_dim(index)?;
                Ok(DimState::Unknown)
            }
            ExprKind::Thunk(body) => {
                self.validate_body_units(body)?;
                Ok(DimState::Unknown)
            }
            ExprKind::Eval { thunk, mode } => {
                self.infer_expr_dim(thunk)?;
                if matches!(mode, ThunkEvalMode::Bool | ThunkEvalMode::Not) {
                    Ok(DimState::Known(UnitDim::NONE))
                } else {
                    Ok(DimState::Unknown)
                }
            }
            ExprKind::Pipe { stages } => {
                for stage in stages {
                    self.infer_expr_dim(stage)?;
                }
                Ok(DimState::Unknown)
            }
            ExprKind::FlowValue => Ok(DimState::Unknown),
            ExprKind::Pack { fields } => {
                for (_, value) in fields {
                    self.infer_expr_dim(value)?;
                }
                Ok(DimState::Unknown)
            }
            ExprKind::Assertion(_) => Ok(DimState::Unknown),
            ExprKind::Formula(_) => Ok(DimState::Unknown),
            ExprKind::Template(_) => Ok(DimState::Unknown),
            ExprKind::StateMachine(_) => Ok(DimState::Unknown),
            ExprKind::TemplateRender { inject, .. } => {
                for (_, value) in inject {
                    self.infer_expr_dim(value)?;
                }
                Ok(DimState::Unknown)
            }
            ExprKind::FormulaEval { inject, .. } => {
                for (_, value) in inject {
                    self.infer_expr_dim(value)?;
                }
                Ok(DimState::Unknown)
            }
            ExprKind::SeedLiteral { body, .. } => {
                self.infer_expr_dim(body)?;
                Ok(DimState::Unknown)
            }
            ExprKind::Nuance { expr, .. } => self.infer_expr_dim(expr),
            ExprKind::Suffix { value, at } => match at {
                AtSuffix::Unit(unit) => {
                    let base_dim = self.infer_expr_dim(value)?;
                    let Some(spec) = unit_spec_from_symbol(unit) else {
                        return Ok(DimState::Unknown);
                    };
                    if let DimState::Known(dim) = base_dim {
                        if dim != UnitDim::NONE && dim != spec.dim {
                            return Err(self.unit_mismatch_error(expr.span, dim, spec.dim));
                        }
                    }
                    Ok(DimState::Known(spec.dim))
                }
                AtSuffix::Asset(_) => {
                    self.infer_expr_dim(value)?;
                    Ok(DimState::Unknown)
                }
            },
            ExprKind::Infix { left, op, right } => {
                let left_dim = self.infer_expr_dim(left)?;
                let right_dim = self.infer_expr_dim(right)?;
                match op.as_str() {
                    "+" | "-" | "%" => self.combine_add_sub(expr.span, left_dim, right_dim),
                    "*" => self.combine_mul(expr.span, left_dim, right_dim),
                    "/" => self.combine_div(expr.span, left_dim, right_dim),
                    "=:=" => Ok(DimState::Unknown),
                    "==" | "!=" | "<" | "<=" | ">" | ">=" => {
                        if let (DimState::Known(left), DimState::Known(right)) =
                            (left_dim, right_dim)
                        {
                            if left != right {
                                return Err(self.unit_mismatch_error(expr.span, left, right));
                            }
                        }
                        Ok(DimState::Known(UnitDim::NONE))
                    }
                    _ => Ok(DimState::Unknown),
                }
            }
        }
    }

    pub(super) fn combine_add_sub(
        &self,
        span: Span,
        left_dim: DimState,
        right_dim: DimState,
    ) -> Result<DimState, ParseError> {
        if let (DimState::Known(left), DimState::Known(right)) = (left_dim, right_dim) {
            if left != right {
                return Err(self.unit_mismatch_error(span, left, right));
            }
            return Ok(DimState::Known(left));
        }
        Ok(DimState::Unknown)
    }

    pub(super) fn combine_mul(
        &self,
        _span: Span,
        left_dim: DimState,
        right_dim: DimState,
    ) -> Result<DimState, ParseError> {
        if let (DimState::Known(left), DimState::Known(right)) = (left_dim, right_dim) {
            return Ok(DimState::Known(left.add(right)));
        }
        Ok(DimState::Unknown)
    }

    pub(super) fn combine_div(
        &self,
        _span: Span,
        left_dim: DimState,
        right_dim: DimState,
    ) -> Result<DimState, ParseError> {
        if let (DimState::Known(left), DimState::Known(right)) = (left_dim, right_dim) {
            return Ok(DimState::Known(left.sub(right)));
        }
        Ok(DimState::Unknown)
    }

    pub(super) fn unit_mismatch_error(&self, span: Span, left: UnitDim, right: UnitDim) -> ParseError {
        ParseError {
            span,
            message: format!(
                "단위 차원이 다릅니다: {} vs {}",
                left.format(),
                right.format()
            ),
        }
    }

    pub(super) fn apply_defaults_in_body(
        &mut self,
        body: &mut Body,
        signatures: &HashMap<String, Vec<ParamPin>>,
        known_seeds: &HashSet<String>,
    ) -> Result<(), ParseError> {
        self.apply_defaults_in_stmts(&mut body.stmts, signatures, known_seeds)
    }

    pub(super) fn apply_defaults_in_stmts(
        &mut self,
        stmts: &mut [Stmt],
        signatures: &HashMap<String, Vec<ParamPin>>,
        known_seeds: &HashSet<String>,
    ) -> Result<(), ParseError> {
        for stmt in stmts {
            match stmt {
                Stmt::SumDefinition { .. } => {}
                Stmt::SumMatch { subject, arms, default_body, .. } => {
                    self.apply_defaults_in_expr(subject, signatures, known_seeds)?;
                    for arm in arms { self.apply_defaults_in_body(&mut arm.body, signatures, known_seeds)?; }
                    if let Some(body) = default_body { self.apply_defaults_in_body(body, signatures, known_seeds)?; }
                }
                Stmt::DeclBlock { items, .. } => {
                    for item in items {
                        if let Some(value) = &mut item.value {
                            self.apply_defaults_in_expr(value, signatures, known_seeds)?;
                        }
                    }
                }
                Stmt::Definition { value, .. } | Stmt::DerivedBinding {value,..} => {
                    self.apply_defaults_in_expr(value, signatures, known_seeds)?;
                }
                Stmt::Mutate { target, value, .. } => {
                    self.apply_defaults_in_expr(target, signatures, known_seeds)?;
                    self.apply_defaults_in_expr(value, signatures, known_seeds)?;
                }
                Stmt::ResultFlow {
                    producer,
                    destination,
                    ..
                } => {
                    self.apply_defaults_in_expr(producer, signatures, known_seeds)?;
                    self.apply_defaults_in_expr(destination, signatures, known_seeds)?;
                }
                Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
                    self.apply_defaults_in_expr(expr, signatures, known_seeds)?
                }
                Stmt::Receive {
                    condition, body, ..
                } => {
                    if let Some(condition) = condition {
                        self.apply_defaults_in_expr(condition, signatures, known_seeds)?;
                    }
                    self.apply_defaults_in_body(body, signatures, known_seeds)?;
                }
                Stmt::Send {
                    sender,
                    payload,
                    receiver,
                    ..
                } => {
                    if let Some(sender) = sender {
                        self.apply_defaults_in_expr(sender, signatures, known_seeds)?;
                    }
                    self.apply_defaults_in_expr(payload, signatures, known_seeds)?;
                    self.apply_defaults_in_expr(receiver, signatures, known_seeds)?;
                }
                Stmt::Return { value, .. } => {
                    self.apply_defaults_in_expr(value, signatures, known_seeds)?
                }
                Stmt::If {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                    self.apply_defaults_in_expr(condition, signatures, known_seeds)?;
                    self.apply_defaults_in_body(then_body, signatures, known_seeds)?;
                    if let Some(body) = else_body {
                        self.apply_defaults_in_body(body, signatures, known_seeds)?;
                    }
                }
                Stmt::Try { action, body, .. } => {
                    self.apply_defaults_in_expr(action, signatures, known_seeds)?;
                    self.apply_defaults_in_body(body, signatures, known_seeds)?;
                }
                Stmt::Choose {
                    branches,
                    else_body,
                    ..
                } => {
                    for branch in branches.iter_mut() {
                        self.apply_defaults_in_expr(
                            &mut branch.condition,
                            signatures,
                            known_seeds,
                        )?;
                        self.apply_defaults_in_body(&mut branch.body, signatures, known_seeds)?;
                    }
                    self.apply_defaults_in_body(else_body, signatures, known_seeds)?;
                }
                Stmt::Repeat { body, .. } => {
                    self.apply_defaults_in_body(body, signatures, known_seeds)?;
                }
                Stmt::BeatBlock { body, .. } => {
                    self.apply_defaults_in_body(body, signatures, known_seeds)?;
                }
                Stmt::Hook { body, .. } => {
                    self.apply_defaults_in_body(body, signatures, known_seeds)?;
                }
                Stmt::HookWhenBecomes {
                    condition, body, ..
                }
                | Stmt::HookWhile {
                    condition, body, ..
                } => {
                    self.apply_defaults_in_expr(condition, signatures, known_seeds)?;
                    self.apply_defaults_in_body(body, signatures, known_seeds)?;
                }
                Stmt::While {
                    condition, body, ..
                } => {
                    self.apply_defaults_in_expr(condition, signatures, known_seeds)?;
                    self.apply_defaults_in_body(body, signatures, known_seeds)?;
                }
                Stmt::ForEach {
                    id,
                    item,
                    item_type,
                    iterable,
                    body,
                    ..
                } => {
                    let item_span = iterable.span;
                    self.apply_defaults_in_expr(iterable, signatures, known_seeds)?;
                    if self.natural_foreach_ids.contains(id) {
                        self.current_item_stack.push((
                            item.clone(),
                            item_type.clone().unwrap_or(TypeRef::Infer),
                            item_span,
                        ));
                        let result = self.apply_defaults_in_body(body, signatures, known_seeds);
                        self.current_item_stack.pop();
                        result?;
                    } else {
                        self.apply_defaults_in_body(body, signatures, known_seeds)?;
                    }
                }
                Stmt::Quantifier { body, .. } => {
                    self.apply_defaults_in_body(body, signatures, known_seeds)?;
                }
                Stmt::Break { .. } | Stmt::ContinueLoop { .. } => {}
                Stmt::Contract {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                    self.apply_defaults_in_expr(condition, signatures, known_seeds)?;
                    if let Some(body) = then_body {
                        self.apply_defaults_in_body(body, signatures, known_seeds)?;
                    }
                    self.apply_defaults_in_body(else_body, signatures, known_seeds)?;
                }
                Stmt::Guard {
                    condition, body, ..
                } => {
                    self.apply_defaults_in_expr(condition, signatures, known_seeds)?;
                    self.apply_defaults_in_body(body, signatures, known_seeds)?;
                }
                Stmt::MetaBlock { .. } => {}
                Stmt::PublicObservation { fields, .. } => {
                    for field in fields {
                        self.apply_defaults_in_expr(&mut field.value, signatures, known_seeds)?;
                    }
                }
                Stmt::Pragma { .. } => {}
            }
        }
        Ok(())
    }

    pub(super) fn apply_defaults_in_expr(
        &mut self,
        expr: &mut Expr,
        signatures: &HashMap<String, Vec<ParamPin>>,
        known_seeds: &HashSet<String>,
    ) -> Result<(), ParseError> {
        match &mut expr.kind {
            ExprKind::SumBranchValue {payload,..} => {
                for value in payload {self.apply_defaults_in_expr(&mut value.value,signatures,known_seeds)?;}
            }
            ExprKind::Call { args, func } => {
                for arg in args.iter_mut() {
                    self.apply_defaults_in_expr(&mut arg.expr, signatures, known_seeds)?;
                }
                if let Some((definition, schema)) = self.product_definitions.get(func) {
                    let category=self.schema_categories.get(func).copied();
                    if category.is_some()&&!self.schema_construction_nodes.contains(&expr.id) {
                        return Err(ParseError{span:expr.span,message:"E_SCHEMA_CONSTRUCTION_SURFACE: 명목 구성은 '(성분=값)인 이름'으로 쓰세요".into()});
                    }
                    let mut seen = HashSet::new();
                    for arg in args.iter() {
                        let pin = arg.resolved_pin.as_ref().ok_or_else(|| ParseError {
                            span: arg.span, message: "E_PRODUCT_NAMED_FIELD_REQUIRED: 구성할 필드 이름을 명시하세요".into()
                        })?;
                        if arg.josa.is_some() || !schema.iter().any(|field| field.pin_name == *pin) {
                            return Err(ParseError { span: arg.span, message:
                                format!("E_PRODUCT_FIELD_UNKNOWN: `{func}`의 필드 `{pin}`가 아닙니다") });
                        }
                        if !seen.insert(pin) {
                            return Err(ParseError { span: arg.span, message:
                                format!("E_PRODUCT_FIELD_DUPLICATE: 필드 `{pin}`가 중복되었습니다") });
                        }
                        let expected = &schema.iter().find(|field| field.pin_name == *pin).unwrap().type_ref;
                        let literal = match &arg.expr.kind {
                            ExprKind::Literal(Literal::Int(value)) => i32::try_from(*value).ok()
                                .map(|value| crate::runtime::Value::Fixed64(Fixed64::from_i32(value))),
                            ExprKind::Literal(Literal::Fixed64(value)) => Some(crate::runtime::Value::Fixed64(*value)),
                            ExprKind::Literal(Literal::String(value) | Literal::Atom(value)) => Some(crate::runtime::Value::String(value.clone())),
                            ExprKind::Literal(Literal::Bool(value)) => Some(crate::runtime::Value::Bool(*value)),
                            ExprKind::Literal(Literal::None) => Some(crate::runtime::Value::None),
                            _ => None,
                        };
                        if let Some(literal) = literal {
                            if let Err(detail) = crate::runtime::check_value_type(&literal, expected) {
                                return Err(ParseError { span: arg.expr.span, message: format!(
                                    "E_RUNTIME_TYPE_MISMATCH: 필드={pin} 기대={} 실제={}", detail.expected, detail.actual) });
                            }
                        }
                    }
                    if seen.len() != schema.len() {
                        return Err(ParseError { span: expr.span, message:
                            format!("E_PRODUCT_FIELD_MISSING: `{func}`의 모든 필드를 명시해야 합니다") });
                    }
                    // Retain authored value evaluation order. Record identity
                    // is owned by the schema, never declaration/source order.
                    expr.kind = ExprKind::ProductConstruction {
                        owner: func.clone(), definition: *definition, category, fields: args.clone(),
                    };
                    return Ok(());
                }
                if self.source_epoch==SourceEpoch::V25&&func=="조작단계"&&!signatures.contains_key(func) {
                    if args.len()!=1||args[0].resolved_pin.as_deref()!=Some("순서")||args[0].josa.is_some(){
                        return Err(ParseError{span:expr.span,message:"E_INPUT_PHASE_PIN: '(순서=원순번) 조작단계'의 필수 받는자리 하나를 명시하세요".into()});
                    }
                    self.manipulation_phase_calls.insert(expr.id,ManipulationPhaseCallBinding{
                        definition:"B1.DEF.INPUT_PHASE.R1",ordinal_pin:"B1.PIN.INPUT_PHASE.ORDINAL.R1"});
                    return Ok(());
                }
                if self.source_epoch==SourceEpoch::V25&&func=="폐기하기" {
                    if signatures.contains_key(func){return Err(ParseError{span:expr.span,message:"E_VC04_RETIRE_AMBIGUOUS: 폐기하기의 사용자 정의와 등록 정의가 경쟁합니다".into()});}
                    if args.len()!=1||args[0].resolved_pin.as_deref()!=Some("대상")||args[0].josa.is_some(){return Err(ParseError{span:expr.span,message:"E_VC04_RETIRE_PIN: '(대상=표적) 폐기하기'의 필수 받는자리 하나만 사용하세요".into()});}
                    self.retirement_commands.insert(expr.id,RetirementCommandBinding{definition:"VC04.DEF.PYEGIHAGI.R1",target_pin:"VC04.PIN.PYEGIHAGI.TARGET.R1"});
                    return Ok(());
                }
                if self.source_epoch==SourceEpoch::V25&&matches!(func.as_str(),"가리키기"|"읽기") {
                    if signatures.contains_key(func){return Err(ParseError{span:expr.span,message:format!("E_VC04_REFERENCE_AMBIGUOUS: 사용자 정의와 등록 `{func}`가 경쟁합니다")});}
                    if args.len()!=1||args[0].resolved_pin.is_some()||args[0].josa.is_some(){return Err(ParseError{span:expr.span,message:format!("E_VC04_REFERENCE_RECEIVER: '{} {func}'의 값 받는자리 하나만 사용하세요",if func=="읽기"{"가리킴값"}else{"임자값"})});}
                    let binding=if func=="가리키기" {SubjectReferenceCallBinding{definition:"VC04.DEF.GARIKIGI.R1",receiver_pin:"VC04.PIN.GARIKIGI.TARGET.R1"}}
                        else {SubjectReferenceCallBinding{definition:"VC04.DEF.ILGI.R1",receiver_pin:"VC04.PIN.ILGI.REFERENCE.R1"}};
                    self.subject_reference_calls.insert(expr.id,binding);
                    return Ok(());
                }
                let func_name = func.clone();
                self.record_registered_call(expr.id, expr.span, &func_name);
                let resolved = self.resolve_call_target(&func_name, known_seeds, expr.span)?;
                if *func != resolved.1 {
                    *func = resolved.1;
                }
                if self.is_transform_call(&resolved.0) {
                    self.normalize_transform_call(args, &resolved.0, expr.span)?;
                } else {
                    self.apply_defaults_to_call(args, &resolved.0, expr.span, signatures)?;
                }
            }
            ExprKind::FieldAccess { target, .. } => {
                self.apply_defaults_in_expr(target, signatures, known_seeds)?;
            }
            ExprKind::CollectionConstructor { elements } | ExprKind::TupleConstructor { elements } => {
                for element in elements {
                    self.apply_defaults_in_expr(element, signatures, known_seeds)?;
                }
            }
            ExprKind::SelectionPlace { target, index } => {
                self.apply_defaults_in_expr(target, signatures, known_seeds)?;
                self.apply_defaults_in_expr(index, signatures, known_seeds)?;
            }
            ExprKind::Infix { left, right, .. } => {
                self.apply_defaults_in_expr(left, signatures, known_seeds)?;
                self.apply_defaults_in_expr(right, signatures, known_seeds)?;
            }
            ExprKind::Suffix { value, .. } => {
                self.apply_defaults_in_expr(value, signatures, known_seeds)?;
            }
            ExprKind::Thunk(body) => {
                self.apply_defaults_in_body(body, signatures, known_seeds)?;
            }
            ExprKind::Eval { thunk, .. } => {
                self.apply_defaults_in_expr(thunk, signatures, known_seeds)?;
            }
            ExprKind::Pipe { stages } => {
                self.apply_defaults_in_pipe(stages, signatures, known_seeds)?;
            }
            ExprKind::Pack { fields } => {
                for (_, value) in fields.iter_mut() {
                    self.apply_defaults_in_expr(value, signatures, known_seeds)?;
                }
            }
            ExprKind::ProductConstruction { fields, .. } => {
                for field in fields {
                    self.apply_defaults_in_expr(&mut field.expr, signatures, known_seeds)?;
                }
            }
            ExprKind::TemplateRender { inject, .. } => {
                for (_, value) in inject.iter_mut() {
                    self.apply_defaults_in_expr(value, signatures, known_seeds)?;
                }
            }
            ExprKind::FormulaEval { inject, .. } => {
                for (_, value) in inject.iter_mut() {
                    self.apply_defaults_in_expr(value, signatures, known_seeds)?;
                }
            }
            ExprKind::Nuance { expr, .. } => {
                self.apply_defaults_in_expr(expr, signatures, known_seeds)?;
            }
            _ => {}
        }
        self.bind_bytes_call(expr)?;
        Ok(())
    }

    pub(super) fn bind_bytes_call(&mut self, expr: &Expr) -> Result<(), ParseError> {
        if let ExprKind::Call { func, args } = &expr.kind {
            if let Some(operation) = crate::bytes::BytesConversion::from_registered_surface(func) {
                operation.validate(args, expr.span)?;
                self.bytes_calls.insert(expr.id, operation);
            }
        }
        Ok(())
    }

    pub(super) fn apply_defaults_in_pipe(
        &mut self,
        stages: &mut Vec<Expr>,
        signatures: &HashMap<String, Vec<ParamPin>>,
        known_seeds: &HashSet<String>,
    ) -> Result<(), ParseError> {
        for index in 0..stages.len() {
            let prev_is_template = if index > 0 {
                matches!(stages[index - 1].kind, ExprKind::Template(_))
            } else {
                false
            };
            let stage = &mut stages[index];
            if matches!(&stage.kind, ExprKind::Call { func, .. } if self.product_definitions.contains_key(func)) {
                // Explicit Product fields have no implicit flow/current-item pin.
                self.apply_defaults_in_expr(stage, signatures, known_seeds)?;
                continue;
            }
            match &mut stage.kind {
                ExprKind::Call { args, func } => {
                    for arg in args.iter_mut() {
                        self.apply_defaults_in_expr(&mut arg.expr, signatures, known_seeds)?;
                    }
                    let func_name = func.clone();
                    self.record_registered_call(stage.id, stage.span, &func_name);
                    let resolved = self.resolve_call_target(&func_name, known_seeds, stage.span)?;
                    if *func != resolved.1 {
                        *func = resolved.1;
                    }
                    if index > 0 && resolved.0 == "채우기" && prev_is_template {
                        return Err(ParseError {
                            span: stage.span,
                            message: "Gate0: 글무늬{...} 해서 (키=값, ...) 채우기는 금지입니다. (<키=값, ...>) 글무늬{...}를 사용하세요".to_string(),
                        });
                    }
                    if index > 0 {
                        if let Some(params) = signatures.get(&resolved.0) {
                            self.inject_flow_into_call(args, params, stage.span)?;
                        } else if self.is_transform_call(&resolved.0) {
                            self.inject_flow_into_transform(args, &resolved.0, stage.span)?;
                        }
                    }
                    if self.is_transform_call(&resolved.0) {
                        self.normalize_transform_call(args, &resolved.0, stage.span)?;
                    } else {
                        self.apply_defaults_to_call(args, &resolved.0, stage.span, signatures)?;
                    }
                }
                _ => {
                    self.apply_defaults_in_expr(stage, signatures, known_seeds)?;
                }
            }
            self.bind_bytes_call(stage)?;
        }
        Ok(())
    }

    pub(super) fn inject_flow_into_call(
        &mut self,
        args: &mut Vec<ArgBinding>,
        params: &[ParamPin],
        span: Span,
    ) -> Result<(), ParseError> {
        if args
            .iter()
            .any(|arg| matches!(arg.binding_reason, BindingReason::FlowInjected))
        {
            return Ok(());
        }
        let used = self.collect_used_pins_for_flow(args, params, span)?;
        let mut object_candidates = Vec::new();
        for (idx, param) in params.iter().enumerate() {
            if used[idx] {
                continue;
            }
            if param.josa_list.iter().any(|j| j == "을" || j == "를") {
                object_candidates.push(idx);
            }
        }
        let target = if object_candidates.len() == 1 {
            Some(object_candidates[0])
        } else if object_candidates.len() > 1 {
            return Err(ParseError {
                span,
                message: "PIPE-FLOW-INJECT-AMBIGUOUS: 흐름값 주입 대상이 모호합니다".to_string(),
            });
        } else {
            let mut missing_required = Vec::new();
            for (idx, param) in params.iter().enumerate() {
                if used[idx] {
                    continue;
                }
                if !param.optional && param.default_value.is_none() {
                    missing_required.push(idx);
                }
            }
            if missing_required.len() == 1 {
                Some(missing_required[0])
            } else if missing_required.len() > 1 {
                return Err(ParseError {
                    span,
                    message: "PIPE-FLOW-INJECT-AMBIGUOUS: 흐름값 주입 대상이 모호합니다"
                        .to_string(),
                });
            } else {
                None
            }
        };

        if let Some(idx) = target {
            let expr = Expr::new(self.next_id(), span, ExprKind::FlowValue);
            args.push(ArgBinding {
                id: self.next_id(),
                span,
                expr,
                josa: None,
                resolved_pin: Some(params[idx].pin_name.clone()),
                binding_reason: BindingReason::FlowInjected,
            });
        }
        Ok(())
    }

    pub(super) fn is_transform_call(&self, func: &str) -> bool {
        matches!(func, "채우기" | "풀기")
    }

    pub(super) fn inject_flow_into_transform(
        &mut self,
        args: &mut Vec<ArgBinding>,
        func: &str,
        span: Span,
    ) -> Result<(), ParseError> {
        if args
            .iter()
            .any(|arg| matches!(arg.binding_reason, BindingReason::FlowInjected))
        {
            return Ok(());
        }
        let expr = Expr::new(self.next_id(), span, ExprKind::FlowValue);
        args.insert(
            0,
            ArgBinding {
                id: self.next_id(),
                span,
                expr,
                josa: None,
                resolved_pin: if func == "채우기" {
                    Some("무늬".to_string())
                } else {
                    Some("식".to_string())
                },
                binding_reason: BindingReason::FlowInjected,
            },
        );
        Ok(())
    }

    pub(super) fn normalize_transform_call(
        &mut self,
        args: &mut Vec<ArgBinding>,
        func: &str,
        span: Span,
    ) -> Result<(), ParseError> {
        let mut value_expr: Option<Expr> = None;
        let mut pack_expr: Option<Expr> = None;
        let mut pack_fields: Vec<(String, Expr)> = Vec::new();
        let mut seen_keys = HashSet::new();

        for arg in args.drain(..) {
            if matches!(arg.binding_reason, BindingReason::FlowInjected) {
                if value_expr.is_some() {
                    return Err(ParseError {
                        span,
                        message: format!("{}: 흐름값이 중복되었습니다", func),
                    });
                }
                value_expr = Some(arg.expr);
                continue;
            }
            if let Some(pin) = &arg.resolved_pin {
                if pin == "무늬" || pin == "식" {
                    if value_expr.is_some() {
                        return Err(ParseError {
                            span,
                            message: format!("{}: 값 인자가 중복되었습니다", func),
                        });
                    }
                    value_expr = Some(arg.expr);
                    continue;
                }
                if pin == "주입" {
                    if pack_expr.is_some() || !pack_fields.is_empty() {
                        return Err(ParseError {
                            span,
                            message: format!("{}: 주입은 하나만 허용됩니다", func),
                        });
                    }
                    pack_expr = Some(arg.expr);
                    continue;
                }
                if seen_keys.contains(pin) {
                    return Err(ParseError {
                        span,
                        message: format!("{}: 키 '{}'가 중복되었습니다", func, pin),
                    });
                }
                seen_keys.insert(pin.clone());
                pack_fields.push((pin.clone(), arg.expr));
                continue;
            }
            if value_expr.is_none() {
                value_expr = Some(arg.expr);
                continue;
            }
            return Err(ParseError {
                span,
                message: format!("{}: 인자는 값:핀 또는 파이프 값만 허용됩니다", func),
            });
        }

        let Some(value_expr) = value_expr else {
            return Err(ParseError {
                span,
                message: format!("{}: 값 인자가 필요합니다", func),
            });
        };
        if func == "채우기" && matches!(value_expr.kind, ExprKind::Template(_)) {
            return Err(ParseError {
                span: value_expr.span,
                message: "Gate0: 글무늬{...} 채우기는 금지입니다. (<키=값, ...>) 글무늬{...}를 사용하세요".to_string(),
            });
        }
        let pack_expr = if let Some(pack_expr) = pack_expr {
            pack_expr
        } else {
            Expr::new(
                self.next_id(),
                span,
                ExprKind::Pack {
                    fields: pack_fields,
                },
            )
        };

        let mut value_arg = self.new_arg_binding(value_expr);
        value_arg.resolved_pin = Some(if func == "채우기" { "무늬" } else { "식" }.to_string());
        let mut pack_arg = self.new_arg_binding(pack_expr);
        pack_arg.resolved_pin = Some("주입".to_string());
        args.push(value_arg);
        args.push(pack_arg);
        Ok(())
    }

    pub(super) fn collect_used_pins_for_flow(
        &self,
        args: &[ArgBinding],
        params: &[ParamPin],
        span: Span,
    ) -> Result<Vec<bool>, ParseError> {
        let mut used = vec![false; params.len()];
        let mut positional = Vec::new();

        for arg in args {
            if let Some(fixed_pin) = &arg.resolved_pin {
                let idx = params
                    .iter()
                    .position(|p| p.pin_name == *fixed_pin)
                    .ok_or_else(|| ParseError {
                        span,
                        message: format!("핀 '{}'을(를) 찾을 수 없습니다", fixed_pin),
                    })?;
                if used[idx] {
                    return Err(ParseError {
                        span,
                        message: format!("핀 '{}' 인자가 중복되었습니다", params[idx].pin_name),
                    });
                }
                if let Some(josa) = &arg.josa {
                    if !params[idx].josa_list.iter().any(|j| j == josa) {
                        return Err(ParseError {
                            span,
                            message: format!(
                                "핀 '{}'에 조사 '{}'를 사용할 수 없습니다",
                                params[idx].pin_name, josa
                            ),
                        });
                    }
                }
                used[idx] = true;
                continue;
            }
            if let Some(josa) = &arg.josa {
                let candidates: Vec<usize> = params
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.josa_list.iter().any(|j| j == josa))
                    .map(|(i, _)| i)
                    .collect();
                if candidates.is_empty() {
                    return Err(ParseError {
                        span,
                        message: format!("조사 '{}'에 해당하는 핀이 없습니다", josa),
                    });
                }
                if candidates.len() > 1 {
                    return Err(ParseError {
                        span,
                        message: format!("조사 '{}'가 모호합니다: 값:핀으로 고정하세요", josa),
                    });
                }
                let idx = candidates[0];
                if used[idx] {
                    return Err(ParseError {
                        span,
                        message: format!("핀 '{}' 인자가 중복되었습니다", params[idx].pin_name),
                    });
                }
                used[idx] = true;
                continue;
            }
            positional.push(arg);
        }

        let mut pos_iter = positional.into_iter();
        for used_slot in used.iter_mut() {
            if *used_slot {
                continue;
            }
            if pos_iter.next().is_some() {
                *used_slot = true;
            } else {
                break;
            }
        }
        Ok(used)
    }

    pub(super) fn resolve_call_target(
        &self,
        func: &str,
        known_seeds: &HashSet<String>,
        span: Span,
    ) -> Result<(String, String), ParseError> {
        if let Some(owner) = self.registered_forms.get(func) {
            return Ok((owner.clone(), owner.clone()));
        }
        let resolved = resolve_known_call_target(func, known_seeds, span)?;
        if crate::bytes::BytesConversion::from_registered_surface(&resolved.0).is_some()
            && func != resolved.0 {
            return Err(ParseError {
                span,
                message: format!("E_REGISTERED_FORM_REQUIRED: '{}'은 등록된 UTF-8 변환 표면이 아닙니다", func),
            });
        }
        if self.explicitly_registered_owners.contains(&resolved.0) {
            return Err(ParseError {
                span,
                message: format!(
                    "E_REGISTERED_FORM_REQUIRED: '{}'은 명시적으로 등록된 호출 표면이 아닙니다",
                    func
                ),
            });
        }
        Ok(resolved)
    }

    pub(super) fn record_registered_call(&mut self, id: NodeId, span: Span, form: &str) {
        if let Some(candidates) = self.registered_form_candidates.get(form) {
            self.registered_call_candidates
                .insert(id, candidates.clone());
            self.registered_call_spans.insert(id, span);
        }
    }

    pub(super) fn apply_defaults_to_call(
        &mut self,
        args: &mut Vec<ArgBinding>,
        func: &str,
        span: Span,
        signatures: &HashMap<String, Vec<ParamPin>>,
    ) -> Result<(), ParseError> {
        let Some(params) = signatures.get(func) else {
            return Ok(());
        };
        let authored_arg_count = args.len();
        let mut bound = vec![None; params.len()];
        let mut used = vec![false; params.len()];
        let mut positional = Vec::new();

        for mut arg in args.drain(..) {
            if let Some(fixed_pin) = &arg.resolved_pin {
                let idx = params
                    .iter()
                    .position(|p| p.pin_name == *fixed_pin)
                    .ok_or_else(|| ParseError {
                        span,
                        message: format!("핀 '{}'을 찾을 수 없습니다", fixed_pin),
                    })?;
                if used[idx] {
                    return Err(ParseError {
                        span,
                        message: format!("핀 '{}'에 인자가 중복되었습니다", params[idx].pin_name),
                    });
                }
                if let Some(josa) = &arg.josa {
                    if !params[idx].josa_list.iter().any(|j| j == josa) {
                        return Err(ParseError {
                            span,
                            message: format!(
                                "핀 '{}'에 조사 '{}'는 허용되지 않습니다",
                                params[idx].pin_name, josa
                            ),
                        });
                    }
                }
                if !matches!(arg.binding_reason, BindingReason::FlowInjected) {
                    arg.binding_reason = BindingReason::UserFixed;
                }
                bound[idx] = Some(arg);
                used[idx] = true;
                continue;
            }
            if let Some(josa) = arg.josa.clone() {
                let candidates: Vec<usize> = params
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.josa_list.iter().any(|j| j == &josa))
                    .map(|(i, _)| i)
                    .collect();
                if candidates.is_empty() {
                    let mut arg = arg;
                    let _ = self.merge_josa_into_expr(&mut arg.expr, &josa);
                    arg.josa = None;
                    if !matches!(arg.binding_reason, BindingReason::FlowInjected) {
                        arg.binding_reason = BindingReason::Positional;
                    }
                    positional.push(arg);
                    continue;
                }
                if candidates.len() > 1 {
                    return Err(ParseError {
                        span,
                        message: format!(
                            "조사 '{}'가 모호합니다. 값:핀 또는 ~조사로 고정하세요",
                            josa
                        ),
                    });
                }
                let idx = candidates[0];
                if used[idx] {
                    return Err(ParseError {
                        span,
                        message: format!("핀 '{}'에 인자가 중복되었습니다", params[idx].pin_name),
                    });
                }
                let mut arg = arg;
                arg.resolved_pin = Some(params[idx].pin_name.clone());
                arg.binding_reason = if matches!(arg.binding_reason, BindingReason::UserFixed) {
                    BindingReason::UserFixed
                } else {
                    BindingReason::Dictionary
                };
                bound[idx] = Some(arg);
                used[idx] = true;
            } else {
                positional.push(arg);
            }
        }

        let mut pos_iter = positional.into_iter();
        let mut missing_required = Vec::new();
        for (idx, slot) in bound.iter_mut().enumerate() {
            if slot.is_some() {
                continue;
            }
            if let Some(mut arg) = pos_iter.next() {
                arg.resolved_pin = Some(params[idx].pin_name.clone());
                arg.binding_reason = BindingReason::Positional;
                *slot = Some(arg);
                continue;
            }
            if let Some(default_value) = &params[idx].default_value {
                let mut arg = self.new_arg_binding(default_value.clone());
                arg.resolved_pin = Some(params[idx].pin_name.clone());
                *slot = Some(arg);
                continue;
            }
            if params[idx].optional {
                let expr = Expr::new(self.next_id(), span, ExprKind::Literal(Literal::None));
                let mut arg = self.new_arg_binding(expr);
                arg.resolved_pin = Some(params[idx].pin_name.clone());
                *slot = Some(arg);
                continue;
            }
            missing_required.push(idx);
        }

        if pos_iter.next().is_some() {
            return Err(ParseError {
                span,
                message: "인자 수가 너무 많습니다".to_string(),
            });
        }

        if let Some((item_name, item_type, item_span)) = self.current_item_stack.last().cloned() {
            if missing_required.is_empty() && authored_arg_count == 0 {
                return Err(ParseError {
                    span,
                    message: "E_CURRENT_ITEM_NO_REQUIRED_PIN: current item을 받을 미충족 필수 pin이 없습니다".to_string(),
                });
            }
            if !missing_required.is_empty() {
                if missing_required.len() != 1 {
                    let pins = missing_required
                        .iter()
                        .map(|idx| params[*idx].pin_name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    return Err(ParseError {
                        span,
                        message: format!(
                            "E_CURRENT_ITEM_AMBIGUOUS: 현재 항목 결속 후보가 하나가 아닙니다: {pins}"
                        ),
                    });
                }
                let idx = missing_required[0];
                let expected = &params[idx].type_ref;
                if matches!(expected, TypeRef::Function { .. }) {
                    return Err(ParseError {
                        span,
                        message: format!(
                            "E_CURRENT_ITEM_OWNERSHIP_UNSAFE: 함수형 핀 '{}'에는 current item을 자동 결속하지 않습니다",
                            params[idx].pin_name
                        ),
                    });
                }
                if !Self::current_item_type_compatible(&item_type, expected) {
                    return Err(ParseError {
                        span,
                        message: format!(
                            "E_CURRENT_ITEM_TYPE_MISMATCH: current item은 핀 '{}' 타입과 안전하게 맞지 않습니다",
                            params[idx].pin_name
                        ),
                    });
                }
                let expr = Expr::new(self.next_id(), item_span, ExprKind::Var(item_name));
                bound[idx] = Some(ArgBinding {
                    id: self.next_id(),
                    span,
                    expr,
                    josa: None,
                    resolved_pin: Some(params[idx].pin_name.clone()),
                    binding_reason: BindingReason::ContextItemInjected,
                });
            }
        } else if !missing_required.is_empty() {
            let pin = &params[missing_required[0]].pin_name;
            return Err(ParseError {
                span,
                message: format!("E_CALL_REQUIRED_PIN_MISSING: 필수 인자 '{pin}' 누락"),
            });
        }

        *args = bound.into_iter().flatten().collect();
        Ok(())
    }

    pub(super) fn current_item_type_compatible(actual: &TypeRef, expected: &TypeRef) -> bool {
        match (actual, expected) {
            (_, TypeRef::Infer) => true,
            (TypeRef::Infer, _) => false,
            _ => actual == expected,
        }
    }

    pub(super) fn merge_josa_into_expr(&self, expr: &mut Expr, josa: &str) -> bool {
        match &mut expr.kind {
            ExprKind::Var(name) => {
                name.push_str(josa);
                true
            }
            ExprKind::FieldAccess { field, .. } => {
                field.push_str(josa);
                true
            }
            _ => false,
        }
    }

}
