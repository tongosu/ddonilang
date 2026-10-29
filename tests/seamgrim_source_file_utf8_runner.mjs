import assert from "node:assert/strict";
import { readTextFromLocalFile } from "../solutions/seamgrim_ui_mvp/ui/screens/editor.js";

const source = "값 <- 1.\n값 보여주기.";
const bytes = new TextEncoder().encode(source);
const file = (data) => ({ arrayBuffer: async () => data.buffer });
assert.equal(await readTextFromLocalFile(file(bytes)), source);
await assert.rejects(
  readTextFromLocalFile(file(new Uint8Array([0xef, 0xbb, 0xbf, ...bytes]))),
  /E_SOURCE_UTF8_BOM_FORBIDDEN/,
);
await assert.rejects(readTextFromLocalFile(file(new Uint8Array([0xff]))));
// A BOM character inside a string is content, not an ingress BOM.
const literal = '값 <- "\uFEFF".';
assert.equal(await readTextFromLocalFile(file(new TextEncoder().encode(literal))), literal);
console.log("SOURCE_FILE_UTF8_PASS normal=1 bom_rejected=1 invalid_utf8=1 literal_preserved=1");
