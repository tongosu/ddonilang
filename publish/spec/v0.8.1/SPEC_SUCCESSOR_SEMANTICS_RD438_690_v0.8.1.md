# v25.137 current semantic projection — Language SPEC v0.8.1 successor R2.1 correction candidate

> 상태: `public current / owner-approved / independently reviewed / docs-first / product-open`
> public current: `v0.8.1`; predecessor: `v0.8.0`
> source internal SSOT: `v25.137.0 / RD-001~690`

> status: `public current / owner-approved / independently reviewed / docs-first / product-open`
> source authority: `v25.137.0 / RD-001~690`
> normative rows in this document: `200`

Historical rows remain traceable. A later successor governs only the scope it
explicitly narrows or supersedes. Product-OPEN does not become product support,
executable evidence, release readiness or Human PASS.

## RD-438 — 횡단 sigil owner/profile contract

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SEMANTICS-CLOSED / SUCCESSOR-SURFACE-CLOSED / PRODUCT-OPEN`

Bare 씨이름은 type/family identity, `Owner.$갈래`와 expected owner가 유일한 `$갈래`는 owner-scoped closed branch, `#원자`는 독립 등록 profile/policy/selector identity다. `$`와 `#`는 closed/open 축이 아니며 상호 migration하지 않는다. SpecialBlock tag bundle은 `(#selector #modifier...)Head{Body}` family이고 selector 0/1, unknown·duplicate·conflict·2+ selector는 fail-closed하며 canon은 selector 뒤 stable modifier identity order다.

## RD-439 — TG-007 되부름 static contract surface

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SEMANTICS-CLOSED / SUCCESSOR-SURFACE-CLOSED / PRODUCT-OPEN`

`두고` static header의 `되부름=$짜임(기준=pin)`과 `되부름=$바닥(기준=pin[, 관계=registeredRelation])`을 정본 surface로 둔다. 기준은 같은 callable header의 ParamPinIdentity이고 local/world/hidden/AI lookup이 아니다. `$짜임`은 structural child strict decrease, `$바닥`은 sealed well-founded relation의 strict decrease를 모든 recursive edge에서 검사한다. 실패는 compile failure이며 ordinary bounded recursion으로 fallback하지 않는다. RD-078 bounded recursion, RD-186 resource ceiling과 RD-418 proof-core firewall을 보존한다.

## RD-440 — callable resource contract와 nested execution-root surface

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / RESOURCE-AUTHORITY-CLOSED / SUCCESSOR-SURFACE-CLOSED / PHYSICAL-SCHEMA-WIRE-OPEN / PRODUCT-OPEN`

새 `울타리=` family를 만들지 않고 RD-181~189·206·255 실행계약 자원축을 재사용한다. callable definition의 `필요자원`은 최소 요구, `자원상한`은 실행 ceiling을 선언하며 스스로 권한을 확대하지 못한다. `(일=실행할일, 실행계약=계약) 실행하기`는 child execution root를 만들고 계약 생략 시 parent 상속, stricter 허용, relaxation은 parent authorization 필수다. 실행 전 capability·resource preflight가 실패하면 body 실행 0이고 semantic unlimited도 명시 승인 필수이며 host safety limit는 남는다. 별도 execution identity·receipt·failure boundary를 보존하고 physical AST·schema·wire·diagnostic은 OPEN이다.

## RD-441 — 일묶음씨·일엮음씨·PlanLiteral successor

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SEMANTICS-CLOSED / SUCCESSOR-SURFACE-CLOSED / PHYSICAL-SCHEMA-WIRE-OPEN / PRODUCT-OPEN`

`일묶음씨`는 아직 실행하지 않은 완결된 일들을 담은 immutable deferred plan value로 유지하고 product `갈래씨` 권고를 current로 승격하지 않는다. `일엮음씨`는 `일묶음씨 + explicit ExecutionRelation`의 별도 nominal composite-work type이고 rename·alias·coercion이 아니다. `(일묶음=묶음, 실행관계=함께시작) 일엮기`가 immutable pre-execution `일엮음씨` 값을 만든다. expected `일묶음씨` expression 자리의 `{...}`는 construction effect 0의 PlanLiteral이며 기존 expected-type 표면을 유지하고 새 `일묶음{}` constructor를 만들지 않는다. immutable explicit capture만 허용하며 hidden mutable/world/Capability capture, auto execution과 auto flatten을 금지한다. physical AST·schema·wire·diagnostic은 OPEN이다.

## RD-442 — `(T)가리킴` acquire/read/failure surface

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SEMANTICS-CLOSED / SUCCESSOR-SURFACE-CLOSED / EXACT-WIRE-OPEN / PRODUCT-OPEN`

`(T)가리킴` 획득은 callable `가리키다`의 호출형 `가리키기`, 읽기는 callable `읽다`의 호출형 `읽기`로 명시한다. 읽기는 resolve와 owner/world/generation/type 검증 뒤 immutable snapshot을 `(T,가리킴안됨)맺음`으로 낸다. `가리킴안됨.$끊김/$세대다름/$누리다름/$씨다름`을 닫고 capability·permission·integrity·contract failure로 silent lowering하지 않는다. implicit acquisition/dereference/field mutation과 silent retarget을 금지하며 exact wire는 OPEN이다.

## RD-443 — fixed-slot 임자 `인` construction

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SEMANTICS-CLOSED / SUCCESSOR-SURFACE-CLOSED / PRODUCT-OPEN`

`이름:임자씨 := (slot=value,...)인 임자씨.`를 fixed-slot 임자 construction 정본으로 둔다. required slot exact match를 요구하고 missing·duplicate·unknown은 실패한다. 같은 NameType의 여러 slot은 ComponentSlotIdentity로 구별하며 붙이 경계 hidden flatten/search와 field/source order identity를 금지한다. authored construction candidate는 atomic publish 뒤에만 runtime SubjectIdentity가 된다. dynamic/optional runtime slots는 DEFER다.

## RD-444 — instance-first alert delivery와 모음 group send

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SEMANTICS-CLOSED / SUCCESSOR-TYPE-RULE-CLOSED / EXACT-DIAGNOSTIC-OPEN / PRODUCT-OPEN`

`알림 ~> 받을임자`와 동치 반대 표면 `<~`의 기본 endpoint는 명시 runtime 임자 instance 하나이며 씨이름은 implicit broadcast가 아니다. finite 임자 `모음`을 target으로 쓰면 한 snapshot에서 membership 확정, stable SubjectIdentity dedup·canonical order·전수 type/capability preflight 뒤 existing point-to-point candidate N개를 staged outbox/atomic publish로 처리한다. partial broadcast를 숨기지 않고 새 broadcast operator를 만들지 않는다.

## RD-445 — 임자-owned ReactionProvider exact authoring surface

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / AUTHORING-OWNERSHIP-CLOSED / SUCCESSOR-SURFACE-CLOSED / PHYSICAL-SCHEMA-WIRE-OPEN / PRODUCT-OPEN`

RD-293의 exact provider declaration source OPEN을 이 successor surface로 닫는다. 임자 정의 안의 named 움직씨가 닫힌 registration shape `{<알림씨>을/를 받으면}`를 소유하고 실제 수신 instance는 `받는이`, typed alert payload는 선언된 payload binder에 결속된다. instance마다 arbitrary closure를 복제하지 않으며 lowering은 `ReceiverRequirement × AlertType × stable ProviderIdentity`다. same ProviderIdentity diamond는 1회 dedup하고 distinct providers는 같은 settled snapshot에서 non-consuming fan-out하며 PatchCandidate를 deterministic merge·atomic publish한다. 구실 dispatch와 합치지 않고 source/import/completion order priority와 hidden/AI provider fallback을 금지한다. physical AST·registry schema·wire·diagnostic은 OPEN이다.

## RD-446 — runtime 모음 enumeration·subject comprehension

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SEMANTICS-CLOSED / SUCCESSOR-SURFACE-CLOSED / PRODUCT-OPEN`

runtime 열거 정본은 `모음{값들}`이고 old `(값들) 모음`은 history/versioned migration-only이며 permanent alias가 아니다. runtime 임자 조건제시는 typed suffix `{x:임자씨 | 참거짓식}모음`만 허용한다. 시작 시 committed world의 `SubjectIndexSnapshot<임자씨>`를 한 번 봉인하고 stable `(T)가리킴` identities를 pure Bool predicate로 거르며 mutation·움직씨·signal·network/time/random·hidden lookup을 금지한다. prefix pipe, untyped suffix, suffix whitespace/newline, general type universe enumeration과 numeric `[]`를 금지한다.

## RD-447 — mathematical 집합 enumeration·set-builder

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SEMANTICS-CLOSED / SUCCESSOR-SURFACE-CLOSED / PRODUCT-OPEN`

수학 열거는 `집합{수학식들}`, 조건제시는 `{x:수학씨 | 명제}집합`이며 fresh BinderIdentity와 PropositionIR를 소유한다. runtime `모음`의 참거짓·materialized collection과 AST/typed IR/identity/hash를 분리하고 Bool↔PropositionIR와 모음↔집합 implicit conversion을 금지한다. prefix pipe, untyped suffix와 suffix whitespace/newline을 금지하며 runtime materialization은 explicit finite enumeration/proof provider만 소유한다. `모든 T` runtime sugar는 R1 DEFER다.

## RD-448 — structural StateMachine R1 exact authored surface

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SEMANTICS-CLOSED / SUCCESSOR-SURFACE-CLOSED / PHYSICAL-SCHEMA-WIRE-OPEN / PRODUCT-OPEN`

`상태머신{ 상태=차림[$갈래...]. 처음=$갈래. 전이=차림[(전=..., 후=..., 사건=알림씨, 내용=payloadBinder, 조건=pureCallable, 행동=action)...]. }`를 StateMachineSpec authored surface로 둔다. `내용`은 사건 payload의 typed binder이고 `행동`은 `없음`, named 움직씨 또는 typed PlanLiteral 중 하나다. local states are spec-owned `$갈래`; spec, runtime instance and world state are distinct. guard는 pure이고 publish 권한이 없으며 enabled 0이면 무전이, 1이면 선택된 state change+action을 하나의 atomic candidate로 실행하고 action 실패 시 전부 rollback, 2+이면 source order 없이 fail-closed한다. hierarchy/parallel/history/timeout/retry/cancel과 physical AST·registry schema·wire·diagnostic은 OPEN이다.

## RD-449 — 글무늬 minimum R1

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SEMANTICS-CLOSED / SUCCESSOR-SURFACE-CLOSED / FORMATTER-SCHEMA-WIRE-OPEN / PRODUCT-OPEN`

`글무늬` owns text template values and bare head has sealed default `#말`; explicit `(#말)글무늬{...}` is the same semantic family with distinct source provenance. Literal/multiline/newline, `{키}`, typed `{키.필드}`, `{{/}}` and registered `{키|@포맷}` are admitted. Immediate `(bindings) 글무늬{...}` and stored `(bindings)인 무늬 채우기` share TemplateIdentity when inputs match. missing·extra·duplicate·unknown, bad path and 0/2+ formatter fail closed; arbitrary expression/file/network/AI evaluation and last-wins are forbidden.

## RD-450 — 말무늬 exact A+B authoring와 C sealed authority

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / THREE-LAYER-AUTHORITY-CLOSED / SUCCESSOR-SURFACE-CLOSED / PHYSICAL-SCHEMA-WIRE-OPEN / PRODUCT-OPEN`

Simple authoring은 `말무늬{ 항목=(무늬="literal {자리:씨}", 대상=callable). }`이고 ordinary structured `말무늬항목` authoring(B)과 함께 validation·ambiguity check·approval·versioned seal을 거쳐 authoritative periphery registry(C)로 투영된다. literal text와 typed slot, versioned finite morphology `말결`만 사용하며 runtime AI/LLM/fuzzy guessing은 authority가 아니다. Only exactly one sealed phrase→typed action/event candidate match succeeds; 0/2+ fail하고 source order/ranking/AI confidence는 권위가 아니다. Replay는 sealed resolution만 사용한다. `역말무늬`는 deterministic profile presentation이고 inverse/identity authority가 아니다. physical registry schema·wire·diagnostic과 editor/LSP는 OPEN이다.

## RD-451 — public Patch R1 rejection과 source clean breaks

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / PUBLIC-PATCH-REJECTED-R1 / MIGRATION-BOUNDARY-CLOSED / PRODUCT-OPEN`

Ordinary 움직씨 mutation lowers to internal PatchOp/PatchCandidate→deterministic merge/conflict→validation→atomic publish; PatchCandidate is not a user value and no public `고침{}` syntax is added. Old `(값들) 모음`, product `갈래씨` plan recommendation, old StateMachine clause grammar and incompatible 글무늬 call forms are history/explicit versioned migration inputs only. Unknown/ambiguous migration fails closed and permanent compatibility aliases are not created.

## RD-452 — ProjectSemanticConfiguration custody

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED A / PROJECT-SEMANTIC-CUSTODY-CLOSED / EXISTING-RD-ADDITIVE-SUCCESSOR / EXACT-MANIFEST-SCHEMA-WIRE-OPEN / PRODUCT-OPEN`

A formal project binds a compiler-supported SourceEpoch identity, explicit entry, semantic project configuration, dependency requirements and declared non-code resource membership into one ProjectSemanticConfiguration identity. Descriptive name, author, tag and UI metadata are separated from semantic fields. CLI flags, WASM callers, Workbench hidden selectors, content sniffing and conventional filenames cannot silently override this identity. A project selects a supported SourceEpoch rather than defining one; dependency-free projects preserve RD-161. Exact manifest file, field spelling, serialization and resource declaration surface remain OPEN.

## RD-453 — immutable ProjectExecutionSnapshot and same-snapshot claim

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED A / PROJECT-EXECUTION-SNAPSHOT-CUSTODY-CLOSED / IDENTITY-BOUNDARY-CLOSED / EXACT-HASH-CAS-DB-OPEN / PRODUCT-OPEN`

An explicit Run binds source logical-tree content identity, ProjectSemanticConfiguration identity, verified dependency-resolution identity and declared resource-content identity into one immutable ProjectExecutionSnapshot. Host checkout path, mtime, cursor, UI state, username, machine identity, cache path and saved-revision ordinal are not semantic identity. Analysis and execution may claim the same project meaning only when bound to the same snapshot identity. HistoryRevisionIdentity and semantic content identity remain distinct, so distinct revisions may have equal semantic content. Exact hash domain, binary encoding and CAS/DB layout remain OPEN.

## RD-454 — sealed Run dependency-resolution boundary

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED A / SEALED-RUN-NO-RESOLVE-NO-UPDATE-NO-NETWORK-CLOSED / RD-222-RD-231-PRESERVED / EXACT-LOCK-RESOLVER-TRANSPORT-OPEN / PRODUCT-OPEN`

An authoritative sealed Run consumes an already verified dependency resolution and does not resolve requirements again, create or update a lock/equivalent seal, mutate resolution or acquire dependencies from the network. Missing, damaged or mismatched resolution fails closed under RD-222 and RD-231. Dependency match/update is a separate explicit action; success produces a new resolution identity and ProjectExecutionSnapshot. A future unsealed live-link development profile is not sealed/reproducible Run evidence. Exact lock filename, schema, wire, resolver transport and UI wording remain OPEN.

## RD-455 — project edit·recovery·save·run lifecycle separation

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED A / PROJECT-LIFECYCLE-ROLE-SEPARATION-CLOSED / IMMUTABLE-RUN-SNAPSHOT-CLOSED / EXACT-AUTOSAVE-STORAGE-GC-OPEN / PRODUCT-OPEN`

Editor generation, recovery checkpoint, durable saved project revision and execution snapshot are distinct logical roles. Editing is not an automatic save, Run is not a save event, and a recovery checkpoint is not an authoritative saved revision. Edits after Run starts do not mutate that Run's immutable ProjectExecutionSnapshot. Exact class/table/file names, autosave cadence, retention, compression, delta format, storage and garbage collection remain OPEN.

## RD-456 — project UX terminology role firewall

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED B+A-LIMITED / PROJECT-UX-PRESENTATION-TERMS-CLOSED / SOURCEEPOCH-CANONICAL-TERM-OPEN / EXISTING-TERM-OWNERS-PRESERVED / NO-PARSER-ALIAS / PRODUCT-OPEN`

The project-owned SourceEpoch semantic contract is active but its preferred user-facing name `글판` remains OPEN pending historical UI and public profile research. In project UX context, `차림새` may present project manifest/configuration, `곳간` the declared non-code resource collection and `가지잠금` the project dependency lock. These role-qualified terms do not supersede `한도차림새` profile/preset meaning, `가지곳간` history/registry ownership or canonical `잠금글`, and do not create parser keywords, permanent compatibility aliases or stable physical fields. `처음.ddn` is a product scaffold default without semantic authority; `첫글` and normative `씨` retain existing meanings.

## RD-457 — MSCS-O1 module-root static directive successor

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / MODULE-ROOT-STATIC-DIRECTIVE-SURFACE-CLOSED / PHASE-AUTHORITY-CLOSED / OLD-BLOCK-MIGRATION-ONLY / EXACT-AST-WIRE-PRODUCT-OPEN`

The canonical successor forms are `(항목들=...) 쓰임.`, `(항목들=...) 드러냄.` and `(실행계약=..., 보개=...) 얼개.`. They reuse only the lexical call shell and are not ordinary Call semantics, 셈씨, 움직씨, runtime callable or constructor. `쓰임` is module 0/1 and uses the shared lexer/token/span authority plus a restricted notation-independent bootstrap parser to produce SourceRef skeletons before full parse, with parser I/O, raw scanning, ad-hoc lexer, full-parser bootstrap, wildcard/unqualified injection and duplicate/conflict resolution forbidden. `드러냄` is module 0/1, default-private and after typed resolution binds external names to exact DefinitionIdentity with explicit re-export and fail-closed private/unknown/duplicate handling. `얼개` is designated-root 0/1 static configuration before runtime; omission is an approved default expansion and Patch/world/runtime dispatch is zero. Old brace/colon block forms, `공개 {}` and `프로젝트 {}` are explicit-version history/migration only with no permanent alias.

