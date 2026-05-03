use regex::Regex;
use serde_json::{json, Value as JsonValue};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct CurrentLineResult {
    pub project_source: String,
    pub context_json: String,
}

#[derive(Debug, Clone, PartialEq)]
enum CellValue {
    Bool(bool),
    Num(i64),
    Str(String),
    Choice {
        variant: String,
        value: Box<CellValue>,
        reason: String,
    },
}

impl CellValue {
    fn display(&self) -> String {
        match self {
            CellValue::Bool(flag) => {
                if *flag {
                    "참".to_string()
                } else {
                    "거짓".to_string()
                }
            }
            CellValue::Num(value) => value.to_string(),
            CellValue::Str(value) => value.clone(),
            CellValue::Choice {
                variant,
                value,
                reason,
            } => {
                if variant == "성공" {
                    value.display()
                } else {
                    reason.clone()
                }
            }
        }
    }

    fn ddn_type(&self) -> &'static str {
        match self {
            CellValue::Bool(_) | CellValue::Num(_) => "수",
            CellValue::Str(_) | CellValue::Choice { .. } => "글",
        }
    }

    fn ddn_literal(&self) -> String {
        match self {
            CellValue::Bool(flag) => {
                if *flag {
                    "1".to_string()
                } else {
                    "0".to_string()
                }
            }
            CellValue::Num(value) => value.to_string(),
            CellValue::Str(value) => json_string(value),
            CellValue::Choice { .. } => json_string(&self.display()),
        }
    }
}

#[derive(Default)]
struct CurrentLineSession {
    state: BTreeMap<String, CellValue>,
    handlers: BTreeMap<String, BTreeMap<String, Vec<String>>>,
    machines: BTreeMap<String, BTreeMap<String, String>>,
    state_hooks: BTreeMap<String, String>,
    outputs: Vec<String>,
    boim_rows: Vec<(String, CellValue)>,
}

pub fn apply_currentline_cell(
    cell_source: &str,
    context_json: Option<&str>,
) -> Result<CurrentLineResult, String> {
    let mut cells = cells_from_context(context_json)?;
    cells.push(cell_source.trim().to_string());
    let project_source = compile_cells(&cells)?;
    let context_json = json!({
        "schema": "ddn.currentline.session.v1",
        "cells": cells,
    })
    .to_string();
    Ok(CurrentLineResult {
        project_source,
        context_json,
    })
}

fn cells_from_context(context_json: Option<&str>) -> Result<Vec<String>, String> {
    let Some(raw) = context_json.map(str::trim).filter(|raw| !raw.is_empty()) else {
        return Ok(Vec::new());
    };
    let value: JsonValue = serde_json::from_str(raw)
        .map_err(|err| format!("E_CURRENTLINE_CONTEXT_JSON {err}"))?;
    let mut cells = Vec::new();
    if let Some(prelude) = value.get("prelude_source").and_then(JsonValue::as_str) {
        if !prelude.trim().is_empty() {
            cells.push(prelude.trim().to_string());
        }
    }
    if let Some(items) = value.get("cells").and_then(JsonValue::as_array) {
        for item in items {
            if let Some(cell) = item.as_str() {
                if !cell.trim().is_empty() {
                    cells.push(cell.trim().to_string());
                }
            }
        }
    }
    Ok(cells)
}

fn compile_cells(cells: &[String]) -> Result<String, String> {
    let mut session = CurrentLineSession::new();
    for cell in cells {
        session.apply_cell(cell)?;
    }
    Ok(session.to_executable_source())
}

impl CurrentLineSession {
    fn new() -> Self {
        Self::default()
    }

    fn apply_cell(&mut self, source: &str) -> Result<(), String> {
        self.outputs.clear();
        self.boim_rows.clear();
        let mut rest = strip_comments(source);
        let (next, chaebis) = remove_named_blocks(&rest, "채비")?;
        rest = next;
        for body in chaebis {
            self.apply_chaebi(&body)?;
        }
        let (next, actors) = remove_imja_blocks(&rest)?;
        rest = next;
        for (actor, body) in actors {
            self.define_actor(&actor, &body)?;
        }
        self.execute_block(&rest, None, &BTreeMap::new())
    }

