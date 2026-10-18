# 공개 상태·비주장·추적 — Language SPEC v0.8.1 공개판

> 상태: `public current / owner-approved / independently reviewed / docs-first / product-open`
> public current: `v0.8.1`; predecessor: `v0.8.0`
> source internal SSOT: `v25.137.0 / RD-001~690`

이 문서는 semantic language promise와 제품·지원·release 상태를 분리한다.
아래 status/trace 행은 기능 지원이나 실행 성공을 주장하지 않는다.

## RD-521 — LICENSE-GOV-U1 repository license allocation and publication-rights gate

- projection: `PUBLIC_GOVERNANCE_NONCLAIM`
- source status: `OWNER-APPROVED / LICENSE-PROFILE-DIRECTION-CLOSED / EXISTING-PROVENANCE-IDENTITIES-REUSED / PHYSICAL-APPLICATION-MISSING / PUBLICATION-BLOCKING / PRODUCT-AND-RELEASE-OPEN`

Repository-authored compiler, runtime, standard-library and tooling source is allocated Apache-2.0; public Language SPEC and general public documentation CC BY 4.0; future inbound contribution DCO 1.1. Rights-cleared educational content may explicitly use CC BY-SA 4.0 but is not blanket-relicensed. RD-425 and RD-428~431 identities are reused; no new source, artifact, provenance or rights identity is created. Unknown ownership, corpus rights, asset origin or vendored-fixture rights fail public-source admission; zero confirmed incompatible dependencies is not clearance and DCO does not cure historical rights. Publication remains blocked until physical license texts, path scopes, metadata, attribution, NOTICE/SBOM, contribution workflow and release-receipt evidence are applied and independently reviewed.

## RD-558 — STDLIB-FLOOR-U1 Owner floor and exact shelf membership

- projection: `PUBLIC_GOVERNANCE_STATUS_ONLY`
- source status: `OWNER-APPROVED / FLOOR-POLICY-CANDIDATE / PRODUCT-OPEN`

V1 has an Owner-approved Standard Library V1 Floor as a required release product gate, not a Core syntax extension, builtin-count increase or API-per-RD catalog. STDLIB-O1=A fixes exactly 20 shelf rows: F01 text/Unicode, F03 collections/range, F04 number/math, F05 deterministic RNG, F06 basic statistics, F07 JSON, F08 CSV, F09 file/path/stdin/stdout, F12 assertion, F13 logging/trace/receipt inspection, F14 CLI program input/helpers, F15 typed table/data processing, F16 basic modeling/solver/integration/numerical diagnostics and F17 basic result table/graph/chart/scene are REQUIRED (14); F02 Regex, F10 time/duration/date-time and F11 encoding/hex/base64/hash are OPTIONAL (3); F18 URL/HTTP, F19 compression/archive and F20 SQLite/local persistence are DEFERRED membership with POST_V1 roadmap (3). This is the shelf-floor axis, not automatic rewriting of VC01~VC18 or the exact VC17 provider rows. Explicit basic F16/F17 floor scope does not promote whole advanced model/solver/graph or 2D families. Regex floor optionality does not make the excluded/deferred V1 Regex capability supported. OPTIONAL and POST_V1 absence do not block V1 floor release; claims about their support still require their own product evidence. Existing numeric, collection, capability, identity and renderer owners are reused. Exact module/provider StableIdentity, Korean names, pins and export layout remain registry/gaji-owned; no API-per-RD, new Core IR, SpecialBlock, source syntax or guessed API contract is authorized.

## RD-559 — STDLIB-FLOOR-U2 starter and breadth blocking partitions

- projection: `PUBLIC_GOVERNANCE_STATUS_ONLY`
- source status: `OWNER-APPROVED / FLOOR-POLICY-CANDIDATE / PRODUCT-OPEN`

STDLIB-O2=A' makes the Starter Golden 24 framework a blocking product gate with explicit BLOCKING_CORE, OPTIONAL_COMPANION and POST_V1_COMPANION dispositions in the candidate release registry. The fixed crosswalk binds SG01~SG19 and SG21~SG23 as 22 BLOCKING_CORE workflows; SG20 Regex and SG24 encoding/content inspection are 2 OPTIONAL_COMPANION workflows; POST_V1_COMPANION count is zero. Every blocking core workflow requires target-bound complete PASS; PARTIAL, FAIL and UNVERIFIED are zero at closure. STDLIB-O3=A applies MISSING=0 and PRODUCT_BROKEN=0 to REQUIRED-related Task80 rows, not all 80. T05~T07 and T37~T43 are 10 optional companions; T73~T78 are 6 post-V1 companions; the remaining 64 are blocking required-related breadth rows. The exact row provenance and predecessor result are preserved in the release registry projection. Optional primitives used inside a required workflow do not promote the whole optional shelf. UNKNOWN is not PASS, and DIRECT/SIMPLE_COMPOSED/COMPLEX_COMPOSED are reachability classifications rather than support or closure. A historical focused PASS is not a new starter oracle. Exact task inputs, typed results, failure evidence and applicable frontdoors must be fixed before executable admission; missing semantics are research/Owner gates rather than implementer guesses.

## RD-560 — STDLIB-FLOOR-U3 product closure and evidence authority

- projection: `PUBLIC_GOVERNANCE_STATUS_ONLY`
- source status: `OWNER-APPROVED / FLOOR-POLICY-CANDIDATE / PRODUCT-OPEN`

