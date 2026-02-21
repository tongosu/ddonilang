use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::run::RunError;
use crate::lang::ast::{Expr, Literal, Stmt};
use crate::lang::lexer::Lexer;
use crate::lang::parser::Parser;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TypeKind {
    Str,
    Num,
    Bool,
    None,
    Unknown,
}

impl TypeKind {
    fn name(&self) -> Option<&'static str> {
        match self {
            TypeKind::Str => Some("글"),
            TypeKind::Num => Some("수"),
            TypeKind::Bool => Some("참거짓"),
            TypeKind::None => Some("없음"),
            TypeKind::Unknown => None,
        }
    }

    fn is_known(&self) -> bool {
        !matches!(self, TypeKind::Unknown)
    }
}

struct SchemaEntry {
    name: String,
    type_name: String,
}

pub struct CheckArgs {
    pub emit_schema: bool,
}

pub fn run(file: &Path, args: CheckArgs) -> Result<(), String> {
    let source = fs::read_to_string(file).map_err(|e| e.to_string())?;
    let tokens = Lexer::tokenize(&source)
        .map_err(|e| RunError::Lex(e).format(&file.display().to_string()))?;
    let default_root = Parser::default_root_for_source(&source);
    let program = Parser::parse_with_default_root(tokens, default_root)
        .map_err(|e| RunError::Parse(e).format(&file.display().to_string()))?;

    let mut symbols: BTreeMap<String, TypeKind> = BTreeMap::new();

    for stmt in &program.stmts {
        let Stmt::Assign { target, value, .. } = stmt else {
            continue;
        };
        let name = target.segments.join(".");
        let value_type = expr_type(value, &symbols);

        if let Some(existing) = symbols.get(&name) {
            if existing.is_known() && value_type.is_known() && existing != &value_type {
                return Err(format!(
                    "E_CHECK_TYPE_MISMATCH {} {} -> {}",
                    name,
                    existing.name().unwrap_or("알수없음"),
                    value_type.name().unwrap_or("알수없음")
                ));
            }
            return Err(format!("E_CHECK_DUPLICATE_SYMBOL {}", name));
        }
        symbols.insert(name, value_type);
    }

    if args.emit_schema {
        let entries = symbols
            .iter()
            .filter_map(|(name, kind)| {
                kind.name().map(|type_name| SchemaEntry {
                    name: name.clone(),
                    type_name: type_name.to_string(),
                })
            })
            .collect::<Vec<_>>();
        write_schema(file, &entries)?;
    }

    Ok(())
}

fn expr_type(expr: &Expr, symbols: &BTreeMap<String, TypeKind>) -> TypeKind {
    match expr {
        Expr::Literal(lit, _) => match lit {
            Literal::Str(_) => TypeKind::Str,
            Literal::Num(_) => TypeKind::Num,
            Literal::Bool(_) => TypeKind::Bool,
            Literal::None => TypeKind::None,
        },
        Expr::Path(path) => symbols
            .get(&path.segments.join("."))
            .cloned()
            .unwrap_or(TypeKind::Unknown),
        Expr::FieldAccess { .. } => TypeKind::Unknown,
        Expr::Atom { .. } => TypeKind::Str,
        Expr::Unary { expr, .. } => expr_type(expr, symbols),
        Expr::Binary { op, .. } => match op {
            crate::lang::ast::BinaryOp::Eq
            | crate::lang::ast::BinaryOp::NotEq
            | crate::lang::ast::BinaryOp::Lt
            | crate::lang::ast::BinaryOp::Lte
            | crate::lang::ast::BinaryOp::Gt
            | crate::lang::ast::BinaryOp::Gte => TypeKind::Bool,
            _ => TypeKind::Num,
        },
        Expr::Call { .. } => TypeKind::Num,
        Expr::Formula { .. } => TypeKind::Unknown,
        Expr::FormulaEval { .. } => TypeKind::Num,
        Expr::FormulaFill { .. } => TypeKind::Num,
        Expr::Template { .. } => TypeKind::Unknown,
        Expr::TemplateFill { .. } => TypeKind::Str,
        Expr::Pack { .. } => TypeKind::Unknown,
        Expr::SeedLiteral { .. } => TypeKind::Unknown,
    }
}

fn write_schema(source_path: &Path, entries: &[SchemaEntry]) -> Result<(), String> {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"version\": 0,\n");
    out.push_str("  \"symbols\": [\n");
    for (idx, entry) in entries.iter().enumerate() {
        out.push_str("    { \"name\": \"");
        out.push_str(&escape_json(&entry.name));
        out.push_str("\", \"type\": \"");
        out.push_str(&escape_json(&entry.type_name));
        out.push_str("\" }");
        if idx + 1 < entries.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ]\n");
    out.push_str("}\n");

    let schema_path = schema_path_for(source_path);
    fs::write(schema_path, out).map_err(|e| e.to_string())
}

fn schema_path_for(source_path: &Path) -> PathBuf {
    match source_path.parent() {
        Some(dir) => dir.join("ddn.schema.json"),
        None => PathBuf::from("ddn.schema.json"),
    }
}

fn escape_json(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            _ => out.push(ch),
        }
    }
    out
}
