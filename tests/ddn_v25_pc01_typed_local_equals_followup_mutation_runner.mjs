import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const parserPath = path.join(root, "lang", "src", "parser.rs");
const original = fs.readFileSync(parserPath);
const cargoTarget = process.env.CARGO_TARGET_DIR;

if (!cargoTarget) {
  throw new Error("CARGO_TARGET_DIR is required; mutation builds must stay outside the worktree");
}

function sha256(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}

function normalizedSource() {
  return fs.readFileSync(parserPath, "utf8").replaceAll("\r\n", "\n");
}

function replaceInFunction(startAnchor, endAnchor, before, after, label) {
  const source = normalizedSource();
  const start = source.indexOf(startAnchor);
  const end = source.indexOf(endAnchor, start + startAnchor.length);
  if (start < 0 || end < 0) throw new Error(`${label}: function boundary missing`);
  const body = source.slice(start, end);
  const first = body.indexOf(before);
  if (first < 0 || first !== body.lastIndexOf(before)) {
    throw new Error(`${label}: mutation anchor must occur exactly once in function`);
  }
  const mutated = body.slice(0, first) + after + body.slice(first + before.length);
  fs.writeFileSync(parserPath, source.slice(0, start) + mutated + source.slice(end), "utf8");
}

function restore() {
  fs.writeFileSync(parserPath, original);
}

function runTargeted() {
  return spawnSync(
    "cargo",
    [
      "test",
      "-p",
      "ddonirang-lang",
      "--test",
      "v25_definition_binding_pack",
      "v25_typed_local_legacy_equals_targeted_matrix",
      "--",
      "--nocapture",
    ],
    {
      cwd: root,
      encoding: "utf8",
      env: { ...process.env, CARGO_NET_OFFLINE: "true", CARGO_TARGET_DIR: cargoTarget },
      shell: process.platform === "win32",
    },
  );
}

function requireGreen(label) {
  const result = runTargeted();
  if (result.status !== 0) {
    throw new Error(`${label} expected GREEN\n${result.stdout}\n${result.stderr}`);
  }
  console.log(`${label}: GREEN`);
}

function requireRed(label) {
  const result = runTargeted();
  if (result.status === 0) throw new Error(`${label} expected RED but passed`);
  console.log(`${label}: RED`);
}

const guard = `        if self.source_epoch == SourceEpoch::V25 && self.check(&TokenKind::Equals) {
            return Err(ParseError {
                span: self.current_span(),
                message: "E_V25_DEFINITION_MIGRATION_REQUIRED: V25 정의에는 \`:=\`가 필요하며 이전 \`=\` 원문은 명시적 마이그레이션 입력으로 분리해야 합니다".to_string(),
            });
        }
        self.expect(&TokenKind::Define, ":=")?;`;

try {
  requireGreen("baseline");

  replaceInFunction(
    "    fn parse_definition_stmt",
    "    fn consume_decl_item_control_suffix_if_present",
    "message: \"E_V25_DEFINITION_MIGRATION_REQUIRED: V25 정의에는 `:=`가 필요하며 이전 `=` 원문은 명시적 마이그레이션 입력으로 분리해야 합니다\".to_string(),",
    "message: \":=\".to_string(),",
    "M1",
  );
  requireRed("M1_TYPED_EQUALS_GENERIC_EXPECTED");
  restore();

  replaceInFunction(
    "    fn parse_definition_stmt",
    "    fn consume_decl_item_control_suffix_if_present",
    guard,
    `        if self.source_epoch == SourceEpoch::V25 && self.check(&TokenKind::Equals) {
            self.advance();
        } else {
            self.expect(&TokenKind::Define, ":=")?;
        }`,
    "M2",
  );
  requireRed("M2_TYPED_EQUALS_ACCEPTED");
  restore();

  replaceInFunction(
    "    fn parse_decl_block",
    "    fn is_definition_stmt_start",
    `            } else if self.check(&TokenKind::Equals) {
                item_kind = DeclKind::Butbak;`,
    `            } else if self.check(&TokenKind::Equals) {
                if self.source_epoch == SourceEpoch::V25 {
                    return Err(self.error("E_V25_DEFINITION_MIGRATION_REQUIRED: invalid Butbak relabel"));
                }
                item_kind = DeclKind::Butbak;`,
    "M3",
  );
  requireRed("M3_BUTBAK_EQUALS_MISCLASSIFIED");
  restore();

  replaceInFunction(
    "    fn parse_definition_stmt",
    "    fn consume_decl_item_control_suffix_if_present",
    "self.source_epoch == SourceEpoch::V25 && self.check(&TokenKind::Equals)",
    "self.source_epoch == SourceEpoch::V25 && self.check(&TokenKind::Define)",
    "M4",
  );
  requireRed("M4_VALID_DEFINE_MISCLASSIFIED");
  restore();

  replaceInFunction(
    "    fn parse_definition_stmt",
    "    fn consume_decl_item_control_suffix_if_present",
    "self.source_epoch == SourceEpoch::V25 && self.check(&TokenKind::Equals)",
    "self.source_epoch == SourceEpoch::V25 && matches!(&type_ref, TypeRef::Named(_)) && self.check(&TokenKind::Equals)",
    "M5",
  );
  requireRed("M5_COMPLEX_TYPE_EQUALS_MISSED");
  restore();

  requireGreen("restored");
} finally {
  restore();
}

if (sha256(fs.readFileSync(parserPath)) !== sha256(original)) {
  throw new Error("mutation residue: lang/src/parser.rs");
}

console.log("DDN_V25_PC01_TYPED_LOCAL_EQUALS_FOLLOWUP_MUTATION_PASS mutations=5 residue=0");