STDLIB_V1_FLOOR_CLOSED holds iff the Owner-approved REQUIRED shelf set is fixed, every REQUIRED shelf is product-closed, the public V1 provider/export registry is nonzero and exact, REQUIRED providers are reachable through official DDN frontdoors, Native/WASM required semantic parity passes, the applicable required Workbench subset passes, blocking Starter Golden core is 100% PASS, REQUIRED-related Task80 MISSING and PRODUCT_BROKEN counts are both zero, clean-machine/offline required workflows pass, and no third-party package is required for the V1 floor. Each REQUIRED shelf binds semantic contract, public provider/export registry, official reachability, type/canon, Native and WASM runtime, applicable Workbench, determinism/capability, focused pack and starter-workflow evidence. This floor is necessary, not sufficient, for the complete V1 release: existing distribution, rights, Owner Human Smoke and other release gates remain. Pack owns executable oracle/golden/mutation/parity evidence; examples/walks own user workflow, not normative API identity. Internal host crates, 256 signature rows / 251 names, docs-only closure or partial upstream PASS do not create public providers or SUPPORTED_FOR_V1. The predecessor fixed-product census has public supported stdlib modules/symbols 0/0; current floor product/release readiness remains NO without new bound evidence. Exact JSON, CSV, file write/path, CLI program input and typed-table contracts remain RESEARCH / OWNER GATE; API spelling, signatures, parse/serialization policies and physical schema are not invented. Internal product/pack closure does not imply external-user success, Human PASS, publication authorization or release activation. Existing current membership, public SPEC, product/support/readiness/maturity/release/Human status is not changed by this candidate.

## RD-591 — STDLIB V1 RNG Solver final-design current-view closure

- projection: `PUBLIC_GOVERNANCE_STATUS_ONLY`
- source status: `OWNER-BOUND CLARIFICATION / DESIGN-CLOSED / PRODUCT-OPEN`

For the fourteen REQUIRED V1 shelves, the seven reused exact-contract slices plus RD-588~590 close the identified deterministic policy, capability, failure, RNG, solver-method and solver-numeric research gaps. RD-560's earlier research and Owner gate is narrowed only for these fixed materials: Owner decision remaining, research required remaining, implementer semantic guess required and STDLIB V1 design open are zero. This does not create a fifteenth shelf or one RD per API. It does not establish exact Korean callable surfaces, physical provider/export registry, API, AST, schema, wire, product conformance, Native/WASM/Workbench parity, SUPPORTED_FOR_V1, readiness, maturity, release or Human PASS. Current saturating or truncating Fixed64 behavior remains a PRODUCT_CONFORMANCE_GAP rather than semantic authority. RNG-P01~10 and SOL-P01~14 remain 24/24 PLANNED_NOT_ORACLE; physical pack and executable oracle remain zero.

## RD-567 — Quechua seed provenance, denominators and Human evidence isolation

- projection: `TRACE_ONLY`
- source status: `OWNER-BOUND / RESEARCH-CUSTODY / MATERIAL-OPEN / HUMAN-NOT-PERFORMED`

Core seed custody is 336 non-Quechua plus 24 Collao rows in a 408-row target, leaving 24 Chanka and 24 Áncash rows missing. Callable seed custody is 210 non-Quechua plus 15 Chanka rows in a 255-row target, leaving 15 Collao and 15 Áncash rows missing. Mixed or undeclared README material cannot promote a variety surface. Chanka, Collao and Áncash Human evidence identities are independent; no PASS inherits across varieties, and all three Human PASS states remain NOT_PERFORMED. No physical pack or executable oracle is created.

## 전역 비주장

```yaml
PRODUCT_IMPLEMENTATION: OPEN
PHYSICAL_PACK: 0
EXECUTABLE_ORACLE: 0
SUPPORTED_FOR_V1: 0
READINESS: UNCLAIMABLE
PUBLICATION_READY: NO
RELEASE_READY: NO
HUMAN_PASS: NOT_GRANTED_BY_THIS_SPEC
```

## v25.91 catch-up governance and trace rows

### RD-615 — ONNX preferred long-term AI model interchange direction

- projection: `TRACE_ONLY`
- source status: `OWNER-BOUND / LONG-TERM-DIRECTION / PHYSICAL-PRODUCT-OPEN`

ONNX is the preferred long-term direction for DDN AI model interchange. Existing model, artifact, provider and RD-561~562 external-boundary owners are reused. No runtime, execution provider, tensor physical schema, training framework, AI-specific provider identity or ProviderRegistry is fixed or introduced.

### RD-616 — WASM Component Model and WIT preferred V2 portable plugin ABI direction

- projection: `TRACE_ONLY`
- source status: `OWNER-BOUND / V2-LONG-TERM-DIRECTION / V1-PHYSICAL-PRODUCT-OPEN`

WASM Component Model and WIT are the preferred long-term direction for a V2 portable plugin ABI. RD-189 Native/WASM semantic parity and RD-561~562 external-boundary authority are reused. No exact ABI, runtime, WASI or engine version, JavaScript substitution, silent fallback or V1/V1.5 implementation obligation is created.

### RD-622 — First external reference vertical

- projection: `PUBLIC_GOVERNANCE_NONCLAIM`
- source status: `OWNER-BOUND / EIO-R1-U1 / PRODUCT-OPEN`

The first official external interop reference vertical is a prebuilt WebAssembly Component with the typed reference signature add(s64,s64)->s64. This is a reference ordering and architecture-binding decision, not WASM-only architecture, broad WASM product support, SUPPORTED_FOR_V1, release or Human PASS. Python prebuilt wheel and Native C or Rust prebuilt artifact follow on the same typed boundary architecture; a sealed external Build Provider remains DEFERRED research.

### RD-627 — Pre-1.0 provisional default publication policy

- projection: `PUBLIC_GOVERNANCE_NONCLAIM`
- source status: `OWNER-BOUND / PRE1-LANG-POLICY-U1 / MATERIAL-STATUS`