## RD-458 — Regex R1 successor selector and general callable operations

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SUCCESSOR-SURFACE-CLOSED / FOUR-GENERAL-CALLABLES-CLOSED / PREDECESSOR-MIGRATION-ONLY / PRODUCT-OPEN`

The canonical carrier is `(#또니정규)글무늬{...}` and public Regex-specific operation-name count is zero. Ordinary typed callable names own the roles: `맞추기` is whole-input Bool match, `찾기` is first matched `(글)있없`, `나누기` removes separator matches without capture insertion and preserves leading/trailing/adjacent empty segments, and `바꾸기` has distinct literal `글` and typed `글무늬<#말>` replacement overloads selected by the replacement static type. `#regex`, `정규맞추기`, `정규찾기`, `정규바꾸기`, `정규나누기` and old `정규식{}` are explicit-version history/migration inputs only; permanent aliases and silent migration are forbidden.

## RD-459 — Regex dispatch·semantic·replay identity and backend firewall

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / REGEXIR-PROFILE-FAMILY-DISPATCH-CLOSED / EXACT-IDENTITY-CLOSURE-CLOSED / BACKEND-CONFORMANCE-ONLY / PHYSICAL-SCHEMA-WIRE-OPEN / PRODUCT-OPEN`

Regex overload dispatch identity is `RegexIR family + #또니정규 family`; exact profile version, backend, source/import/definition order and return type are not overload-selection evidence. Regex semantic/replay/receipt closure binds RegexIR canon, exact RegexProfileIdentity and version, Unicode data version, canonical flags, admitted feature and selection/zero-length/position contract, semantic ceilings, WorkCost formula identity, execution-contract/resource budget and conformance evidence in their declared roles. RegexIR/Profile is semantic authority and a backend is only a conformance implementation. Erasure to ordinary 글무늬 TypeId, host-regex authority and silent fallback are forbidden. Exact hash algorithm, physical schema, wire and receipt encoding remain OPEN.

## RD-460 — Regex literal/typed replacement and capture-shape projection

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / REPLACEMENT-OVERLOAD-SEMANTICS-CLOSED / CAPTURE-SHAPE-PHASE-PROJECTION-CLOSED / PUBLIC-CAPTURE-TYPE-0 / PRODUCT-OPEN`

In the literal `글` replacement overload every character, including `$1`, `${name}` and `\1`, is literal and no host replacement mini-language or callback is interpreted. Only the `글무늬<#말>` overload owns typed capture-aware rendering. Named-capture identity, optionality and required replacement slots are a deterministic phase-only static projection derived from RegexIR for pre-execution template validation; they do not create a public nominal capture type or numeric ordinal semantic identity. Duplicate named capture, unknown or wrong-type slot and implicit optional-capture-to-empty flattening fail closed; runtime capture-name guessing is forbidden. Physical capture metadata schema remains OPEN.

## RD-461 — Regex admitted grammar·flags·Unicode·bounded execution and scoped syntax census

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / REGEX-R1-EXACT-SEMANTIC-SURFACE-CLOSED / BOUNDED-EXECUTION-CLOSED / SCOPED-SYNTAX-RESEARCH-CLOSED / PRODUCT-OPEN`

R1 admits `*`, `+`, `?`, `{n}`, `{n,m}`, `{n,}` with greedy default and explicit lazy suffix, and forbids backreference, lookaround, recursion/subroutine, atomic/possessive/conditional constructs and host callbacks. An optional single `(?ims)` header may occur only at normalized Regex body offset 0 and canon orders flags `ims`; mid/scoped/disable, x/U/u, duplicate and unknown flags fail. Matching and zero-length progress use Unicode scalar boundaries with pinned Unicode data, no automatic normalization and no grapheme fallback. Finite input, RegexIR/count/match/output ceilings and versioned WorkCost/resource bounds produce typed exhaustion failure; silent no-match, truncation and fallback are forbidden. Scope-limited census records `NEW_SPECIAL_SYNTAX_FAMILY_DISCOVERED=0`, `MODULE_ROOT_NEW_SPECIALBLOCK_REQUIRED=0` and remaining special-syntax research excluding separately-owned meta/advanced-StateMachine programs as zero; it does not erase or reopen existing 말무늬, StateMachine or meta owners.

## RD-462 — META/JIEUM U1 typed declaration-generation gate and hygiene

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / META-GOVERNANCE-CONSOLIDATION-CLOSED / NAME-HYGIENE-AND-AUTHORITY-FIREWALL-CLOSED / PRODUCT-OPEN`

Compiler-private synthetic identity does not participate in user identifier namespace, lookup, public semantic identity, canon or hash, and source-order temporary identity cannot change user declaration collision or output identity. A user-visible generated declaration gains no parser, SpecialBlock, plugin, capability, effect, trust, checker or semantic-registry authority merely by being generated. Every declaration-producing facility uses one compiler-owned typed Declaration Generation Contract: typed intent, immutable DeclarationBundle, collision check, ordinary resolve/type/generic/kusil/effect/canon/lock validation and all-success atomic integration, without raw source generation, parser re-entry or arbitrary AST insertion. RD-368 direct typed SpecialBlock lowering remains distinct. Exact private representation, physical gate/schema/wire/diagnostic and product landing remain OPEN.

## RD-463 — META/JIEUM U2 GenerationRequestStmt successor surface

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SUCCESSOR-SURFACE-CLOSED / GENERATION-CALL-SPECIALBLOCK-ROLE-SEPARATION-CLOSED / RD-248-CLAUSE-ONLY-SUPERSEDE / PRODUCT-OPEN`

Only RD-248's top-level `지음 { 대상=..., 원형=..., 설정=(이름: 값) }.` request surface and its one-block-one-request wording are superseded. The canonical successor is `(대상=..., 원형=..., 설정=(이름: 값)) 지음.`; `대상`, `원형`, `설정` use fixed pin `=`, open settings fields retain `:`, and one GenerationRequestStmt owns one request. It is a top-level compile/tool phase directive, not an ordinary runtime call, CallExpr, ExprStmt, callable value, SpecialBlock or TagBundle. The `지음` head is resolved by this grammar slot without expected-type, callable, body-parser or AI guessing. RD-245~247 and all remaining RD-248 builder, first-recipe, field-policy, canon/display, whole-failure, ordinary-validation and atomic-integration contracts are inherited. Old brace source is version-pinned migration-only, never a silent or permanent alias. Exact parser enum, diagnostic, migration encoding and product landing remain OPEN.

## RD-464 — META/JIEUM U3 typed authoring materials and logical builders

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / TYPED-JIEUM-AUTHORING-MATERIALS-CLOSED / RAW-SOURCE-TOKEN-AST-API-0 / PRODUCT-OPEN`

V1 custom `지음씨` authoring exposes only registered typed declaration recipes and immutable `선언묶음` composition, not raw source, token, compiler AST, general expression builder, quote/unquote or a forgeable declaration-candidate type. `⟦선언후보⟧` is an inference-only opaque phase value accepted only by `표준지음.선언보태기`. Logical signatures are `표준지음.빈선언묶음 : () --> 선언묶음`, `표준지음.필드설명 : (대상=이름씨보기) --> ⟦선언후보⟧`, and `표준지음.선언보태기 : (묶음=선언묶음, 선언=⟦선언후보⟧) --> 선언묶음`. The first recipe uses one closed 이름씨보기, public displayable fields, an empty versioned settings schema and the U4 fixed generated name to emit one pure named 셈씨 candidate with `대상:T` and result `글`. Unsupported, secret, resource, reference, recursive, unresolved-generic or ambiguous-formatter paths fail closed. Builders are phase-only, immutable and scope/base-snapshot checked; commit authority remains U1 and RD-245~246. Exact physical type/layout/schema and product implementation remain OPEN.

## RD-465 — META/JIEUM U4 GenerationScope and GeneratedName identity

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / GENERATION-SCOPE-AND-GENERATED-NAME-CLOSED / AMPERSAND-SIGIL-CLOSED / COLLISION-FAIL-CLOSED / PRODUCT-OPEN`

GenerationScopeIdentity binds SourceEpochIdentity, containing OrdinaryDeclarationScopeIdentity, GenerationRequestIdentity and immutable BaseSnapshotIdentity. Generated declarations are added to that ordinary declaration scope, never injected into a target type body, another module/import owner, SpecialBlock registry or capability/trust registry. GeneratedNameIdentity is distinct from OrdinaryNameIdentity; canonical source is `&이름` with no whitespace. `&` is only the generated-name sigil, not an ordinary prefix operator, capability, effect or trust marker, and `%` is not a generated-name sigil. The first empty-settings field-description recipe owns canonical `&설명하다`; `&설명하기` is its registered independent invocation morphology form under the existing versioned callable morphology contract and is not a second identity. Raw 글 and string concatenation cannot create GeneratedNameIdentity. Ordinary/generated namespaces are distinct. Generated identity/signature, duplicate or bundle collision fails the whole generation without auto-number, dedup, source-order or last-wins; only ordinary callable overload rules apply. Exact hash/encoding/tokenization/schema/wire and product landing remain OPEN.

## RD-466 — META/JIEUM U5 four-layer composition and growth gate

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / JIEUM-FOUR-LAYER-COMPOSITION-CLOSED / R1-GROWTH-BUDGET-AND-ARCHITECTURE-GATE-CLOSED / PRODUCT-OPEN`

The standard generation system has four authority-separated layers: L1 standard generated declarations, L2 standard 지음씨, L3 public standard generation materials and L4 compiler-owned standard base materials, with dependency direction L4 to L3 to L2 to L1. Higher layers do not mint parser, type, trust or capability authority for lower layers. R1 growth budgets are L2 generators 9, L3 public materials 10 and L4 Core 9 plus extension 2; these are governance budgets, not permanent ABI, enum, registry-row, release-membership or V1 counts. At least 80 percent of Core standard generators must be compositionally expressible through admitted public materials, excluding compiler-sealed authority. A new Core base primitive is admitted only when existing materials cannot express it and at least two distinct upper material families repeatedly require it. Architecture review is mandatory before Core base materials exceed 12. Catalog rows and exact public names are registry/gaji/pack concerns, not separate RD rows.

## RD-467 — META/JIEUM U6 phase views and typed settings

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / JIEUM-PHASE-VIEW-AND-TYPED-SETTINGS-CLOSED / PREFLIGHT-FAIL-CLOSED / PRODUCT-OPEN`

RD-247's phase-only typed view family is additively extended with `고름씨보기`, `갈래보기` and `움직씨보기`. They are immutable narrow views, not one dynamic reflection object, and expose no raw source or compiler AST. Every L2 지음씨 owns a versioned typed settings schema. Unknown, missing-required, duplicate or type-mismatched settings fail before generator-body execution. An empty schema is valid and owns the first recipe's empty-settings vertical; it does not make every generator empty-only. Exact view field catalog, settings syntax, schema encoding, registry, wire and product implementation remain OPEN.

## RD-468 — META/JIEUM U7 standard families and boundary profiles

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / JIEUM-STANDARD-FAMILY-GOVERNANCE-CLOSED / WRAPPER-FIRST-AND-BOUNDARY-PROFILE-CLOSED / PRODUCT-OPEN`

R1 governance covers 설명, 검증, 외부표현, 변환, 고름도구, 구실맞춤, 실행관측 and CLI composition families without one kernel per catalog item. Generated 움직씨 begins with typed input-match, pure condition, immutable work-bundle/action-dispatch and execution-observation wrappers; arbitrary mutation AST, PatchOp, world publish, capability and trust generation are forbidden. JSON, CLI, CSV, CBOR and later boundary formats are registered members of one boundary-format profile family rather than private generator kernels. Text generation and presentation reuse RD-449 글무늬 and registered formatter authority; hidden host formatters are forbidden. Core equality, hash and copy are not generation customization points. Exact catalog membership/spelling, codec/Unicode/number policy, physical profile schema, wire and release membership remain OPEN.

## RD-469 — META/JIEUM U8 semantic execution mark and physical evidence firewall

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / JIEUM-EXECUTION-OBSERVATION-FIREWALL-CLOSED / PHYSICAL-PERFORMANCE-NONAUTHORITY-CLOSED / PRODUCT-OPEN`

Semantic execution observation uses a versioned deterministic SemanticExecutionMark bound to sealed semantic inputs and deterministic logical ordering or causal relation. Wall time, CPU time, worker/thread identity, scheduler order, host counters and similar measurements are PhysicalPerformanceEvidence. They may be retained as diagnostics but do not participate in semantic state/hash, control flow, replay equivalence, branch/result selection, canon or success authority. Generated behavior that uses physical performance evidence to choose a semantic branch, result or identity fails closed. Exact mark enum, event schema, clock/counter collection, receipt encoding, storage and product implementation remain OPEN.

## RD-470 — GM-A1 module identity three-layer separation

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / MODULE-IDENTITY-SEMANTICS-CLOSED / EXACT-ENCODING-HASH-SCHEMA-WIRE-OPEN / PRODUCT-OPEN`

`ModuleLogicalIdentity`, `ResolvedModuleIdentity` and consumer-local `LocalModuleName` are distinct. ModuleLogicalIdentity is a stable module-local semantic identity under a logical gaji dependency; ResolvedModuleIdentity binds it to the actually resolved gaji content identity; LocalModuleName is only the current source namespace binding name. Filesystem/source URL/checkout path/local alias are not module semantic identity. Local rename, file move or checkout move alone creates neither a new ModuleLogicalIdentity nor a new ResolvedModuleIdentity.

## RD-471 — GM-A2 dependency requirement and module binding separation

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / DEPENDENCY-BINDING-SEPARATION-CLOSED / DEFAULT-LOCAL-NAME-CLOSED / EXACT-AUTHORED-SURFACE-OPEN / PRODUCT-OPEN`

A Gaji dependency requirement states which logical gaji dependency is required; a module binding selects a module supplied by an available resolved gaji into the current source namespace. One gaji may supply multiple modules and dependency presence never exposes them automatically. An eventual shorthand must lower to the two distinct typed roles. Explicit module selection with omitted alias uses the canonical module name as LocalModuleName; name conflict fails closed and only an explicit alias resolves it. Auto-number, last-wins, source/import/registry order are forbidden.

## RD-472 — GM-A3 V1 module-level selective binding

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / V1-MODULE-LEVEL-SELECTION-CLOSED / DEFINITION-LEVEL-IMPORT-DEFERRED / PRODUCT-OPEN`

V1 selective binding is module-level only and definition use remains module-qualified. Definition-level selective import and unqualified consumer-namespace injection are not V1 canonical surfaces. `MODULE_SELECTIVE_DEFINITION_IMPORT_V2` is a named `DEFERRED / SUCCESSOR-RESEARCH-AND-PRODUCT` item covering collision, provenance, re-export, AI/codegen and notation interaction; V1 exclusion is not permanent feature denial.

## RD-473 — GM-A4 re-export ownership and visibility

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / REEXPORT-OWNERSHIP-PROVENANCE-CLOSED / EXPLICIT-NOTATION-ACTIVATION-DIRECTION-CLOSED / EXPORTED-PROVIDER-VISIBILITY-CLOSED / EXACT-SCHEMA-SURFACE-OPEN / PRODUCT-OPEN`

An explicitly re-exported API belongs to the re-exporting module's public surface while original definition/provider identity remains origin provenance; consumer namespace authority does not revert to the origin. Notation is not activated merely by module presence: explicit activation is the V1 default refinement of RD-278. Provider resolution candidates include only the module's exported provider surface; private providers do not leak through dependency or binding. Exact provenance chain, activation and provider-selection surfaces remain OPEN.

## RD-474 — GM-A5 exact-first and authenticity policy

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / V1-EXACT-FIRST-CLOSED / INTEGRITY-AUTHENTICITY-SEPARATION-PRESERVED / SOURCE-CLASS-POLICY-DIRECTION-CLOSED / EXACT-TRUST-SCHEMA-WIRE-OPEN / PRODUCT-OPEN`

V1 authored dependency requirements are EXACT-FIRST; ambient latest and silent fallback remain forbidden. Version ranges remain future capability `GAJI_VERSION_RANGE_V2`. Publisher authenticity/authorization is distinct from content integrity and may be governed by a sealed source-class policy for local development, official registry, enterprise/private registry and external third-party sources; it cannot vary as hidden environment state. Exact trust root, signature, publisher schema, authorization, rotation/revocation, policy wire and license integration remain OPEN.

## RD-475 — U1 Regex exact R1 package and RD-461 narrow correction

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / COMPILE-READY-SURFACE-CLOSED / RD-461-EXACT-CLOSURE-NARROWED / PRODUCT-OPEN`

The existing `#또니정규` carrier admits only group, noncapture, named capture, class/range, alternation, anchor, bounded repetition, pinned Unicode General_Category and White_Space properties, and scalar escape. Its scanner owns escaped, class, splice, repetition, group, header and terminal outer-brace states. Phase capture projection is `잡음.전체 : 글` and `잡음.<named> : (글)있없`; ordinal capture is not semantic identity. Flag `i` uses a pinned locale-independent Unicode Simple_Case_Folding profile; full 1:N folding is absent. Host regex authority, `\w`, `\b`, loose property aliases, backreferences, lookaround, recursion and host callbacks fail closed. RD-461 and v25.18 custody remain historical, while only the overly broad exact-closure claim is superseded.

## RD-476 — U2 strict selection and safe lookup separation

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SAFE-LOOKUP-SURFACE-CLOSED / MIGRATION-ONLY-LEGACY / PRODUCT-OPEN`

Strict `[]` selection keeps compile failure for statically known invalid selection and strict-contract abort for runtime-only violations; it never silently returns absence or typed failure. Canonical safe lookup is the ordinary typed overload family `찾기`: `((T)차림, 위치) -> (T)있없`, `((K,V)짝맞춤, 열쇠) -> (V)있없`, and `(글, 위치) -> (글)있없`. `찾기?` is versioned migration/history-only and is not a permanent alias.

## RD-477 — U3 RD-079 normalization and bootstrap preservation

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / RD-079-PRESERVED / NO-HIDDEN-NORMALIZATION / PRODUCT-OPEN`

말무늬 decodes a 글 literal and then preserves exact scalar content; automatic NFC, NFKC, trimming, collapsing and fuzzy authority are zero. The `#!` language bootstrap begins at raw byte zero, terminates at LF or EOF, admits exactly one registered language name and fails closed on unknown or redeclared language. UTF-8 BOM exception and automatic BOM stripping are both zero; absence selects Korean under the existing contract.

## RD-478 — U4 ExecutionRelation exact grouping and canon

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / EXECUTION-RELATION-GROUPING-CLOSED / PRODUCT-OPEN`

The active ExecutionRelation family has exactly twelve relations and excludes RD-401's retired `~지 못하면`; recovery remains `해보고 안되면`. An ungrouped chain may flatten to one N-ary relation only when every edge has the same normalized relation family. Heterogeneous relation families require ordinary parentheses. Hidden precedence, declaration order, completion order and fallback tables have no semantic authority.

## RD-479 — U5 multiple Guseil requirement bundle

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / MULTIPLE-GUSEIL-REQUIREMENT-SURFACE-CLOSED / PRODUCT-OPEN`

The existing single requirement grammar is reused. Multiple requirements use one nonempty parenthesized comma bundle. `guseil_argument_list := type_ref (',' type_ref)*`, `applied_guseil_identity := '(' guseil_argument_list ')' guseil_qname`, and a requirement atom is nominal or applied GuseilIdentity. Each `type_ref` consumes balanced nested application parentheses; only commas at the immediate bundle depth separate requirements. Empty or trailing elements, duplicate identity, conflicting application, naive comma splitting, set deduplication and last-wins fail closed.

## RD-480 — U6 seven structural surface-shape classes

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / CLOSED-SHAPE-CLASSIFICATION / NO-USER-CLASS-SYNTAX / PRODUCT-OPEN`

Registered multisurface skeletons classify structurally as exactly one of ATOM, PREFIX, POSTFIX, BINARY, CIRCUMFIX, MIXFIX or KOREAN_RELATION_PARTICLE. BINARY requires two placeholders, zero outer literal islands, exactly one internal literal island and declared role marker count zero. Zero or two-or-more matches fail closed. Import/declaration order, return type, runtime value, hidden priority and user-defined precedence, associativity or priority cannot choose the class. No new user-facing class syntax is introduced.

## RD-481 — U7 anonymous FunctionLiteral source and grouping

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / FUNCTION-LITERAL-SOURCE-CLOSED / ONE-PHYSICAL-TOKEN / NO-NEW-SEMANTIC-OPERATOR / PRODUCT-OPEN`

Canonical anonymous FunctionLiteral uses `\|->`, scanned as one maximal-munch physical token for the existing FunctionLiteral identity. It has the lowest expression binding and is right associative, so `x \|-> y \|-> E` means `x \|-> (y \|-> E)`. The RHS consumes a full expression to the same-depth container delimiter. Selection may occur inside the RHS; applying call, field or selection postfix to the FunctionLiteral value requires parentheses. The left side is a binder head, not an arbitrary expression. Old `{x \| E}` is versioned migration-only; `->`, `-->` and C-style call are not aliases.

## RD-482 — U8 conservative identifier and pronunciation profile

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / IDENTIFIER-ADMISSION-CLOSED / PRONUNCIATION-NONAUTHORITY / PRODUCT-OPEN`

R1 identifier start admits ASCII letter, underscore, Hangul syllable and Hangul compatibility jamo; continuation additionally admits ASCII digit. Pure ASCII digits remain number authority. Latin or mixed identifiers acquire no automatic Korean pronunciation and use existing role markers or named pins when disambiguation is needed. Bare `_` may tokenize but resolves only as existing Infer in `type_ref`; it is rejected as ordinary NameIdentity and gains no wildcard, hole, generated-name or AI meaning. `_name` is an ordinary identifier subject to the existing public-name normalization phase. Broad Unicode, NFD combining names, emoji, default-ignorable controls and non-ASCII numeric admission fail closed in R1.

## RD-483 — U9 raw 글무늬 ordered Option B normalization

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / RAW-BODY-PROFILE-CLOSED / ORDERED-NORMALIZATION / PRODUCT-OPEN`

Raw 글무늬 applies this order: CRLF and lone CR become LF; remove exactly one first LF if present; remove exactly one last LF if present; find the minimum leading SPACE/TAB scalar count among nonblank and non-whitespace-only lines and remove that many where present; remove a final U+005C and the immediately following LF; then apply no NFC, NFKC, tab expansion, trim or blank-line collapse. Ordering is semantic. An EOF-final U+005C remains literal. The current product remnant that deletes it is a product gap and cannot be imported as SSOT meaning.

## RD-484 — U-01 StructureIR carrier와 StructureProfile host

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / STRUCTURE-PROFILE-HOST-CLOSED / NEW-CORE-IR-0 / EXACT-PHYSICAL-OPEN / PRODUCT-OPEN`

Higher Structure는 새 Core IR family가 아니라 existing StructureIR carrier에 적용되는 versioned StructureProfile policy contract다. StructureIR carrier가 StructureIdentity, CellIdentity와 IncidenceIdentity를 계속 소유하며 ProfileIdentity는 StructureIdentity와 독립이다. grade는 arity나 universe level이 아니다. source, target, composition, identity, associativity, globularity, simplicial, cubical 또는 Path 법칙은 universal core law가 아니며 admitted profile, provider와 checker가 소유한다. Exact physical StructureIR schema, ABI와 제품 구현은 OPEN이다.

## RD-485 — U-02 Profile schema, inheritance와 identity canon

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / PROFILE-SCHEMA-IDENTITY-CLOSED / TOP-LEVEL-6 / SUBAXIS-12 / BASE-0-TO-1 / MULTIPLE-BASE-DEFERRED / EXACT-DIGEST-WIRE-OPEN`

StructureProfile logical schema는 base, structure, cell, capabilities, validation, projections의 6 top-level과 바탕, 켜, 만들기, 테, 뜻머리, 값, 연산, 파생, 법칙, 제약, 검사, 투영의 12 semantic sub-axis를 가진다. R1 base는 0개 또는 1개이며 sparse inheritance 뒤 완전 정규화해 봉인한다. 같은 provider key의 상이한 identity나 version, 모순 law나 constraint와 unresolved reference는 hidden priority, source order와 last-write-wins 없이 fail closed한다. ProfileIdentity에는 schema와 profile version, resolved base와 policies, provider, checker와 evidence identities, assumption closure, scoped boundary semantics, laws, constraints와 checked projections가 참여한다. field order, whitespace, comments, presentation aliases와 editor, debug, cache metadata 및 physical layout은 참여하지 않는다. Permanent digest와 wire는 OPEN이다.

## RD-486 — U-03 Structure operations와 strict canonical privilege

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / STRUCTURE-OPERATION-LAW-CLOSED / AMBIGUITY-FAIL-CLOSED / EXACT-TOKEN-GRAMMAR-OPEN / PRODUCT-OPEN`

Structural pasting, composition, semantic identity operation 제자리, inverse operation 되돌이와 reversal은 서로 다르다. self-loop는 identity가 아니고 reversal은 inverse가 아니다. Omitted composition interface는 compatible candidate가 정확히 하나일 때만 허용하며 0개는 실패하고 2개 이상은 ambiguity failure다. Weak profile은 composition tree를 보존한다. Flattening 또는 strict canonical privilege는 admitted provider, checker와 sealed evidence를 모두 요구한다. Exact tokens, precedence, parser AST와 diagnostics는 OPEN이다.

## RD-487 — U-04 construction-family scoped boundary policy

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SCOPED-BOUNDARY-NORMALIZATION-CLOSED / FLAT-GLOBAL-UNION-PROHIBITED / PROFILE-IDENTITY-PARTICIPATION-CLOSED / EXACT-SCHEMA-WIRE-OPEN`

Boundary policy는 construction-family scope에서 봉인한다. 서로 다른 scope는 독립 공존하고 같은 scope의 동일 policy는 normalize하며 nonempty subset narrowing은 허용한다. 같은 scope의 incompatible closure, direction 또는 reversal과 seal 뒤 unresolved scope는 fail closed한다. Flat global union과 cross-product는 금지한다. Scope semantics는 ProfileIdentity에 참여한다. C-R2-02가 이 normative correction을 소유하고 exact detjson, physical schema와 wire는 OPEN이다.

## RD-488 — U-05 typed derived capability와 provider admission authority

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / TYPED-DERIVED-CAPABILITY-CLOSED / UNIQUE-ADMISSION-AUTHORITY-CLOSED / RD-360-AUTHORITY-PRESERVED / EXTERNAL-VERIFICATION-OPEN / PRODUCT-OPEN`

기존 파생 sub-axis는 stable provider identity, provider version과 typed family를 가진 derived-capability map이다. 최소 family는 derived_structure, derived_predicate와 sealed_principle이며 missing 또는 unknown family는 fail closed한다. Actual admission identity는 정확히 하나의 authoritative StructureProfile/provider registry가 소유한다. MATH_SURFACE_COVERAGE_REGISTRY는 admitted entry를 reference할 수 있으나 admission을 소유하지 않는다. Reference는 admission ownership이나 provider signature, checker, kernel 또는 runtime 검증이 아니다. Exact registry filename, row schema, encoding과 wire는 OPEN이다.

## RD-489 — U-06 checked projection과 cross-family firewall

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / CHECKED-PROJECTION-GATE-CLOSED / CROSS-FAMILY-FIREWALL-CLOSED / HOTT-DEMAND-GATE-PRESERVED / PHYSICAL-CHECKER-OPEN / PRODUCT-OPEN`

Cell은 Path가 아니고 self-loop는 Refl이 아니며 reversal은 inverse가 아니다. Path는 DefEq가 아니고 Equiv는 sealed Univalence evidence 없이 Universe Path가 아니다. StructureIdentity는 StateMachineSpec 또는 StateMachineInstance가 아니다. Projection 성공은 profile eligibility, assumption closure, required law evidence, checked adapter와 target checker가 모두 일치해야 한다. C-R2-03은 provider reference와 external semantic verification을 분리한다. HoTT-1과 HoTT-2 demand gate, exact checker receipt, provider execution, proof kernel과 제품 구현은 OPEN이다.

## RD-490 — U-07 Profile authoring과 terminology governance

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / PROFILE-AUTHORING-GOVERNANCE-CLOSED / TYPED-TERMINOLOGY-PROFILE-CLOSED / EXACT-SURFACE-OPEN / PRODUCT-OPEN`

Profile descriptor는 ordinary DDN value 또는 bundle을 profile role에서 정적으로 validate, resolve와 seal하는 방향이며 새 차림새 literal을 요구하지 않는다. Higher-structure authored entry는 profile-qualified role을 요구하고 bare 짜임 block은 AGE5 JjaimBlock owner를 유지한다. 구실은 RD-292, 고쳐쓰기는 RD-310, function type arrow는 RD-376 owner를 유지한다. n켜 조각, n-셀, n-cell과 n차원 셀은 typed terminology profile 안에서 같은 semantic identity를 가리킬 수 있으나 bare dimension을 global 켜 alias로 만들지 않는다. Final public terminology, exact source spelling, block head, grammar, AST와 canon은 OPEN이다.

## RD-491 — PC-IR-U1 explicit declared-resource closure

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / PROJECT-RESOURCE-CLOSURE-CLOSED / LOGICAL-PATH-PORTABILITY-FAIL-CLOSED / EXACT-SOURCE-SCHEMA-WIRE-OPEN / PRODUCT-OPEN`

A formal project's non-code semantic resource membership is an explicit closed set of canonical logical resources. Incidental undeclared physical files are identity-neutral and unavailable to authoritative Run. Declared missing, canonical logical identity duplicate, case/NFC portability collision, and project-root or reparse escape fail closed. Directory convenience is materialized into a deterministic explicit logical set before snapshot sealing. A declared resource content change creates a new resource-content identity and requires a new ProjectExecutionSnapshot. Exact source syntax, folder layout, path encoding and physical reparse policy remain OPEN.

## RD-492 — PC-IR-U2 semantic participation·SourceEpoch·common invocation context

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / PROJECT-SEMANTIC-PARTICIPATION-CLOSED / EXPLICIT-SOURCEEPOCH-FRONTDOOR-CLOSED / HIDDEN-DEFAULT-0 / EXACT-FIELD-SCHEMA-WIRE-OPEN / PRODUCT-OPEN`

A ProjectSemanticConfiguration field is semantic when changing it within the same logical source tree can change resolve, type or runtime meaning, declared-resource membership, or effective target profile. Descriptive, provenance, UI and host-local fields are excluded. A formal project persists a compiler-supported SourceEpoch; missing legacy input requires explicit migration and unknown or unsupported epochs fail closed. A standalone invocation may bind the epoch without a manifest. CLI, WASM and Workbench share one explicit logical root, input set, epoch and project-or-standalone invocation context and may not create hidden authority or infer epoch from current defaults, content or public SPEC. Exact field names, source spelling and user-facing Korean epoch term remain OPEN.

## RD-493 — PC-IR-U3 atomic save·reopen·recovery integrity

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / ATOMIC-COMPLETE-SAVE-CLOSED / REOPEN-INTEGRITY-CLOSED / STALE-SAVE-FAIL-CLOSED / EXACT-STORAGE-TECHNOLOGY-OPEN / PRODUCT-OPEN`

Authoritative durable-save visibility is exactly OLD_COMPLETE or NEW_COMPLETE; a HALF manifest/content/resource revision is never authoritative HEAD or Run input. Reopen verifies stored identity against actual content recomputed by sealed canon; mismatch fails closed without silent repair, overwrite or accept. RecoveryCheckpoint is not authoritative HEAD, and adopting recovery followed by durable save creates a new revision. A stale writer whose ExpectedHead differs from ActualHead fails closed and V1 does not require auto-merge. Exact DB/CAS/file layout, journal or rename technology, autosave cadence, retention, compression and GC remain OPEN.

## RD-494 — V1-PRODUCT-U1 사용자 제품 정의·필수 frontdoor·제품 형식

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / V1-PRODUCT-DEFINITION-DIRECTION-CLOSED / REQUIRED-ROLE-DIRECTION-CLOSED / ACTUAL-C2-MEMBERSHIP-0 / PRODUCT-OPEN`

V1 release claim은 외부 사용자가 공식 배포물을 설치하고 공개된 지원범위를 학습하여 single-file source와 formal Local Project를 작성·실행하고, 프로젝트를 저장·종료·재개한 뒤 같은 sealed semantic input에 대해 같은 의미 결과와 필요한 실행 근거를 확인할 수 있는 사용자 제품을 뜻한다. Native CLI와 Workbench는 V1 REQUIRED role direction이며, Workbench와 WASM transport는 별도 parser, type, canon 또는 runtime authority를 만들지 않고 하나의 shared product semantic authority와 explicit invocation context를 소비한다. Standalone WASM은 별도 Owner membership 없이는 REQUIRED product가 아니다. 이 logical role direction은 actual RELEASE_CAPABILITY_REGISTRY membership row를 만들지 않으며 exact command, binary, crate와 UI surface는 C2 product profile이 소유한다.

## RD-495 — V1-PRODUCT-U2 public SPEC·제품 지원·비지원 방화벽

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SPEC-SUPPORT-SEPARATION-CLOSED / UNSUPPORTED-DETERMINISTIC-FAIL-CLOSED / EXACT-C2-STATUS-SPELLING-OPEN / PRODUCT-OPEN`

Public Language SPEC의 docs-first surface와 V1 product-supported surface는 동일하지 않다. Official support profile은 각 surface의 계획, 검토중 구현, 지원, 비지원과 이월 상태를 구분한다. SUPPORTED_FOR_V1은 Owner-approved membership, required product path, target-bound evidence와 required independent verdict 없이 선언하지 않는다. 공식 ingress에서 비지원 표면은 가능한 의미 경계에서 deterministic explicit unsupported failure로 닫으며 silent fallback이나 host-dependent semantic inference를 허용하지 않는다. Exact status enum과 spelling, membership row와 required closure gate는 C2 product registry가 소유한다.

## RD-496 — V1-PRODUCT-U3 distribution·public usability closure

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / DISTRIBUTION-AND-PUBLIC-USABILITY-GATE-CLOSED / VERIFIED-USER-DOCUMENTATION-REQUIRED / EXACT-C2-TARGET-PACKAGING-OPEN / PRODUCT-AND-RELEASE-OPEN`

V1 distribution closure는 sealed public-source and provenance-consistent build, ReleaseArtifactIdentity와 integrity, target-profile clean install and first run, version identification, required offline workflow와 dependency declaration, license and NOTICE, SBOM 또는 동등한 component evidence, truthful supported-surface declaration과 verified Quickstart를 요구한다. Public user documentation은 release capability이며 설치, 지원범위, single-file and project workflow, 결정성, receipt and replay와 오류를 설명해야 한다. 모든 code example은 official supported product path의 fixed evidence에 결속한다. Windows x64 exact target profile, installer and package technology, physical document names와 tutorial inventory는 C2 product, release와 public-document registry가 소유한다.

## RD-497 — V1-PRODUCT-U4 closure dependency·Human gate

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / V1-CLOSURE-DEPENDENCY-CLOSED / PARTIAL-EVIDENCE-NONEXPANSION-CLOSED / OWNER-HUMAN-SMOKE-RELEASE-BLOCKER / PRODUCT-OPEN`

V1 release adjudication은 shared semantic and frontdoor authority, supported V1-Core ingress, Local Project custody, official Workbench, determinism, failure, receipt and replay, distribution closure와 Owner Human Smoke RC의 논리 의존 순서를 보존한다. 앞 단계의 partial evidence는 뒤 단계나 whole-product PASS로 확대하지 않는다. Owner Human Smoke는 automated test, pack과 independent review와 별도인 release blocker다. 최초 release는 external-user success count를 선행 blocker로 요구하지 않으며 그 값 0은 Human PASS나 release authorization을 대신하지 않는다. Actual capability membership, required gate, readiness, numeric maturity와 release authorization은 별도 C2 and Owner transition을 요구한다.

## RD-498 — ARROW-R2-U1 `<-` PlaceAssignment과 `->` ResultFlow source role 분리

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SOURCE-ROLE-AND-CANON-SEPARATION-CLOSED / RD-180-RD-205-ALIAS-CLAUSE-SUPERSEDED / PRODUCT-OPEN`

RD-180·RD-205의 `<-`와 `->` 동일 대입 alias 및 단일 `<-` canon 부분을 명시적으로 supersede한다. `자리 <- 값.`은 place-centered Binding/Rebinding이고 RD-370 callable ResultPlace 기록도 이 표면만 쓴다. `완성된 식/호출 -> writable place.`는 하나의 producer 결과를 하나의 writable place로 보내는 statement-level ResultFlow이며 source AST와 canon을 보존한다. 두 역할은 typed write machinery를 공유할 수 있으나 서로 alias가 아니고 자동 canonical rewrite하지 않는다. arbitrary chain/pipeline은 금지한다. SpecialBlock body `->`는 RD-368 profile grammar가 소유하고 ordinary ResultFlow와 분리하며, `-->` FunctionType/Signature와 `\|->` FunctionLiteral은 RD-407 의미를 유지한다. old alias source는 version-pinned migration-only이고 silent guessing과 permanent alias는 없다. exact AST/schema/wire/diagnostic, parser/runtime/CLI/WASM/Workbench parity와 executable oracle은 PRODUCT-OPEN이다.

## RD-499 — nominal product schema ownership

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / NOMINAL-SCHEMA-OWNERSHIP-CLOSED / STRUCTURAL-NOMINAL-SEPARATION-PRESERVED / EXACT-SURFACE-OPEN / PRODUCT-OPEN`

`묶음씨/묶음값` remains the structural-record family. A nominal product is an existing `이름씨` DefinitionIdentity bound to its product field schema; no new nominal-record core identity family is created. Equal field schemas do not make distinct nominal DefinitionIdentity values equal, and a structural record value does not implicitly convert to a nominal value. Exact declaration, constructor, AST, schema and wire spelling remain OPEN.

## RD-500 — explicit nominal construction owner evidence

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / NOMINAL-CONSTRUCTION-OWNER-CLOSED / NO-HIDDEN-INFERENCE / EXACT-SURFACE-OPEN / PRODUCT-OPEN`

Nominal construction source must contain direct authored evidence of the nominal owner identity. Ambient expected type, return or target context, structural-shape search, import/source/registry order, global candidate search, hidden environment lookup and AI inference have no owner-selection authority. StructuralRecordConstruction and NominalConstruction are distinct typed semantic categories. Exact constructor spelling and physical AST/IR names remain OPEN.

## RD-501 — definition-owned product field default

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / PRODUCT-FIELD-DEFAULT-SEMANTICS-CLOSED / FAILURE-CONTRACT-CLOSED / EXACT-CANON-WIRE-OPEN / PRODUCT-OPEN`

A product field default is a definition-owned typed default and is not a callable-pin default. Required omission, defaulted omission and optional absence are distinct. Missing required, duplicate and unknown fields fail closed; last-write-wins, ambient or stdlib/provider hidden defaults and AI completion are forbidden. A closed branch is owner-qualified as `색.$검정`; `$검정` is admitted only when the expected owner is unique, and `#검정` is not a branch spelling. Default canon, expression identity, serialization, diagnostics and wire remain OPEN.

## RD-502 — MALMUNUI-U1 stable PhrasePatternIdentity·SlotIdentity와 sealed catalog context

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / PHRASE-PATTERN-IDENTITY-CLOSED / EXPLICIT-CATALOG-CONTEXT-CLOSED / AMBIGUITY-FAIL-CLOSED / EXACT-PHYSICAL-OPEN / PRODUCT-OPEN`

PhrasePatternIdentity와 그 typed SlotIdentity는 surface literal, source 위치·순서, formatter 배치와 presentation 철자와 분리된 안정 identity다. Resolution은 실행 또는 프로젝트가 명시적으로 봉인한 finite catalog set과 profile identity만 사용한다. hidden global catalog, install·import·source order, ranking, capability 또는 AI confidence는 선택 권위가 아니다. 동일 입력이 0개 후보이면 NO_MATCH, 2개 이상이면 AMBIGUOUS로 실행 0이며 exact physical ID, catalog schema, wire와 제품 구현은 OPEN이다.

## RD-503 — MALMUNUI-U2 RD-477 exact-scalar와 finite registered phrase morphology

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / RD-477-PRESERVED / EXACT-SCALAR-FIREWALL-CLOSED / FINITE-REGISTERED-MORPHOLOGY-CLOSED / PRODUCT-OPEN`

Resolver는 decoded 글 literal의 Unicode scalar sequence에 automatic NFC, NFKC, trim, whitespace collapse, punctuation removal 또는 fuzzy normalization을 적용하지 않는다. 조사·어미·생략·띄어쓰기와 punctuation 차이는 pattern 또는 slot morphology role에 결속된 finite registered variant일 때만 같은 PhrasePatternIdentity 후보를 만들 수 있고 universal language equality나 productive guessing을 만들지 않는다. 도구 suggestion은 비권위이며 수정된 입력은 authoritative resolver에 새로 들어간다. Exact authoring syntax, registry storage, diagnostic와 제품 parity는 OPEN이다.

## RD-504 — MALMUNUI-U3 역말무늬 L0~L3 deterministic presentation projection

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / REVERSE-PRESENTATION-LEVELS-CLOSED / SEMANTIC-INVERSE-0 / IDENTITY-AUTHORITY-0 / EXACT-SURFACE-OPEN / PRODUCT-OPEN`

역말무늬는 stable ActionIdentity를 profile별 human presentation으로 투영하며 provider callable 이름, 실행의 역함수 또는 semantic identity authority가 아니다. L0는 표현 없음 허용, L1은 preferred presentation만, L2는 같은 sealed context에서 원 identity가 candidate set에 포함됨, L3는 같은 catalog/profile에서 presentation을 다시 resolve했을 때 정확히 하나의 원 ActionIdentity가 됨을 뜻한다. L1과 L2는 L3 round trip을 주장하지 않는다. Exact public spelling, profile schema, formatter와 제품 구현은 OPEN이다.

## RD-505 — MALMUNUI-U4 stable ActionIdentity·exact-one named provider·typed Slot bridge

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / ACTION-IDENTITY-PROVIDER-SEPARATION-CLOSED / EXACT-ONE-PROVIDER-CLOSED / TYPED-SLOT-PARAMPIN-BRIDGE-CLOSED / EVENT-BRANCH-OPEN / PRODUCT-OPEN`

PhrasePatternIdentity, ActionIdentity, ActionProviderIdentity와 CallableDefinitionIdentity는 서로 다른 identity family다. R1의 한 ActionIdentity에는 stable named 셈씨 또는 named 움직씨 active provider가 정확히 하나이며 provider kind를 보존한다. anonymous, hidden, temporary AI-generated, order-selected 또는 0·2개 이상 provider는 fail closed한다. SlotIdentity는 explicit ParamPinIdentity에 type-checked binding되고 missing, duplicate, unknown 또는 type mismatch가 있으면 실행 0이다. 말무늬는 capability, permission, resource, approval 또는 world-mutation authority를 만들지 않는다. EventIdentity/EventProvider bridge, multi-provider dispatch, exact receipt/schema/wire와 제품 구현은 OPEN이다.

## RD-506 — MSAES-U1 MathIR·PropositionIR construction과 explicit evaluation

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / EXACT-CONSTRUCTION-SURFACE-CLOSED / EVALUATION-FIREWALL-CLOSED / PRODUCT-OPEN`

`수식{E}`는 E를 typed resolve하여 Typed MathIR value를 구성하며 실행·수치평가·정리·증명을 수행하지 않는다. `(변수=x)인 수식{E}`는 x occurrence에 distinguished free-variable identity를 결속한 MathIR을 구성하고 evaluation이나 substitution이 아니다. `세움{P}`는 well-typed PropositionIR을 구성한다. MathIR 또는 PropositionIR의 계산은 명시 `풀기` operation만 소유하며 구성 문맥이 evaluation을 암묵 요청하지 않는다. Exact physical AST·schema·wire·evaluator와 제품 구현은 OPEN이다.

## RD-507 — MSAES-U2 registered multisurface binder·capture·Definition-token 방화벽

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / BINDER-IDENTITY-SURFACE-CLOSED / CAPTURE-FIREWALL-CLOSED / DEFINITION-TOKEN-FIREWALL-CLOSED / PRODUCT-OPEN`

수학 표면은 RD-317~325의 declaration-multisurface만 재사용하며 별도 notation language나 operator macro system이 아니다. Placeholder는 string substitution이 아니라 declared ParamPinIdentity 또는 LocalBinderIdentity를 참조하고 양화·합·적분·집합제시 등 nested binder는 fresh BinderIdentity와 lexical scope를 소유한다. 같은 typed binder identity의 Korean·symbolic 표면을 함께 등록할 수 있지만 alpha-renaming과 substitution은 spelling이 아닌 identity/scope로 판정하고 capture 가능성은 fail closed한다. Active `:=` token은 registered surface pattern 안에서 금지하며 Definition binder owner를 유지한다. Quoted data 안의 문자 `:=`는 active token이 아니다.

## RD-508 — MSAES-U3 official HoTT Path·PropTrunc symbolic surface

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / HOTT-SYMBOLIC-SURFACE-CLOSED / PATH-PROOF-FIREWALL-PRESERVED / DEMAND-GATED / PRODUCT-OPEN`

Typed Path context에서 `x =p= y`는 Path type surface이고 paired delimiter `\|:A:\|`는 PropTrunc(A) surface다. `p·q`는 endpoint가 맞는 p 다음 q의 Path composition이며 endpoint mismatch나 ordinary multiplication과의 복수 해소는 fail closed한다. 이는 RD-342·343·375의 research/pressure-test status를 해당 세 표면에 한해 supersede하지만 Proof R1 `=:=`, 일반 equality, product 또는 norm 표면으로 재해석하지 않는다. HOTT-1/2 provider·checker·assumption과 RD-345 demand gate, kernel/product OPEN을 유지한다.

## RD-509 — MSAES-U4 callable forward composition과 traditional projection

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / COMPOSITION-SURFACE-CLOSED / DIRECTION-CLOSED / PRODUCT-OPEN`

`f 잇고 g 잇기`와 `f⨾g`는 먼저 f, 다음 g를 적용하는 하나의 forward CompositionIdentity surface다. 전통 표기 `g∘f`는 같은 typed composite의 presentation projection이며 별도 의미나 반대 traversal authority가 아니다. Path `p·q`, Proof equality chain과 Higher Structure connector composition은 각 owner를 유지한다. Domain/codomain 또는 typed interface가 맞지 않거나 candidate가 복수이면 source order나 coercion 추측 없이 fail closed한다.

## RD-510 — MSAES-U5 Tensor Korean named-pin type·conversion surface

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / TENSOR-NAMED-PIN-SURFACE-CLOSED / RD-385-399-PRESERVED / PRODUCT-OPEN`

Static Tensor type surface는 `(원소씨=T, 모양=(d0,...,dn)) 텐서씨`이고 shape tuple은 RD-386의 ordered finite static axis sizes다. Nested collection conversion은 `(차림=xs, 원소씨=T, 모양=(d0,...,dn)) 텐서로`, reverse conversion은 `(텐서=A) 차림으로`다. 모든 이름은 stable named pin이고 positional fallback, ambient shape invention, automatic broadcasting, Vector/Matrix alias와 hidden host conversion을 금지한다. Rank-0, empty/ragged, failure, canon과 provider 의미는 RD-385~399를 그대로 따른다. Exact physical AST·registry wire와 제품 landing은 OPEN이다.

## RD-511 — MSAES-U6 StructureProfile ordinary value·bundle과 selector surface

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / PROFILE-SELECTOR-SURFACE-CLOSED / ORDINARY-VALUE-BUNDLE-CLOSED / PUBLIC-PROFILE-TYPE-0 / PRODUCT-OPEN`

StructureProfile descriptor는 ordinary DDN value 또는 bundle이며 public `짜임차림새`나 `범주차림새` type을 만들지 않는다. Profile-qualified authored structure는 `(차림새=P) 짜임`을 preferred surface로 사용한다. `설정`은 이 profile-selector pin scope에서만 같은 pin identity의 accepted input alias이고 machine canon은 `차림새`로 수렴하며 일반 configuration spelling이나 global alias가 아니다. bare `짜임 {}`의 AGE5 owner와 RD-484~490의 ProfileIdentity·validation·admission semantics를 바꾸지 않는다.

## RD-512 — MSAES-U7 Higher Structure connector·channel·operation exact surface

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / HIGHER-STRUCTURE-SURFACE-CLOSED / FOUR-CHANNEL-CLOSED / OPERATION-SEPARATION-CLOSED / PRODUCT-OPEN`

Registered StructureProfile grammar island에서 ordinary undecorated connector는 `A -- B`, decorated ordinary connector는 `A -<head>[incidence](payload){body}-> B`, higher-directed connector는 `p =<head>[incidence](payload){body}=> q`다. 네 channel은 head `< >`, incidence `[ ]`, typed payload `( )`, body `{ }`이고 같은 semantic item의 중복 channel 결속은 실패한다. Identity alternates는 `A -\|-> A`와 `f =\|=> f`, inverse alternate는 postfix `f↩`이며 self-loop는 identity가 아니다. `뒤집기` structural reversal과 `되돌이` algebraic inverse를 합치지 않는다. `α -[R]:[L]- β`는 pasting이고 `α -[R:L]- β`와 `α[R] 잇고 β[L] 잇기`는 composition이다.

## RD-513 — MSAES-U8 fixed precedence·grammar islands·shared semantic lowering

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / PARSING-ARCHITECTURE-CLOSED / RECOVERY-FAIL-CLOSED / SHARED-LOWERING-CLOSED / PHYSICAL-PARSER-OPEN / PRODUCT-OPEN`

Core token의 precedence·associativity와 longest-match는 versioned core grammar가 고정하며 registered surface/provider/profile은 이를 주입·변경하지 않는다. HoTT delimiter, binder, Tensor named-pin과 Higher Structure connector는 각각 typed grammar island에서만 활성화하고 nested ordinary DDN grammar로 복귀하는 경계를 명시한다. Recovery는 semantic candidate를 선택하거나 token을 삽입해 성공시키지 않고 deterministic diagnostic과 no-semantic-output으로 fail closed한다. Native, WASM과 Workbench는 같은 resolved typed AST와 lowering contract를 소비하며 독립 parser·string rewrite·host fallback 의미 권위를 만들지 않는다. Exact parser representation과 제품 착지는 OPEN이다.

## RD-514 — MSAES-U9 DefEq·quotient congruence/canon·decidability clarification

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / KERNEL-INTERNAL-DEFEQ-PRESERVED / QUOTIENT-OBLIGATION-CLOSED / DECIDABILITY-MATERIALIZATION-SEPARATED / PRODUCT-OPEN`

DefEq는 RD-417 KernelProfile 내부 checker computation이며 public equality·rewrite·notation surface가 아니다. Quotient 위 n항 operation은 모든 quotient argument position에 대한 congruence/respectfulness evidence를 요구하고 semantic identity, state 또는 replay에 참여하려면 representative 선택과 무관한 canonical representation/encoding evidence를 별도로 요구한다. Decidability evidence는 membership/equality 판단 가능성을 뜻할 뿐 finite enumeration, 전수 materialization, canonical representative 또는 bounded storage를 자동 제공하지 않는다. Missing obligation이나 evidence는 hidden assumption 없이 fail closed한다.

## RD-515 — MSAES-U10 StructureIR ABI 방화벽과 logical provider-registry row

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / PERMANENT-PUBLIC-ABI-0 / LOGICAL-PROVIDER-ROW-CLOSED / UNIQUE-ADMISSION-AUTHORITY-PRESERVED / PRODUCT-OPEN`

StructureIR은 semantic carrier lineage이고 permanent public ABI가 아니다. Authoritative StructureProfile/provider registry의 logical row는 최소 stable provider identity, provider version, typed family, admitted profile identity/version range, signature, required laws/assumption closure, checker identity와 evidence identity, deterministic canon/projection contract와 status를 결속한다. Unknown, duplicate 또는 conflicting active row는 fail closed하고 MATH_SURFACE_COVERAGE_REGISTRY는 reference/coverage만 소유한다. Exact filename, physical schema, encoding, wire, ABI와 provider execution은 OPEN이다.

## RD-516 — NT-U1 terminology projection and identity-role separation

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / TERMINOLOGY-PROJECTION-CLOSED / PARSER-AUTHORITY-0 / PRODUCT-OPEN`

DDN terminology projection may version ddn-native, common-ko, expert-ko and technical-en explanation profiles without creating parser aliases, morphology registration, source keywords, types, AST kinds or wire renames. `이것임` is the ddn-native explanation of generic semantic identity, while qualified DefinitionIdentity, SubjectIdentity, ProviderIdentity, InvocationIdentity and BinderIdentity owners remain distinct and may be named with owner qualification. `제같음` remains Equality and differs from `이것임`. `받는자리` remains Parameter; `넘긴값` is an additive Argument-role term only.

## RD-517 — NT-U2 fixed slot, root state, default and state-value terminology boundary

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / FIXED-COMPONENTSLOT-TERM-CLOSED / DEFAULT-OWNER-SEPARATION-CLOSED / PRODUCT-OPEN`

The ddn-native canonical term for fixed ComponentSlot is `붙이`. Bare `바탕` remains the root/system-state source owner and is neither the ComponentSlot term nor a parser alias. Dynamic attachment terminology and attach/detach surface remain OPEN. `바탕값` may be owner-qualified but callable-pin default and definition-owned nominal product-field default remain separate semantic owners. `첫값` denotes the first committed value of a stateful slot or instance and `지닌값` the value held by current committed state; neither creates a new value identity or storage duty. `첫바탕` is not canonical, and `제다움`, `참이름`, `참모습` remain explanatory-only.

## RD-518 — VIEW-SHELF-U1 standard shelf, data and immutable view-edit contract

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SHELF-CATALOG-SUCCESSOR-CORRECTION / AUTHORED-SURFACE-CLOSED / PRODUCT-OPEN`

The V1 standard view shelf contains one result table plus line, timeline, coordinate, point-cloud, bar, histogram, box, heatmap, range-band, error-range and pie views, each returning public `보기값`. Result order preserves authored column order when explicit and otherwise public observation order plus canonical sample order without filling present, gap, missing or pruned states. `값=` and `계열들=` are XOR where applicable; XY data shares sample identity; histogram `칸수=` and `경계들=` are XOR; duplicate heatmap cells require `모으기=`; bounds are absolute and pie values are nonnegative with positive valid sum. `손질` is immutable typed field transformation and `갈라보기` creates one ordered-panel view and one bogae. Local UI/backend/cache state is outside semantic hash. Exact physical schema and product evidence remain OPEN.

## RD-519 — VIEW-SCENE-U2 plane2/space3 primitive, framing and camera roles

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SCENE-SEMANTIC-ROLE-CLOSED / EXACT-LEXICAL-AND-PHYSICAL-SIGNATURE-OPEN / PRODUCT-OPEN`

`scene+plane2` primitives are point, segment, polyline, circle, polygon and text; `scene+space3` primitives are point, segment, polyline, sphere, face and text. A point owns `자리`, a segment `처음` and `끝`, a polyline ordered `점들`, and a sphere `중심` and `반지름`; unlisted detailed roles remain OPEN rather than inferred. Plane2 composition owns `요소들`, `가로범위`, `세로범위` and forbids hidden auto-fit authority. Space3 composition owns `요소들`, `카메라`; camera owns `자리`, `바라봄`, `위`, `투영`, `시야각`, and missing authored camera cannot be repaired by hidden auto-camera. Camera/view motion is not world mutation. Exact lexical grammar, signatures, wire and renderer implementation remain OPEN.

## RD-520 — VIEW-BOGAE-U3 exact authored bogae relation roles

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / BOGAE-AUTHORING-SURFACE-CLOSED / NEW-CORE-TYPE-0 / PRODUCT-OPEN`

V1 authored roles are `표준보개.으뜸(보기=...)`, `겹(보기=..., 기준=..., 순서=...)`, `곁(보기=..., 자리=...)`, `돋(대상=..., 보기법=...)` and `짜임(관계들=...)`. `보기법` is a pure typed sealed callable projecting a semantic target to `보기값`, not a new Core view type, arbitrary renderer callback or hidden closure. Composition preserves exactly one 으뜸, zero or more 겹·곁·돋, a shared absolute scene cursor, and authored/local/world separation. Exact relation schema, runtime instance, UI layout and product parity remain OPEN.

## RD-522 — BOGAE-LX-U1 semantic validity versus renderer projection capability

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SEMANTIC-PROJECTION-LAYER-SEPARATION-CLOSED / EXACT-DIAGNOSTIC-OPEN / PRODUCT-OPEN`

Semantic validity of 보기값, 보개관계값 and 보개짜임값 is orthogonal to a renderer/backend's projection capability: a value may be VALID while projection is UNSUPPORTED. Renderer, backend or projection-profile differences do not change semantic identity or hash. Unsupported projection cannot become empty/no-data view, silent fallback or semantic success and must fail closed. Language Presentation Profile owns preferred source/presentation surface; View Projection/Rendering Profile owns renderer capability projection; local dock, pan, zoom, cursor, selection, window and cache state remains outside authored semantic composition. Exact Korean renderer-profile term, code, schema, wire and receipt fields remain product-owned and OPEN.

## RD-523 — O1 authored static-directive clean break

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SOURCE-SURFACE-CLOSED / MIGRATION-ONLY-PREDECESSOR / PRODUCT-OPEN`

RD-457's call-shaped static-directive phase remains. Canonical authored statements are `(pins) 얽음.`, `(항목=value) 쓰임.`, and `(항목=value) 드러냄.`. They are not ordinary calls or runtime dispatch. Old `얼개` and `항목들` spellings are history or versioned migration only and are not permanent parser aliases. Shared lexer, token and span authority remains mandatory; parser I/O, ad-hoc scanners, wildcard injection and source-order resolution remain forbidden.

## RD-524 — O2 authored work root and seven-field configuration

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / PROJECT-ROOT-CONTRACT-CLOSED / PRODUCT-OPEN`

`작품` is the common authored unit for programs, reusable gaji and library-only gaji. A formal root uses exactly the fields `가지`, `판본`, `모듬`, `내놓음`, `시작`, `실행계약`, `보개`. When present, `가지` uses only the scoped canonical name `이름범위.가지이름`; Registry short aliases are forbidden in declaration position and are available only in consumption-side `쓰임`. `가지` and `판본` occur together. Formal `모듬` is nonempty and every module declaration has at least `이름`, `머리`, `원문`. `내놓음` is gaji-only and defaults to the empty public-module set; `시작` is optional and resolves only to an executable DefinitionIdentity owned by a local module. Existing execution-contract and view defaults are inherited. A source closure has zero roots for standalone dependency-free execution, one root for a formal work, and fails closed at two or more. The formal root is configuration-only.

## RD-525 — O3 module source closure, namespace and DAG

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SOURCE-CLOSURE-AND-GRAPH-CLOSED / PRODUCT-OPEN`

A module source class is file, recursive directory or a nonempty list of those. Multi-source modules share one namespace. The frontend first seals the module source closure, then collects all top-level Definition skeletons, constructs the namespace and only then binds and type-checks bodies. File, declaration and filesystem order have no authority. Inter-module dependencies form a DAG and direct or transitive cycles fail closed. Directly designating a link or junction as a source fails; a link or junction encountered during recursive traversal is not followed. Thus target bytes never silently extend discovery. A source belonging to more than one module fails closed. Exact encountered-link diagnostics and physical storage remain product-owned.

## RD-526 — O4 Korean source, launcher and invocation-head surface

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / KOREAN-AUTHORED-INGRESS-CLOSED / PRODUCT-OPEN`

`.또니` is the Korean preferred source extension and `.ddn` is the toolchain-global technical extension. `또니` is the Korean preferred launcher and `ddn` its technical surface. `또니 실행 a.또니` makes `a.또니` the invocation's authored work-head source. Directory proximity, filename convention and Workbench hidden selection are not root authority. Physical spelling does not create a second semantic identity.

## RD-527 — O5 module-head directives, import, exposure and visibility axes

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / MODULE-HEAD-AUTHORITY-CLOSED / PRODUCT-OPEN`

Every formal module has one authored head path, which is location metadata rather than ModuleLogicalIdentity. `쓰임` and `드러냄` are module-owned and may each occur zero or one time only in that head; use in non-head sources fails closed. `쓰임` names a local module or an external exact gaji, exact version and module; omitted alias uses ModuleCanonicalName, an approved short gaji alias must bind immutable GajiId, and collision, range, latest, wildcard or silent rebind fail closed. `드러냄` explicitly names local or imported bindings, preserves exact provider and re-export provenance, and does not auto-publish imports. Module `드러냄` and gaji `내놓음` are independent visibility axes. Definition-selective import is deferred.

## RD-528 — O6 DDL `옮김` surface and ownership granularity

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / DDL-SURFACE-AND-GRANULARITY-CLOSED / PRODUCT-OPEN`

Technical semantic and artifact identity remains DDL; Korean preferred surface is `옮김`. `.옮김` and `.ddl` are language and technical spellings of the same owner, and competing artifacts for one owner fail closed. R1 ownership is gaji-level and module-level, never source-path identity. Gaji-level mapping covers public gaji and exported-module surfaces; module-level mapping covers public definitions, pins, role markers, callable surfaces and language-facing notation while private implementation coverage is not required. Notation is attached to the callable DefinitionIdentity and is not activated merely by `쓰임`; exact module-level activation spelling, DDL grammar, wire and escape remain OPEN.

## RD-529 — O7 LanguageProfile and CLI authority separation

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / PROFILE-AUTHORITY-SEPARATION-CLOSED / PRODUCT-OPEN`

LANGUAGE_PROFILE, LANGUAGE_SURFACE, CLI_LANGUAGE_SURFACE, PROFILE_ADMISSION and TOOLCHAIN_SOURCE_EXTENSION are distinct owners. LanguageProfile owns identity, version, status, native name, specific extension, grammar and morphology, formatter and diagnostic profile, but not CLI spelling. `.ddn` is global technical source, not profile-owned. `.또니` selects Korean; `.ddn` with one registered bootstrap selects that profile; headerless `.ddn` uses the inherited Korean default; a specific extension with a mismatching bootstrap fails closed. OS locale, UI locale, plugin presence and path have no selection authority. CommandIdentity and OptionIdentity are stable across localized CLI spellings, and CLI display language does not change source language.

## RD-530 — O8 Korean CLI surface, exposure and sealed execution

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / CLI-LANGUAGE-SURFACE-CLOSED / PRODUCT-OPEN`

Preferred Korean command surfaces bind stable CommandIdentity values: `RUN→실행`, `CHECK→검사`, `BUILD→짓기`, `TEST→시험`, `FORMAT→다듬기`, `REPLAY→재현`, `VERIFY→검증`, `DOCTOR→진단`, `HELP→도움`, `VERSION→판본`; `PROJECT.NEW→작품 새로`, `PROJECT.INFO→작품 보기`; `GAJI.RESOLVE→가지 풀기`, `GAJI.INFO→가지 보기`, `GAJI.PACK→가지 묶기`, `GAJI.PUBLISH→가지 올리기`, `GAJI.ADD→가지 더하기`, `GAJI.REMOVE→가지 빼기`, `GAJI.UPDATE→가지 갱신`; `LANGUAGE.LIST→말씨 목록`, `LANGUAGE.INFO→말씨 보기`, `LANGUAGE.CHECK→말씨 검사`; `DDL.INFO→옮김 보기`, `DDL.CHECK→옮김 검사`, `DDL.NEW→옮김 새로`; `RECEIPT.INFO→증표 보기`. Localized spellings never create new CommandIdentity values. `GAJI.ADD/REMOVE` and `DDL.NEW` are convenience tools, not semantic authority. Receipt identity and schema remain unchanged. Developer `PARSE` and `CANON` project as `개발 구문` and `개발 정본`. Exposure class is PUBLIC, ADVANCED, DEVELOPER or INTERNAL and does not change capability identity. Preferred global options are `--도움`, `--판본`, `--조용히`, `--자세히`, `--출력`, `--대상`, `--표시말씨`, `--망없이`, `--기계출력`; technical and localized forms share OptionIdentity. `--표시말씨` changes CLI presentation only. Machine schemas and semantic outputs are language-invariant. Sealed `실행` and `재현` perform no hidden resolve, update or network. Japanese surfaces require native review.

## RD-531 — RANGE-R1-U1 SelectionRange and ProgressionRange separation with half-open boundaries

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / RANGE-SEMANTICS-CLOSED / EXACT-AUTHORED-SURFACE-OPEN / PRODUCT-OPEN`

Target-owned SelectionRange and independent finite ProgressionRange are distinct identities with separate admission and failure semantics, though both use ordered half-open boundaries. SelectionRange is 0-based, requires `0 <= start <= end <= target_length`, admits empty start=end, rejects negative, reversed or out-of-range bounds without clipping, and follows target Selection failure; Tensor range remains static, nonnegative and extent-determining. ProgressionRange identity contains profile, exact admitted numeric domain/type, start, end and step; it is finite, discrete, exact, ordered and integer-compatible, defaults step to +1 only when admitted, rejects step 0, direction mismatch, approximate floating progression and overflow, and uses no hidden reversal. Exact DDN source must follow arguments-before-provider call-tail rules; prefix foreign-call notation is not canonical.

## RD-532 — V1 public unsupported diagnostic logical contract

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / PUBLIC-UNSUPPORTED-DIAGNOSTIC-BOUNDARY-CLOSED / EXACT-PRODUCT-CODE-OPEN / PRODUCT-OPEN`

Official ingress distinguishes MALFORMED, UNKNOWN, KNOWN_UNSUPPORTED and PROFILE_CAPABILITY_UNSUPPORTED semantic failure classes, with the last admitting reason-specific projection for a known identity rejected by target/profile capability. These are logical classes, not fixed product enums, numeric codes, wire spellings or localized messages. Failure is decided at the earliest authoritative BYTE_INGRESS, PARSE, TYPED_ADMISSION, CAPABILITY_ADMISSION or EXECUTION phase, with stable semantic identity, SourceEpoch, target/profile, span where available, reason and provenance sufficient for deterministic evidence. Unsupported input never silently falls back, host-infers or partially publishes; exact public diagnostic registry remains product-owned.

## RD-533 — zero-latency logical-tick reaction boundary

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / ZERO-LATENCY-REACTION-CLOSURE-CLOSED / PRODUCT-OPEN`

Finite zero-latency work settles inside one logical madi's deterministic reaction closure. Work emitted by reaction r first becomes eligible at r+1 and same-stack direct reentry is forbidden. Only dependency relations own causal partial order; source, registration, worker, completion and traversal order have no semantic authority. Stable identity canonizes enumeration, receipts and replay but grants no priority. Equal DAGs yield equal result, state, failure and receipt digest independent of traversal and worker count. Static zero-delay cycles fail admission; dynamic cycles or versioned reaction, causal-depth, WorkCost, pending-event or PatchCandidate ceiling violations abort the whole current-madi candidate without truncation or deferral. Enabled work produces PatchCandidate values, and conflict/failure prevents partial state or outbox publication.

## RD-534 — strict source ingress decoder and physical-versus-semantic source identity

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / STRICT-UTF8-INGRESS-CLOSED / RD-477-CLAUSE-CORRECTED / PRODUCT-OPEN`

Each SourceEpochIdentity binds exactly one DecoderProfileIdentity. V1 admits strict UTF-8 only and forbids alternate-encoding detection, UTF-8 BOM, BOM stripping, replacement insertion, locale/codepage fallback, editor metadata authority and content sniffing. Structurally invalid UTF-8 fails at byte ingress. Structural newline accepts LF, CRLF and EOF; CR belongs only to a CRLF pair and lone CR is not a bootstrap terminator. This successor corrects only RD-477's terminator clause while retaining raw-byte-zero and registered-language constraints. Raw 글무늬 keeps its separate CRLF/lone-CR body normalization owner. PhysicalSourceByteIdentity records exact bytes and provenance while decoded/semantic identity records authorized processing; LF and CRLF may differ physically yet share structural meaning, and receipts preserve both roles. Exact hashes, fields and wire remain product-owned.

## RD-535 — GOER-R1-U1 guard outcome, selection and evidence-only emission

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / GUARD-SELECTION-SEMANTICS-CLOSED / EXACT-DIAGNOSTIC-AND-WIRE-OPEN / PRODUCT-OPEN`

One settled InputSnapshot evaluates the complete canonical eligible TransitionIdentity set without source-order, completion-order or first-wins authority. GUARD_FALSE disables only that transition. GUARD_UNRESOLVED, GUARD_FAILURE and host/internal fault abort the entire selection set. Zero true guards is normal NO_TRANSITION; exactly one yields one atomic state-change-plus-action candidate; two or more fail closed without selection. Guard outcomes are receipt evidence and, when needed, diagnostic evidence only: they are not semantic events, actions, world mutations, patches, outbox entries or capability authority. Guard evaluation, failure and abort publish no partial state, action or outbox. Exact diagnostic code, registry schema and wire remain product-owned and OPEN.

## RD-536 — SOURCE-COORD-R1-U1 authoritative source coordinate and projection boundary

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SOURCE-COORDINATE-IDENTITY-CLOSED / PROJECTION-NONAUTHORITY-CLOSED / PHYSICAL-SCHEMA-OPEN / PRODUCT-OPEN`

An authoritative source coordinate is exactly `SourceSnapshotIdentity × ByteRange[start,end)`. ByteRange alone is never authoritative. The byte range is 0-based, half-open and bounded by the exact accepted UTF-8 SourceSnapshot; the same byte range in a different snapshot is a different coordinate. UTF-16 offsets, line/column, Unicode-scalar and grapheme-cluster coordinates are deterministic projections from that authoritative byte coordinate and are not independent semantic or source authority. Generated or synthetic nodes must use explicit absent or derived provenance and never fabricate a source coordinate. Exact physical schema, wire, LSP/browser adapter and cache representation remain product-owned and OPEN.

## RD-537 — DDL-ACTIVATION-R1-U1 module-binding exact language-surface activation

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / MODULE-BINDING-ACTIVATION-SURFACE-CLOSED / DDL-IDENTITY-PRESERVED / EXACT-PHYSICAL-SCHEMA-WIRE-OPEN / PRODUCT-OPEN`

Module-level DDL activation reuses the existing `쓰임` module-binding record and creates no new module identity. Its optional `옮김=(대상말씨=<LanguageProfileIdentity>, 대상판본=<exact LanguageProfileVersion>)` field has cardinality 0..1 and activates only the exported DDL surface of the current `쓰임` binding record and its bound ResolvedModuleIdentity for the consuming module. `쓰임` alone, re-export, filesystem/source/import order, latest/range/default profile and hidden current do not activate notation; multiple profiles or duplicate activation fail closed. A separate nonempty `(주인=..., 대상말씨=..., 대상판본=..., 항목=차림[...]) 옮김.` artifact defines owner-bound language-facing projections by public DefinitionIdentity and ParamPinIdentity under RD-317~323 typed multisurface rules and creates no callable, precedence, associativity, token macro or raw string/token/AST rewrite authority. Source LanguageProfile and target projection LanguageProfile are distinct. No new binding identity family is created. Exact AST, registry schema, wire, cache, diagnostics and product implementation remain OPEN.

## RD-538 — VIEW-SCENE-EXACT-R1-U1 primitive roles and exact scene construction surface

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SCENE-ROLE-AND-SOURCE-SURFACE-CLOSED / ORDERED-GEOMETRY-CLOSED / RENDERER-NONAUTHORITY / PRODUCT-OPEN`

The remaining primitive roles are circle `중심, 반지름`, polygon ordered `꼭짓점들`, text `자리, 내용`, and face ordered `꼭짓점들`; predecessor point, segment, polyline and sphere roles remain unchanged. Exact standard named-pin providers are `표준장면.원`, `표준장면.다각형`, `표준장면.글`, `표준장면.면`, `표준장면.평면`, `표준장면.카메라`, `표준장면.입체`, and `표준보기.장면`. Circle radius is finite positive; polygon and face require at least three distinct ordered vertices and valid simple nonzero geometry, with face planarity; text position and content are required. Plane owns `요소들, 가로범위, 세로범위` with finite increasing bounds and no hidden auto-fit. Camera owns `자리, 바라봄, 위, 투영, 시야각` with a valid basis and `0 < 시야각 < 180`; space3 owns `요소들, 카메라` and no hidden auto-camera. Scene-to-view owns `장면`. Missing, duplicate, unknown, wrong-type, unit/profile mismatch, nonfinite and ambiguous inputs fail closed; renderer repair and source order have no semantic authority. Exact physical AST, schema, wire and product landing remain OPEN.

## RD-539 — VIEW-PROJECTION-TERM-R1-U2 `보기투영법` and three-layer projection boundary

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / CANONICAL-KOREAN-TERM-CLOSED / SEMANTIC-RENDERER-FIREWALL-CLOSED / LOCAL-WORKSPACE-EXCLUDED / PRODUCT-OPEN`

The canonical Korean term for View Projection/Rendering Profile is `보기투영법`. Language Presentation Profile owns preferred source and presentation surface; 보기투영법 is a versioned projection contract from semantic 보기/보개 values to renderer/backend capability; Local Workspace State owns dock, pan, zoom, cursor, selection, window, cache and other nonauthored local state. These three layers are distinct. A renderer or 보기투영법 never owns or changes semantic view identity/hash. Camera pin `투영` is a camera projection value and is not 보기투영법. A semantically valid view may have projection capability UNSUPPORTED, which must not be lowered to empty/no-data view, silent fallback or semantic success. Exact renderer identity inventory, diagnostic code, schema, wire and product implementation remain OPEN.

## RD-540 — MALMUNUI-EVENT-R1-U1 typed EventCandidate provider bridge and publication boundary

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / MALMUNUI-EVENT-BRIDGE-CLOSED / EXACT-ONE-PROVIDER-SCOPED / DELIVERY-TRIGGER-PRESERVED / PRODUCT-OPEN`

Exact-one active stable named pure EventProvider admission applies only when a sealed MALMUNUI phrase/request constructs a typed EventCandidate; zero or two-plus providers fail closed without order selection. Existing typed RD-293/RD-297 publication may stage an explicit EventCandidate/delivery effect without invoking EventProvider. EventIdentity, AlertTypeIdentity, EventProviderIdentity, EventCandidateIdentity and DeliveryIdentity remain distinct; no hidden provider acquisition, identity merge or provider-order authority exists. Before publication commit, provider, capability, payload or admission failure produces zero state/world/outbox/resource publication and zero DeliveryIdentity. After a triggering DeliveryIdentity was published, downstream StateMachine guard or action abort preserves that triggering delivery as evidence but publishes no new state, action, outbox or secondary delivery. EventProvider output, receipt and diagnostic evidence do not themselves gain world-mutation, outbox, reaction, retry or capability authority. Future effectful providers and multi-provider dispatch require a versioned successor. Exact source, registry schema, wire, receipt fields and product implementation remain OPEN.

## RD-541 — RANGE-SURFACE-R1-U1 exact ProgressionRange authored multisurface

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / EXACT-RANGE-SURFACE-CLOSED / BRACKET-GRAMMAR-ISLAND / BARUNSU-ONLY-R1 / FUTURE-BIG-INTEGER-ADDITIVE / PRODUCT-OPEN`

ProgressionRange has exactly the named-pin surface `(시작=START, 끝=END) 범위 만들기` or `(시작=START, 끝=END, 간격=STEP) 범위 만들기`, with citation morphology `범위를 만들다`, and exactly two bracket surfaces `[START, END)` and `[START, STEP, END)`. Bracket forms are independent value-form grammar islands, not postfix Selection; `TARGET[...]` mixing fails closed. All four forms resolve to one ProgressionRange identity/hash, with Korean named-pin canon by default and bracket projection permitted only by an admitted presentation profile. The R1 operand domain is exactly `바른수`; step is signed and nonzero, omitted step is +1, equal start/end is empty, direction mismatch and overflow fail closed, and no hidden reversal occurs. Mixed domains and bare `큰수` fail; `큰바른수` is FUTURE_ADDITIVE and requires a separate successor.

## RD-542 — VC17-SURFACE-R1-U2 exact 22-row public provider surface package

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / VC17-EXACT-AUTHORED-SURFACE-CLOSED / CALL-TAIL-PRESERVED / PHRASE-PATTERN-TYPED / MEMBERSHIP-UNCHANGED / PRODUCT-OPEN`

The fixed VC17 authored-surface package covers exactly 22 current provider rows: 15 REQUIRED and 7 OPTIONAL, while the full partition remains 44 = 15 REQUIRED + 7 OPTIONAL + 1 DEFERRED + 21 NOT_VC17. Public citation form and registered invocation morphology are two surfaces of the same internal callable/provider identity. Arguments precede the provider and invocation remains call-tail/postpositive; prefix foreign-call notation is nonauthority. Multi-token surfaces such as `시작하는지 보다` are finite registered typed phrase patterns with exact slot/ParamPin binding, not whitespace-bearing identifiers or natural-language guessing. Zero or multiple pattern/provider resolutions fail closed; source/import/registration order, fuzzy NLP and AI confidence have no selection authority. `바꾸기` and `붙이기` typed child DefinitionIdentity rows refine source/catalog dispatch and do not create new membership rows.

## RD-543 — VC17-SEMANTICS-R1-U3 exact semantic and failure package

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / VC17-SEMANTIC-FAILURE-PACKAGE-CLOSED / SENTINEL-AND-HIDDEN-FALLBACK-0 / PARTIAL-PUBLISH-0 / PRODUCT-OPEN`

The fixed 22-row VC17 package preserves each provider's typed result, absence and failure contract. Typed absence is distinct from contract or provider failure. Host debug formatting, OS locale behavior, hidden Regex, Unicode normalization or case-folding, sentinel `-1`, last-write-wins, hidden mutation, recursive flattening, source/import-order ambiguity resolution and partial publication are forbidden unless a separately admitted contract explicitly owns them. `바꾸기` has distinct typed child definitions for text replacement and collection transform; `붙이기` has distinct typed child definitions for text join and sequence concatenation. Ambiguous overloads fail closed without coercion or fallback. This closure changes neither the 44-row membership partition nor SUPPORTED_FOR_V1. Exact diagnostic code, physical schema, wire and product implementation remain OPEN.

## RD-544 — JSM-U1 typed host binding identity

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / TYPED-HOST-BINDING-IDENTITY-CLOSED / IDENTITY-OWNERS-SEPARATED / EXISTING-INSTANCE-DEFERRED / EXACT-PHYSICAL-SURFACE-OPEN / PRODUCT-OPEN`

A Jjaim host binds a StateMachine through a versioned typed `StateMachineHostBindingIdentity` that seals the host Jjaim identity, existing component-slot identity, StateMachineSpec identity, explicit adapter contract and lifecycle contract. The binding is distinct from the bare JjaimBlock, StateMachineSpec, runtime StateMachineInstance, StructureIdentity and StructureProfile. Admission validates the complete binding before instance creation; missing, duplicate, ambiguous or order-selected binding fails closed. R1 creates a fresh StateMachineInstance from the bound spec at lifecycle start and does not bind an already-existing instance. The binding is a composition identity, not a new Core IR family or SpecialBlock. Exact hash encoding, physical record and authored host syntax remain OPEN; the existing typed component/slot surface is reused unless independently proven insufficient.

## RD-545 — JSM-U2 explicit port event output adapter

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / TYPED-ADAPTER-CONTRACT-CLOSED / EVENT-IDENTITY-EXACT / STRING-AND-ORDER-SELECTION-0 / PRODUCT-OPEN`

StateMachine host ingress and egress use an explicit versioned typed adapter contract. Each admitted host port is bound by exact PortIdentity and value type to an exact EventIdentity and typed payload shape; each admitted machine result or action output is bound by exact identity and type to an explicit host output candidate. Event selection by text, display name, source order, registration order, import order or hidden provider lookup is forbidden. Unknown, missing, duplicate, type-mismatched or multiply matching adapter bindings fail before machine execution and publish nothing. Existing typed EventCandidate/Delivery paths retain RD-540 authority and do not require a new EventProvider; an adapter creates no callable, event, provider, capability or delivery identity by string conversion. Adapter results remain candidates until the outer Jjaim commit.

## RD-546 — JSM-U3 lifecycle binding

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / HOST-LIFECYCLE-CONTRACT-CLOSED / FRESH-INSTANCE-START / HIDDEN-PAUSE-BUFFER-0 / EXACT-PHYSICAL-SURFACE-OPEN / PRODUCT-OPEN`

The host binding owns explicit bind/admit, start, pause, resume and stop lifecycle roles. Admission validates the spec, adapters, lifecycle contract, required capabilities and resource contract without creating observable machine state. Start creates one fresh instance at the spec-owned initial state. Pause admits and evaluates no new delivery and buffers no hidden input; an input arriving while paused must be rejected or handled by a separately explicit upstream contract. Resume continues the same instance from its last committed state and does not reinitialize or replay hidden input. Stop prevents further delivery and seals the final lifecycle evidence. Automatic restart, retry, timeout and rebinding of an already-existing instance are not implied. Exact lifecycle enum spelling, physical storage and authored syntax remain OPEN.

## RD-547 — JSM-U4 failure propagation after rollback boundary

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / MACHINE-LOCAL-ROLLBACK-CLOSED / OUTER-FAILURE-PROPAGATION-CLOSED / PARTIAL-PUBLISH-0 / TRIGGERING-DELIVERY-PRESERVED / PRODUCT-OPEN`

A StateMachine guard, action, validation, adapter or host/internal failure first closes the machine-local rollback boundary: no candidate machine state, action output, host output, outbox entry or resource effect from the failed selection is published. The typed failure and its cause/provenance then propagate to the containing Jjaim host and abort the current outer candidate; the host may not reinterpret it as GUARD_FALSE, NO_TRANSITION or empty output. A triggering DeliveryIdentity that was authoritatively published before the selection remains preserved evidence under RD-540, while the failed selection publishes no secondary delivery. Facts committed before the current outer candidate are not retroactively erased. Exact diagnostic code, failure schema and wire remain product-owned and OPEN.

## RD-548 — JSM-U5 Jjaim outer atomic commit

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / OUTER-ATOMIC-COMMIT-CLOSED / MACHINE-CANDIDATE-NONVISIBLE / PATCH-CANDIDATE-AUTHORITY-REUSED / CROSS-JJAIM-DEFERRED / PRODUCT-OPEN`

Within one Jjaim execution candidate, the selected StateMachine state-change-plus-action candidate, explicit adapter outputs and sibling component candidates remain staged until the Jjaim host performs deterministic merge/conflict resolution, validation and one outer atomic publish using the existing PatchCandidate/atomic-commit authority. Success publishes the complete accepted set once; any machine, adapter, sibling, capability, resource, conflict or invariant failure preserves the pre-candidate world and state hash and publishes none of the current candidate. A machine-local atomic candidate is not an independently visible commit inside the same host candidate. Cross-Jjaim distributed atomicity is not created.

## RD-549 — JSM-U6 direct RD-535 reuse

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / RD-535-DIRECT-REUSE / COMPETING-GUARD-MODEL-0 / FULL-SET-EVALUATION / EVIDENCE-ONLY-OUTCOME / PRODUCT-OPEN`

Every hosted StateMachine selection directly consumes RD-535 without a host-specific guard model: one settled InputSnapshot evaluates the complete canonical eligible TransitionIdentity set; GUARD_FALSE disables only that transition; GUARD_UNRESOLVED, GUARD_FAILURE and host/internal fault abort the entire selection set; zero true is normal NO_TRANSITION; exactly one yields one atomic state-change-plus-action candidate; two or more fail closed without selection. Source order, completion order, first-wins and semantic short-circuit have no authority. Guard outcomes are receipt/diagnostic evidence only and create no event, action, mutation, patch, outbox or capability authority. This unit adds no competing guard identity, enum or registry.

## RD-550 — JSM-U7 capability effect firewall

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / CAPABILITY-EFFECT-PREFLIGHT-CLOSED / GUARD-PURE / AUTHORITY-AMPLIFICATION-0 / STRUCTURE-PROFILE-TAKEOVER-0 / PRODUCT-OPEN`

A host binding must declare the capabilities, effects and resource contract required by its spec, adapters and action paths and pass preflight before execution. A spec, port, adapter, event, guard result, receipt, diagnostic, profile or machine output cannot acquire, infer, amplify or transfer capability authority. Guards remain pure. Pure adapters may only construct or validate typed candidates. Effectful work is represented by an admitted typed action/effect candidate and can become observable only through the U5 outer commit under the already-authorized capability context. Missing capability, effect mismatch, authority escalation or hidden host fallback fails closed with execution and partial publication zero. StructureProfile and its provider registry gain no StateMachine admission, effect or capability authority.

## RD-551 — JSM-U8 nesting flattening replay firewall

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / NESTED-IDENTITY-PRESERVATION-CLOSED / FLATTENING-NONAUTHORITY / REPLAY-IDENTITY-SEALED / ADVANCED-STATE-MACHINE-DEFERRED / PRODUCT-OPEN`

Nested host composition preserves every Jjaim host identity, StateMachineHostBindingIdentity, StateMachineSpec, StateMachineInstance, TransitionIdentity, InputSnapshot, adapter identity, lifecycle evidence, candidate boundary, receipt and causal parent. Storage or execution optimization may not flatten these logical boundaries, merge names, discard provenance or turn nested machine semantics into hierarchical/parallel/history StateMachine semantics. Replay binds the same sealed host/spec/instance-generation, adapter, lifecycle, capability/resource, InputSnapshot, delivery and parent-receipt identities; it performs no live provider fallback, string re-resolution, source-order reselection, hidden input recovery or capability acquisition. Cross-Jjaim distributed atomicity, hierarchy/parallel/history machines and timeout/retry semantics remain deferred.

## RD-552 — REL-PROFILE-U1 definition-owned relation ownership and LanguageProfile denominator

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SEMANTICS-CLOSED / LANGUAGEPROFILE-DENOMINATOR-100 / HYBRID-IDENTITY-SPLIT / NEW-STABLE-IDENTITY-0 / PRODUCT-OPEN`

19개 relation surface family는 global natural-language semantic-role dictionary가 아니다. 17개 pure definition-owned family는 successor LanguageProfile global translation denominator에서 제외하고, `~을/~를` 및 `~마다` 두 family는 각각 existing core grammar identity와 definition-owned pin projection identity로 분리하되 같은 철자를 허용한다. successor official denominator는 `98 core + 2 hybrid core-side = 100`이며 predecessor 117은 successor activation 전까지 predecessor/history evidence로 보존한다. definition-local public projection은 exact LanguageProfileVersion + existing DefinitionIdentity + ParamPinIdentity에 결속하며 compiler는 PURPOSE/RECIPIENT/SOURCE 같은 자연어 ontology나 새 stable identity를 만들지 않는다. 새 standalone identity registry는 만들지 않고 이 RD와 existing SSOT_LANG/SSOT_TERMS projection이 normative owner이며 exact physical registry schema/wire와 product materialization은 OPEN이다.

## RD-553 — REL-SURFACE-U2 SPACE_NATIVE exact boundary, call resolution and migration

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SEMANTICS-CLOSED / SPACE-NATIVE / EXACT-COMPONENT-SEQUENCE / NO-WINNER / NO-MIXING / MIGRATION-BOUNDED / PRODUCT-OPEN`

preferred multi-token relation source는 `~에 대해`, `~을 위해`, `~와 함께`이며 공백 포함 identifier가 아니라 exact token-component sequence다. component spelling은 exact scalar identity이고 component 사이는 U+0020 또는 U+0009 하나 이상을 허용하되 canonical output은 U+0020 한 칸이다. newline과 comment는 phrase 내부 HSPACE가 아니며 NFC/NFKC/semantic trim/synonym·fuzzy normalization은 금지한다. underscore unit은 preferred 또는 general accepted alias가 아니고 SPACE↔underscore 자동 변환을 금지하며 실재 legacy corpus에 의한 별도 admission과 BOTH policy는 각각 OPEN/DEFERRED다. resolver는 registry-admitted finite relation/head partition을 모두 보존하여 callable head, arity/default/optional, binding mode, explicit ParamPinIdentity, current TypeIdentity/equivalence와 authorized generic specialization, exact relation surface, unique argument↔pin mapping의 전체 제약을 적용한다. fully-resolved candidate 0/1/2+는 각각 NO_MATCH/SUCCESS/AMBIGUOUS이고 unresolved type obligation은 SUCCESS가 아니다. longest/shortest/source/declaration/registry/HashMap order, NLP/AI inference, invented OOP subtype-specificity는 winner authority가 아니며 RD-339의 explicit/relation-bound와 positional-only 암묵 혼합 금지를 계승한다. 새 SPACE_NATIVE lane의 0 candidate는 NO_MATCH이고 positional/expression fallback을 하지 않으며 기존 single-token/bare lane은 profile/version-bound compatibility로만 남고 merge_josa_into_expr-class fallback은 새 lane에 계승하지 않는다. same-head multimap, stable DefinitionIdentity materialization, lossless HSPACE/comment/source provenance, exact phrase AST/IR, partition parser, typed resolution, unique pin matching, canonical round-trip, tool/teul/WASM/browser parity는 PRODUCT_OPEN이다. corpus 31건은 PLANNED_NOT_ORACLE이고 physical pack과 executable oracle은 0이다.

## RD-554 — BOGAE-U1-A authored presentation composition and owner firewall

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / DIRECT-CLARIFICATION / SEMANTIC-DELTA-0 / AUTHORED-COMPOSITION-OWNER-CLOSED / PRODUCT-OPEN`

보개짜임값은 typed 보기값을 이름 있는 보개관계값으로 조립하고, authored 관계의 identity·순서·시간정책·초기 상태·참조를 보존하는 renderer-independent presentation composition이다. 보개짜임의 semantic 소유 범위는 authored composition이며 runtime instance, Local Workspace 상태, renderer/backend와 보기투영법은 포함하지 않는다. 보개는 renderer, 보기투영법, world, Jjaim, StructureProfile 또는 Host Capability와 동일하지 않으며 그 owner를 가져오거나 재정의하지 않는다.

## RD-555 — BOGAE-U1-B renderer projection semantic firewall

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / DIRECT-CLARIFICATION / SEMANTIC-DELTA-0 / SEMANTIC-VALIDITY-PROJECTION-INDEPENDENT / UNSUPPORTED-FAIL-CLOSED / PRODUCT-OPEN`

보기값·보개관계값·보개짜임값의 semantic 유효성은 renderer/backend projection의 성공이나 지원 여부를 전제로 하지 않는다. semantic 보기·보개 admission과 renderer projection capability admission은 서로 다른 단계다. semantic하게 유효한 값에 지원되는 projection이 없어도 그 값을 empty/no-data나 semantic success로 바꾸지 않으며 semantic identity 또는 validity를 재작성하지 않는다. `VALID semantic value + UNSUPPORTED projection`은 양립할 수 있다. Bogae instance 0 또는 Renderer instance 0 official mode는 만들지 않는다.

## RD-556 — BOGAE-U1-C interaction and Local Workspace firewall

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / DIRECT-CLARIFICATION / SEMANTIC-DELTA-0 / TYPED-INPUTSNAPSHOT-INGRESS / LOCAL-WORKSPACE-NONAUTHORITY / NEW-INTERACTION-IR-0 / PRODUCT-OPEN`

보개에서 시작되어 세계에 영향을 주는 상호작용은 `view event → typed input recipe → InputSnapshot → world execution → accepted commit → typed observation → new frame` 경계만 통과한다. 열기·닫기·이동·pan/zoom·cursor scrub·selection·focus처럼 세계 변경을 일으키지 않는 보기 상호작용은 Local Workspace 상태에 머물며 authored composition과 world state_hash를 바꾸지 않는다. raw host input/device family, GUI IR, ViewModel IR 또는 Interaction IR을 새로 만들지 않는다.

## RD-557 — BOGAE-U1-D determinism and renderer nonauthority

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / DIRECT-CLARIFICATION / SEMANTIC-DELTA-0 / SEALED-EXECUTION-DETERMINISM / AUTHORED-IDENTITY-ORDER-GEOMETRY / RENDERER-REPAIR-0 / PRODUCT-OPEN`

semantic result·committed state·failure·receipt·replay의 결정성은 실행계약과 봉인된 입력·artifact·identity가 소유하며 renderer projection은 그 결정성의 의미 권위를 갖지 않는다. 보기·보개의 semantic identity와 authored order, 장면 primitive의 typed semantic geometry는 canonical authored input과 current validation 규칙이 정하며 renderer/backend가 이를 추정·수선·재정렬하지 않는다. pixel identity, font raster identity, GPU identity, device-scale identity 또는 pixel certification을 새 Core guarantee로 만들지 않는다.

## RD-561 — external dependency node and boundary contract

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / SEMANTICS-CLOSED / PHYSICAL-SCHEMA-WIRE-OPEN / PRODUCT-OPEN`

DDN gaji and external-provider dependencies resolve in one dependency-resolution graph whose typed node kinds are DDN_GAJI and EXTERNAL_ARTIFACT. Node kind is a contract over existing logical dependency, provenance, artifact, provider, callable, graph and execution-contract identities, not a new ecosystem identity family. Python/native/WASM differences are qualified metadata and boundary/target/execution-contract differences. RD-231/453/454 and RD-220/470~474 remain direct predecessor owners; no predecessor meaning is redefined.

## RD-562 — external adapter boundary identity and sealed execution closure

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-APPROVED / ONE-NEW-LOGICAL-CONCEPT / CANONICAL-NAME-OPEN / PRODUCT-OPEN`

Exactly one external adapter/boundary contract identity slot connects the resolved artifact and provenance, provider/callable owners, typed input-output boundary, target/execution contract, capability/effect limits and sealed execution evidence. Its canonical name is OPEN; AdapterIdentity and ExternalCallIdentity are candidate labels only. It is distinct from ArtifactIdentity, ProviderIdentity, CallableDefinitionIdentity and InvocationIdentity. Sealed Run preserves resolve/update/network/ambient-discovery zero. Physical ddn.lock schema, wire, hash fields, serialization and exact diagnostics remain OPEN; no ecosystem-specific identity, new lock wire, product claim or executable oracle is created.

## RD-563 — Callable R2 child definition and scoped pin identity closure

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / EXACT-IDENTITY / PHYSICAL-REGISTRY-OPEN / PRODUCT-OPEN`

The public families 바꾸기 and 붙이기 retain their existing VC17 membership rows while four child DefinitionIdentity values and nine DefinitionIdentity-scoped ParamPinIdentity values are authoritative exactly as listed in the normative table below. Resolution uses the child DefinitionIdentity together with its scoped ParamPinIdentity; source, import or registry order, hidden coercion and return-only inference have no selection authority. The four children create no new callable family or membership row.

## RD-564 — Callable R2 preferred multilingual surfaces and review firewall

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / PRESENTATION-SURFACE / NATIVE-REVIEW-REQUIRED / PRODUCT-OPEN`

The 25 citation and invocation replacements in the normative table below are the preferred R2 surfaces. Their R1 predecessors are neither preferred nor permanent aliases. This owner binding closes the listed presentation choice but does not confer native Human PASS. The fixed 15-language research matrix remains 225 rows with projected 196 nonprovisional and 29 provisional rows; the 17-language successor denominator is 255 and has 30 missing rows. Operational laws for empty text, replacement overlap/order, transform cardinality, joining and concatenation remain with existing owners or OPEN; no law is inferred from this surface binding.

## RD-565 — 17-language strategic target and Quechua profile identities

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / STRATEGIC-TARGET / PHYSICAL-PROFILE-REGISTRY-OPEN / PRODUCT-OPEN`

The exact strategic localization target is the ordered 17-member list below. Quechua Chanka, Collao and Áncash have distinct registered source LanguageProfile identities LANGUAGE_PROFILE.QUECHUA.CHANKA.R1, LANGUAGE_PROFILE.QUECHUA.COLLAO.R1 and LANGUAGE_PROFILE.QUECHUA.ANCASH.R1. They share the same DDN semantic identities, Korean Semantic IR, runtime and canon; the split adds no runtime or semantic universe. Strategic target membership is not physical profile lifecycle official status, product landing, Human PASS, SUPPORTED_FOR_V1 or release readiness. Exact visible headers and the physical profile registry remain OPEN.

## RD-566 — generic Quechua ambiguity and shared lowering firewall

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / FAIL-CLOSED / NO-HIDDEN-ALIAS / PRODUCT-OPEN`

Generic qu, generic Quechua or an unqualified Quechua header is an ambiguous legacy denominator and cannot select Chanka, Collao or Áncash. Selection fails closed until a versioned exact profile identity is supplied. Migration requires explicit row or document provenance and is not an alias. No cross-variety automatic spelling conversion, alias, semantic/runtime duplication, source-order choice or product inference is introduced.

## RD-568 — DDL supported-language admission and fail-closed coverage

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / PHYSICAL-OPEN / PRODUCT-OPEN`

Each gaji explicitly declares the set of LanguageIdentity values it supports. For every declared language, every required public Definition surface and public ParamPin surface is complete; missing coverage is admission failure. Use from an unsupported source language fails. Korean, technical-name, locale, AI and other-language fallback are forbidden. Private non-exposed definitions are outside the coverage denominator. This decision does not change product, V1 support or public SPEC status.

## RD-569 — DDL TranslationEntry and stable scoped surface ownership

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / LOGICAL-MODEL-CLOSED / PHYSICAL-SCHEMA-OPEN / PRODUCT-OPEN`

A logical TranslationEntry has exactly the minimum fields OwnerIdentity, LanguageIdentity, PreferredSurface, ExplicitAliases, PublicParamPinSurfaces, OptionalRelationSurfaces, OptionalNotationSurfaces, Provenance and ReviewStatus. ExplicitAliases defaults to empty. Technical .ddl and Korean .옮김 are the same file kind, semantics and data model; competing definitions for one logical owner fail with no precedence. Authority binds to stable gaji or module owners and DefinitionIdentity, never source path. Relation surfaces are scoped to DefinitionIdentity times ParamPinIdentity, and notation requires explicit activation. Exact physical schema, wire and parser representation remain OPEN.

## RD-570 — explicit registered surface-set authority and morphology nonauthority

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / CLARIFICATION / LINGUISTIC-NONVALIDATION / PRODUCT-OPEN`

The form A{A1~...~An} denotes an explicit registered accepted-surface set, not exception-only inflection. A parser or compiler does not judge natural-language roots, grammaticality, linguistic regularity or derivational legitimacy. Admission checks only exact registration, typed context, unique resolution and ambiguity. Thus 먹{괚괘~히히} is admissible when those gates pass. A versioned morphology profile is an authoring or convenience expansion aid only; it has no semantic identity, veto, admission, hidden generation or fallback authority, and cannot create global token, precedence or grammar authority.

## RD-571 — whole public Definition translation projection

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / EXACT-SYNTAX-OPEN / PRODUCT-OPEN`

DDL translation is a target-language public declaration projection bound to one stable DefinitionIdentity. It preserves type parameters and requirements, the ParamPinIdentity set, semantic pin order, pin types and default semantics, result contract, effect and kind, while allowing localized occurrence order and registered definition, pin, marker, relation, exceptional-surface and notation presentation. It never copies or owns the implementation body. Semantic pin order is distinct from localized textual occurrence order. Exact language-header spelling, DDL punctuation and grammar, morphology descriptor, physical schema, wire and parser/canon/IDE parity remain OPEN. Unsupported-language fallback remains zero.

## RD-572 — required-input-only type-slot inference and unique call completion

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / PHYSICAL-SCHEMA-OPEN / PRODUCT-OPEN`

V1 type-slot inference begins only from actually-bound required execution input pins after finite callable and stable pin mapping. Optional pins, callable value defaults, result places, return or assignment targets, expected context, natural-language appearance, locale and AI are not inference authority. RD-202 current-item supply occurs only after explicit bindings and value-default processing identify exactly one structurally unsatisfied required pin; its supplied value and TypeIdentity may then be evidence. Remaining contiguous trailing type slots may use declaration-owned type defaults only after required-input inference. Guseil validates or finitely filters obtained bindings and never searches an ambient type or provider universe. RD-083 widening applies only to an already-concrete target and cannot invent a common supertype. Complete solutions are 0=UNRESOLVED, 1=success and 2+=AMBIGUOUS; only fully resolved candidates enter RD-553 final call resolution. Inferred arguments, result, evidence pin, current-item use, trailing defaults and chosen Guseil provider remain explainable in typed IR or specialization custody. Exact physical field, schema and wire remain OPEN, and no new explicit type-argument call surface is created.

## RD-573 — 마다 Korean-native iteration and plurality nonauthority

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / CLARIFICATION / NO-NEW-SYNTAX / PRODUCT-OPEN`

RD-201's 대상마다 body semantics, once-only iterable evaluation, same-name current-item binder and outer-binding restoration on every approved exit remain authoritative. Korean 마다 already carries distributive meaning, so forms such as 학생마다, 사람마다, 집마다 and 나무마다 are preferred explanatory surfaces. The presence or absence of 들 is not compiler authority for collection, plurality, iterability or binder identity. The compiler performs no automatic singularization or pluralization and does not use dictionaries, statistics or AI to invent a binder name. Existing explicit binder iteration such as (학생) 학생목록에 대해 remains available. No new (학생) 학생목록마다 surface, fixed current-item keyword, 제 alias or historical (x)마다 alias is introduced; RD-408 self vocabulary remains a separate owner.

## RD-574 — DDL PublicGajiSurface denominator closure

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / PRODUCT-OPEN`

For every declared supported LanguageIdentity, PublicGajiSurface is the identity-closed set of the gaji public name, every module exposed by 내놓음, every public Definition exposed by each module's 드러냄 and every DefinitionKind-required public child identity and surface. It includes calculation/action Definitions, immutable named-value Definitions, public nominal/product/sum types, OwnedBranch children, type parameters and requirements, current-permitted public model/spec Definitions, Definition-owned notation and explicit same-identity surfaces. Ordinary <- bindings, private/local Definitions, runtime SubjectIdentity or state, mutable world locations and actual reference/capability values are outside the denominator. Any missing required identity or surface fails closed; source or declaration order cannot repair closure mismatch.

## RD-575 — DDL contextual payload omission and public type-schema firewall

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / EXACT-SURFACE-CLOSED / PHYSICAL-AST-OPEN / PRODUCT-OPEN`

The exact omission sequence :={...}, containing exactly three ASCII U+002E dots, is valid only in a resolved .ddl or .옮김 Definition payload slot and has no omission meaning in ordinary .ddn. Its only logical forms are OmittedPayload(CALLABLE_BODY) and OmittedPayload(VALUE_INITIALIZER); the omitted node owns no body, initializer, literal, statement or payload hash. :={}, :={code}, :={…}, :={..} and :={....} fail. Public type, field, branch, branch-payload and default schema remain visible API contracts and cannot use OmittedPayload or hide the whole schema. Exact physical DDL AST/schema/wire remains OPEN.

## RD-576 — DDL-only multi-profile sections and duplicate-profile failure

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / HEADER-REGISTRY-OPEN / PRODUCT-OPEN`

Only .ddl and .옮김 may contain multiple top-level registered LanguageProfile sections, and only for distinct exact LanguageProfileIdentity values; ordinary .ddn remains one-language-per-file. Repeating the same profile is whole-document FAIL CLOSED, while unknown profiles and generic Quechua or ambiguous qu fail. Apparent headers in strings, comments, nested constructs or Definition bodies are not section boundaries and fail when exact classification is not unique. Section order is presentation only. Exact visible spellings for all 17 headers and the physical LanguageProfile registry remain OPEN and cannot be invented by this unit.

## RD-577 — DDL stable-identity reuse and ordering nonauthority

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / CLARIFICATION / NO-NEW-IDENTITY / PRODUCT-OPEN`

DDL public-surface closure reuses GajiId, ModuleLogicalIdentity and bound ResolvedModuleIdentity, DefinitionIdentity, ParamPinIdentity, TypeIdentity and NameTypeIdentity, and OwnedBranchIdentity. It creates no ValueIdentity, NotationIdentity, generic SymbolIdentity or source-path identity; value and notation surfaces remain children or presentations of their DefinitionIdentity owner. Stable identity closure is authoritative. Source order, section order, spelling equality, longest match, locale, import order and AI are not resolution authority. A formatter may preserve authored presentation order for reviewable diffs without making that order semantic. Collao and Áncash selected callable surfaces remain OPEN and cross-variety guessing is forbidden.

## RD-578 — Family-preserving generic Jjaim host contract

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / PRODUCT-OPEN`

A generic Jjaim typed-host contract standardizes only typed ingress and egress, lifecycle declaration, capability/resource/effect preflight, outer commit, replay and provenance obligations. Every semantic family retains its own versioned *HostBindingIdentity and family semantics. No universal ComponentIdentity, JjaimComponentBindingIdentity, HostBindingIdentity, Component IR, Core IR or SpecialBlock is created. JjaimBlock, StateMachineHostBindingIdentity, StateMachineSpec, StateMachineInstance, StructureIdentity and StructureProfile remain distinct, and RD-544~551 remains the first concrete StateMachine specialization.

## RD-579 — Explicit typed ingress and egress adapter contract

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / EXACT-SURFACE-OPEN / PRODUCT-OPEN`

A family host binding connects exact typed port, event, input and output identities and admits only a complete mapping with unique resolution. String or display-name matching, language spelling, locale, source/import/provider/registry order, AI inference, automatic translation, hidden provider discovery, hidden coercion, optional/default/result-context inference, ambient Guseil search, first-wins and invented common supertypes are non-authority. Unknown, missing, duplicate, type-mismatched, zero-match or multiple-match bindings fail closed with publication zero. RD-572 required-input-only inference is reused directly.

## RD-580 — Family-owned lifecycle contract

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / PHYSICAL-SCHEMA-OPEN / PRODUCT-OPEN`

Lifecycle is a shared host axis, but each family owns its exact roles and versioned transition semantics. The common host contract does not impose the StateMachine lifecycle enum or create a universal lifecycle enum. Unsupported roles fail explicitly. Hidden input buffering during pause, automatic restart, implicit retry and source-order lifecycle selection are forbidden. RD-546 remains the StateMachine specialization owner.

## RD-581 — Capability resource effect and Jjaim outer commit contract

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / PRODUCT-OPEN`

Resource and effect candidates are admitted only under explicit approved capability and family-owned adapter/execution contracts. Components, adapters, events, receipts, diagnostics, profiles and AI cannot acquire, infer, amplify or transfer capability. One Jjaim outer atomic commit owns visibility of admitted internal state, effect, output and outbox candidates. Failure, abort or conflict preserves the pre-candidate world and state and publishes no partial result. Cross-Jjaim distributed atomicity is not created.

## RD-582 — External request response two-reaction protocol

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / PRODUCT-OPEN`

External provider execution follows this exact logical boundary: construct and validate a typed RequestCandidate; bind provider, version, capability, correlation, provenance and receipt/outbox evidence; complete the Jjaim outer commit; then perform the external call; seal typed response evidence; deliver it as typed input to a later Jjaim reaction; adapt and admit it; and complete a later outer commit. Pre-commit external calls, same-reaction response success and treating irreversible external side effects as rollbackable transaction results are forbidden. RD-561~562 integrated dependency resolution and its single external adapter/boundary contract identity slot are reused without canonical-name closure or competing identity families.

## RD-583 — Replay nesting and provenance firewall

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / PRODUCT-OPEN`

Replay consumes only sealed request, response and provenance evidence. Missing or identity-mismatched response evidence fails deterministically; live provider recall is a new capability-authorized live execution with a new execution identity, never replay. Nested execution preserves family, host-binding, request, response, causal-parent, receipt and provenance identities. Physical flattening is storage or optimization only and cannot erase, infer or reconstruct semantic identities. Exact host syntax, physical AST/schema/wire, diagnostics and registry spelling remain OPEN; streaming, retry/backoff, dynamic discovery/plugins, shared instances, distributed transactions, agent memory/planning and universal result selection remain DEFERRED.

## RD-584 — ExecutionRelation Wave 1 success-only completion and ordering gates

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / PRODUCT-OPEN`

For Wave 1, complete(A) is the SUCCESS_COMPLETION_ONLY event after body calculation and definite-result confirmation; FAILED, ABORTED and CANCELLED are not complete(A), and complete precedes publish. The three existing relations remain distinct: SEQUENCE `~고` requires start(A) before start(B); AFTER_COMPLETION `~고서` requires successful complete(A) before start(B); AFTER_PUBLISH `~고 나서` requires successful publish(A) before start(B). If the required predecessor event does not occur, B admission is zero. Source, scheduler, worker, completion or physical timing order is non-authority. No new event identity, source syntax, scheduler identity or wall-clock authority is created.

## RD-585 — ExecutionRelation Wave 1 structured-group failure and committed-state boundary

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / PRODUCT-OPEN`

RD-259/RD-286 general structured execution-group all-or-nothing is the Wave 1 default and has no partial-success override. For SEQUENCE, once A and B start, A failure or cancellation cancels and joins remaining B, stops further admission, discards unpublished group Patch and staged outbox, and preserves pre-world. AFTER_COMPLETION permits `A complete -> B start -> A publish reject/failure`; the same cancel/join/discard boundary applies. AFTER_PUBLISH starts B only after successful A publish; later B failure cannot retroactively roll back A committed state, and correction requires explicit compensation. RD-286 separate relation family is not all twelve active ExecutionRelations; RD-284 SelectionRelation and exact override-family attribution remain FUTURE_WAVE. In Jjaim, RD-581 outer atomic commit remains the containing visibility owner and Wave 1 creates no independent child publish.

## RD-586 — ExecutionRelation Wave 1 result logical-time and identity firewall

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / PHYSICAL-REPRESENTATION-OPEN / PRODUCT-OPEN`

The Wave 1 core result baseline is stable member identity to MemberResult tree. Relation-specific implicit projection, completion-order arrays and bare nested-plan automatic execution or flattening are not added. NOT_STARTED and CANCELLED physical representation remains OPEN and no new runtime enum is created. Ordering reuses RD-285/RD-287 logical time `(madi, reaction)`; thread, worker, CPU/GPU, network arrival and completion order are non-authority, while global same-madi identity is FUTURE_WAVE. PlanIdentity, ExecutionInstanceIdentity, 일묶음씨/일엮음씨 and result-tree custody remain distinct. CW-001~020 stay byte-preserved PLANNED_NOT_ORACLE; CW-012/013 planned coverage includes the post-complete publish-reject path without new pack identity, slot, runner or executable oracle.

## RD-587 — V1 current-only active-tree and explicit-migration contract

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / C1 CORE AUTHORITY / PRODUCT-OPEN / PURGE-NOT-AUTHORIZED`

V1 active product and guidance selectors, and current Strict parser/runtime admission, use only CURRENT source grammar. Retired source surfaces are rejected as current input and are not silently aliased, canonicalized or inferred from source content. Active positive tests, current packs and walks, examples, formatter/LSP inputs and unversioned or current public guidance converge to current syntax or are excluded from active selectors; fixed or versioned historical evidence is not rewritten. DEFERRED semantic owners, identities, decisions and plans remain preserved but gain no V1 admission, membership, support, readiness or executable-oracle status. Historical migration, if provided, is owned only by an explicit separate tool or command receiving an exact version, SourceEpoch or profile; current parser/runtime fallback is forbidden. CURRENT, DEFERRED and REJECTED are admission dispositions, while HISTORY_ONLY and MIGRATION_ONLY are custody or tooling dispositions and do not merge semantic, product or evidence lifecycles. Versioned SSOT and SPEC, fixed review evidence, manifest, seal, builder, checker, backup and their protected dependencies remain immutable custody; Git history alone is not a replacement. UNKNOWN is not zero or automatically deletable, bulk replacement is forbidden, and each product cleanup PR requires an exact-path lease, dependency closure, backup and distinct-root restore with hash parity, product-behavior DoD, focused tests, mutation, independent review and separate Owner exact-path deletion approval before any purge.

## RD-588 — STDLIB V1 deterministic RNG exact profile

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / PRODUCT-OPEN`

The V1 deterministic simulation profile is `DDN-RNG-v1/ChaCha12`, not a cryptographic API claim. State contains ChaCha constants, eight little-endian seed words, a 64-bit block counter and 64-bit RandomStreamIdentity; output is sixteen little-endian u32 words after six double-rounds. Admitted seed forms are SeedBytes32 and SeedU64, with SeedU64 expanded by SHA-256 of the fixed domain string plus U64_LE. Public position is block counter plus word index, wrap is forbidden, and every operation requires an explicit positive word budget. Bool, bounded rejection sampling, choice, descending Fisher-Yates and exact rational grid projection have the fixed consumption rules. Hidden entropy, fallback, schedule or host ordering and hidden reservoirs are non-authority. V-RNG-01~05 are fixed discrimination vectors; product execution and executable oracle remain zero.

## RD-589 — STDLIB V1 canonical solver method matrix

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / PRODUCT-OPEN`

The V1 canonical matrix is exact Brent-Dekker for bracketed scalar roots, classical fixed-step RK4 for non-stiff ODE, composite Simpson with positive even partition for definite integration, and deterministic partial-pivot LU/Gaussian elimination for square linear systems. Evaluation, branch, accumulation and pivot tie order are fixed by the source-owner material. Automatic method selection, fallback, hidden endpoint adjustment and backend-dependent semantic results are forbidden. Unsupported stiff ODE and multivariate nonlinear systems fail explicitly. Failure, count, branch trace, termination and backend descriptor remain distinct receipt fields; physical API, registry and implementation remain OPEN.

## RD-590 — STDLIB solver Fixed64 numeric profile and authority vectors

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / PHYSICAL-PUBLIC-WIRE-OPEN / PRODUCT-OPEN`

`DDN-SOLVER-FIXED64-V1` version 1 is signed Q31.32 over a signed 64-bit two's-complement raw value with 32 fractional bits, ULP 2^-32 and canonical corpus scalar encoding as exactly eight little-endian bytes. Admission and every precision-loss boundary use checked roundTiesToEven. Addition and subtraction are checked raw operations; multiplication and division use exact widened intermediates before ties-to-even quantization. Division by zero, overflow and unrepresentable conversion keep distinct numeric-owner failures. Binary64, FMA, extended precision, reassociation, saturation and truncation are non-authority. Brent representable resolution is one raw ULP. The fixed solver corpus is BR-01~06, RK-01~02, SM-01~02 and LU-01~02, 12/12 source-owner vectors; this is conformance material, not a public wire claim.

## RD-592 — ordinary callable V1 disjoint overload admission

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / PHYSICAL-PRODUCT-OPEN`

Within one V1 ordinary-callable effective overload set, every declaration pair must be deterministically proven disjoint at admission. Proven overlap and unknown, unresolved or insufficient sealed evidence reject the overload set; UNKNOWN is neither disjoint nor deferred to call-site selection. Specificity, Guseil, provider, return/result or expected context, effect/capability, source/import/declaration/registry order, lexical proximity, priority, host state and AI are non-authority. Existing call-site exact-one 0/1/2+ remains a defensive invariant. Selection, StateMachine, Event, Provider and other family-owned resolvers are unchanged. Exact overlap algorithm, solver/index/cache, diagnostics, AST/schema/wire and product implementation remain OPEN. OCV1-01~15 are PLANNED_NOT_ORACLE; physical pack and executable oracle are zero.

## RD-593 — ExecutionIR R1 architecture and profile boundary

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / PHYSICAL-PRODUCT-OPEN`

ExecutionIR (`실행짜임`) is the typed internal representation after semantic resolution, not source syntax, raw AST, bytecode, backend opcode or a metaprogramming language. The R1 profile is a versioned closed set; unknown profile/op/terminator fails closed and extensions require a successor profile. Backend lowering and optimization are non-authority. No new source syntax, Core IR family or SpecialBlock is created.

## RD-594 — ExecutionIR R1 exact ExecOp profile

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / PHYSICAL-PRODUCT-OPEN`

The exact R1 ExecOp set is eight: 값놓기, 가져오기, 구성하기, 부르기, 고르기, 두기, 바꿈후보, 바깥일후보. Their fixed typed roles and current Definition/Provider/Invocation, Place/Selection/ResultPlace, PatchCandidate, RequestCandidate, capability and receipt owners are reused. Calls do not publish world state or perform host calls, 바꿈후보 only stages PatchOp-kind intent, and 바깥일후보 only stages typed request/outbox intent before outer commit.

## RD-595 — ExecutionIR R1 terminator block result and failure contract

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / PHYSICAL-PRODUCT-OPEN`

The exact R1 Terminator set is four: 이어가기, 가르기, 끝내기, 실패하기. A block is ExecOp* plus exactly one Terminator and is synchronous/non-suspending. 끝내기 is value-less normal completion after ResultPlace binding; typed 맺음.$안됨 remains a normal value, while 실패하기 denotes hard failure/abort. ExecutionRelation, Selection, ResultPlace/publish, external response and StateMachine guard owners remain distinct.

## RD-596 — ExecutionIR R1 provenance human projection and OPEN boundary

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / PHYSICAL-PRODUCT-OPEN`

Authoritative source coordinate is SourceSnapshotIdentity × ByteRange[start,end); ByteRange alone, file path, line/column and Unicode projections are non-authority. Korean predicate-final typed renderings are human presentation only. Physical enum/AST/IR/schema/wire/hash/diagnostic/profile registry/backend lowering, evaluator migration, multi-file provenance, pack/oracle, product and public projection remain OPEN. EA-001~015 are PLANNED_NOT_ORACLE; physical pack and executable oracle are zero.

## RD-597 — intentional absence of generic semantic heap ceiling

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / PHYSICAL-PRODUCT-OPEN`

Actual RAM/RSS, allocator overhead, refcount/COW/arena/pool representation and device/GPU memory are host/runtime safety facts, not DDN semantic resource identities, deterministic ceilings or canonical-result authority. Semantic resources are explicit implementation-independent logical quantities only. Domain logical-size metrics require an explicit successor and are not inferred from physical heap use. MEM-R1-G1 is INTENTIONAL_NO_GENERIC_SEMANTIC_HEAP_CEILING. No memory grade, generic heap meter or heap source syntax is created; host OOM/preflight/abort keeps its existing failure boundary.

## RD-598 — cooperative structured cancellation minimum

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / PRODUCT-ADAPTER-OPEN`

After cancellation is confirmed, new child admission is zero; started children receive cancellation requests and may observe them at registered cooperative safe points. Normal parent scope exit requires all children terminated or quiesced and joined. Failed/cancelled scopes discard unpublished PatchCandidate work and release no staged outbox while preserving pre-world and committed-predecessor boundaries. Physical preemption timing is non-authority, and resource ceiling/host abort identities are not merged with cancellation. Safe-point catalog, polling/executor/OS strategy, solver/provider ABI, WASM and ResultPlace W2 product behavior remain OPEN. CM-001~012 are PLANNED_NOT_ORACLE; physical pack and executable oracle are zero.

## RD-599 — shared SourceProgram and orthogonal source roles

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / C2-PRODUCT-OPEN`

Every admitted role-bearing file is one ordinary SourceProgram under the current lexer, parser, typed resolver, canon and runtime authority. StartRole, ModuleRole and GajiRole are independent optional roles and do not create separate languages, Program kinds, AST universes, semantic hashes, type authorities or runtimes. Discovery is set-valued before validation; duplicate, conflicting or unresolved role bindings fail closed, and source/filesystem/import/registry order has no winner authority. A role-oriented filename is only a frontdoor selector after separate C2 admission and is not semantic identity.

## RD-600 — Start Module Gaji and FormalWork role boundary

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / PHYSICAL-DISCOVERY-OPEN`

StartRole is directory/project default-entry authority only; explicit ordinary-file execution does not require it. ModuleRole owns current static module boundaries and implies neither entry nor GajiRole. GajiRole owns distribution/dependency identity and implies neither entry nor module export. All eight role sets are semantically admissible, including GajiRole-only only with current formal identity and resolver/lock qualification; metadata-free publication and replacement of gaji.toml are forbidden. FormalWorkRole is optional at source closure, while a present 작품 preserves RD-524 seven-field validation, exactly-one valid root, configuration-only status and non-subsumption of the three roles. Korean exact role filenames remain C2 candidates; non-Korean spellings remain OPEN.

## RD-601 — named-pin shell reuse for nominal construction

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / PHYSICAL-AST-SCHEMA-WIRE-OPEN`

Product/configuration construction may reuse the ordinary named-pin lexical shell, with direct authored nominal-owner evidence mandatory. Typed lowering preserves a NominalConstruction-equivalent category distinct from ordinary Call and may not use global structural search, expected/return/target context, source/registry order or AI as owner authority. No new raw 작품 block grammar is created. Exact AST/IR enum, parser node, schema, wire and diagnostic remain product choices. SR-001~020 are PLANNED_NOT_ORACLE; physical pack and executable oracle are zero.

## RD-602 — same-name public pin and local binder exact surface elision

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SEMANTIC-CLOSED / PHYSICAL-PRODUCT-OPEN`

For an ordinary definition pin, full `public=local:T~role=default` and short `name:T~role=default` are admitted. Before typed resolution, short expands exactly to `name=name:T~role=default`; when the public ParamPin spelling and local Binder spelling are exactly equal, full and short authored inputs canonicalize to short. ParamPinIdentity and local BinderIdentity remain distinct and canon preserves both identities, their binding, source spelling and byte range, type, relation role and optional/default contract. Duplicate public pins, duplicate or captured binders, unresolved obligations, zero or multiple candidates, fuzzy/case/Unicode approximate equality, order winners, AI/NLP/host inference and distinct-spelling shortening fail closed. Exact AST/schema/wire and product landing remain OPEN. SRE-001~016 are PLANNED_NOT_ORACLE; physical pack and executable oracle are zero.

## RD-603 — DDN-native IR terminology and family-preserving architecture

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / TERMINOLOGY-CONSISTENCY-CLOSED / UNIVERSAL-SHAPE-0 / PHYSICAL-PRODUCT-OPEN`

`짜임` is the DDN-native typed semantic representation family term. Typed MathIR/셈짜임, PropositionIR/세움짜임, StructureIR/구조짜임, PathIR/같음길짜임, RegexIR/정규무늬짜임 and ExecutionIR/실행짜임 each denote one logical owner within the pair, without creating a keyword, parser alias or AST/wire rename. Typed and stable-identity principles do not require a universal semantic or physical record; UniversalIR, UniversalNode, UniversalRecord, universal Component IR and Core IR are not introduced. Each family retains its semantic owner and natural physical representation. RD-593~596 and ExecutionIR R1 exact 8+4 remain literal predecessors.

## RD-604 — FormalWork executability separation and Definition-owned ExecutionIR

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / ARCHITECTURE-NORM-CLOSED / PHYSICAL-OPTIMIZER-OPEN / PRODUCT-OPEN`

FormalWork (`작품`) is an authored top-level root, not an executable-program identity. Library-only, definition-only and tool works are allowed. Executability is classified separately by existing StartRole and execution owners; exact classification schema remains OPEN. ExecutionIR semantic ownership is per executable Definition and no whole work must flatten into one mandatory ExecutionIR. Inlining, fusion, splitting, LTO and code-generation grouping are permitted physical optimizations only when observable semantics and Definition ownership are preserved.

## RD-605 — whole-program graph reuse lifecycle and self-host boundary

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / NO-SEMANTIC-WHOLEPROGRAMIR-DEFAULT / EXISTING-OWNER-REUSE / PRODUCT-OPEN`

No semantic WholeProgramIR or 프로그램짜임 identity is created until concrete necessity beyond existing module/definition dependency, call, per-definition CFG, ExecutionRelation and artifact/receipt dependency graphs is proved. Compiler-private versioned BuildGraph, LinkGraph and OptimizationGraph are allowed but are not language semantic IR. Lifecycle composes existing owners as source -> semantic resolution -> required semantic/execution jjaim -> execution -> result -> committed state -> artifact -> receipt -> replay/audit, reusing receipt/replay/provenance identities without duplicates. Future self-host compatibility is preserved, while V1 self-host implementation and schedule remain OPEN. IRWP-001~024 are PLANNED_NOT_ORACLE; physical pack and executable oracle are zero.

## RD-606 — stdlib RNG Package C canonical public logical surface

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / LOGICAL-SURFACE-CLOSED / PHYSICAL-PRODUCT-OPEN`

OD-SURFACE-1=C binds STDLIB-RNG-PUBLIC-SURFACE-U1: gaji 결정난수 owns six qualified callable logical identities 결정난수.만들기, 결정난수.범위정수, 결정난수.참거짓, 결정난수.하나뽑기, 결정난수.섞기 and 결정난수.눈금값, with 20 definition-scoped required ParamPinIdentity rows and the relevant NFC display spellings. The logical provider direction is stdlib.rng.ddn-rng-v1 and RD-588's DDN-RNG-v1/ChaCha12 profile is reused without redefinition. Per-call 프로필 pin, alias, adapter and fallback are zero; physical registry/type wire and product landing remain OPEN.

## RD-607 — stdlib Solver Package C canonical public logical surface

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / LOGICAL-SURFACE-CLOSED / PHYSICAL-PRODUCT-OPEN`

OD-SURFACE-1=C binds STDLIB-SOLVER-PUBLIC-SURFACE-U2: gaji 수치풀이 owns four qualified callable logical identities 수치풀이.브렌트구간근, 수치풀이.룬게쿠타4고정풀이, 수치풀이.심프슨적분 and 수치풀이.엘유연립풀이, with 21 definition-scoped required ParamPinIdentity rows and the relevant NFC display spellings. The logical provider direction is stdlib.solver.ddn-solver-fixed64-v1; RD-589~590's DDN-SOLVER-FIXED64-V1@1 method and Fixed64 contracts are reused without redefinition, and method identity is callable-owned. Per-call 방법/수프로필 pins, automatic solver selection, alias, adapter and fallback are zero; physical registry/type wire and product landing remain OPEN.

## RD-608 — official full-headless Bogae execution configuration

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / ADDITIVE-SEMANTIC-CLOSED / PHYSICAL-PRODUCT-OPEN`

A DDN execution configuration with BogaeRuntimeInstance=0 and RendererProjectionInstance=0 is officially valid: world execution and accepted atomic commit remain available, and the execution host may emit textual diagnostics and receipts. FULL_HEADLESS is distinct from CLI textual output, terminal presentation, graphical presentation and a no-op renderer; no ConsoleBogae, TerminalBogae or hidden view is created, renderer resources are not initialized, and an explicit Bogae projection request still requires valid composition, projection profile and capable-renderer evidence. RD-555 is preserved as immutable predecessor text and is superseded only by this additive decision.

## RD-609 — Native and WASM first-class semantic target architecture

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / ARCHITECTURE-CLOSED / PHYSICAL-TARGET-PRODUCT-OPEN`

Native and WASM are first-class semantic execution targets. Web-first rollout is permitted, while Web-only is not semantic architecture authority. Both targets reuse one source language and shared typed semantic resolution, lowering authority, runtime meaning, identity, failure, receipt and replay contracts. Physical adapters may differ, but JavaScript-only grammar or source rewrite, string or host fallback, separate target language/parser/IR identity, Android/iOS semantic families and pixel certification are not introduced. BIC-001~028 remain PLANNED_NOT_ORACLE; BOGAE-IMPL-01~12 remain planned product work; physical pack and executable oracle are zero.

## RD-610 — PGSM qualified sum branch construction and joint predecessor narrowing

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / PHYSICAL-SURFACE-CONTRACT-CLOSED / PRODUCT-OPEN`

PG-CLI-05 closed-sum declarations own exact OwnedBranchIdentity children. Branch values require explicit TypeIdentity.$branch and resolve by exact TypeIdentity times OwnedBranchIdentity; bare $branch fails even when the expected owner is unique. The immutable RD-438 and RD-501 expected-owner bare-branch clauses are jointly superseded only for PG-CLI-05 construction and typed sum-match patterns. Duplicate declarations, unknown branches, wrong owners and payload mismatch fail closed. No other RD-438 or RD-501 clause is changed.

## RD-611 — PGSM statement-only typed sum match, default and exhaustiveness

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / PHYSICAL-SURFACE-CONTRACT-CLOSED / EXPRESSION-MATCH-DEFERRED / PRODUCT-OPEN`

A typed sum match is statement-only and every branch pattern requires explicit TypeIdentity.$branch-if. Dispatch is exact typed identity; declaration order, match order, source/import/registry order, string or numeric tags, host and AI inference are nonauthority. At most one explicit trailing contextual otherwise arm is admitted; implicit, duplicate or non-trailing default fails. Without a default, every declared branch must be covered exactly once. Expression-valued match remains DEFERRED.

## RD-612 — PGSM diagnostic, atomic failure and UTF-8 span boundary

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / FAILURE-AND-PROVENANCE-CONTRACT-CLOSED / PHYSICAL-SCHEMA-OPEN / PRODUCT-OPEN`

PGSM requires stable typed diagnostic identities for duplicate declared branch, unknown branch, duplicate match branch, missing exhaustive branch without default, non-sum subject and branch/type mismatch. Exact diagnostic IDs and schema/wire remain OPEN. Any failed uncommitted match publishes state, stdout, state_hash and trace as zero. The branch-role dollar token preserves its exact UTF-8 byte range; dollar occurrences in strings and comments are non-semantic. Exact parser AST, lowering and runtime representation remain OPEN.

## RD-613 — PGSM exact authored grammar and exact-name subset payload-pattern binding

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / PHYSICAL-GRAMMAR-CLOSED / PRODUCT-OPEN`

PG-CLI-05 uses the fixed SumTypeDefinition, SumBranchDecl, BranchConstruction and statement-only TypedSumMatchStmt grammar. Ordinary TypeName, TypeRef, PayloadPinDecl, Expression, Identifier and Block are reused. For a qualified typed arm, absent payload parentheses bind none; present parentheses contain one or more exact-name identifiers and empty parentheses fail. Each identifier selects one branch-owned ParamPinIdentity and creates one same-spelled arm-local BinderIdentity; any subset may be selected and omitted pins are unbound. ParamPinIdentity and BinderIdentity remain distinct. Rename, positional or order mapping, fuzzy or normalization mapping, hidden inference and AI/NLP authority are forbidden; unknown, duplicate and capture states fail closed. Canon orders selected binders by stable ParamPinIdentity. The contiguous dollar branch token, named construction arguments, raw trailing unique default `아니면`, exhaustiveness and RD-610~612 family firewalls are preserved. Exact AST/schema/wire/diagnostic IDs/lowering/runtime/CLI-WASM product implementation remain OPEN.

## RD-614 — solver semantic authority and replaceable backend separation

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / ARCHITECTURE-NORM-CLOSED / PHYSICAL-PRODUCT-OPEN`

DDN owns typed solver problem, constraint, result, failure, versioned method and checker semantics. DDN may contain deterministic reference paths, bounded educational or validation solvers, proof/checkers and the minimum implementation required for portable fallback. Professional or high-performance solvers may be replaceable external backends separated from semantic authority; neither an internal reference implementation nor a particular external backend is canonical semantic authority. The historical all-solver-external requirement is superseded. External execution reuses RD-561~562 and existing execution-contract, conformance, receipt and replay owners.

## RD-617 — CLI human and machine output projection separation

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / CLI-OUTPUT-R1 / PHYSICAL-PRODUCT-OPEN`

The official CLI output projection has three explicit modes. Successful default human mode writes only ordered output_log_texts to stdout with one LF per entry, and zero entries produce empty stdout. Explicit --json writes exactly one structured summary without human prefix or suffix. Explicit --summary-json <path> preserves human stdout and atomically writes the same logical summary; both options may coexist. Output mode is a post-runtime projection and cannot alter execution meaning, accepted commit, output order, state/trace hashes, receipt or replay identity. Parse/type/runtime failure is nonzero and publishes zero uncommitted program stdout; exact structured-failure channel, schema, wire, diagnostic and filesystem technology remain OPEN. Ordinary full-headless execution without a requested Bogae projection is valid, while an explicit unsupported projection remains distinct. ddonilang and 또니랑 share one CLI identity and output contract; executable spelling, environment, order and hidden compatibility are nonauthority. Implicit JSON default compatibility is not preserved without separate evidence and Owner disposition.

## RD-618 — ProgressionRange exact authored multisurface and canonicalization

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / RANGE-SURFACE-R1.1 / PHYSICAL-PRODUCT-OPEN`

ProgressionRange retains RD-531 semantics and admits exactly three complete callable surface families: positional (START, END_EXCLUSIVE[, STEP]) 범위짓기, named-pin (시작=START, 끝앞=END_EXCLUSIVE[, 간격=STEP]) 범위짓기, and Korean-particle START부터 END_EXCLUSIVE전까지 [STEP씩] 범위짓기. Their three-argument semantic order is 시작, 끝앞, 간격; arbitrary argument-level mixing, zero or multiple pattern resolution, fuzzy matching, order authority and AI/NLP inference fail closed. All three callable families canon to named-pin 범위짓기. RD-541 historical text is preserved: bracket [START, END) and [START, STEP, END) remain admitted presentation profiles with their distinct START, STEP, END order, while 범위를 만들다/범위 만들기 and public pin 끝 are history-only with no permanent alias. ProgressionRange remains distinct from SelectionRange. ExecutionIR/실행짜임, authored 짜임 block/JjaimBlock, StructureIR/구조짜임 and MathIR/셈짜임 remain separate. Exact AST, schema, wire, diagnostic and parser/formatter/LSP/product landing remain OPEN.

## RD-619 — Semantic identity, Korean presentation and physical projection separation

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / IR-PROJ-01 / PRESENTATION-ONLY`

Stable semantic identity, Korean canonical presentation, current Rust projection, future DDN projection and wire, ABI or artifact projection are distinct authority layers. A canonical Korean presentation denotes the same logical owner but neither renames nor replaces its stable semantic identity. Source spelling, parser aliases, AST names, schema, wire and product projections do not follow automatically. No new semantic identity, language syntax, Core IR or SpecialBlock is created.

## RD-620 — Owner-approved Korean logical vocabulary presentations

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / IR-PROJ-02 / CANONICAL-PRESENTATION`

The following canonical Korean presentations bind to their existing logical owners exactly once: SourceEpoch=글판; SourceSnapshot=글고정본; SourceCoordinate=글자리; DefinitionIdentity=정의식별값; ParamPinIdentity=받는자리식별값; ResultPlace=결과자리; Plan=계획; ExecutionInstance=실행건; PatchCandidate=변경후보; AtomicCommit=원자반영; StateHash=상태해시; InputSnapshot=입력순간본; Receipt=증표; Capability=권한; AssetRef=자산참조; ProjectExecutionSnapshot=프로젝트실행고정본; Artifact=산출물. Capability remains distinct from general permission sets, roles, ACLs and allow or deny policy. Plan remains distinct from the immutable typed deferred PlanValue whose presentation is 일묶음씨. MathIR=셈짜임, PropositionIR=세움짜임, StructureIR=구조짜임, PathIR=같음길짜임, RegexIR=정규무늬짜임 and ExecutionIR=실행짜임 remain governed by their current owners, including RD-603, and are not duplicated or renamed.

## RD-621 — Self-host implementation semantic-equivalence firewall

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / IR-PROJ-03 / SELFHOST-SCHEDULE-OPEN`

Stage0 Rust and any future Stage1 DDN compiler are compared by semantic equivalence rather than physical AST, schema, binary or implementation identity. Existing owner, carrier, provenance and observable-semantics contracts remain authoritative. This does not require wholesale Rust renaming, freeze a stable IR wire, create a universal Core IR or semantic IR family, make self-hosting V1 REQUIRED, or close any self-host milestone or schedule. Exact implementation, physical projection and validation evidence remain product-owned and OPEN.

## RD-623 — V1 prebuilt-only external artifact policy

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / EIO-R1-U2 / SOURCE-BUILD-DEFERRED`

V1 external interop verifies and consumes exact prebuilt artifacts, provenance, typed interface and target or execution contract. A V1 sealed Run performs source resolve, source build, update, network access and ambient discovery zero times. Source build in V1 is DEFERRED; any future source-build path prefers an external sealed Build Provider that produces an exact artifact for DDN verification and consumption, without creating ecosystem-specific Core dependency identities or execution models.

## RD-624 — 바깥이음 single boundary contract slot

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / EIO-R1-U3 / SINGLE-IDENTITY-SLOT`

RD-562's exactly one external adapter or boundary contract identity slot has canonical Korean presentation 바깥이음, technical semantic description ExternalBoundaryContractIdentity and implementation-visible physical field boundary_contract_id. The slot is distinct from ArtifactIdentity, ProviderIdentity, CallableDefinitionIdentity and InvocationIdentity. AdapterIdentity and ExternalCallIdentity are historical or research labels for this same slot and cannot coexist as additional semantic authorities. 바깥이음 is not a source keyword, SpecialBlock, runtime invocation record or simple code adapter.

## RD-625 — One versioned dependency lock and seal envelope

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / EIO-R1-U4 / PHYSICAL-PROFILE`

The one semantic dependency-resolution graph is projected through one versioned physical lock and seal envelope identified by schema ddn.dependency-lock.v1. Nodes use explicit tagged variants DDN_GAJI and EXTERNAL_ARTIFACT; an external artifact cannot masquerade as a gaji row and no separate authoritative external.lock graph exists. Required graph, resolution, conformance, node, edge and seal fields, canonical identity order, SHA-256 seal domain DDN-DEPENDENCY-LOCK-V1, duplicate or unknown required fields and unresolved references follow the approved physical proposal. Legacy lock forms are not silently relabeled or migrated; sealed Run migration or resolution is forbidden.

## RD-626 — Immutable external request and response evidence

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / EIO-R1-U5 / PHYSICAL-PROFILE`

ExternalRequestEvidence and ExternalResponseEvidence are separate immutable records sharing an ExternalEvidenceHeader and exact execution, provenance, boundary contract, provider, artifact and correlation binding. Response evidence binds the admitted request integrity identity exactly; mutable last-wins response update is forbidden. DDN-EXTERNAL-EVIDENCE-CANONICAL-JSON-R1 uses UTF-8 without BOM, one terminal LF, duplicate-key rejection, deterministic key and set-like array order, and distinct SHA-256 domains DDN-EXTERNAL-REQUEST-EVIDENCE-V1 and DDN-EXTERNAL-RESPONSE-EVIDENCE-V1. Missing, malformed, forged or mismatched evidence fails closed with publication zero. Replay consumes sealed request and response evidence only and performs live provider recall zero times.

## RD-628 — LanguageProfile canonical header and explicit alias policy

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / LANG-HEADER-ALIAS-U1 / LOGICAL-PROFILE`

Each existing LanguageProfileIdentity owns exactly one canonical header and one finite explicit alias set with default empty. Canonical and registered aliases exact-match to the same existing identity, while canon and formatter output only the canonical header. Canonical cardinality 0 or 2+, cross-profile surface duplication, unregistered or inferred/fuzzy aliases, locale or prefix guessing, source/import/registry-order winner and AI/NLP inference fail admission. No HeaderAliasIdentity, global alias resolver, alias-specific semantic or wire identity, or language-specific parser universe is created. Generic qu, Quechua and unqualified Quechua remain ambiguous fail-closed; Chanka, Collao and Áncash header sets are separate and cross-variety alias or Human PASS inheritance is zero. Exact spelling, lists, registry/schema/wire, parser/formatter/LSP, translation and product remain OPEN.

## RD-629 — Solver Wave 1 provider-neutral typed runtime bridge

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SOLVER-PHYS-R1-U1 / PHYSICAL-CONTRACT / PRODUCT-OPEN`

One provider-neutral runtime bridge consumes only the existing Solver ProviderIdentity, an exact CallableDefinitionIdentity, definition-scoped ParamPinIdentity typed bindings and existing invocation/provenance context after common typed resolution. Wave 1 admits exactly stdlib.solver.ddn-solver-fixed64-v1.rk4-fixed-step@1, stdlib.solver.ddn-solver-fixed64-v1.composite-simpson@1 and stdlib.solver.ddn-solver-fixed64-v1.partial-pivot-lu@1. Unknown, mismatched, missing, duplicate, 0-candidate or 2+-candidate identity/binding fails closed. Brent is excluded. New provider or callable identity, provider-selection syntax, global registry authority, permanent-key grammar, alias, display/order selection and fallback are zero; exact Rust carrier, symbol, module, dispatch table and registry serialization remain product-owned.

## RD-630 — Solver Wave 1 failure mapping and Native/WASM shared-path bridge

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SOLVER-PHYS-R1-U2 / PHYSICAL-CONTRACT / PRODUCT-OPEN`

The Wave 1 bridge preserves existing Result<SolverOutcome<Trajectory<Fixed64>>>, Result<SolverOutcome<Fixed64>> and Result<SolverOutcome<Vector<Fixed64>>> shapes and keeps solver.invalid_argument, solver.resource_ceiling_required, solver.resource_ceiling_exhausted, solver.evaluation_ceiling_exhausted, solver.iteration_ceiling_exhausted, solver.bracket_invalid, solver.singular_pivot and solver.unsupported_shape mutually distinct. Fixed64 checked overflow, division by zero, exact-to-Fixed64 conversion failure and unsupported NumericProfileIdentity/version reuse their existing numeric/profile failure owners. Native and WASM consume the same typed invocation, provider-neutral bridge, checked-Fixed64 kernel and mapping. Separate JS/WASM solver, host-float meaning, source/import/registry/environment order and silent fallback have zero authority. Exact diagnostic schema/transport, complete export wire, product implementation, public SPEC, pack/oracle, Solver closure, VC17/C2, V1, release and Human status remain OPEN or unchanged.

## RD-631 — P3 Workbench saved-snapshot persistence and parity physical contract

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / P3-WORKBENCH-PHYSICAL-U1 / PHYSICAL-CONTRACT / PRODUCT-OPEN`

P3-WB-O1=A, P3-WB-O2=A and P3-WB-O3=A bind one physical unit without a new semantic identity. Official ingress reuses ddn.virtual_project_source.v1 with strict UTF-8/no-BOM raw text, NFC project-relative slash paths, no empty/dot/dotdot/absolute/drive/URI segment, duplicate and ASCII-case-fold collision rejection, ascending normalized path order, shared project graph validation and exact-one saved official entry. Durable save uses ddn.workbench.project_snapshot_revision.v1 and existing ProjectIdentity, HistoryRevisionIdentity, SourceEpochIdentity, SourceSnapshotIdentity, ProjectSemanticConfigurationIdentity, ProjectExecutionSnapshotIdentity and DefinitionIdentity. The complete envelope binds expected parent, official entry, ddn.virtual_project_manifest.v1, raw-blob byte length/SHA-256, dependency/resource closure, canonical_manifest_sha256 and revision_integrity_sha256. One shared owner emits BOM-free UTF-8 canonical JSON with one LF, code-point-sorted keys, no insignificant whitespace, duplicate/unknown/non-integer/invalid-Unicode rejection, explicit nulls and defined array order; digests are over the corresponding canonical object with its digest field empty. Validation is strict decode/schema, path/role/cardinality, raw blob integrity, project/root/entry consistency, source identity/canon, dependency/resource closure, manifest digest, envelope digest, ExpectedHead equality, durable commit, then pointer advance. The pointer contains project_identity, history_revision_identity and revision_integrity_sha256 and may expose only OLD_COMPLETE or NEW_COMPLETE, never HALF or last-wins. Official Run revalidates the saved envelope/blobs and exact-one entry and passes only the saved SourceSnapshotIdentity set to the common frontdoor; unsaved buffer, recovery state, selected tab, focus, order and hidden entry are non-authoritative. Exact diagnostic mapping preserves E_PROJECT_SAVE_REQUIRED, E_VIRTUAL_PROJECT_JSON_INVALID, E_VIRTUAL_PROJECT_SCHEMA, E_VIRTUAL_PROJECT_KIND, E_PROJECT_JSON_INVALID, E_PROJECT_ROOT_MISSING, E_PROJECT_BLOCK_MISSING, E_PROJECT_ENTRY_MISSING, E_PROJECT_ENTRY_CONFLICT, E_PROJECT_REVISION_CONFLICT, E_PROJECT_OBJECT_MISSING, E_PROJECT_OBJECT_BYTE_LENGTH_MISMATCH, E_PROJECT_OBJECT_SHA_MISMATCH and E_PROJECT_REOPEN_CORRUPT plus existing shared leaf codes. Snapshot persistence contains no stdout, result/failure, diagnostic, state hash, trace, receipt, frame, view or result UI state; reopen restores saved content and sets result observations to NOT_PRODUCED, with no stale reuse. ddn.project_run_parity_record.v1 binds surface, existing project/revision/configuration/source/entry identities, execution snapshot, outcome, typed result/failure, diagnostic, stdout bytes, state/trace/receipt hashes and saved head before/after. PRESENT requires a value, ABSENT means authoritative optional absence, NOT_PRODUCED means failure produced no artifact; Native/WASM/Workbench compare every field except surface exactly. Failure preserves last saved snapshot and committed state and publishes no success state/stdout/hash/trace/receipt. Exact product paths, storage technology and UI remain OPEN; product, public SPEC, pack/oracle, P3/V1/release/Human closure remain unchanged.

## RD-632 — BOGAE-IMPL-02 shared host configuration, zero-instance observation and Native/WASM fixture admission

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / BOGAE-IMPL-02-PHYSICAL-U1 / PHYSICAL-CONTRACT / PRODUCT-OPEN`

BOGAE-IMPL-02-HOST-O1=A, O2=A, O3=A, O4=A and O5=A bind exactly one physical unit. The internal versioned ddn.bogae_host_configuration.v1 record contains target_adapter NATIVE or WASM, mode FULL_HEADLESS or EXPLICIT_PROJECTION, and projection_request status NOT_REQUESTED or PRESENT with null or one existing typed request. FULL_HEADLESS admits only NOT_REQUESTED/null and EXPLICIT_PROJECTION only PRESENT/non-null; unknown, missing, duplicate and cross-invalid values fail closed. Native and WASM normalize through the same pre-construction admission point before any Bogae runtime, projection, renderer or device allocation; target adapter is presentation-only and cannot alter program meaning, result or state hash. ddn.bogae_host_observation.v1 records adapter, mode, request status, nonnegative Bogae and renderer instance counts, projection disposition NOT_REQUESTED/PRODUCED/FAILED and typed failure ABSENT/PRESENT. Successful full-headless execution is exactly FULL_HEADLESS, NOT_REQUESTED, 0/0, disposition NOT_REQUESTED and failure ABSENT. NoopBogae, Option<BogaeMode>, run summary and E_PRIMARY_VIEW_ABSENT are not this authority. Invalid config and unavailable or unsupported explicit projection reuse E_BOGAE_UNSUPPORTED; multiple eligible View owners use E_BOGAE_SELECTION_REQUIRED; entry type, duplicate and profile mismatch retain E_BOGAE_ENTRY_TYPE, E_BOGAE_ENTRY_DUPLICATE and E_BOGAE_PROFILE_MISMATCH; other composition failures retain their existing stable owner. Non-Bogae failure families are never collapsed. Rejected pre-execution configuration publishes no world patch, program stdout, state hash, trace, renderer output or successful projection receipt. BIC-HEADLESS-01 Native and BIC-HEADLESS-02 WASM admit the same raw pack/age2_kernel_smoke_v1/input.ddn material at 63 bytes and SHA-256 2f7a11414b873a67e4aa1ca7650297a36c4888807070fe1978f15a46555cc8c4 and require the same accept/result/stdout/committed-state-hash and 0/0 observation. Exact product artifact/output/receipt bytes remain OPEN; BIH-001~016 are PLANNED_NOT_ORACLE, physical pack/oracle are 0, and product, public SPEC, V1, publication/release/Human status remain unchanged.

## RD-633 — Ordered control events and settled held state

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / CLI-TEXT-R1-U1 / PRODUCT-OPEN`

OI-CLI-TEXT-01=A binds Press, Repeat and Release as distinct ordered logical events, with same-madi order carried by sealed monotonic logical receive ordinal. held is the applicable madi's settled state, not an event alias. OS repeat is not synthesized as Press; missing release, focus loss and disconnect are not repaired by fake Release. Terminal polling, rendering FPS and wall clock have no world-madi authority.

## RD-634 — Committed text and IME local-host boundary

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / CLI-TEXT-R1-U2 / PRODUCT-OPEN`

OI-CLI-TEXT-02=C keeps IME preedit, candidate UI and composition cursor in local host/UI state. One host IME commit is one committed Unicode text transaction, and only that committed transaction is eligible for typed recipe to InputSnapshot world ingress. Preedit, last_key_name and raw scan code are not canonical text or tape authority.

## RD-635 — Exact Unicode scalars and explicit normalization

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / CLI-TEXT-R1-U3 / PRODUCT-OPEN`

OI-CLI-TEXT-03=C preserves the exact Unicode scalar sequence by default. Hidden NFC, NFD and NFKC normalization are forbidden. NFC requires explicit input-recipe or project-policy authority with provenance; NFKC requires separate explicit authority. Grapheme-aware editing does not replace scalar payload or provenance identity.

## RD-636 — Typed context ownership for edit and control actions

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / CLI-TEXT-R1-U4 / PRODUCT-OPEN`

OI-CLI-TEXT-04=C assigns Backspace, Delete and Enter roles to the active typed input context or recipe. Host focus, widget inference, source/import order and AI/NLP have no context-selection authority. Escape is world input and Ctrl+C is host stop; double admission is forbidden.

## RD-637 — Distinct committed paste transaction

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / CLI-TEXT-R1-U5 / PRODUCT-OPEN`

OI-CLI-TEXT-05=C preserves paste as a distinct committed transaction rather than synthetic key repeat. A finite ceiling is mandatory and excess rejects the whole transaction before world application. The exact number and byte/scalar/grapheme accounting unit remain product-owned and OPEN.

## RD-638 — Input queue integrity and fail-closed loss

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / CLI-TEXT-R1-U6 / PRODUCT-OPEN`

OI-CLI-TEXT-06=C forbids silent oldest/newest drop, merge and reorder on overflow, loss or integrity uncertainty. The boundary produces explicit input-integrity failure or session-abort evidence and cannot continue as normal success. Failed ingress publishes no partial world state, success stdout, complete-tape seal or replay success.

## RD-639 — Versioned event-oriented input tape successor

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / CLI-TEXT-R1-U7 / PRODUCT-OPEN`

OI-CLI-TEXT-07=B preserves Tape V1 as control-state/control-only compatibility evidence and requires a successor capable of profile identity, madi rate/index, same-madi ordinal, logical channel, Press/Repeat/Release, committed text, paste, recording/integrity policy and COMPLETE/ABORTED/INCOMPLETE termination/seal. Migration never invents missing V1 event order; silent dual read/write and fallback are forbidden.

## RD-640 — Secret and no-record input firewall

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / CLI-TEXT-R1-U8 / PRODUCT-OPEN`

OI-CLI-TEXT-08=C keeps raw password and secret text at the local/host capability boundary and outside world input and persistent replay tape. Only a derived typed authentication or validation result may enter the world. World-affecting no-record raw secret input cannot support an exact replay-equivalence claim.

## RD-641 — Requested execution-strategy law-permission bridge

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / JLPR-U1 / PRODUCT-OPEN`

An execution contract or execution plan explicitly requesting a semantics-preserving strategy that requires law privileges is permission-eligible only when exact admitted Profile/provider/checker evidence sealed to the requested operation, provider, profile and version satisfies every required privilege. Profile, provider, checker, law declaration and evidence do not select the strategy or own, acquire, amplify or transfer dispatch, effect, capability, resource or mutation authority. Missing privilege is DENY without weaker-strategy silent fallback; malformed, forged, unresolved or mismatched evidence fails closed. Available evidence and actually consumed permission evidence remain distinct and are projected through the existing execution receipt owner. Permission eligibility is not executable dispatch.

## RD-642 — ExecutionIR R1 official versioned execution-profile ingress physical contract

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / EXECUTIONIR-OFFICIAL-INGRESS-U1 / PRODUCT-OPEN`

The optional official execution profile is the exact typed pair DDN-EXECUTION-IR-R1 and unsigned version 1, owned by the shared runtime configuration and admission boundary. Selector absence preserves AST-direct. CLI uses exactly --execution-profile DDN-EXECUTION-IR-R1@1 and WASM uses execution_profile as null or the typed identity/version object; both project one shared identity and dispatch owner. Unknown, incomplete and duplicate/conflicting selectors fail as E_EXECUTION_PROFILE_UNKNOWN, E_EXECUTION_PROFILE_DATA_MISSING and E_EXECUTION_PROFILE_AMBIGUOUS. Existing ExecutionIR unsupported-node/operation failures remain typed, with AST fallback zero. Existing receipt provenance records selected_execution_profile; receipt identity distinguishes the profile while state/trace hash formulas do not change. The sealed parity and zero-publication failure boundaries are required, while physical product landing and protected status axes remain OPEN.

## RD-643 — SourceSnapshot provider entry and shared admission-identity custody physical contract

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-BOUND / SOURCE-SNAPSHOT-PROVIDER-ENTRY-R1_1-U1 / PRODUCT-OPEN`

Within the existing stdlib provider family, the canonical namespace `원문고정본` admits five exact callable identities and seven definition-scoped required/no-default ParamPinIdentity rows. Calls return invocation-bound current-entry snapshot, byte and range values with exact raw bytes, strict UTF-8 without BOM, 0-based half-open ranges and typed atomic failure. SourceSnapshotIdentity is unforgeable and is not a content hash. Identity allocation, replay custody and exact diagnostic schema or wire are private implementation contracts and are not public language semantics.

## RD-656 — VC06 Bytes Unicode and exact UTF-8 storage physical contract

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-GATED CANDIDATE / VC06-BYTES-UNICODE-PHYSICAL-R1_5-U1 / PHYSICAL-CONTRACT / PRODUCT-OPEN`

Bytes, String and List<number> are distinct and have no implicit conversion. Unicode-sensitive results use Unicode 17.0.0. `UTF8바이트열로` with required pin `글` encodes the exact scalar sequence as UTF-8 with no normalization or BOM insertion. `UTF8NFC글로` with required pin `바이트열` consumes the complete payload with strict UTF-8 and then applies explicit Unicode 17.0.0 NFC. Aliases, a third helper, fuzzy or prefix resolution, registry-order authority and namespace or receiver inference are forbidden. The private Bytes storage tag, frame, hash domains and replay encoding are not public wire commitments.

## RD-658 — ordinary and hook while authored-surface unification

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-GATED CANDIDATE / WHILE-SURFACE-UNIFICATION-R1-U1 / PRODUCT-OPEN`

The ordinary sequential while canonical authored surface is exactly `{ 조건 }인 동안 { ... }.` and the continuing-condition hook surface is exactly `(조건)인 동안 { ... }.`. The two meanings remain distinct. Ordinary `{ 조건 }인것 동안 { ... }.` and its colon variant are rejected; compatibility alias, deprecated alias, warning-only admission and silent canonical fallback are all absent. Unrelated `{ ... }인것` Bool-thunk meanings remain unchanged. Product AST names and corpus-migration mechanics are not public language syntax contracts.

## RD-682 — Basic Tensor element-type canonical identity physical owner R2

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-GATED CANDIDATE / BASIC-TENSOR-ELEMENT-TYPE-IDENTITY-CANONICAL-OWNER-PHYSICAL-R2-U1`

V1 Basic Tensor admits only canonical `셈수` as its element TypeIdentity. Display spelling, file path, source order and registry order are not identity authority. Tensor is not a nested-list alias and rank-0 Tensor is not a scalar. The physical TypeIdentity byte preimage, digest and issuer or verifier encoding are private custody and are not public wire commitments.

## RD-684 — VC17 REQUIRED15 operational laws and typed callable bridge R1.1

- projection: `NORMATIVE_SEMANTIC`
- source status: `OWNER-GATED CANDIDATE / VC17-REQUIRED15-OPERATIONAL-LAWS-AND-TYPED-CALLABLE-BRIDGE-R1_1-U1 / PRODUCT-OPEN`

The VC17 REQUIRED 15 surfaces keep their existing DefinitionIdentity and definition-scoped ParamPinIdentity membership. Empty-needle search, empty-delimiter failure, left-to-right non-overlapping replacement, transform input order, exactly-one output per input, first-failure whole-operation abort, and empty or singleton join and concat laws are normative. Collection transform accepts only a registered typed named PURE callable with no ambient or mutable capture, closure environment, string-name fallback or Lambda auto-promotion. The private callable-carrier profile and provider/version/definition/pin-set/request/result byte grammar are not public ABI or wire commitments.
