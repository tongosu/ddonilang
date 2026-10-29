//! Existing port graph validation with its private carriers; no registry or graph algorithm changes.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PortRef {
    instance: String,
    port: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct JjaimPortRegistry {
    input_ports: HashSet<String>,
    output_ports: HashSet<String>,
    input_port_types: HashMap<String, String>,
    output_port_types: HashMap<String, String>,
}

impl JjaimPortRegistry {
    fn has_any(&self) -> bool {
        !(self.input_ports.is_empty() && self.output_ports.is_empty())
    }
}

#[derive(Debug, Default, Clone)]
struct JjaimPortSchema {
    by_type: HashMap<String, JjaimPortRegistry>,
}

impl JjaimPortSchema {
    fn registry_for_type(&self, type_name: &str) -> Option<&JjaimPortRegistry> {
        self.by_type.get(type_name)
    }
}

pub(super) fn build_guseong_flatten_plan_surface(
    stmts: &[SurfaceStmt],
) -> Result<GuseongFlattenPlan, CanonError> {
    let mut declared_instances: HashSet<String> = HashSet::new();
    let mut instance_types: HashMap<String, String> = HashMap::new();
    let mut links: Vec<(PortRef, PortRef)> = Vec::new();
    let mut edges: HashMap<String, HashSet<String>> = HashMap::new();
    let mut sink_sources: HashMap<PortRef, PortRef> = HashMap::new();
    let mut linked_instances: HashSet<String> = HashSet::new();
    let mut nodes: BTreeSet<String> = BTreeSet::new();

    collect_guseong_from_surface_stmts(
        stmts,
        &mut declared_instances,
        &mut instance_types,
        &mut links,
        &mut edges,
        &mut sink_sources,
        &mut linked_instances,
        &mut nodes,
        0,
    )?;

    let port_schema = collect_jjaim_port_schema(stmts, &instance_types)?;

    let mut undeclared: Vec<String> = linked_instances
        .difference(&declared_instances)
        .cloned()
        .collect();
    undeclared.sort();
    if !undeclared.is_empty() {
        return Err(CanonError::new(
            "E_GUSEONG_INSTANCE_UNDECLARED",
            format!(
                "짜임 연결에 선언되지 않은 인스턴스가 있습니다: {}",
                undeclared.join(", ")
            ),
        ));
    }

    validate_guseong_port_links(&links, &instance_types, &port_schema)?;

    let topo_order = topo_sort_or_cycle(&edges, &nodes)?;
    let mut instances: Vec<GuseongFlattenInstance> = declared_instances
        .iter()
        .map(|name| GuseongFlattenInstance {
            name: name.clone(),
            type_name: instance_types
                .get(name)
                .cloned()
                .unwrap_or_else(|| "UNKNOWN".to_string()),
        })
        .collect();
    instances.sort_by(|a, b| {
        a.name
            .cmp(&b.name)
            .then_with(|| a.type_name.cmp(&b.type_name))
    });

    let mut plan_links: Vec<GuseongFlattenLink> = links
        .iter()
        .map(|(dst, src)| GuseongFlattenLink {
            dst_instance: dst.instance.clone(),
            dst_port: dst.port.clone(),
            src_instance: src.instance.clone(),
            src_port: src.port.clone(),
        })
        .collect();
    plan_links.sort_by(|a, b| {
        a.dst_instance
            .cmp(&b.dst_instance)
            .then_with(|| a.dst_port.cmp(&b.dst_port))
            .then_with(|| a.src_instance.cmp(&b.src_instance))
            .then_with(|| a.src_port.cmp(&b.src_port))
    });

    Ok(GuseongFlattenPlan {
        topo_order,
        instances,
        links: plan_links,
    })
}

