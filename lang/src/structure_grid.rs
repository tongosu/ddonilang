//! RD-511/512 grammar island for the registered finite-chain profile.
use crate::ast::Span as SourceSpan;
use crate::parser::ParseError;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

const REGISTRATION: &str = include_str!("structure_grid_r1.json");

#[derive(Clone, Copy, Debug, Serialize)]
struct Span {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy)]
struct Token<'a> {
    text: &'a str,
    span: Span,
}

#[derive(Debug, Serialize)]
pub struct Diagnostic {
    code: &'static str,
    detail: &'static str,
    span: Span,
}

fn reject(token: Token<'_>, code: &'static str, detail: &'static str) -> Diagnostic {
    Diagnostic {
        code,
        detail,
        span: token.span,
    }
}

struct Parser<'a> {
    source: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Self {
        Self { source, pos: 0 }
    }

    fn next(&mut self) -> Token<'a> {
        loop {
            self.pos = self.source.len() - self.source[self.pos..].trim_start().len();
            let rest = &self.source[self.pos..];
            if rest.starts_with("//") {
                self.pos += rest.find('\n').unwrap_or(rest.len());
            } else {
                break;
            }
        }
        let start = self.pos;
        let rest = &self.source[start..];
        let word = |ch: char| ch.is_alphanumeric() || ch == '_';
        let count = if let Some(symbol) = ["=<", "=>", "->", "-["]
            .into_iter()
            .find(|s| rest.starts_with(s))
        {
            symbol.len()
        } else if rest.starts_with(word) {
            rest.find(|ch| !word(ch)).unwrap_or(rest.len())
        } else {
            rest.chars().next().map_or(0, char::len_utf8)
        };
        self.pos += count;
        Token {
            text: &self.source[start..self.pos],
            span: Span {
                start,
                end: self.pos,
            },
        }
    }

    fn peek(&self) -> &'a str {
        Self {
            source: self.source,
            pos: self.pos,
        }
        .next()
        .text
    }

    fn take(&mut self, expected: &str) -> Result<Token<'a>, Diagnostic> {
        let token = self.next();
        if token.text == expected {
            Ok(token)
        } else {
            Err(reject(
                token,
                "E_STRUCTURE_SYNTAX",
                "등록 차림새의 작성 꼴과 맞지 않습니다",
            ))
        }
    }

    fn name(&mut self) -> Result<Token<'a>, Diagnostic> {
        let token = self.next();
        if token.text.starts_with(char::is_alphabetic) {
            Ok(token)
        } else {
            Err(reject(
                token,
                "E_STRUCTURE_SYNTAX",
                "선언 이름이 필요합니다",
            ))
        }
    }

    fn number(&mut self) -> Result<(u32, Token<'a>), Diagnostic> {
        let token = self.next();
        token
            .text
            .parse::<u32>()
            .map(|number| (number, token))
            .map_err(|_| {
                reject(
                    token,
                    "E_STRUCTURE_SYNTAX",
                    "범주 대상은 부호 없는 정수입니다",
                )
            })
    }

    fn numbers(&mut self) -> Result<Vec<u32>, Diagnostic> {
        self.take("(")?;
        let mut values = vec![self.number()?.0];
        while self.peek() == "," {
            self.take(",")?;
            values.push(self.number()?.0);
        }
        self.take(")")?;
        Ok(values)
    }
}

#[derive(Clone)]
struct Functor<'a> {
    declaration: Token<'a>,
    from: &'a str,
    to: &'a str,
    images: BTreeMap<u32, u32>,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
struct Arrow {
    from: u32,
    to: u32,
}

#[derive(Clone)]
struct Cell<'a> {
    from: Functor<'a>,
    to: Functor<'a>,
    components: BTreeMap<u32, Arrow>,
}

#[derive(Default)]
struct Grid<'a> {
    categories: BTreeMap<&'a str, Vec<u32>>,
    functors: BTreeMap<&'a str, Functor<'a>>,
    cells: BTreeMap<&'a str, Cell<'a>>,
    names: BTreeSet<&'a str>,
    results: BTreeMap<&'a str, Value>,
    output: Vec<(SourceSpan, String)>,
}