A project-owned Core, LanguageProfile or official standard-gaji candidate surface with sufficient evidence and provenance and exactly one preferred candidate may be presented as PROVISIONAL_DEFAULT before Human review. This status is changeable before 1.0 and is not HUMAN_PASS, FINAL_1_0, product support, SUPPORTED_FOR_V1, publication readiness or release readiness. Human PASS is necessary but not sufficient for FINAL_1_0. Ambiguity, weak provenance, preferred-candidate cardinality 0 or 2+, or dependence on another language or Quechua variety stays HOLD_FOR_AMBIGUITY; UNKNOWN, MISSING, AI, machine translation and locale are not selection authority. Earlier provisional spelling does not become a permanent alias automatically.

## Public cadence custody

`PUBLIC-SPEC-CADENCE-O1=B` requires impact classification for every SSOT promotion.
Normative public change uses a minor version, custody-only correction uses a patch, and
no public meaning delta creates no public version. Projection is never automatic.

```yaml
PUBLIC_CURRENT: v0.7.0
CANDIDATE: v0.8.0
PRODUCT_SUPPORT_V1_RELEASE_HUMAN_INHERITANCE: 0
PUBLICATION_READY: NO
RELEASE_READY: NO
```

## v25.137 governance/status successor rows

### RD-644 — V1 VC03 Binding/Rebinding complete-row product evidence binding

- projection: `PUBLIC_GOVERNANCE_STATUS_ONLY`
- source status: `OWNER-GATED CANDIDATE / V1-VC03-COMPLETE-ROW-R1-U1 / CLOSED_TARGET_BOUND`

The existing REQUIRED VC03 Binding/Rebinding <- row binds immutable product target 1417483399d37c4e5dda3dabfc9cb9e5ec635bfa and independent review cc35dbaad5e4e1a2bfb38be0693d6210b6496fde, report SHA-256 d96eea427365691d656c09c146a8665dee012482401350ad1208d9df9c4392c5, with PASS_TARGET_BOUND and blocking findings zero. Its existing complete-row target, frontdoors, shared authority, tests and failure contract are unchanged. Complete REQUIRED rows derive 1/28 and blockers 27. Membership, SUPPORTED_FOR_V1, readiness, maturity, public SPEC, publication/release and Human states do not transition; P0, VC05 and VC18 are not merged and inherit no verdict.

### RD-645 — V1 VC18 basic diagnostics complete-row product evidence binding

- projection: `PUBLIC_GOVERNANCE_STATUS_ONLY`
- source status: `OWNER-GATED CANDIDATE / V1-VC18-COMPLETE-ROW-R1-U1 / CLOSED_TARGET_BOUND`

The existing REQUIRED VC18 basic diagnostics row binds immutable product target d2940e284fb4f3ddce678e425217a03036050938 and independent review b5e46d7a7bf050b07bca007521dca2bc9782b8ce, report SHA-256 4d43c04b8103c18b4652f087ab8b79c2e0d5bf1562728cc924b6b08f1274a67d, with PASS_TARGET_BOUND, blocking findings zero and VC18_CLOSURE_READY=YES. Its existing target, frontdoors, shared authority, P1 diagnostic suite, deterministic rejection and publication contract are unchanged. Complete REQUIRED rows derive 2/28 and blockers 26. VC03 remains bound independently; P0, VC05, VC13 and VC16 are not merged. Membership, support, readiness, maturity, public SPEC, product, publication/release and Human states do not transition.

### RD-646 — V1 VC16 canonicalization complete-row product evidence binding

- projection: `PUBLIC_GOVERNANCE_STATUS_ONLY`
- source status: `OWNER-GATED CANDIDATE / V1-VC16-COMPLETE-ROW-R1-U1 / CLOSED_TARGET_BOUND`

The existing REQUIRED VC16 closed multisurface and canonicalization row binds immutable product target e2467dfe1c8ec8fa30a9d77032b6e0e1432e2ee1 and independent review 5347c93bdfce297352fcf72d802bec7c538b7fca, report SHA-256 1620faf0d7022bf4881e82a7ba744b704524c1e0e2dd81ad873ed933cc498acd, with PASS_TARGET_BOUND, blocking findings zero, one inherited nonblocking finding and VC16_CLOSURE_READY=YES. Its existing target, frontdoors, shared authority, canonicalization evidence, deterministic rejection and publication contract are unchanged. Complete REQUIRED rows derive 3/28 and blockers 25. VC03 and VC18 remain bound independently; P0, VC05 and VC13 are not merged. Membership, support, readiness, maturity, public SPEC, product, publication/release and Human states do not transition.

### RD-648 — V1 VC11 control-flow complete-row product evidence binding

- projection: `PUBLIC_GOVERNANCE_STATUS_ONLY`
- source status: `OWNER-GATED CANDIDATE / V1-VC11-COMPLETE-ROW-R1-U1 / CLOSED_TARGET_BOUND`

The existing REQUIRED VC11 조건·반복 row binds immutable product target b852b2e8db26fcffc4044f80ddb1f8129ce03e70 and independent review 018643395f12b59fa1edb27745139807057d981c, report SHA-256 9c14ebb0d600f9f4438dab9def0c97fc4bb6f58f61feab84324f12941a1a1494, with PASS_TARGET_BOUND and blocking findings zero. Its existing target, official Native CLI and Workbench frontdoors, shared parser/type/canon/runtime authority, executed evidence, deterministic rejection and failure-publication contract are unchanged. Complete REQUIRED rows derive 4/28 and blockers 24. VC03, VC18 and VC16 remain bound independently; RD-647 is unrelated; P0, VC05 and VC13 are not merged. Membership, support, readiness, maturity, public SPEC, product, publication/release and Human states do not transition.

### RD-650 — V1 P0 official Native CLI complete-row R5 product evidence binding