    fn apply_chaebi(&mut self, body: &str) -> Result<(), String> {
        let decl_re = Regex::new(r"(?s)^([가-힣A-Za-z_][가-힣A-Za-z0-9_.]*)\s*:\s*[가-힣A-Za-z_][가-힣A-Za-z0-9_]*\s*<-\s*(.+)$").unwrap();
        for stmt in split_statements(body) {
            if let Some(caps) = decl_re.captures(stmt.trim()) {
                let key = flatten_name(caps.get(1).unwrap().as_str(), None);
                let value = self.eval_expr(caps.get(2).unwrap().as_str(), None, &BTreeMap::new())?;
                self.state.insert(key, value);
            }
        }
        Ok(())
    }

    fn define_actor(&mut self, actor: &str, body: &str) -> Result<(), String> {
        self.handlers.entry(actor.to_string()).or_default();
        let decl_re = Regex::new(r"(?s)^([가-힣A-Za-z_][가-힣A-Za-z0-9_]*)\s*:\s*[가-힣A-Za-z_][가-힣A-Za-z0-9_]*\s*<-\s*(.+)$").unwrap();
        let handler_re = Regex::new(r#"(?s)^"([^"]+)"라는\s+알림이\s+오면\s*\{(.*)\}\s*$"#).unwrap();
        let hook_re = Regex::new(r"(?s)^\(지금상태\)바뀔때마다\s*\{(.*)\}\s*$").unwrap();
        for stmt in split_statements(body) {
            let stmt = stmt.trim();
            if let Some(caps) = decl_re.captures(stmt) {
                let key = format!("{actor}__{}", caps.get(1).unwrap().as_str());
                let value =
                    self.eval_expr(caps.get(2).unwrap().as_str(), Some(actor), &BTreeMap::new())?;
                self.state.insert(key, value);
            } else if let Some(caps) = handler_re.captures(stmt) {
                self.handlers
                    .entry(actor.to_string())
                    .or_default()
                    .insert(caps[1].to_string(), split_statements(&caps[2]));
            } else if stmt.starts_with("상태머신") {
                self.define_state_machine(actor, stmt)?;
            } else if let Some(caps) = hook_re.captures(stmt) {
                self.state_hooks
                    .insert(actor.to_string(), caps[1].to_string());
            } else {
                self.execute_statement(stmt, Some(actor), &BTreeMap::new())?;
            }
        }
        Ok(())
    }

    fn define_state_machine(&mut self, actor: &str, stmt: &str) -> Result<(), String> {
        let Some(open) = stmt.find('{') else {
            return Ok(());
        };
        let close = find_matching(stmt, open, '{', '}')?;
        let mut transitions = BTreeMap::new();
        for line in split_statements(&stmt[open + 1..close]) {
            let trimmed = line.trim();
            if let Some(initial) = trimmed.strip_prefix("처음:") {
                self.state.insert(
                    format!("{actor}__지금상태"),
                    CellValue::Str(initial.trim().to_string()),
                );
            } else if let Some((from, to)) = trimmed.split_once("->") {
                transitions.insert(from.trim().to_string(), to.trim().to_string());
            }
        }
        self.machines.insert(actor.to_string(), transitions);
        Ok(())
    }

    fn execute_block(
        &mut self,
        body: &str,
        actor: Option<&str>,
        binds: &BTreeMap<String, CellValue>,
    ) -> Result<(), String> {
        let body = strip_comments(body);
        if body.trim_start().starts_with("고르기:") {
            return self.execute_statement(body.trim(), actor, binds);
        }
        let statements = split_statements(&body);
        let mut idx = 0usize;
        while idx < statements.len() {
            let stmt = statements[idx].trim();
            let next_stmt = statements.get(idx + 1).map(|s| s.trim());
            if next_stmt.is_some_and(|it| it.starts_with("아니면"))
                && self.execute_if(stmt, next_stmt, actor, binds)?
            {
                idx += 2;
                continue;
            }
            self.execute_statement(stmt, actor, binds)?;
            idx += 1;
        }
        Ok(())
    }

    fn execute_if(
        &mut self,
        stmt: &str,
        next_stmt: Option<&str>,
        actor: Option<&str>,
        binds: &BTreeMap<String, CellValue>,
    ) -> Result<bool, String> {
        let re = Regex::new(r"(?s)^만약\s+(.+?)\s+이라면\s*\{(.*)\}\s*$").unwrap();
        let Some(caps) = re.captures(stmt) else {
            return Ok(false);
        };
        let else_body = next_stmt
            .and_then(|it| Regex::new(r"(?s)^아니면\s*\{(.*)\}\s*$").unwrap().captures(it))
            .map(|caps| caps[1].to_string());
        if truthy(&self.eval_expr(&caps[1], actor, binds)?) {
            self.execute_block(&caps[2], actor, binds)?;
        } else if let Some(body) = else_body {
            self.execute_block(&body, actor, binds)?;
        }
        Ok(true)
    }

    fn execute_statement(
        &mut self,
        stmt: &str,
        actor: Option<&str>,
        binds: &BTreeMap<String, CellValue>,
    ) -> Result<(), String> {
        let stmt = stmt.trim();
        if stmt.is_empty() {
            return Ok(());
        }
        if self.execute_if(stmt, None, actor, binds)? {
            return Ok(());
        }
        if stmt.starts_with("덩이") {
            return self.execute_braced_body(stmt, actor, binds);
        }
        if stmt.starts_with("계약") {
            return self.execute_contract(stmt, actor, binds);
        }
        if stmt.contains("에 따라") {
            return self.execute_choice(stmt, actor, binds);
        }
        if stmt.starts_with("고르기:") {
            return self.execute_choose(stmt, actor, binds);
        }
        let send_re = Regex::new(r"(?s)^\((.*?)\)\s*([가-힣A-Za-z_][가-힣A-Za-z0-9_]*)\s*~~>\s*([가-힣A-Za-z_][가-힣A-Za-z0-9_]*)\s*$").unwrap();
        if let Some(caps) = send_re.captures(stmt) {
            let payload = self.eval_expr(&caps[1], actor, binds)?;
            return self.dispatch_signal(&caps[3], &caps[2], payload);
        }
        let assign_re =
            Regex::new(r"(?s)^([가-힣A-Za-z_][가-힣A-Za-z0-9_.]*)\s*<-\s*(.+)$").unwrap();
        if let Some(caps) = assign_re.captures(stmt) {
            let value = self.eval_expr(&caps[2], actor, binds)?;
            self.state
                .insert(flatten_name(&caps[1], actor), value);
            return Ok(());
        }
        let show_re = Regex::new(r"(?s)^(.+?)\s+보여주기\s*$").unwrap();
        if let Some(caps) = show_re.captures(stmt) {
            let value = self.eval_expr(&caps[1], actor, binds)?;
            self.outputs.push(value.display());
            return Ok(());
        }
        if stmt.starts_with("보임") {
            return self.execute_boim(stmt, actor, binds);
        }
        Ok(())
    }

    fn execute_braced_body(
        &mut self,
        stmt: &str,
        actor: Option<&str>,
        binds: &BTreeMap<String, CellValue>,
    ) -> Result<(), String> {
        let Some(open) = stmt.find('{') else {
            return Ok(());
        };
        let close = find_matching(stmt, open, '{', '}')?;
        self.execute_block(&stmt[open + 1..close], actor, binds)
    }

    fn execute_contract(
        &mut self,
        stmt: &str,
        actor: Option<&str>,
        binds: &BTreeMap<String, CellValue>,
    ) -> Result<(), String> {
        let Some(open) = stmt.find('{') else {
            return Ok(());
        };
        let close = find_matching(stmt, open, '{', '}')?;
        let body = &stmt[open + 1..close];
        let premise = extract_label_expr(body, "전제:");
        let post = extract_label_expr(body, "후행:");
        let fallback = extract_named_block(body, "물림 이라면")?;
        let ok = premise
            .as_deref()
            .map(|expr| self.eval_expr(expr, actor, binds).map(|v| truthy(&v)))
            .transpose()?
            .unwrap_or(true)
            && post
                .as_deref()
                .map(|expr| self.eval_expr(expr, actor, binds).map(|v| truthy(&v)))
                .transpose()?
                .unwrap_or(true);
        if !ok {
            if let Some(body) = fallback {
                self.execute_block(&body, actor, binds)?;
            }
        }
        Ok(())
    }

    fn execute_choice(
        &mut self,
        stmt: &str,
        actor: Option<&str>,
        binds: &BTreeMap<String, CellValue>,
    ) -> Result<(), String> {
        let re = Regex::new(r"(?s)^([가-힣A-Za-z_][가-힣A-Za-z0-9_]*)에\s+따라\s*\{(.*)\}\s*$").unwrap();
        let Some(caps) = re.captures(stmt) else {
            return Ok(());
        };
        let result = self
            .state
            .get(&caps[1])
            .cloned()
            .unwrap_or(CellValue::Choice {
                variant: "실패".to_string(),
                value: Box::new(CellValue::Num(0)),
                reason: "결과 없음".to_string(),
            });
        let branch_re = Regex::new(r"(?s)#(성공|실패)\(([^)]*)\)이면\s*\{(.*?)\}").unwrap();
        for branch in branch_re.captures_iter(&caps[2]) {
            let CellValue::Choice {
                variant,
                value,
                reason,
            } = result.clone()
            else {
                continue;
            };
            if variant != branch[1] {
                continue;
            }
            let bind_name = branch[2].split(':').next().unwrap_or("").trim().to_string();
            let mut local = binds.clone();
            if !bind_name.is_empty() {
                local.insert(
                    bind_name,
                    if variant == "성공" {
                        *value
                    } else {
                        CellValue::Str(reason)
                    },
                );
            }
            self.execute_block(&branch[3], actor, &local)?;
            break;
        }
        Ok(())
    }

    fn execute_choose(
        &mut self,
        stmt: &str,
        actor: Option<&str>,
        binds: &BTreeMap<String, CellValue>,
    ) -> Result<(), String> {
        let mut branches: Vec<(Option<String>, Vec<String>)> = Vec::new();
        let mut current_cond: Option<String> = None;
        let mut current_body: Vec<String> = Vec::new();
        let branch_re = Regex::new(r"^\{\s*(.*?)\s*\}인것:").unwrap();
        for raw in stmt.lines().skip(1) {
            let line = raw.trim();
            if let Some(caps) = branch_re.captures(line) {
                if current_cond.is_some() || !current_body.is_empty() {
                    branches.push((current_cond.take(), std::mem::take(&mut current_body)));
                }
                current_cond = Some(caps[1].to_string());
            } else if line == "아니면:" {
                if current_cond.is_some() || !current_body.is_empty() {
                    branches.push((current_cond.take(), std::mem::take(&mut current_body)));
                }
                current_cond = None;
            } else if !line.is_empty() {
                current_body.push(line.to_string());
            }
        }
        if current_cond.is_some() || !current_body.is_empty() {
            branches.push((current_cond, current_body));
        }
        for (cond, body) in branches {
            if cond
                .as_deref()
                .map(|expr| self.eval_expr(expr, actor, binds).map(|v| truthy(&v)))
                .transpose()?
                .unwrap_or(true)
            {
                self.execute_block(&body.join("\n"), actor, binds)?;
                break;
            }
        }
        Ok(())
    }

    fn execute_boim(
        &mut self,
        stmt: &str,
        actor: Option<&str>,
        binds: &BTreeMap<String, CellValue>,
    ) -> Result<(), String> {
        let Some(open) = stmt.find('{') else {
            return Ok(());
        };
        let close = find_matching(stmt, open, '{', '}')?;
        for row in split_statements(&stmt[open + 1..close]) {
            if let Some((key, expr)) = row.split_once(':') {
                let value = self.eval_expr(expr, actor, binds)?;
                self.boim_rows.push((sanitize_key(key.trim()), value));
            }
        }
        Ok(())
    }

    fn dispatch_signal(
        &mut self,
        actor: &str,
        signal: &str,
        payload: CellValue,
    ) -> Result<(), String> {
        let body = self
            .handlers
            .get(actor)
            .and_then(|items| items.get(signal))
            .cloned()
            .unwrap_or_default();
        let mut binds = BTreeMap::new();
        for key in ["값", "내용", "이름", "정보"] {
            binds.insert(key.to_string(), payload.clone());
        }
        for stmt in body {
            self.execute_statement(stmt.trim_end_matches('.'), Some(actor), &binds)?;
        }
        Ok(())
    }

    fn eval_expr(
        &self,
        expr: &str,
        actor: Option<&str>,
        binds: &BTreeMap<String, CellValue>,
    ) -> Result<CellValue, String> {
        let expr = expr.trim();
        if expr.is_empty() {
            return Ok(CellValue::Str(String::new()));
        }
        if expr.starts_with('(') && expr.ends_with(')') {
            if find_matching(expr, 0, '(', ')').ok() == Some(expr.len() - 1) {
                return self.eval_expr(&expr[1..expr.len() - 1], actor, binds);
            }
        }
        if let Some(left) = expr.strip_suffix("~을 숫자로") {
            let raw = self.eval_expr(left, actor, binds)?.display();
            return match raw.parse::<i64>() {
                Ok(value) => Ok(CellValue::Choice {
                    variant: "성공".to_string(),
                    value: Box::new(CellValue::Num(value)),
                    reason: String::new(),
                }),
                Err(_) => Ok(CellValue::Choice {
                    variant: "실패".to_string(),
                    value: Box::new(CellValue::Num(0)),
                    reason: format!("숫자로 바꿀 수 없음: {raw}"),
                }),
            };
        }
        if let Some(value) = binds.get(expr) {
            return Ok(value.clone());
        }
        if let Some(key) = expr.strip_prefix("알림.정보.") {
            return Ok(binds
                .get(key)
                .cloned()
                .unwrap_or(CellValue::Str(String::new())));
        }
        if let Some(key) = expr.strip_prefix("정보.") {
            return Ok(binds
                .get(key)
                .cloned()
                .unwrap_or(CellValue::Str(String::new())));
        }
        if expr == "참" {
            return Ok(CellValue::Bool(true));
        }
        if expr == "거짓" {
            return Ok(CellValue::Bool(false));
        }
        if let Ok(value) = expr.parse::<i64>() {
            return Ok(CellValue::Num(value));
        }
        if expr.starts_with('"') && expr.ends_with('"') && expr.len() >= 2 {
            return Ok(CellValue::Str(expr[1..expr.len() - 1].to_string()));
        }
        for op in ["==", "!=", "<=", ">=", "<", ">"] {
            if let Some((left, right)) = split_binary(expr, op) {
                let left = self.eval_expr(&left, actor, binds)?;
                let right = self.eval_expr(&right, actor, binds)?;
                return compare_values(&left, &right, op);
            }
        }
        for op in ["+", "-"] {
            if let Some((left, right)) = split_binary(expr, op) {
                let left = as_i64(self.eval_expr(&left, actor, binds)?);
                let right = as_i64(self.eval_expr(&right, actor, binds)?);
                return Ok(CellValue::Num(if op == "+" {
                    left + right
                } else {
                    left - right
                }));
            }
        }
        let key = flatten_name(expr, actor);
        Ok(self
            .state
            .get(&key)
            .cloned()
            .unwrap_or(CellValue::Str(expr.to_string())))
    }

    fn to_executable_source(&self) -> String {
        let mut lines = vec!["채비 {".to_string()];
        for (key, value) in &self.state {
            if matches!(value, CellValue::Choice { .. }) {
                continue;
            }
            if matches!(value, CellValue::Str(text) if text.parse::<i64>().is_ok()) {
                continue;
            }
            lines.push(format!(
                "  {key}: {} <- {}.",
                value.ddn_type(),
                value.ddn_literal()
            ));
        }
        lines.push("}.".to_string());
        lines.push(String::new());
        for output in &self.outputs {
            lines.push(format!("{} 보여주기.", json_string(output)));
        }
        if !self.boim_rows.is_empty() {
            lines.push("보임 {".to_string());
            for (key, value) in &self.boim_rows {
                lines.push(format!("  {key}: {}.", value.ddn_literal()));
            }
            lines.push("}.".to_string());
        }
        lines.join("\n") + "\n"
    }
}

fn strip_comments(source: &str) -> String {
    source
        .lines()
        .map(|line| line.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

fn split_statements(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut brace = 0i32;
    let mut paren = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    let chars: Vec<(usize, char)> = body.char_indices().collect();
    for (pos, ch) in &chars {
        if in_string {
            if escaped {
                escaped = false;
            } else if *ch == '\\' {
                escaped = true;
            } else if *ch == '"' {
                in_string = false;
            }
            continue;
        }
        match *ch {
            '"' => in_string = true,
            '{' => brace += 1,
            '}' => brace -= 1,
            '(' => paren += 1,
            ')' => paren -= 1,
            '.' if brace == 0 && paren == 0 => {
                let prev = body[..*pos].chars().next_back().unwrap_or('\0');
                let next = body[*pos + 1..].chars().next().unwrap_or('\0');
                if is_ident_char(prev) && is_ident_char(next) {
                    continue;
                }
                let chunk = body[start..*pos].trim();
                if !chunk.is_empty() {
                    out.push(chunk.to_string());
                }
                start = *pos + 1;
            }
            _ => {}
        }
    }
    let tail = body[start..].trim();
    if !tail.is_empty() {
        out.push(tail.to_string());
    }
    out
}

fn remove_named_blocks(source: &str, name: &str) -> Result<(String, Vec<String>), String> {
    let mut out = String::new();
    let mut bodies = Vec::new();
    let mut pos = 0usize;
    while let Some(rel) = source[pos..].find(name) {
        let start = pos + rel;
        let Some(open_rel) = source[start..].find('{') else {
            break;
        };
        let open = start + open_rel;
        let close = find_matching(source, open, '{', '}')?;
        let mut end = close + 1;
        while source[end..].starts_with(char::is_whitespace) {
            end += source[end..].chars().next().unwrap().len_utf8();
            if end >= source.len() {
                break;
            }
        }
        if end < source.len() && source[end..].starts_with('.') {
            end += 1;
        }
        out.push_str(&source[pos..start]);
        bodies.push(source[open + 1..close].to_string());
        pos = end;
    }
    out.push_str(&source[pos..]);
    Ok((out, bodies))
}

fn remove_imja_blocks(source: &str) -> Result<(String, Vec<(String, String)>), String> {
    let re = Regex::new(r"([가-힣A-Za-z_][가-힣A-Za-z0-9_]*)\s*:\s*임자\s*=").unwrap();
    let mut out = String::new();
    let mut bodies = Vec::new();
    let mut pos = 0usize;
    while let Some(caps) = re.captures(&source[pos..]) {
        let whole = caps.get(0).unwrap();
        let start = pos + whole.start();
        let open = source[start..]
            .find('{')
            .map(|rel| start + rel)
            .ok_or_else(|| "E_CURRENTLINE_IMJA_BLOCK".to_string())?;
        let close = find_matching(source, open, '{', '}')?;
        let mut end = close + 1;
        while end < source.len()
            && source[end..]
                .chars()
                .next()
                .is_some_and(char::is_whitespace)
        {
            end += source[end..].chars().next().unwrap().len_utf8();
        }
        if end < source.len() && source[end..].starts_with('.') {
            end += 1;
        }
        out.push_str(&source[pos..start]);
        bodies.push((caps[1].to_string(), source[open + 1..close].to_string()));
        pos = end;
    }
    out.push_str(&source[pos..]);
    Ok((out, bodies))
}

fn find_matching(text: &str, open_index: usize, open_ch: char, close_ch: char) -> Result<usize, String> {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    for (idx, ch) in text[open_index..].char_indices() {
        let idx = open_index + idx;
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        if ch == '"' {
            in_string = true;
        } else if ch == open_ch {
            depth += 1;
        } else if ch == close_ch {
            depth -= 1;
            if depth == 0 {
                return Ok(idx);
            }
        }
    }
    Err("E_CURRENTLINE_BRACE_MATCH".to_string())
}

fn extract_label_expr(body: &str, label: &str) -> Option<String> {
    let start = body.find(label)? + label.len();
    let end = body[start..].find('.')? + start;
    Some(body[start..end].trim().to_string())
}

fn extract_named_block(body: &str, name: &str) -> Result<Option<String>, String> {
    let Some(start) = body.find(name) else {
        return Ok(None);
    };
    let Some(open_rel) = body[start..].find('{') else {
        return Ok(None);
    };
    let open = start + open_rel;
    let close = find_matching(body, open, '{', '}')?;
    Ok(Some(body[open + 1..close].to_string()))
}

fn flatten_name(name: &str, actor: Option<&str>) -> String {
    let name = name.trim();
    if let Some(field) = name.strip_prefix("제.") {
        if let Some(actor) = actor {
            return format!("{actor}__{field}");
        }
    }
    if name == "지금상태" {
        if let Some(actor) = actor {
            return format!("{actor}__지금상태");
        }
    }
    if let Some((left, right)) = name.split_once('.') {
        return format!("{left}__{right}");
    }
    name.to_string()
}

fn split_binary(expr: &str, op: &str) -> Option<(String, String)> {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut prev = '\0';
    for (idx, ch) in expr.char_indices() {
        if ch == '"' && prev != '\\' {
            in_string = !in_string;
        }
        if !in_string {
            if ch == '(' {
                depth += 1;
            } else if ch == ')' {
                depth -= 1;
            } else if depth == 0 && expr[idx..].starts_with(op) {
                let before = expr[..idx].chars().next_back().unwrap_or(' ');
                let after = expr[idx + op.len()..].chars().next().unwrap_or(' ');
                if (op == "+" || op == "-")
                    && (!before.is_whitespace() || !after.is_whitespace())
                {
                    prev = ch;
                    continue;
                }
                return Some((
                    expr[..idx].trim().to_string(),
                    expr[idx + op.len()..].trim().to_string(),
                ));
            }
        }
        prev = ch;
    }
    None
}

fn compare_values(left: &CellValue, right: &CellValue, op: &str) -> Result<CellValue, String> {
    let result = match op {
        "==" => left.display() == right.display(),
        "!=" => left.display() != right.display(),
        "<" => as_i64(left.clone()) < as_i64(right.clone()),
        ">" => as_i64(left.clone()) > as_i64(right.clone()),
        "<=" => as_i64(left.clone()) <= as_i64(right.clone()),
        ">=" => as_i64(left.clone()) >= as_i64(right.clone()),
        _ => false,
    };
    Ok(CellValue::Bool(result))
}

fn as_i64(value: CellValue) -> i64 {
    match value {
        CellValue::Bool(flag) => i64::from(flag),
        CellValue::Num(value) => value,
        CellValue::Str(value) => value.parse::<i64>().unwrap_or(0),
        CellValue::Choice { value, .. } => as_i64(*value),
    }
}

fn truthy(value: &CellValue) -> bool {
    match value {
        CellValue::Bool(flag) => *flag,
        CellValue::Num(value) => *value != 0,
        CellValue::Str(value) => !value.is_empty(),
        CellValue::Choice { variant, .. } => variant == "성공",
    }
}

fn is_ident_char(ch: char) -> bool {
    ch == '_' || ch.is_alphanumeric() || ('가'..='힣').contains(&ch)
}

fn sanitize_key(key: &str) -> String {
    let out: String = key
        .chars()
        .map(|ch| if is_ident_char(ch) { ch } else { '_' })
        .collect();
    if out.is_empty() {
        "값".to_string()
    } else {
        out
    }
}

fn json_string(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_string())
}

#[cfg(test)]
mod tests {
    use super::apply_currentline_cell;

    #[test]
    fn currentline_accepts_imja_field_decl_and_assignment() {
        let out = apply_currentline_cell(
            r#"
문지기:임자 = {
  체력: 수 <- 100.
  제.체력 <- 제.체력 - 10.
}.
"#,
            None,
        )
        .expect("currentline");
        assert!(out.project_source.contains("문지기__체력: 수 <- 90."));
    }

    #[test]
    fn currentline_accepts_choice_conversion() {
        let out = apply_currentline_cell(
            r#"
전력문자 <- "82a".
전력결과 <- 전력문자 ~을 숫자로.
전력결과에 따라 {
  #성공(값)이면 { 값 보여주기. }
  #실패(이유)이면 { 이유 보여주기. }
}.
"#,
            None,
        )
        .expect("currentline");
        assert!(out.project_source.contains("숫자로 바꿀 수 없음"));
    }
}
