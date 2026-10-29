//! Existing expression canonicalization and call rendering; source bodies unchanged.
use super::*;

pub(super) fn canonicalize_expr(
    expr: Expr,
    declared: &HashSet<String>,
    bridge: bool,
    default_root: &str,
    root_hide: bool,
) -> Expr {
    match expr {
        Expr::Path(path) => Expr::Path(canonicalize_path(
            path,
            declared,
            bridge,
            default_root,
            root_hide,
        )),
        Expr::Literal(literal) => Expr::Literal(literal),
        Expr::Resource(path) => Expr::Resource(path),
        Expr::Template { body } => Expr::Template { body },
        Expr::PromptExpr { expr } => Expr::PromptExpr {
            expr: Box::new(canonicalize_expr(
                *expr,
                declared,
                bridge,
                default_root,
                root_hide,
            )),
        },
        Expr::PromptBlock { body } => Expr::PromptBlock { body },
        Expr::TemplateApply { bindings, body } => Expr::TemplateApply {
            bindings: bindings
                .into_iter()
                .map(|binding| Binding {
                    name: binding.name,
                    value: canonicalize_expr(
                        binding.value,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    ),
                })
                .collect(),
            body,
        },
        Expr::Formula { tag, body } => Expr::Formula { tag, body },
        Expr::Call { name, args } => Expr::Call {
            name: canonicalize_stdlib_alias(&name).to_string(),
            args: args
                .into_iter()
                .map(|arg| canonicalize_expr(arg, declared, bridge, default_root, root_hide))
                .collect(),
        },
        Expr::CallIn { name, bindings } => Expr::CallIn {
            name: canonicalize_stdlib_alias(&name).to_string(),
            bindings: bindings
                .into_iter()
                .map(|binding| Binding {
                    name: binding.name,
                    value: canonicalize_expr(
                        binding.value,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    ),
                })
                .collect(),
        },
        Expr::FieldAccess { target, field } => Expr::FieldAccess {
            target: Box::new(canonicalize_expr(
                *target,
                declared,
                bridge,
                default_root,
                root_hide,
            )),
            field,
        },
        Expr::Pack { bindings } => Expr::Pack {
            bindings: bindings
                .into_iter()
                .map(|binding| Binding {
                    name: binding.name,
                    value: canonicalize_expr(
                        binding.value,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    ),
                })
                .collect(),
        },
        Expr::Unit { value, unit } => Expr::Unit {
            value: Box::new(canonicalize_expr(
                *value,
                declared,
                bridge,
                default_root,
                root_hide,
            )),
            unit,
        },
        Expr::Unary { op, expr } => Expr::Unary {
            op,
            expr: Box::new(canonicalize_expr(
                *expr,
                declared,
                bridge,
                default_root,
                root_hide,
            )),
        },
        Expr::Binary { left, op, right } => Expr::Binary {
            left: Box::new(canonicalize_expr(
                *left,
                declared,
                bridge,
                default_root,
                root_hide,
            )),
            op,
            right: Box::new(canonicalize_expr(
                *right,
                declared,
                bridge,
                default_root,
                root_hide,
            )),
        },
        Expr::Pipe { left, kind, right } => Expr::Pipe {
            left: Box::new(canonicalize_expr(
                *left,
                declared,
                bridge,
                default_root,
                root_hide,
            )),
            kind,
            right: Box::new(canonicalize_expr(
                *right,
                declared,
                bridge,
                default_root,
                root_hide,
            )),
        },
        Expr::SeedLiteral { param, body } => Expr::SeedLiteral {
            param,
            body: Box::new(canonicalize_expr(
                *body,
                declared,
                bridge,
                default_root,
                root_hide,
            )),
        },
        Expr::Grouped { expr } => Expr::Grouped {
            expr: Box::new(canonicalize_expr(
                *expr,
                declared,
                bridge,
                default_root,
                root_hide,
            )),
        },
    }
}

pub(super) fn canonicalize_stdlib_alias(name: &str) -> &str {
    ddonirang_lang::stdlib::canonicalize_stdlib_alias(name)
}

pub(super) fn canonicalize_path(
    path: Path,
    declared: &HashSet<String>,
    bridge: bool,
    default_root: &str,
    root_hide: bool,
) -> Path {
    let mut segments = path.segments;
    let has_root = matches!(
        segments.first().map(|seg| seg.as_str()),
        Some("살림" | "바탕" | "샘")
    );
    if let Some(root) = segments.first_mut() {
        if root == "살림" {
            *root = "바탕".to_string();
        }
    }
    if !has_root {
        if root_hide {
            segments.insert(0, default_root.to_string());
        } else if bridge && segments.len() == 1 {
            let name = &segments[0];
            if !matches!(name.as_str(), "살림" | "바탕") && declared.contains(name) {
                segments.insert(0, default_root.to_string());
            }
        }
    }
    Path { segments }
}