- projection: `PUBLIC_GOVERNANCE_STATUS_ONLY`
- source status: `OWNER-GATED CANDIDATE / V1-P0-COMPLETE-ROW-R5-U1 / CLOSED_TARGET_BOUND`

The existing REQUIRED P0 official Native CLI row binds immutable product target 9f02e989cc6d24105ffe55ccb82788fad385a536 and independent review 836f8ba7010b64e1008d3106a386d486e64fecc9, report SHA-256 17a3c492a177efec958f31626efae03fd956a1f7155825576844a5d8a232d3cb, with PASS_TARGET_BOUND, blocking findings zero and row closure ready. Actual tracked-WASM execution, freshness, strict-selection failure atomicity and official CLI evidence close the R5 target only. Prior P0 fail/rework custody remains historical. Complete REQUIRED derives 5/28 and blockers 23; VC03, VC18, VC16 and VC11 remain independently bound, RD-647/RD-649 are unrelated, and VC05/VC13 are not merged. Membership, support, readiness, maturity, public SPEC, product, publication/release and Human states do not transition.

### RD-651 — V1 VC01 Korean-source complete-row product evidence binding

- projection: `PUBLIC_GOVERNANCE_STATUS_ONLY`
- source status: `OWNER-GATED CANDIDATE / V1-VC01-COMPLETE-ROW-R1-U1 / CLOSED_TARGET_BOUND`

The existing REQUIRED VC01 한국어 기본 원문 row binds immutable product target 994e0c19102ff338cd738662e9921494ee32e13a and independent review 92a1533e88f8bca166e9b081b65fef589aadc6ad, report SHA-256 2e6df345245d62181ff48d11c374774fbecb7e409101bac90363112d55a6d8a1, with PASS_TARGET_BOUND, blocking findings zero and row closure ready. Handoff and review are sibling direct children of the product; the review consumed the exact handoff object. Definition pack 9/9 and VC01 cases 17/17 cover official Native CLI, tracked WASM and Chromium Workbench with failure-publication zero; broad suites retain their inherited failures and are not claimed GREEN. Complete REQUIRED rows derive 6/28 and blockers 22. VC03, VC18, VC16, VC11 and P0 remain independently bound; RD-647/RD-649 are unrelated; VC05 and VC13 are not merged. Membership, support, readiness, maturity, public SPEC, product, publication/release and Human states do not transition.

### RD-652 — V1 VC02 Definition complete-row product evidence binding

- projection: `PUBLIC_GOVERNANCE_STATUS_ONLY`
- source status: `OWNER-GATED CANDIDATE / V1-VC02-COMPLETE-ROW-R1-U1 / CLOSED_TARGET_BOUND`

The existing REQUIRED VC02 Definition := row binds immutable product target c3bb9630e8e4bdc757ac58fe4ddb61c920d84957 and report commit 809c88de0eefcf09def3a483a752cad55b6df270, report SHA-256 128e89d1f9c84bb7a9a77a464ebf99c678f9326e36954ccd54ab4a3d749dc44b, with PASS_TARGET_BOUND, blocking findings zero and row closure ready. The exact chain is product to direct-child handoff 6729b76551460ea7cb3cdd2937298c8ea80e8cbf, review seal 88ae5c1b0c51dde1c0148f827512ee3ffdc4d1c1, then report commit. The 18-case product matrix, Definition pack 8/8, RD-457 roundtrip 6/6, official Native CLI, tracked WASM and Chromium Workbench preserve deterministic rejection and failure-publication zero. Two effective mutations produced RED and exact restore GREEN. Complete REQUIRED rows derive 7/28 and blockers 21. VC03, VC18, VC16, VC11, P0 and VC01 remain independently bound; RD-647/RD-649 are unrelated; other rows are not merged. Membership, support, readiness, maturity, public SPEC, product, publication/release and Human states do not transition.

### RD-653 — V1 VC08 Sum optional typed Result complete-row product evidence binding

- projection: `PUBLIC_GOVERNANCE_STATUS_ONLY`
- source status: `OWNER-GATED CANDIDATE / V1-VC08-COMPLETE-ROW-R1-U1 / CLOSED_TARGET_BOUND`

The existing REQUIRED VC08 Sum, optional and typed Result row binds immutable product target 60d47bef1c5c0c1603ee3d92b1a8b0d34dea6aee and direct-child handoff 77c57fbeacee341bf028ea28c819222a8fdd7a8a plus direct-child independent review 80dcb6bb343206280d910692cae5fbf8eafd431a, report SHA-256 1c6553ed61ae7807000beda38cf5fd530964ecac4de0ca215691039880c279f0, with PASS_TARGET_BOUND, blocking findings zero and row closure ready. Shared typed-Sum 3/3, structural result-owner 2/2 and the 30-case official Native CLI, tracked WASM and Chromium Workbench matrix preserve exact owner matching, canonical roundtrip, deterministic rejection, hard-abort separation and failure-publication zero. Two independent behavior mutations produced RED and exact restore GREEN. Complete REQUIRED rows derive 8/28 and blockers 20. All earlier closed rows remain independently bound; VC10 and VC07 are not allocated or merged. Membership, support, readiness, maturity, public SPEC, product, publication/release and Human states do not transition.

### RD-654 — V1 VC10 ResultPlace complete-row product evidence binding

- projection: `PUBLIC_GOVERNANCE_STATUS_ONLY`
- source status: `OWNER-GATED CANDIDATE / V1-VC10-COMPLETE-ROW-R1-U1 / CLOSED_TARGET_BOUND`