fn collect_guseong_from_surface_stmts(
    stmts: &[SurfaceStmt],
    declared_instances: &mut HashSet<String>,
    instance_types: &mut HashMap<String, String>,
    links: &mut Vec<(PortRef, PortRef)>,
    edges: &mut HashMap<String, HashSet<String>>,
    sink_sources: &mut HashMap<PortRef, PortRef>,
    linked_instances: &mut HashSet<String>,
    nodes: &mut BTreeSet<String>,
    link_scope_depth: usize,
) -> Result<(), CanonError> {
    for stmt in stmts {
        match stmt {
            SurfaceStmt::Assign { target, value } => {
                if let Some(instance_name) = extract_instance_decl_target(target) {
                    if matches!(value, Expr::CallIn { .. }) {
                        if !declared_instances.insert(instance_name.clone()) {
                            return Err(CanonError::new(
                                "E_GUSEONG_INSTANCE_DUP",
                                format!("짜임 인스턴스 이름 중복: {}", instance_name),
                            ));
                        }
                        if let Expr::CallIn { name, .. } = value {
                            instance_types.insert(instance_name.clone(), name.clone());
                        }
                        nodes.insert(instance_name);
                    }
                }

                let Some((dst, src)) = extract_link_from_assign(target, value) else {
                    continue;
                };
                if link_scope_depth > 0 {
                    return Err(CanonError::new(
                        "E_GUSEONG_LINK_TOPLEVEL_ONLY",
                        format!(
                            "짜임 연결은 최상위 문장에서만 허용됩니다: {}.{} <- {}.{}",
                            dst.instance, dst.port, src.instance, src.port
                        ),
                    ));
                }
                links.push((dst.clone(), src.clone()));

                nodes.insert(src.instance.clone());
                nodes.insert(dst.instance.clone());
                linked_instances.insert(src.instance.clone());
                linked_instances.insert(dst.instance.clone());
                edges
                    .entry(src.instance.clone())
                    .or_default()
                    .insert(dst.instance.clone());

                if let Some(existing_src) = sink_sources.get(&dst) {
                    if existing_src != &src {
                        return Err(CanonError::new(
                            "E_GUSEONG_PORT_DIR",
                            format!(
                                "입력 포트 다중 연결: {}.{} <- {}.{} / {}.{}",
                                dst.instance,
                                dst.port,
                                existing_src.instance,
                                existing_src.port,
                                src.instance,
                                src.port
                            ),
                        ));
                    }
                } else {
                    sink_sources.insert(dst, src);
                }
            }
            SurfaceStmt::SeedDef { body, .. } => collect_guseong_from_surface_stmts(
                body,
                declared_instances,
                instance_types,
                links,
                edges,
                sink_sources,
                linked_instances,
                nodes,
                link_scope_depth + 1,
            )?,
            SurfaceStmt::Receive { body, .. } => collect_guseong_from_surface_stmts(
                body,
                declared_instances,
                instance_types,
                links,
                edges,
                sink_sources,
                linked_instances,
                nodes,
                link_scope_depth + 1,
            )?,
            SurfaceStmt::If {
                then_body,
                else_body,
                ..
            } => {
                collect_guseong_from_surface_stmts(
                    then_body,
                    declared_instances,
                    instance_types,
                    links,
                    edges,
                    sink_sources,
                    linked_instances,
                    nodes,
                    link_scope_depth + 1,
                )?;
                if let Some(body) = else_body {
                    collect_guseong_from_surface_stmts(
                        body,
                        declared_instances,
                        instance_types,
                        links,
                        edges,
                        sink_sources,
                        linked_instances,
                        nodes,
                        link_scope_depth + 1,
                    )?;
                }
            }
            SurfaceStmt::Contract {
                then_body,
                else_body,
                ..
            } => {
                if let Some(body) = then_body {
                    collect_guseong_from_surface_stmts(
                        body,
                        declared_instances,
                        instance_types,
                        links,
                        edges,
                        sink_sources,
                        linked_instances,
                        nodes,
                        link_scope_depth + 1,
                    )?;
                }
                collect_guseong_from_surface_stmts(
                    else_body,
                    declared_instances,
                    instance_types,
                    links,
                    edges,
                    sink_sources,
                    linked_instances,
                    nodes,
                    link_scope_depth + 1,
                )?;
            }
            SurfaceStmt::Choose {
                branches,
                else_body,
                ..
            } => {
                for branch in branches {
                    collect_guseong_from_surface_stmts(
                        &branch.body,
                        declared_instances,
                        instance_types,
                        links,
                        edges,
                        sink_sources,
                        linked_instances,
                        nodes,
                        link_scope_depth + 1,
                    )?;
                }
                if let Some(else_body) = else_body {
                    collect_guseong_from_surface_stmts(
                        else_body,
                        declared_instances,
                        instance_types,
                        links,
                        edges,
                        sink_sources,
                        linked_instances,
                        nodes,
                        link_scope_depth + 1,
                    )?;
                }
            }
            SurfaceStmt::PromptChoose {
                branches,
                else_body: Some(else_body),
            } => {
                for branch in branches {
                    collect_guseong_from_surface_stmts(
                        &branch.body,
                        declared_instances,
                        instance_types,
                        links,
                        edges,
                        sink_sources,
                        linked_instances,
                        nodes,
                        link_scope_depth + 1,
                    )?;
                }
                collect_guseong_from_surface_stmts(
                    else_body,
                    declared_instances,
                    instance_types,
                    links,
                    edges,
                    sink_sources,
                    linked_instances,
                    nodes,
                    link_scope_depth + 1,
                )?;
            }
            SurfaceStmt::PromptChoose {
                else_body: None,
                branches,
            } => {
                for branch in branches {
                    collect_guseong_from_surface_stmts(
                        &branch.body,
                        declared_instances,
                        instance_types,
                        links,
                        edges,
                        sink_sources,
                        linked_instances,
                        nodes,
                        link_scope_depth + 1,
                    )?;
                }
            }
            SurfaceStmt::PromptAfter { body, .. }
            | SurfaceStmt::PromptCondition { body, .. }
            | SurfaceStmt::PromptBlock { body }
            | SurfaceStmt::Repeat { body }
            | SurfaceStmt::While { body, .. }
            | SurfaceStmt::ForEach { body, .. }
            | SurfaceStmt::Hook { body, .. }
            | SurfaceStmt::EventReact { body, .. }
            | SurfaceStmt::OpenBlock { body } => collect_guseong_from_surface_stmts(
                body,
                declared_instances,
                instance_types,
                links,
                edges,
                sink_sources,
                linked_instances,
                nodes,
                link_scope_depth + 1,
            )?,
            SurfaceStmt::RootDecl { .. }
            | SurfaceStmt::Decl { .. }
            | SurfaceStmt::Send { .. }
            | SurfaceStmt::Show { .. }
            | SurfaceStmt::Inspect { .. }
            | SurfaceStmt::Boim { .. }
            | SurfaceStmt::BogaeMadangBlock { .. }
            | SurfaceStmt::ExecPolicyBlock { .. }
            | SurfaceStmt::JjaimBlock { .. }
            | SurfaceStmt::BogaeDraw
            | SurfaceStmt::Return { .. }
            | SurfaceStmt::Expr { .. }
            | SurfaceStmt::Break
            | SurfaceStmt::ContinueLoop => {}
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct NamedJjaimPortRegistry {
    type_name: Option<String>,
    registry: JjaimPortRegistry,
}

fn collect_jjaim_port_schema(
    stmts: &[SurfaceStmt],
    instance_types: &HashMap<String, String>,
) -> Result<JjaimPortSchema, CanonError> {
    let mut named: Vec<NamedJjaimPortRegistry> = Vec::new();
    collect_jjaim_port_blocks_from_surface(stmts, &mut named)?;

    let mut by_type: HashMap<String, JjaimPortRegistry> = HashMap::new();
    let mut unnamed_merged = JjaimPortRegistry::default();
    for block in named {
        if let Some(type_name) = block.type_name {
            if let Some(existing) = by_type.get(&type_name) {
                if existing != &block.registry {
                    return Err(CanonError::new(
                        "E_JJAIM_TYPE_SCHEMA_CONFLICT",
                        format!(
                            "같은 형식의 포트 스키마가 충돌합니다: {} (기존 입력/출력: {}/{}, 신규 입력/출력: {}/{})",
                            type_name,
                            sorted_ports_text(&existing.input_ports),
                            sorted_ports_text(&existing.output_ports),
                            sorted_ports_text(&block.registry.input_ports),
                            sorted_ports_text(&block.registry.output_ports)
                        ),
                    ));
                }
            } else {
                by_type.insert(type_name, block.registry);
            }
        } else {
            merge_port_registry(&mut unnamed_merged, &block.registry);
        }
    }

    if unnamed_merged.has_any() {
        let unique_types: HashSet<String> = instance_types.values().cloned().collect();
        if unique_types.len() > 1 {
            let mut types: Vec<String> = unique_types.into_iter().collect();
            types.sort();
            return Err(CanonError::new(
                "E_JJAIM_TYPE_TAG_REQUIRED",
                format!(
                    "형식 태그 없는 짜임 포트 스키마는 인스턴스 타입이 1종일 때만 허용됩니다. 현재 타입: {}",
                    types.join(", ")
                ),
            ));
        }
        if unique_types.len() == 1 {
            if let Some(type_name) = unique_types.into_iter().next() {
                let entry = by_type.entry(type_name).or_default();
                merge_port_registry(entry, &unnamed_merged);
            }
        }
    }

    Ok(JjaimPortSchema { by_type })
}

fn collect_jjaim_port_blocks_from_surface(
    stmts: &[SurfaceStmt],
    out: &mut Vec<NamedJjaimPortRegistry>,
) -> Result<(), CanonError> {
    for stmt in stmts {
        match stmt {
            SurfaceStmt::JjaimBlock { body, .. } => out.push(collect_ports_from_jjaim_body(body)?),
            SurfaceStmt::SeedDef { body, .. }
            | SurfaceStmt::PromptAfter { body, .. }
            | SurfaceStmt::PromptCondition { body, .. }
            | SurfaceStmt::PromptBlock { body }
            | SurfaceStmt::Repeat { body }
            | SurfaceStmt::While { body, .. }
            | SurfaceStmt::ForEach { body, .. }
            | SurfaceStmt::Hook { body, .. }
            | SurfaceStmt::Receive { body, .. }
            | SurfaceStmt::EventReact { body, .. }
            | SurfaceStmt::OpenBlock { body } => collect_jjaim_port_blocks_from_surface(body, out)?,
            SurfaceStmt::If {
                then_body,
                else_body,
                ..
            } => {
                collect_jjaim_port_blocks_from_surface(then_body, out)?;
                if let Some(body) = else_body {
                    collect_jjaim_port_blocks_from_surface(body, out)?;
                }
            }
            SurfaceStmt::Contract {
                then_body,
                else_body,
                ..
            } => {
                if let Some(body) = then_body {
                    collect_jjaim_port_blocks_from_surface(body, out)?;
                }
                collect_jjaim_port_blocks_from_surface(else_body, out)?;
            }
            SurfaceStmt::Choose {
                branches,
                else_body,
                ..
            } => {
                for branch in branches {
                    collect_jjaim_port_blocks_from_surface(&branch.body, out)?;
                }
                if let Some(else_body) = else_body {
                    collect_jjaim_port_blocks_from_surface(else_body, out)?;
                }
            }
            SurfaceStmt::PromptChoose {
                branches,
                else_body: Some(else_body),
            } => {
                for branch in branches {
                    collect_jjaim_port_blocks_from_surface(&branch.body, out)?;
                }
                collect_jjaim_port_blocks_from_surface(else_body, out)?;
            }
            SurfaceStmt::PromptChoose {
                branches,
                else_body: None,
            } => {
                for branch in branches {
                    collect_jjaim_port_blocks_from_surface(&branch.body, out)?;
                }
            }
            SurfaceStmt::RootDecl { .. }
            | SurfaceStmt::Decl { .. }
            | SurfaceStmt::Assign { .. }
            | SurfaceStmt::Send { .. }
            | SurfaceStmt::Show { .. }
            | SurfaceStmt::Inspect { .. }
            | SurfaceStmt::Boim { .. }
            | SurfaceStmt::BogaeMadangBlock { .. }
            | SurfaceStmt::ExecPolicyBlock { .. }
            | SurfaceStmt::BogaeDraw
            | SurfaceStmt::Return { .. }
            | SurfaceStmt::Expr { .. }
            | SurfaceStmt::Break
            | SurfaceStmt::ContinueLoop => {}
        }
    }
    Ok(())
}

fn collect_ports_from_jjaim_body(body: &str) -> Result<NamedJjaimPortRegistry, CanonError> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Mode {
        Input,
        Output,
        Formula,
    }
    let mut depth: i32 = 0;
    let mut mode: Option<Mode> = None;
    let mut mode_depth: i32 = 0;
    let mut type_name: Option<String> = None;
    let mut registry = JjaimPortRegistry::default();
    for raw_line in body.lines() {
        let code = strip_line_comment(raw_line);
        let trimmed = code.trim();
        let line_start_depth = depth;

        if line_start_depth == 0 {
            if type_name.is_none() {
                type_name = extract_jjaim_type_tag(trimmed);
            }
            if let Some(open_idx) = trimmed.find('{') {
                let mut head = trimmed[..open_idx].trim();
                if let Some(stripped) = head.strip_suffix(':') {
                    head = stripped.trim_end();
                }
                if head == "입력" {
                    mode = Some(Mode::Input);
                    mode_depth = line_start_depth + 1;
                } else if head == "출력" {
                    mode = Some(Mode::Output);
                    mode_depth = line_start_depth + 1;
                } else if head == "수식" {
                    mode = Some(Mode::Formula);
                    mode_depth = line_start_depth + 1;
                }
            }
        }

        if let Some(active) = mode {
            if line_start_depth == mode_depth {
                match active {
                    Mode::Input | Mode::Output => {
                        if !(trimmed.is_empty() || trimmed == "}" || trimmed.starts_with("}")) {
                            if !contains_assign_operator(trimmed) {
                                return Err(CanonError::new(
                                    "E_JJAIM_PORT_DECL_INVALID",
                                    format!("포트 선언 형식 오류(대입식 필요): {}", trimmed),
                                ));
                            }
                            if has_type_colon_without_type(trimmed) {
                                return Err(CanonError::new(
                                    "E_JJAIM_PORT_TYPE_MISSING",
                                    format!("포트 타입 누락: {}", trimmed),
                                ));
                            }
                            if let Some((name, type_name)) = extract_decl_lhs_name_and_type(trimmed)
                            {
                                match active {
                                    Mode::Input => {
                                        if !registry.input_ports.insert(name.clone()) {
                                            return Err(CanonError::new(
                                                "E_JJAIM_PORT_DUP",
                                                format!("입력 포트 이름 중복: {}", name),
                                            ));
                                        }
                                        if let Some(type_name) = type_name {
                                            registry.input_port_types.insert(name, type_name);
                                        }
                                    }
                                    Mode::Output => {
                                        if !registry.output_ports.insert(name.clone()) {
                                            return Err(CanonError::new(
                                                "E_JJAIM_PORT_DUP",
                                                format!("출력 포트 이름 중복: {}", name),
                                            ));
                                        }
                                        if let Some(type_name) = type_name {
                                            registry.output_port_types.insert(name, type_name);
                                        }
                                    }
                                    Mode::Formula => {}
                                }
                            } else {
                                return Err(CanonError::new(
                                    "E_JJAIM_PORT_DECL_INVALID",
                                    format!("포트 선언에서 이름을 해석할 수 없습니다: {}", trimmed),
                                ));
                            }
                        }
                    }
                    Mode::Formula => {
                        // 수식 블록은 타입 명시된 항목만 외부 출력 포트로 간주한다.
                        if let Some((name, type_name)) =
                            extract_typed_decl_lhs_name_and_type(trimmed)
                        {
                            registry.output_ports.insert(name.clone());
                            registry.output_port_types.insert(name, type_name);
                        }
                    }
                }
            }
        }

        for ch in code.chars() {
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
            }
        }
        if mode.is_some() && depth < mode_depth {
            mode = None;
            mode_depth = 0;
        }
    }
    Ok(NamedJjaimPortRegistry {
        type_name,
        registry,
    })
}

fn merge_port_registry(dst: &mut JjaimPortRegistry, src: &JjaimPortRegistry) {
    dst.input_ports.extend(src.input_ports.iter().cloned());
    dst.output_ports.extend(src.output_ports.iter().cloned());
    for (name, type_name) in &src.input_port_types {
        dst.input_port_types
            .entry(name.clone())
            .or_insert_with(|| type_name.clone());
    }
    for (name, type_name) in &src.output_port_types {
        dst.output_port_types
            .entry(name.clone())
            .or_insert_with(|| type_name.clone());
    }
}

fn validate_guseong_port_links(
    links: &[(PortRef, PortRef)],
    instance_types: &HashMap<String, String>,
    port_schema: &JjaimPortSchema,
) -> Result<(), CanonError> {
    for (dst, src) in links {
        validate_tuple_port_projection(&dst.instance, &dst.port, "입력")?;
        validate_tuple_port_projection(&src.instance, &src.port, "출력")?;
        if let Some(dst_type) = instance_types.get(&dst.instance) {
            if let Some(registry) = port_schema.registry_for_type(dst_type) {
                let dst_base = base_port_name(&dst.port);
                if !registry.input_ports.contains(dst_base) {
                    return Err(CanonError::new(
                        "E_GUSEONG_INPUT_PORT_UNDECLARED",
                        format!(
                            "선언되지 않은 입력 포트입니다: {}.{} (타입 {}, 기대 입력 포트: {})",
                            dst.instance,
                            dst.port,
                            dst_type,
                            sorted_ports_text(&registry.input_ports)
                        ),
                    ));
                }
                validate_tuple_projection_type(
                    &dst.instance,
                    &dst.port,
                    "입력",
                    registry.input_port_types.get(dst_base),
                )?;
            }
        }
        if let Some(src_type) = instance_types.get(&src.instance) {
            if let Some(registry) = port_schema.registry_for_type(src_type) {
                let src_base = base_port_name(&src.port);
                if !registry.output_ports.contains(src_base) {
                    return Err(CanonError::new(
                        "E_GUSEONG_OUTPUT_PORT_UNDECLARED",
                        format!(
                            "선언되지 않은 출력 포트입니다: {}.{} (타입 {}, 기대 출력 포트: {})",
                            src.instance,
                            src.port,
                            src_type,
                            sorted_ports_text(&registry.output_ports)
                        ),
                    ));
                }
                validate_tuple_projection_type(
                    &src.instance,
                    &src.port,
                    "출력",
                    registry.output_port_types.get(src_base),
                )?;
            }
        }
    }
    Ok(())
}

fn validate_tuple_port_projection(
    instance: &str,
    port: &str,
    direction: &str,
) -> Result<(), CanonError> {
    let mut parts = port.split('.');
    let _base = parts.next().unwrap_or(port);
    for suffix in parts {
        if suffix != "0" && suffix != "1" {
            return Err(CanonError::new(
                "E_GUSEONG_TUPLE_INDEX_INVALID",
                format!(
                    "튜플 포트 접근은 .0/.1만 허용됩니다: {}.{} ({} 접근)",
                    instance, port, direction
                ),
            ));
        }
    }
    Ok(())
}

fn validate_tuple_projection_type(
    instance: &str,
    port: &str,
    direction: &str,
    declared_type: Option<&String>,
) -> Result<(), CanonError> {
    if !port.contains('.') {
        return Ok(());
    }
    let Some(type_name) = declared_type else {
        return Ok(());
    };
    if !is_tuple_type_name(type_name) {
        return Err(CanonError::new(
            "E_GUSEONG_TUPLE_ACCESS_ON_SCALAR",
            format!(
                "스칼라 포트에는 튜플 인덱스 접근을 사용할 수 없습니다: {}.{} ({} 포트 타입: {})",
                instance, port, direction, type_name
            ),
        ));
    }
    Ok(())
}

fn is_tuple_type_name(type_name: &str) -> bool {
    let trimmed = type_name.trim();
    trimmed.starts_with('(') && trimmed.ends_with(')') && trimmed.contains(',')
}

fn strip_line_comment(raw: &str) -> &str {
    raw.split_once("//").map(|(head, _)| head).unwrap_or(raw)
}

fn contains_assign_operator(line: &str) -> bool {
    line.contains("<-") || line.contains('=')
}

fn has_type_colon_without_type(line: &str) -> bool {
    let op_idx = line.find("<-").or_else(|| line.find('='));
    let Some(op_idx) = op_idx else {
        return false;
    };
    let lhs = line[..op_idx].trim();
    let Some((_, type_name)) = lhs.split_once(':') else {
        return false;
    };
    type_name.trim().is_empty()
}

fn extract_jjaim_type_tag(line: &str) -> Option<String> {
    let head = line.trim();
    let (key, rhs) = head.split_once(':')?;
    let key = key.trim();
    if key != "형식" && key != "짜임이름" {
        return None;
    }
    let rhs = rhs.trim().trim_end_matches('.').trim();
    if rhs.is_empty() {
        return None;
    }
    let mut out = String::new();
    for ch in rhs.chars() {
        if is_ident_continue(ch) {
            out.push(ch);
        } else {
            break;
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn extract_decl_lhs_name_and_type(line: &str) -> Option<(String, Option<String>)> {
    if line.is_empty() || line.starts_with('{') || line.starts_with('}') {
        return None;
    }
    let op_idx = line.find("<-").or_else(|| line.find('='))?;
    let lhs = line[..op_idx].trim();
    if lhs.is_empty() {
        return None;
    }
    if let Some((name, type_name)) = lhs.split_once(':') {
        let parsed_name = parse_ident_head(name.trim())?;
        let type_name = type_name.trim();
        if type_name.is_empty() {
            return Some((parsed_name, None));
        }
        return Some((parsed_name, Some(type_name.to_string())));
    }
    parse_ident_head(lhs).map(|name| (name, None))
}

fn extract_typed_decl_lhs_name_and_type(line: &str) -> Option<(String, String)> {
    if line.is_empty() || line.starts_with('{') || line.starts_with('}') {
        return None;
    }
    let op_idx = line.find("<-").or_else(|| line.find('='))?;
    let lhs = line[..op_idx].trim();
    let (name, type_name) = lhs.split_once(':')?;
    let parsed_name = parse_ident_head(name.trim())?;
    let parsed_type = type_name.trim();
    if parsed_type.is_empty() {
        return None;
    }
    Some((parsed_name, parsed_type.to_string()))
}

fn parse_ident_head(raw: &str) -> Option<String> {
    if raw.is_empty() {
        return None;
    }
    let mut out = String::new();
    for ch in raw.chars() {
        if is_ident_continue(ch) {
            out.push(ch);
        } else {
            break;
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn base_port_name(port: &str) -> &str {
    port.split('.').next().unwrap_or(port)
}

fn sorted_ports_text(ports: &HashSet<String>) -> String {
    let mut values: Vec<String> = ports.iter().cloned().collect();
    values.sort();
    if values.is_empty() {
        "(없음)".to_string()
    } else {
        values.join(", ")
    }
}

fn extract_instance_decl_target(path: &Path) -> Option<String> {
    let segments = normalize_root_segments(&path.segments);
    (segments.len() == 1).then(|| segments[0].clone())
}

fn extract_port_ref_from_path(path: &Path) -> Option<PortRef> {
    let segments = normalize_root_segments(&path.segments);
    if segments.len() < 2 {
        return None;
    }
    Some(PortRef {
        instance: segments[0].clone(),
        port: segments[1..].join("."),
    })
}

fn extract_port_ref_from_expr(expr: &Expr) -> Option<PortRef> {
    match expr {
        Expr::Path(path) => extract_port_ref_from_path(path),
        Expr::FieldAccess { target, field } => {
            let path = flatten_field_access(target, field)?;
            extract_port_ref_from_path(&path)
        }
        _ => None,
    }
}

fn extract_link_from_assign(target: &Path, value: &Expr) -> Option<(PortRef, PortRef)> {
    let direct_dst = extract_port_ref_from_path(target);
    let direct_src = extract_port_ref_from_expr(value);
    if let (Some(dst), Some(src)) = (direct_dst, direct_src) {
        return Some((dst, src));
    }

    let Expr::Call { name, args } = value else {
        return None;
    };
    if name != "짝맞춤.바꾼값" || args.len() != 3 {
        return None;
    }

    let Expr::Path(map_path) = &args[0] else {
        return None;
    };
    let map_instance = extract_instance_from_path(map_path)?;
    let Expr::Literal(Literal::Str(port_name)) = &args[1] else {
        return None;
    };
    let src = extract_port_ref_from_expr(&args[2])?;
    let dst = PortRef {
        instance: map_instance,
        port: port_name.clone(),
    };
    Some((dst, src))
}

fn extract_instance_from_path(path: &Path) -> Option<String> {
    let segments = normalize_root_segments(&path.segments);
    (!segments.is_empty()).then(|| segments[0].clone())
}