fn lookup<'b, 'a, T>(map: &'b BTreeMap<&'a str, T>, token: Token<'a>) -> Result<&'b T, Diagnostic> {
    map.get(token.text)
        .ok_or_else(|| reject(token, "E_STRUCTURE_NAME", "해소할 선언이 없습니다"))
}

impl<'a> Grid<'a> {
    fn category(&mut self, parser: &mut Parser<'a>, name: Token<'a>) -> Result<(), Diagnostic> {
        let objects = parser.numbers()?;
        if !objects.windows(2).all(|pair| pair[0] < pair[1]) {
            return Err(reject(
                name,
                "E_STRUCTURE_CHAIN",
                "대상은 엄격한 오름차순으로 한 번씩 선언합니다",
            ));
        }
        self.categories.insert(name.text, objects);
        Ok(())
    }

    fn functor(&mut self, parser: &mut Parser<'a>, name: Token<'a>) -> Result<(), Diagnostic> {
        let first = parser.name()?;
        let functor = if parser.peek() == "->" {
            parser.take("->")?;
            let second = parser.name()?;
            let domain = lookup(&self.categories, first)?;
            let codomain = lookup(&self.categories, second)?;
            let images = parser.numbers()?;
            if domain.len() != images.len()
                || !images
                    .iter()
                    .all(|image| codomain.binary_search(image).is_ok())
                || !images.windows(2).all(|pair| pair[0] <= pair[1])
            {
                return Err(reject(
                    name,
                    "E_STRUCTURE_FUNCTOR",
                    "전체 대상 대응과 순서 보존이 필요합니다",
                ));
            }
            Functor {
                declaration: name,
                from: first.text,
                to: second.text,
                images: domain.iter().copied().zip(images).collect(),
            }
        } else {
            parser.take("잇고")?;
            let second = parser.name()?;
            parser.take("잇기")?;
            let left = lookup(&self.functors, first)?;
            let right = lookup(&self.functors, second)?;
            if left.to != right.from {
                return Err(reject(
                    second,
                    "E_STRUCTURE_BOUNDARY",
                    "함자의 중간 범주 선언이 다릅니다",
                ));
            }
            Functor {
                declaration: name,
                from: left.from,
                to: right.to,
                images: left
                    .images
                    .iter()
                    .map(|(&object, image)| (object, right.images[image]))
                    .collect(),
            }
        };
        self.functors.insert(name.text, functor);
        Ok(())
    }

    fn cell(
        &mut self,
        parser: &mut Parser<'a>,
        name: Token<'a>,
        registration: &Value,
    ) -> Result<(), Diagnostic> {
        let left = parser.name()?;
        parser.take("=<")?;
        parser.take(registration["head"].as_str().expect("sealed head"))?;
        parser.take(">")?;
        parser.take("[")?;
        let incidence = parser.next();
        let duplicate = match incidence.text {
            "공통경계" => None,
            "양끝" => {
                parser.take("=")?;
                parser.take("(")?;
                let first = parser.name()?;
                parser.take(",")?;
                parser.name()?;
                parser.take(")")?;
                Some(first)
            }
            _ => {
                return Err(reject(
                    incidence,
                    "E_STRUCTURE_SYNTAX",
                    "공통경계 접속이 필요합니다",
                ))
            }
        };
        parser.take("]")?;
        parser.take("(")?;
        let mut authored = Vec::new();
        loop {
            let (object, object_token) = parser.number()?;
            parser.take(":")?;
            let (from, from_token) = parser.number()?;
            parser.take("->")?;
            let (to, to_token) = parser.number()?;
            authored.push((
                object,
                object_token,
                Arrow { from, to },
                Token {
                    text: &parser.source[from_token.span.start..to_token.span.end],
                    span: Span {
                        start: from_token.span.start,
                        end: to_token.span.end,
                    },
                },
            ));
            if parser.peek() != "," {
                break;
            }
            parser.take(",")?;
        }
        parser.take(")")?;
        parser.take("{")?;
        parser.take(registration["body_rule"].as_str().expect("sealed rule"))?;
        parser.take("}")?;
        parser.take("=>")?;
        let right = parser.name()?;
        if let Some(token) = duplicate {
            return Err(reject(
                token,
                "E_STRUCTURE_CHANNEL_DUPLICATE",
                "양끝 함자 참조가 접속에도 공급되었습니다",
            ));
        }
        let from = lookup(&self.functors, left)?.clone();
        let to = lookup(&self.functors, right)?.clone();
        if (from.from, from.to) != (to.from, to.to) {
            return Err(reject(
                right,
                "E_STRUCTURE_BOUNDARY",
                "자연변환의 양끝에 공통 함자 서명이 필요합니다",
            ));
        }
        let mut components = BTreeMap::new();
        for (object, object_token, arrow, arrow_token) in authored {
            let (Some(&start), Some(&end)) = (from.images.get(&object), to.images.get(&object))
            else {
                return Err(reject(
                    object_token,
                    "E_STRUCTURE_COMPONENT_DOMAIN",
                    "정의역 대상이 아닙니다",
                ));
            };
            if arrow
                != (Arrow {
                    from: start,
                    to: end,
                })
                || arrow.from > arrow.to
            {
                return Err(reject(
                    arrow_token,
                    "E_STRUCTURE_COMPONENT_ENDPOINT",
                    "성분의 두 끝이 함자 대응과 맞지 않습니다",
                ));
            }
            if components.insert(object, arrow).is_some() {
                return Err(reject(
                    object_token,
                    "E_STRUCTURE_COMPONENT_DUPLICATE",
                    "같은 대상의 성분을 다시 공급했습니다",
                ));
            }
        }
        if components.len() != from.images.len() {
            return Err(reject(
                name,
                "E_STRUCTURE_COMPONENT_MISSING",
                "정의역의 모든 대상에 성분이 필요합니다",
            ));
        }
        // In a chain each x <= y has exactly one arrow. The checked functors
        // give both horizontal arrows, and the checked components give both
        // vertical arrows. Both paths for every x <= y have the same ends.
        self.cells.insert(
            name.text,
            Cell {
                from,
                to,
                components,
            },
        );
        Ok(())
    }

    fn show(&mut self, parser: &mut Parser<'a>, name: Token<'a>) -> Result<(), Diagnostic> {
        let first = parser.name()?;
        let mut cell = lookup(&self.cells, first)?.clone();
        let mut path = vec![first.text];
        let mut joins = Vec::new();
        while parser.peek() == "-[" {
            parser.take("-[")?;
            parser.take("오른쪽")?;
            parser.take(":")?;
            parser.take("왼쪽")?;
            parser.take("]")?;
            parser.take("-")?;
            let operand = parser.name()?;
            let next = lookup(&self.cells, operand)?;
            if cell.to.declaration.span.start != next.from.declaration.span.start {
                return Err(reject(
                    operand,
                    "E_STRUCTURE_COMPOSITION_BOUNDARY",
                    "중간 경계가 같은 함자 선언이 아닙니다",
                ));
            }
            joins.push(json!({
                "declaration":cell.to.declaration.text,
                "span":cell.to.declaration.span
            }));
            for (object, arrow) in &mut cell.components {
                arrow.to = next.components[object].to;
            }
            cell.to = next.to.clone();
            path.push(operand.text);
        }
        let rendered = cell
            .components
            .iter()
            .map(|(object, arrow)| format!("{object}: {}→{}", arrow.from, arrow.to))
            .collect::<Vec<_>>()
            .join(" · ");
        self.output.push((
            SourceSpan::new(name.span.start, name.span.end),
            format!("{} {rendered}", name.text),
        ));
        let components: Vec<_> = cell
            .components
            .iter()
            .map(|(object, arrow)| json!({"object":object,"from":arrow.from,"to":arrow.to}))
            .collect();
        self.results.insert(
            name.text,
            json!({"path":path,"joins":joins,"components":components}),
        );
        Ok(())
    }
}

pub(crate) struct ResolvedGrid {
    pub(crate) canonical_source: String,
    pub(crate) output: Vec<(SourceSpan, String)>,
    report: Value,
}

impl Diagnostic {
    fn into_parse_error(self) -> ParseError {
        ParseError {
            span: SourceSpan::new(self.span.start, self.span.end),
            message: format!("{}: {}", self.code, self.detail),
        }
    }
}

pub(crate) fn diagnostic_code(message: &str) -> Option<&'static str> {
    let (code, _) = message.split_once(':')?;
    [
        "E_STRUCTURE_SYNTAX",
        "E_STRUCTURE_NAME",
        "E_STRUCTURE_CHAIN",
        "E_STRUCTURE_FUNCTOR",
        "E_STRUCTURE_BOUNDARY",
        "E_STRUCTURE_CHANNEL_DUPLICATE",
        "E_STRUCTURE_COMPONENT_DOMAIN",
        "E_STRUCTURE_COMPONENT_ENDPOINT",
        "E_STRUCTURE_COMPONENT_DUPLICATE",
        "E_STRUCTURE_COMPONENT_MISSING",
        "E_STRUCTURE_COMPOSITION_BOUNDARY",
        "E_STRUCTURE_PROFILE_UNREGISTERED",
        "E_STRUCTURE_DECLARATION_DUPLICATE",
        "E_STRUCTURE_SOURCE_LIMIT",
    ]
    .into_iter()
    .find(|known| *known == code)
}