The existing REQUIRED VC10 ResultPlace row binds immutable product target f966c33ef8c8ce06070f56afade671d50b6f2d29 and direct-child handoff bff6a823d63055ac842e633d6a749c50b4d6ccd9 plus direct-child independent review cfb5b97e145333422d234e7437e131f7030be658, report SHA-256 4a0b4a9a5d6f536bbefb256f756b8d5e45498ea80e8b46225929871cfd2dd2aa, with PASS_TARGET_BOUND, blocking findings zero and row closure ready. Language 6/6, runtime custody/type/publication 3/3 and 34 authored cases across official Native CLI, tracked WASM and Chromium Workbench preserve per-invocation ResultPlace, resultful/resultless separation, definite result, write-only access, typed assignment, immediate finish, child isolation and failure-publication zero. Two counted independent behavior mutations produced RED and exact restore GREEN; the unselected trial mutation is not evidence. Complete REQUIRED rows derive 9/28 and blockers 19. All earlier closed rows remain independently bound; VC07 and ProjectExecutionSnapshotIdentity are not allocated or merged. Membership, support, readiness, maturity, public SPEC, product, publication/release and Human states do not transition.

### RD-655 — V1 VC15 registered Korean form complete-row product evidence binding

- projection: `PUBLIC_GOVERNANCE_STATUS_ONLY`
- source status: `OWNER-GATED CANDIDATE / V1-VC15-COMPLETE-ROW-R1-U1 / CLOSED_TARGET_BOUND`

The existing REQUIRED VC15 registered Korean form row binds immutable correction product 90d146f2fa18e39216fca548126cb7354fc68c50, direct-child handoff f91fd2b3b78e24f2c8d5ec78c89ab974b5a23417 and direct-child independent review 1b5b9bdc099c6c406209f2e715624ab38615d2ec, report SHA-256 fc2eb5cd7ea5caf670146ae9847c8fa64d325aab1f621b202724118854a7b41e, with PASS_TARGET_BOUND, blocking findings zero and row closure ready. Historical BF-01 remains immutable and is reproduced on the failed product. The preferred surface preserves its complete registered family, each alternate remains its exact subset, and order, fuzzy matching, hidden morphology and inference are nonauthoritative. Twenty-six authored cases pass across three official Native CLI names, tracked WASM with freshness gates and actual Chromium Workbench; failure publication is atomic. Four reviewer-owned behavioral mutations produced RED and exact restore GREEN. Broad regression failures remain target-parent identical at seven targets and 48 test names, so whole-suite GREEN is not claimed. Complete REQUIRED rows derive 10/28 and blockers 18. All earlier closed rows remain independently bound; VC06 and every other row are not allocated or merged. Membership, support, readiness, maturity, public SPEC, product, publication/release and Human states do not transition.

### RD-657 — V1 VC07 record persistence complete-row product evidence binding

- projection: `PUBLIC_GOVERNANCE_STATUS_ONLY`
- source status: `OWNER-GATED CANDIDATE / V1-VC07-COMPLETE-ROW-R1-U1 / CLOSED_TARGET_BOUND`

The existing REQUIRED VC07 row binds immutable product deeb7c81b6172b9b4c710e072bc3c50202d05c97, direct-child handoff f85b1e7a5e75338d148c256089c0d0a7cba5e6da and direct-child review e0d4c90058fe506990c74987c8a7d1e3bec5755e, report SHA-256 cd17bdf45869c84811002932f4de584360c0f7dc38e35e5e9994bbbb720a1a0a, PASS_TARGET_BOUND with zero blockers and row closure ready. Legacy Map bytes/hash remain unchanged; explicit versioned record identity/persistence, typed restore/consumer execution, official Native CLI, tracked WASM, Chromium Workbench and failure atomicity evidence are bounded to VC07. VC06 RD-656 and all other rows remain independent. Complete REQUIRED derives 11/28 and blockers 17. Membership, support, readiness, maturity, public SPEC, product, release and Human states do not transition.

### RD-666 — V1 VC09 named callable and collection complete-row product evidence binding

- projection: `PUBLIC_GOVERNANCE_STATUS_ONLY`
- source status: `OWNER-GATED CANDIDATE / V1-VC09-COMPLETE-ROW-R1-U1 / CLOSED_TARGET_BOUND`

The existing REQUIRED VC09 row binds exact product 2bf3fba48e56d8ee36f66dc85a918d628da4b275, prior independent review 9e8a58fb9bcfa98593d7b693db9432a8b21ebd98 and direct-child custody successor 56ff9e36a745c62dc0f2622eb2575050e4a3d263. The 5,859-byte report SHA-256 is c50c029aa8162af63604f974b247fc064775398452c22398ed4fc8aed5c9fae4, verdict PASS_TARGET_BOUND, blocking product findings zero and complete-row binding ready. Evidence is bounded to VC09; product/public/support/release/Human transitions and other-row inheritance are zero.

### RD-667 — V1 VC12 event hook and conditional dispatch complete-row product evidence binding

- projection: `PUBLIC_GOVERNANCE_STATUS_ONLY`
- source status: `OWNER-GATED CANDIDATE / V1-VC12-COMPLETE-ROW-R1-U1 / CLOSED_TARGET_BOUND`

The existing REQUIRED VC12 row binds exact product f7dc66b59bc05939dc0ed9b01462d3c2b8000c9d, prior independent review 7415be859cc79efaceda383a1044b2fdc9e164fe and direct-child custody successor 33ad0d17c8b88c8bb5fd6f6e6458162caa7f9178. The 5,847-byte report SHA-256 is 0af7d6ce78832e70eeae73cc6aca27dbeff88c9d4fc8c377cf56c012f37932ed, verdict PASS_TARGET_BOUND, blocking product findings zero and complete-row binding ready. Evidence is bounded to VC12. Together with RD-666 it derives complete REQUIRED 13/28 and blockers 15; all protected axes and other rows remain unchanged.

