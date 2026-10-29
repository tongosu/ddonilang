import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, "..");
const cargoTarget = process.env.CARGO_TARGET_DIR;

if (!cargoTarget) {
  throw new Error("CARGO_TARGET_DIR is required; mutation builds must stay outside the worktree");
}

const parserPath = path.join(root, "lang", "src", "parser.rs");
const normalizerPath = path.join(root, "lang", "src", "normalizer.rs");
const originals = new Map([
  [parserPath, fs.readFileSync(parserPath)],
  [normalizerPath, fs.readFileSync(normalizerPath)],
]);

function sha256(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}

function normalizedText(file) {
  return fs.readFileSync(file, "utf8").replace(/\r\n/g, "\n");
}

function writeSingleReplacement(file, before, after) {
  const source = normalizedText(file);
  const first = source.indexOf(before);
  const last = source.lastIndexOf(before);
  if (first < 0 || first !== last) {
    throw new Error(`mutation anchor must occur exactly once: ${path.relative(root, file)}`);
  }
  fs.writeFileSync(file, source.slice(0, first) + after + source.slice(first + before.length), "utf8");
}

function writeSingleReplacementInRange(file, startAnchor, endAnchor, before, after) {
  const source = normalizedText(file);
  const start = source.indexOf(startAnchor);
  const end = source.indexOf(endAnchor, start + startAnchor.length);
  if (start < 0 || end < 0) {
    throw new Error(`mutation range missing: ${path.relative(root, file)}`);
  }
  const range = source.slice(start, end);
  const first = range.indexOf(before);
  const last = range.lastIndexOf(before);
  if (first < 0 || first !== last) {
    throw new Error(`mutation anchor must occur exactly once in range: ${path.relative(root, file)}`);
  }
  const mutated = range.slice(0, first) + after + range.slice(first + before.length);
  fs.writeFileSync(file, source.slice(0, start) + mutated + source.slice(end), "utf8");
}

function runPack() {
  return spawnSync(
    "cargo",
    ["test", "-p", "ddonirang-lang", "--test", "v25_definition_binding_pack", "--", "--nocapture"],
    {
      cwd: root,
      encoding: "utf8",
      env: { ...process.env, CARGO_NET_OFFLINE: "true", CARGO_TARGET_DIR: cargoTarget },
      shell: process.platform === "win32",
    },
  );
}

function requireGreen(label) {
  const result = runPack();
  if (result.status !== 0) {
    throw new Error(`${label} expected GREEN\n${result.stdout}\n${result.stderr}`);
  }
  console.log(`${label}: GREEN`);
}

function requireRed(label) {
  const result = runPack();
  if (result.status === 0) {
    throw new Error(`${label} expected RED but pack passed`);
  }
  console.log(`${label}: RED`);
}

function restoreAll() {
  for (const [file, bytes] of originals) {
    fs.writeFileSync(file, bytes);
  }
}

const definitionBuilder = `        Ok(Stmt::Definition {
            id: self.next_id(),
            span: start.merge(&self.previous_span()),
            mood,
            name,
            type_ref,
            value,
        })`;
const collapsedBuilder = `        let _ = type_ref;
        Ok(Stmt::Mutate {
            id: self.next_id(),
            span: start.merge(&self.previous_span()),
            mood,
            target: Expr {
                id: self.next_id(),
                span: start,
                kind: ExprKind::Ident(name),
            },
            value,
            deferred: false,
        })`;

const v25DefineArm = `            SourceEpoch::V25 => self.expect(&TokenKind::Define, ":=")?,`;
const v25AliasArm = `            SourceEpoch::V25 => {
                if self.check(&TokenKind::Equals) {
                    self.advance();
                } else {
                    self.expect(&TokenKind::Define, ":=")?;
                }
            }`;
const migrationGuard = [
  "        if self.source_epoch == SourceEpoch::V25 && self.check(&TokenKind::Equals) {",
  "            return Err(ParseError {",
  "                span: self.current_span(),",
  '                message: "E_V25_DEFINITION_MIGRATION_REQUIRED: V25 정의에는 `:=`가 필요하며 이전 `=` 원문은 명시적 마이그레이션 입력으로 분리해야 합니다".to_string(),',
  "            });",
  "        }",
  "",
].join("\n");

const definitionCanon = `                self.write(" := ");
                self.normalize_expr(value);
                self.write_stmt_terminator(stmt);`;
const bindingCanon = `                self.write(" <- ");
                self.normalize_expr(value);
                self.write_stmt_terminator(stmt);`;

try {
  requireGreen("baseline");

  writeSingleReplacement(parserPath, definitionBuilder, collapsedBuilder);
  requireRed("M1_DEFINITION_TO_BINDING_COLLAPSE");
  restoreAll();

  writeSingleReplacementInRange(
    parserPath,
    "    fn parse_seed_def",
    "    fn parse_params",
    migrationGuard,
    "",
  );
  writeSingleReplacement(parserPath, v25DefineArm, v25AliasArm);
  requireRed("M2_OLD_EQUALS_SILENT_ALIAS");
  restoreAll();

  writeSingleReplacement(normalizerPath, definitionCanon, bindingCanon);
  requireRed("M3_DEFINITION_CANON_BINDING_SYMBOL");
  restoreAll();

  requireGreen("restored");
} finally {
  restoreAll();
}

for (const [file, bytes] of originals) {
  const actual = fs.readFileSync(file);
  if (sha256(actual) !== sha256(bytes)) {
    throw new Error(`mutation residue: ${path.relative(root, file)}`);
  }
}

console.log("DDN_V25_PC01_MUTATION_RED_GREEN_PASS residue=0");