/// The math notebook presents the same checked result as the shared parser.
pub fn observe(source: &str, path: &str) -> Option<Result<Value, ParseError>> {
    resolve(source, path).map(|result| result.map(|grid| grid.report))
}

pub(crate) fn resolve(source: &str, path: &str) -> Option<Result<ResolvedGrid, ParseError>> {
    let mut parser = Parser::new(source);
    if parser.next().text != "(" || !matches!(parser.next().text, "차림새" | "설정") {
        return None;
    }
    let mut lookahead = Parser {
        source,
        pos: parser.pos,
    };
    if lookahead.next().text != "="
        || !lookahead.next().text.starts_with(char::is_alphabetic)
        || lookahead.next().text != ")"
        || lookahead.next().text != "짜임"
    {
        return None;
    }
    Some(run(parser, path).map_err(Diagnostic::into_parse_error))
}

fn run(mut parser: Parser<'_>, path: &str) -> Result<ResolvedGrid, Diagnostic> {
    parser.take("=")?;
    let profile = parser.name()?;
    if parser.source.len() > 65_536 {
        return Err(reject(
            profile,
            "E_STRUCTURE_SOURCE_LIMIT",
            "격자 원문 크기 한도를 넘었습니다",
        ));
    }
    parser.take(")")?;
    parser.take("짜임")?;
    let registration: Value = serde_json::from_str(REGISTRATION).expect("built-in registration");
    if registration["status"] != "active" || registration["selector"] != profile.text {
        return Err(reject(
            profile,
            "E_STRUCTURE_PROFILE_UNREGISTERED",
            "활성 차림새 등록을 찾을 수 없습니다",
        ));
    }
    parser.take("{")?;
    let body = Parser {
        source: parser.source,
        pos: parser.pos,
    };
    let mut grid = Grid::default();
    while parser.peek() != "}" {
        let kind = parser.next();
        let name = parser.name()?;
        if !grid.names.insert(name.text) {
            return Err(reject(
                name,
                "E_STRUCTURE_DECLARATION_DUPLICATE",
                "이미 사용한 선언 이름입니다",
            ));
        }
        parser.take("=")?;
        match kind.text {
            "순서범주" => grid.category(&mut parser, name)?,
            "함자" => grid.functor(&mut parser, name)?,
            "이음" => grid.cell(&mut parser, name, &registration)?,
            "관찰" => grid.show(&mut parser, name)?,
            _ => {
                return Err(reject(
                    kind,
                    "E_STRUCTURE_SYNTAX",
                    "이 차림새의 선언 종류가 아닙니다",
                ))
            }
        }
        parser.take(".")?;
    }
    parser.take("}")?;
    parser.take(".")?;
    parser.take("")?;
    Ok(ResolvedGrid {
        canonical_source: canonical_source(profile.text, body),
        output: grid.output,
        report: json!({
        "schema":"ddn.structure.finite-chain.observation.v1",
        "status":"accepted",
        "path":path,
        "source_revision":blake3::hash(parser.source.as_bytes()).to_hex().to_string(),
        "registration":registration,
        "observation_only":true,
        "results":grid.results
        }),
    })
}

// Render checked island tokens, never the lowered display statements.
fn canonical_source(profile: &str, mut body: Parser<'_>) -> String {
    let mut output = format!("(차림새={profile}) 짜임 {{\n");
    let mut previous = "";
    while body.peek() != "" {
        let token = body.next().text;
        if previous.is_empty() && token == "}" {
            output.push_str("}.\n");
            break;
        }
        if previous.is_empty() {
            output.push_str("    ");
        } else if !matches!(token, "." | ")" | "]" | "}" | "," | ">" | ":" | "-")
            && !matches!(previous, "(" | "-[" | "[" | "{" | "=<" | ":" | "->")
            && !(matches!(previous, ">" | "]" | ")") && matches!(token, "[" | "(" | "{"))
        {
            output.push(' ');
        }
        output.push_str(token);
        if token == "." {
            output.push('\n');
            previous = "";
        } else {
            previous = token;
        }
    }
    output
}