### RD-673 — V1 P1 single-file complete-row product evidence binding

- projection: `PUBLIC_GOVERNANCE_STATUS_ONLY`
- source status: `OWNER-GATED CANDIDATE / V1-P1-SINGLE-FILE-COMPLETE-ROW-R1-U1 / CLOSED_TARGET_BOUND`

The existing REQUIRED P1 single-file row binds immutable product 4b422af68804c3f02c1d2e5f7c27a37ef4722420, direct-child handoff 4027a78142b08addda758fe8c356c314d0ddb5cf and direct-child complete-row review 6944bbf1caf14e3f3c1d2d2f36e0c8b5d020b707, report SHA-256 c85ccf06ee9d4d713e01b1e4c5d0b20e0c6a37077d3504107ee2f2b05a761c70, PASS_TARGET_BOUND with zero blockers and row closure ready. Official Native CLI 3/3, tracked WASM and freshness, actual Chromium Workbench, strict raw-source admission, SourceEpoch single-file invocation, stale-read protection and failure atomicity are bounded to P1. Earlier b2698e3f narrow review remains supporting only. P1 alone becomes CLOSED_TARGET_BOUND, deriving complete REQUIRED 14/28 and blockers 14. P2/P3 and all other rows remain independent. Membership, support, readiness, maturity, public SPEC, product, publication/release and Human states do not transition.

### RD-677 — Development Program Governance R1

- projection: `PUBLIC_GOVERNANCE_NONCLAIM`
- source status: `OWNER-GATED CANDIDATE / DEVELOPMENT-PROGRAM-GOVERNANCE-R1-U1 / GOVERNANCE-CONTRACT / NOT-PRODUCT / NOT-RELEASE`

RD-432~437, RD-100, RD-300 and RD-304 are reused. SSOTVersion, ProductVersion, RoadmapHorizon, DevelopmentProgram/Wave and WorkLeaf identities remain independent; RESEARCH, CONTRACT, PRODUCT, REVIEW and RELEASE remain independent axes. Future requirements use closed NOW_COMPAT, INTERFACE_LATER or DOMAIN_LATER classification, with unknown/ambiguous fail-closed and NOW_COMPAT opening only an RD-435 affected-dependency review. The default G1~G8 order is research, fixed proposal/contract, independent contract review, authority promotion, product implementation, fixed product review, row/capability closure and release admission; N/A requires exact OwnerIdentity, reason and evidence. Current V1 work and valid PASS are neither automatically expanded nor reopened. Calendar and roadmap are non-authoritative. Product, membership, support/readiness/maturity, release and Human status remain unchanged.

### RD-681 — V1 VC06 Bytes Unicode complete-row product evidence binding

- projection: `PUBLIC_GOVERNANCE_STATUS_ONLY`
- source status: `OWNER-GATED CANDIDATE / V1-VC06-COMPLETE-ROW-R1-U1 / CLOSED_TARGET_BOUND`

The existing REQUIRED VC06 row binds immutable product 30f19f2380b037e487e7064a23063ea7ef2b7699, direct-child handoff b7b9ec7e9d427c12a6dde2555020983174d5fddd and direct-child review 43386f9892fcf042544d29e4ee3404357fe2fca0, report SHA-256 84ead056a63c916625a85df345c037f904638264926ecbf09e7b6a4866c1060d, PASS_TARGET_BOUND with zero blockers and row closure ready. Unicode 17, strict UTF-8/NFC/EGC, distinct Bytes identity, exact 0x0C storage, saved transport, official Native CLI, tracked WASM, Chromium Workbench, deterministic rerun and failure atomicity evidence are bounded to VC06. RD-680 and all other rows remain independent. Complete REQUIRED derives 15/28 and blockers 13. Membership, support, readiness, maturity, public SPEC, product, release and Human states do not transition.

### RD-688 — V1 VC17 PURE_MINIMAL REQUIRED15 complete-row product evidence binding

- projection: `PUBLIC_GOVERNANCE_STATUS_ONLY`
- source status: `OWNER-GATED CANDIDATE / V1-VC17-PURE-MINIMAL-REQUIRED15-COMPLETE-ROW-R1-U1 / CLOSED_TARGET_BOUND`

The existing REQUIRED VC17 row binds immutable product 6a1e6032ff77ba41a66314eae1d9b186aecf9691, direct-child handoff b02dc37f780b831ba306e6fa61bf429a28717fa2 and direct-child review e6452012e894beaf825ac0c9a4a0c2bf031bd134, exact review report SHA-256 2f54865850fa1171422d5702536393857688e50d7ec8e6fbe5ef522404d13a25, PASS_TARGET_BOUND with zero blockers and row closure ready. Evidence covers the current REQUIRED 15, 17 definitions, 30 scoped pins, operational laws, typed named PURE carrier, closed runtime TypeIdentity, actual CollectionTransform, official Native CLI, tracked WASM, Chromium Workbench and failure atomicity. RD-686, RD-687 and every other row remain independent; PASS inheritance is zero. Complete REQUIRED derives 16/28 and blockers 12. Membership, support, readiness, maturity, public SPEC, product, release and Human states do not transition.

## v25.137 trace-only successor rows

These rows preserve physical, custody, fixture and status lineage. Their details are
not public language semantics or public wire commitments.