pub(super) fn canonicalize_condition(
    condition: Condition,
    declared: &HashSet<String>,
    bridge: bool,
    default_root: &str,
    root_hide: bool,
) -> Condition {
    Condition {
        expr: canonicalize_expr(condition.expr, declared, bridge, default_root, root_hide),
        style: condition.style,
        negated: condition.negated,
    }
}

pub(super) fn canonicalize_body(
    body: Vec<SurfaceStmt>,
    declared: &HashSet<String>,
    bridge: bool,
    default_root: &str,
    root_hide: bool,
) -> Result<Vec<Stmt>, CanonError> {
    let mut out = Vec::new();
    for stmt in body {
        match stmt {
            SurfaceStmt::RootDecl { items } => {
                let items = items
                    .into_iter()
                    .map(|item| DeclItem {
                        name: item.name,
                        kind: item.kind,
                        type_name: item.type_name,
                        value: item.value.map(|expr| {
                            canonicalize_expr(expr, declared, bridge, default_root, root_hide)
                        }),
                        maegim: item.maegim.map(|spec| MaegimSpec {
                            fields: spec
                                .fields
                                .into_iter()
                                .map(|binding| Binding {
                                    name: binding.name,
                                    value: canonicalize_expr(
                                        binding.value,
                                        declared,
                                        bridge,
                                        default_root,
                                        root_hide,
                                    ),
                                })
                                .collect(),
                        }),
                    })
                    .collect();
                out.push(Stmt::RootDecl { items });
            }
            SurfaceStmt::Decl {
                name,
                type_name,
                value,
            } => {
                if !is_bridge_alias_type_name(&type_name) {
                    return Err(CanonError::new(
                        "E_CANON_BAD_ALIAS",
                        format!("알 수 없는 별칭입니다: {}", type_name),
                    ));
                }
                let target = Path {
                    segments: vec![default_root.to_string(), name],
                };
                let value = canonicalize_expr(value, declared, bridge, default_root, root_hide);
                out.push(Stmt::Assign { target, value });
            }
            SurfaceStmt::SeedDef {
                name,
                kind,
                params,
                body,
            } => {
                let params = params
                    .into_iter()
                    .map(|param| Param {
                        name: param.name,
                        type_name: param.type_name,
                        default: param.default.map(|expr| {
                            canonicalize_expr(expr, declared, bridge, default_root, root_hide)
                        }),
                    })
                    .collect();
                let body = canonicalize_body(body, declared, bridge, default_root, root_hide)?;
                out.push(Stmt::SeedDef {
                    name,
                    kind,
                    params,
                    body,
                });
            }
            SurfaceStmt::Send {
                sender,
                payload,
                receiver,
            } => {
                out.push(Stmt::Send {
                    sender: sender.map(|expr| {
                        canonicalize_expr(expr, declared, bridge, default_root, root_hide)
                    }),
                    payload: canonicalize_expr(payload, declared, bridge, default_root, root_hide),
                    receiver: canonicalize_expr(
                        receiver,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    ),
                });
            }
            SurfaceStmt::Assign { target, value } => {
                if root_hide
                    && !matches!(
                        target.segments.first().map(|seg| seg.as_str()),
                        Some("살림" | "바탕" | "샘")
                    )
                {
                    if let Some(name) = target.segments.first() {
                        if !declared.contains(name) {
                            return Err(CanonError::new(
                                "E_CANON_ROOT_HIDE_UNDECLARED",
                                format!("채비에 선언되지 않은 전역 쓰기: {}", name),
                            ));
                        }
                    }
                }
                out.push(Stmt::Assign {
                    target: canonicalize_path(target, declared, bridge, default_root, root_hide),
                    value: canonicalize_expr(value, declared, bridge, default_root, root_hide),
                });
            }
            SurfaceStmt::Show { value } => {
                out.push(Stmt::Show {
                    value: canonicalize_expr(value, declared, bridge, default_root, root_hide),
                });
            }
            SurfaceStmt::Inspect { value } => {
                out.push(Stmt::Inspect {
                    value: canonicalize_expr(value, declared, bridge, default_root, root_hide),
                });
            }
            SurfaceStmt::Boim { entries } => {
                let entries = entries
                    .into_iter()
                    .map(|entry| Binding {
                        name: entry.name,
                        value: canonicalize_expr(
                            entry.value,
                            declared,
                            bridge,
                            default_root,
                            root_hide,
                        ),
                    })
                    .collect();
                out.push(Stmt::Boim { entries });
            }
            SurfaceStmt::BogaeMadangBlock { body, .. } => {
                out.push(Stmt::BogaeMadangBlock { body });
            }
            SurfaceStmt::JjaimBlock { body, .. } => {
                out.push(Stmt::JjaimBlock { body });
            }
            SurfaceStmt::BogaeDraw => {
                out.push(Stmt::BogaeDraw);
            }
            SurfaceStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                let then_body =
                    canonicalize_body(then_body, declared, bridge, default_root, root_hide)?;
                let else_body = match else_body {
                    Some(body) => Some(canonicalize_body(
                        body,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    )?),
                    None => None,
                };
                out.push(Stmt::If {
                    condition: canonicalize_condition(
                        condition,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    ),
                    then_body,
                    else_body,
                });
            }
            SurfaceStmt::Contract {
                kind,
                mode,
                condition,
                then_body,
                else_body,
            } => {
                let else_body =
                    canonicalize_body(else_body, declared, bridge, default_root, root_hide)?;
                let then_body = match then_body {
                    Some(body) => Some(canonicalize_body(
                        body,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    )?),
                    None => None,
                };
                out.push(Stmt::Contract {
                    kind,
                    mode,
                    condition: canonicalize_condition(
                        condition,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    ),
                    then_body,
                    else_body,
                });
            }
            SurfaceStmt::Choose {
                branches,
                else_body,
                exhaustive,
            } => {
                let mut mapped = Vec::new();
                for branch in branches {
                    mapped.push(ChooseBranch {
                        condition: canonicalize_condition(
                            branch.condition,
                            declared,
                            bridge,
                            default_root,
                            root_hide,
                        ),
                        body: canonicalize_body(
                            branch.body,
                            declared,
                            bridge,
                            default_root,
                            root_hide,
                        )?,
                    });
                }
                let else_body = match else_body {
                    Some(body) => Some(canonicalize_body(
                        body,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    )?),
                    None => None,
                };
                out.push(Stmt::Choose {
                    branches: mapped,
                    else_body,
                    exhaustive,
                });
            }
            SurfaceStmt::PromptChoose {
                branches,
                else_body,
            } => {
                let mut mapped = Vec::new();
                for branch in branches {
                    mapped.push(ChooseBranch {
                        condition: canonicalize_condition(
                            branch.condition,
                            declared,
                            bridge,
                            default_root,
                            root_hide,
                        ),
                        body: canonicalize_body(
                            branch.body,
                            declared,
                            bridge,
                            default_root,
                            root_hide,
                        )?,
                    });
                }
                let else_body = match else_body {
                    Some(body) => Some(canonicalize_body(
                        body,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    )?),
                    None => None,
                };
                out.push(Stmt::PromptChoose {
                    branches: mapped,
                    else_body,
                });
            }
            SurfaceStmt::PromptAfter { value, body } => {
                out.push(Stmt::PromptAfter {
                    value: canonicalize_expr(value, declared, bridge, default_root, root_hide),
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::PromptCondition { condition, body } => {
                out.push(Stmt::PromptCondition {
                    condition: canonicalize_condition(
                        condition,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    ),
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::PromptBlock { body } => {
                out.push(Stmt::PromptBlock {
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::ExecPolicyBlock { body } => {
                out.push(Stmt::ExecPolicyBlock { body });
            }
            SurfaceStmt::Repeat { body } => {
                out.push(Stmt::Repeat {
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::While { condition, body } => {
                out.push(Stmt::While {
                    condition: canonicalize_condition(
                        condition,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    ),
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::ForEach {
                item,
                iterable,
                body,
            } => {
                out.push(Stmt::ForEach {
                    item,
                    iterable: canonicalize_expr(
                        iterable,
                        declared,
                        bridge,
                        default_root,
                        root_hide,
                    ),
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::Hook { name, suffix, body } => {
                out.push(Stmt::Hook {
                    name,
                    suffix,
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::Receive {
                kind,
                binding,
                condition,
                body,
            } => {
                out.push(Stmt::Receive {
                    kind,
                    binding,
                    condition: condition.map(|expr| {
                        canonicalize_expr(expr, declared, bridge, default_root, root_hide)
                    }),
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::EventReact { kind, body } => {
                out.push(Stmt::EventReact {
                    kind,
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::OpenBlock { body } => {
                out.push(Stmt::OpenBlock {
                    body: canonicalize_body(body, declared, bridge, default_root, root_hide)?,
                });
            }
            SurfaceStmt::Return { value } => {
                out.push(Stmt::Return {
                    value: canonicalize_expr(value, declared, bridge, default_root, root_hide),
                });
            }
            SurfaceStmt::Expr { value } => {
                out.push(Stmt::Expr {
                    value: canonicalize_expr(value, declared, bridge, default_root, root_hide),
                });
            }
            SurfaceStmt::Break => {
                out.push(Stmt::Break);
            }
            SurfaceStmt::ContinueLoop => {
                out.push(Stmt::ContinueLoop);
            }
        }
    }
    Ok(out)
}