| RD | title | source status | decision sha256 |
|---|---|---|---|
| RD-647 | External Interop bounded typed-payload canonical TLV physical profile | OWNER-BOUND CANDIDATE / EXTERNAL-TYPED-PAYLOAD-TLV-R1_1-U1 / PRODUCT-OPEN | `a340fd4f4fc14678de918da3aa48689bd271080f98e13ee19deb3dee4467bb0a` |
| RD-649 | CLI live-input Owner ballot provenance forward correction | FORWARD-ONLY / TRACEABILITY-CORRECTION / CLI-TEXT-R1_1-OWNER-TRACE-CORRECTION-U1 | `7a91bc6df0a0512570e611c11e7cb139b00d9ef94dec7ddb2298cc53595cd2bf` |
| RD-659 | ProjectExecutionSnapshotIdentity exact physical profile | OWNER-GATED CANDIDATE / PROJECT-EXECUTION-SNAPSHOT-IDENTITY-PHYSICAL-R1-U1 / PRODUCT-OPEN | `7c85da6b71a982459333ca0a9a98e4f3b6d01b2082a679199c0e3447dc0ec87e` |
| RD-660 | VC04 bounded basic value and type identity physical contract | OWNER-GATED CANDIDATE / VC04-BASIC-VALUE-TYPE-IDENTITY-PHYSICAL-R1-U1 / PHYSICAL-CONTRACT / NOT-PRODUCT / COMPLETE-ROW-OPEN | `62d9769f7e771ad4115d5d42c741803bb118b6ea0747d363a2f3397b4571f2d9` |
| RD-661 | VC06 storage identity and saved Bytes physical binding | OWNER-GATED CANDIDATE / VC06-STORAGE-IDENTITY-PHYSICAL-BINDING-R1-U1 / PHYSICAL-CONTRACT / NOT-PRODUCT / COMPLETE-ROW-OPEN | `12ca58db60c0617eca78ca90d6f54639913039e8ce923d8f67b0b0338b406980` |
| RD-662 | VC04 Subject/reference and registered-atom supply owners physical binding | OWNER-GATED CANDIDATE / VC04-SUPPLY-OWNERS-PHYSICAL-R1_2-U1 / PHYSICAL-CONTRACT / NOT-PRODUCT / COMPLETE-ROW-OPEN | `f89bd9455a1e4e38bdd9e8534c8470dfbc9d135f571abf2c4e95d4c5716b8362` |
| RD-663 | PESI constituent identity owners physical binding | OWNER-GATED CANDIDATE / PESI-CONSTITUENT-IDENTITY-OWNERS-PHYSICAL-R1-U1 / PHYSICAL-CONTRACT / NOT-PRODUCT / DOWNSTREAM-CLOSURE-OPEN | `452794542477d99ccb97501013e3443a0a519185090cfe0536f958ea0346e0ca` |
| RD-664 | VC04/VC07 canonical ResourceValue tag collision forward correction | OWNER-GATED CANDIDATE / VC04-VC07-CANONICAL-TAG-COLLISION-CORRECTION-R1-U1 / FORWARD-ONLY / PHYSICAL-CONTRACT / NOT-PRODUCT | `2f18cb74edb0104bdb5be191ebcbec7a28a0e1b122c432c7a5b398d312aa8497` |
| RD-665 | Subject identity persistent physical-owner contract | OWNER-GATED CANDIDATE / SUBJECT-IDENTITY-PHYSICAL-OWNER-R1-U1 / PHYSICAL-CONTRACT / NOT-PRODUCT | `9873bd22833dff8e77c35bfe4268976da4ccbf839fed44fc841e9230acdab804` |
| RD-668 | VC04 snapshot_v2 complete custody physical contract R2.1 | OWNER-GATED CANDIDATE / VC04-SNAPSHOT-V2-COMPLETE-CUSTODY-R2_1-U1 / PHYSICAL-CONTRACT / NOT-PRODUCT / DURABILITY-OPEN | `ed437813fc56f3ced1c0cc3306113d4c57194b034a3ed283037362ef7ab6d5a4` |
| RD-669 | VC04 snapshot_v2 shared Rust durable transaction owner R1.3 | OWNER-GATED CANDIDATE / VC04-SNAPSHOT-V2-DURABLE-TRANSACTION-OWNER-R1_3-U1 / PHYSICAL-CONTRACT / NOT-PRODUCT / BACKEND-OPEN | `767f1fc28aed018ba237d3290065e00e5c3218e71c95010f9f0f957210ee3cf2` |
| RD-670 | VC04 snapshot_v2 codec/host transport bridge R1.4 | OWNER-GATED CANDIDATE / VC04-SNAPSHOT-V2-CODEC-HOST-TRANSPORT-BRIDGE-R1_4-U1 / PHYSICAL-CONTRACT / NOT-PRODUCT / BACKEND-OPEN | `599c8d2010e3cee3f5b43338103affa50089cb60b4d4f6f7015589313a8203bb` |
| RD-671 | VC04 SQLite durability backend R1.2 mechanics-only adapter child | OWNER-GATED CANDIDATE / VC04-SQLITE-DURABILITY-BACKEND-R1_2-U1 / PHYSICAL-CONTRACT / NOT-PRODUCT / ADAPTER-CHILD | `42f4121b253aaab70b6a2f1874196d6ed7041833cdf73f41b5e7c3acc902eca7` |
| RD-672 | VC04 revision-envelope ingress physical binding R1 | OWNER-GATED CANDIDATE / VC04-REVISION-ENVELOPE-INGRESS-PHYSICAL-BINDING-R1-U1 / PHYSICAL-CONTRACT / NOT-PRODUCT | `59b44df664b2209b21e1b7e1e70180dedb020bcc8b5aab6ea582bad153722567` |
| RD-674 | Basic Tensor physical contract R2.1 | OWNER-GATED CANDIDATE / BASIC-TENSOR-PHYSICAL-CONTRACT-R2_1-U1 / PHYSICAL-CONTRACT / NOT-PRODUCT | `177dc3f66c0258c1af2f2f4e3264133bfbc4577db7679dfffcbf47b2bb4501c2` |
| RD-675 | VC17 PURE_MINIMAL REQUIRED15 provider/export physical contract R1 | OWNER-GATED CANDIDATE / VC17-PURE-MINIMAL-REQUIRED15-PHYSICAL-R1-U1 / PHYSICAL-CONTRACT / NOT-PRODUCT | `8c274023dc43783e4e217a525c21d783a89b6790fee1e90dff3941af6f03378a` |
| RD-676 | P5 public release-source boundary physical contract R2 | OWNER-GATED CANDIDATE / P5-PUBLIC-RELEASE-SOURCE-BOUNDARY-R2-U1 / PHYSICAL-CONTRACT / NOT-RELEASE | `ae4d6cd7a091c01fcb24d75e857df23938f9aa7427cccd131d085675351eacd0` |
| RD-678 | Development Program Contract physical profile R1.1 | OWNER-GATED CANDIDATE / DEVELOPMENT-PROGRAM-CONTRACT-PHYSICAL-R1_1-U1 / PHYSICAL-CONTRACT / AUTHORITATIVE-PERIPHERY / NOT-PRODUCT | `6481d053a74bfb9c107c174f8983e4770f87b518b00693350e0f20dfcc3fe66d` |
| RD-679 | VC04 RD-631/RD-672 crosscheck supply owner R1 | OWNER-GATED CANDIDATE / VC04-RD631-RD672-CROSSCHECK-SUPPLY-OWNER-R1-U1 / PHYSICAL-BINDING / NOT-PRODUCT | `53a4b81a51935b9654e66c300934a1d0165658fff07e87c9dd796f8c4c3f9672` |
| RD-680 | VC06 saved Bytes transport projection physical binding R1.1 | OWNER-GATED CANDIDATE / VC06-SAVED-BYTES-TRANSPORT-PROJECTION-PHYSICAL-R1_1-U1 / PHYSICAL-BINDING / NOT-PRODUCT | `b260c84208f0d422dd29f28dfe9cca9de2aaab0beaebbd691014fb37a1c45bbd` |
| RD-683 | VC14 saved Tensor private transport binding R1 | OWNER-GATED CANDIDATE / VC14-SAVED-TENSOR-TRANSPORT-BINDING-R1-U1 / PRODUCT-OPEN | `62489316b0531a1816ba43caa51100fd8b9fabbdfb7a6ba6e3189f2e647ffb4f` |
| RD-685 | VC17 canonical type and named PURE registry bridge R1.1 | OWNER-GATED CANDIDATE / VC17-CANONICAL-TYPE-AND-NAMED-PURE-REGISTRY-BRIDGE-R1_1-U1 / PRODUCT-OPEN | `18af3ba255a14c643c2b3e2916f5bc5118e0bb6990f1bee6bc09cfcdd5c50dc0` |
| RD-686 | VC14 Workbench project snapshot runtime attachment physical R1.1a custody binding | OWNER-GATED CANDIDATE / VC14-WORKBENCH-PROJECT-SNAPSHOT-RUNTIME-ATTACHMENT-PHYSICAL-R1_1A-U1 / PRODUCT-OPEN | `4c963365a9dd0c982dc3f628b4a682f872dd34361a95f5364a67a88cbdf820e2` |
| RD-687 | VC04 RD-663 current-source custody forward correction R1 binding | OWNER-GATED CANDIDATE / VC04-RD663-CURRENT-SOURCE-CUSTODY-FORWARD-CORRECTION-R1-U1 / PRODUCT-OPEN | `3302b54ee7f6a14673ace96b72307d747530b963bd7ca53a9f3d23471e8cc0a8` |
| RD-689 | VC04 RD-631 complete revision envelope owner R1.1 physical capsule binding | OWNER-GATED CANDIDATE / VC04-RD631-COMPLETE-REVISION-ENVELOPE-OWNER-R1_1-U1 / PHYSICAL-CAPSULE-BOUND / PRODUCT-OPEN | `c8ebe4d5e105fdd19e2e2d878b2e2277b7cf4f44752c59177f820d3d586857b2` |
| RD-690 | VC04 RD-669/RD-689 P01/P02 transaction digest fixture correction R1 | OWNER-GATED CANDIDATE / VC04-RD669-RD689-P01-P02-TRANSACTION-DIGEST-FIXTURE-CORRECTION-R1-U1 / FIXTURE-CORRECTION-ONLY / PRODUCT-UNCHANGED | `f810822e42b276b54c937d8b1f5484a72c663d03dd2d48c432be71c9764bb19e` |

```yaml
PUBLIC_CURRENT: v0.7.0
CANDIDATE: v0.8.0
PRODUCT_SUPPORT_V1_RELEASE_HUMAN_INHERITANCE: 0
PUBLICATION_READY: NO
RELEASE_READY: NO
```

## R2.1 filtered private-contract custody

The following mixed-purpose rows retain their full decision identity in the sealed
internal-source custody and trace. These physical details are not public normative wire.

| RD | private custody retained outside normative prose | decision sha256 |
|---|---|---|
| RD-643 | ordinal allocation and seal-before-identity mechanics | `81461658cac62ca2d05d567c651ffc1bfe5d6fb958068beb3ffd7d52e7eab443` |
| RD-656 | Bytes storage frame and payload/frame hash domains | `1a17bdd6a4e78464beb053b7e11b9460f7f1f1159ec1c8ea6994858f009ac4a3` |
| RD-682 | physical TypeIdentity preimage and digest | `342f031c3528d51f23f42440be61a556d190afb8e87c3bd6ea8538b486685bd0` |
| RD-684 | private named-PURE carrier and registry byte grammar | `12253734fa823ee955bd0683e4ca91cc04904137dbf1b2c90f22b9d055ab87bc` |
