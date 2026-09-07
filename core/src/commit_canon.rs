//! Canonical DetJson and domain-separated BLAKE3 records for atomic commits.

use unicode_normalization::UnicodeNormalization;

use crate::atomic_commit::{
    analyze_patch_effects, CandidateDisposition, PatchCandidate, PatchCandidateStatus,
};
use crate::fixed64::Fixed64;
use crate::platform::{
    InputSnapshot, NetEvent, Patch, PatchOp, ResourceValue, SeulgiIntent, SeulgiPacket,
};
use crate::signals::{ArithmeticFaultKind, DiagEvent, ExprTrace, FaultContext, Signal, SourceSpan};

pub const PATCH_OPS_DOMAIN: &str = "ddn.patch_ops.v1";
pub const PATCH_CANDIDATE_DOMAIN: &str = "ddn.patch_candidate.v1";
pub const PATCH_CANDIDATE_SET_DOMAIN: &str = "ddn.patch_candidate_set.v1";
pub const EMISSION_BUFFER_DOMAIN: &str = "ddn.emission_buffer.v1";
pub const COMMIT_RECEIPT_DOMAIN: &str = "ddn.commit_receipt.v1";
pub const INPUT_SNAPSHOT_DOMAIN: &str = "ddn.input_snapshot.v1";
pub const EXECUTION_BINDING_DOMAIN: &str = "ddn.execution_binding.v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalCandidate {
    pub ops_text: String,
    pub ops_hash: String,
    pub candidate_text: String,
    pub candidate_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalCandidateSet {
    pub canonical_text: String,
    pub hash: String,
    pub ordered_candidate_ids: Vec<String>,
    pub candidates: Vec<CanonicalCandidate>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalCommitReceiptRecord {
    pub candidate_dispositions: Vec<(String, String)>,
    pub candidate_set_hash: String,
    pub commit_id: String,
    pub committed_state_hash: String,
    pub conflict_keys: Vec<String>,
    pub diagnostic_sequence_range: Option<(u64, u64)>,
    pub emission_digest: String,
    pub error_code: Option<String>,
    pub failure_consequence_ids: Vec<String>,
    pub initial_state_hash: String,
    pub madi: u64,
    pub ordered_candidate_ids: Vec<String>,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalExecutionBindingRecord {
    pub route: String,
    pub source_revision_digest: String,
    pub input_snapshot_digest: String,
}

pub fn domain_hash_text(domain: &str, canonical_text: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    hasher.update(&[0]);
    hasher.update(canonical_text.as_bytes());
    format!("blake3:{}", hasher.finalize().to_hex())
}

pub(crate) fn normalize_nfc_text(input: &str) -> String {
    input.nfc().collect()
}

fn compare_nfc_text(left: &str, right: &str) -> std::cmp::Ordering {
    normalize_nfc_text(left)
        .as_bytes()
        .cmp(normalize_nfc_text(right).as_bytes())
}

fn json_string(input: &str) -> String {
    let normalized = normalize_nfc_text(input);
    let mut output = String::with_capacity(normalized.len() + 2);
    output.push('"');
    for ch in normalized.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{0008}' => output.push_str("\\b"),
            '\u{000c}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            ch if ch <= '\u{001f}' => {
                use std::fmt::Write;
                let _ = write!(&mut output, "\\u{:04x}", ch as u32);
            }
            _ => output.push(ch),
        }
    }
    output.push('"');
    output
}

fn array(items: impl IntoIterator<Item = String>) -> String {
    let mut output = String::from("[");
    let mut first = true;
    for item in items {
        if !first {
            output.push(',');
        }
        first = false;
        output.push_str(&item);
    }
    output.push(']');
    output
}

fn object(mut fields: Vec<(&'static str, String)>) -> String {
    fields.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
    let mut output = String::from("{");
    for (index, (key, value)) in fields.into_iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&json_string(key));
        output.push(':');
        output.push_str(&value);
    }
    output.push('}');
    output
}

fn optional_string(value: Option<&str>) -> String {
    value.map(json_string).unwrap_or_else(|| "null".to_string())
}

fn string_array(values: impl IntoIterator<Item = String>) -> String {
    array(values.into_iter().map(|value| json_string(&value)))
}

fn normalized_sorted_strings(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut values = values
        .into_iter()
        .map(|value| normalize_nfc_text(&value))
        .collect::<Vec<_>>();
    values.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    values.dedup();
    values
}

fn fixed64_value(value: Fixed64) -> String {
    if value.frac_part() == 0 {
        value.int_part().to_string()
    } else {
        object(vec![
            ("kind", json_string("fixed64")),
            ("raw", value.raw_i64().to_string()),
        ])
    }
}

fn seulgi_intent_value(intent: &SeulgiIntent) -> String {
    match intent {
        SeulgiIntent::None => object(vec![("kind", json_string("none"))]),
        SeulgiIntent::MoveTo { x, y } => object(vec![
            ("kind", json_string("move_to")),
            ("x", fixed64_value(*x)),
            ("y", fixed64_value(*y)),
        ]),
        SeulgiIntent::Attack { target_id } => object(vec![
            ("kind", json_string("attack")),
            ("target_id", target_id.to_string()),
        ]),
        SeulgiIntent::Say { text } => object(vec![
            ("kind", json_string("say")),
            ("text", json_string(text)),
        ]),
    }
}

fn seulgi_packet_value(packet: &SeulgiPacket) -> String {
    object(vec![
        ("accepted_madi", packet.accepted_madi.to_string()),
        ("agent_id", packet.agent_id.to_string()),
        ("intent", seulgi_intent_value(&packet.intent)),
        ("recv_seq", packet.recv_seq.to_string()),
        ("source", packet.source.code_u8().to_string()),
        ("target_madi", packet.target_madi.to_string()),
    ])
}

fn net_event_value(event: &NetEvent) -> String {
    object(vec![
        ("order_key", json_string(&event.order_key)),
        ("payload_detjson", json_string(&event.payload_detjson)),
        ("sender", json_string(&event.sender)),
        ("seq", event.seq.to_string()),
        ("source", event.source.code_u8().to_string()),
    ])
}

pub fn canonical_input_snapshot_text(input: &InputSnapshot) -> String {
    object(vec![
        (
            "ai_injections",
            array(input.ai_injections.iter().map(seulgi_packet_value)),
        ),
        ("dt", fixed64_value(input.dt)),
        ("frame_source", input.frame_source.code_u8().to_string()),
        ("keys_pressed", input.keys_pressed.to_string()),
        ("last_key_name", json_string(&input.last_key_name)),
        (
            "net_events",
            array(input.net_events.iter().map(net_event_value)),
        ),
        ("pointer_x_i32", input.pointer_x_i32.to_string()),
        ("pointer_y_i32", input.pointer_y_i32.to_string()),
        ("rng_seed", input.rng_seed.to_string()),
        ("tick_id", input.tick_id.to_string()),
    ])
}

pub fn canonical_input_snapshot_hash(input: &InputSnapshot) -> String {
    domain_hash_text(INPUT_SNAPSHOT_DOMAIN, &canonical_input_snapshot_text(input))
}

pub fn canonical_execution_binding_text(record: &CanonicalExecutionBindingRecord) -> String {
    object(vec![
        (
            "input_snapshot_digest",
            json_string(&record.input_snapshot_digest),
        ),
        ("route", json_string(&record.route)),
        (
            "source_revision_digest",
            json_string(&record.source_revision_digest),
        ),
    ])
}

pub fn canonical_execution_binding_hash(record: &CanonicalExecutionBindingRecord) -> String {
    domain_hash_text(
        EXECUTION_BINDING_DOMAIN,
        &canonical_execution_binding_text(record),
    )
}

fn resource_value(value: &ResourceValue) -> String {
    match value {
        ResourceValue::None => "null".to_string(),
        ResourceValue::Bool(value) => value.to_string(),
        ResourceValue::Fixed64(value) => fixed64_value(*value),
        ResourceValue::Unit(value) => object(vec![
            ("dim", json_string(&value.dim.format())),
            ("kind", json_string("unit")),
            ("value", fixed64_value(value.value)),
        ]),
        ResourceValue::String(value) => json_string(value),
        ResourceValue::ResourceHandle(handle) => object(vec![
            ("kind", json_string("handle")),
            ("raw", handle.raw().to_string()),
        ]),
        ResourceValue::List(items) => object(vec![
            ("items", array(items.iter().map(resource_value))),
            ("kind", json_string("list")),
        ]),
        ResourceValue::Set(items) => object(vec![
            ("items", array(items.values().map(resource_value))),
            ("kind", json_string("set")),
        ]),
        ResourceValue::Map(entries) => object(vec![
            (
                "entries",
                array(entries.values().map(|entry| {
                    object(vec![
                        ("key", resource_value(&entry.key)),
                        ("value", resource_value(&entry.value)),
                    ])
                })),
            ),
            ("kind", json_string("map")),
        ]),
    }
}

fn source_span(span: &SourceSpan) -> String {
    object(vec![
        (
            "end_col",
            span.end_col
                .map(|value| value.to_string())
                .unwrap_or_else(|| "null".to_string()),
        ),
        ("end_line", span.end_line.to_string()),
        ("file", json_string(&span.file)),
        (
            "start_col",
            span.start_col
                .map(|value| value.to_string())
                .unwrap_or_else(|| "null".to_string()),
        ),
        ("start_line", span.start_line.to_string()),
    ])
}

fn expr_trace(expr: &ExprTrace) -> String {
    object(vec![
        ("tag", json_string(&expr.tag)),
        ("text", optional_string(expr.text.as_deref())),
    ])
}

fn fault_context(ctx: &FaultContext) -> String {
    object(vec![
        (
            "expr",
            ctx.expr
                .as_ref()
                .map(expr_trace)
                .unwrap_or_else(|| "null".to_string()),
        ),
        ("location", json_string(ctx.location)),
        (
            "source_span",
            ctx.source_span
                .as_ref()
                .map(source_span)
                .unwrap_or_else(|| "null".to_string()),
        ),
        ("tick_id", ctx.tick_id.to_string()),
    ])
}

fn arithmetic_kind(kind: &ArithmeticFaultKind) -> String {
    match kind {
        ArithmeticFaultKind::DivByZero => object(vec![("kind", json_string("DivByZero"))]),
        ArithmeticFaultKind::DimensionMismatch { left, right } => object(vec![
            ("kind", json_string("DimensionMismatch")),
            ("left", json_string(&left.format())),
            ("right", json_string(&right.format())),
        ]),
    }
}

fn diag_event(event: &DiagEvent) -> String {
    object(vec![
        (
            "contract_kind",
            optional_string(event.contract_kind.as_deref()),
        ),
        (
            "expr",
            event
                .expr
                .as_ref()
                .map(expr_trace)
                .unwrap_or_else(|| "null".to_string()),
        ),
        ("fault_id", json_string(&event.fault_id)),
        ("madi", event.madi.to_string()),
        ("message", optional_string(event.message.as_deref())),
        ("mode", optional_string(event.mode.as_deref())),
        ("origin", json_string(&event.origin)),
        ("reason", json_string(&event.reason)),
        ("rule_id", json_string(&event.rule_id)),
        ("sam_hash", optional_string(event.sam_hash.as_deref())),
        ("seq", event.seq.to_string()),
        (
            "source_span",
            event
                .source_span
                .as_ref()
                .map(source_span)
                .unwrap_or_else(|| "null".to_string()),
        ),
        ("sub_reason", optional_string(event.sub_reason.as_deref())),
        ("targets", string_array(event.targets.clone())),
    ])
}

pub fn canonical_signal_text(signal: &Signal) -> String {
    match signal {
        Signal::Alrim { name } => object(vec![
            ("kind", json_string("Alrim")),
            ("name", json_string(name)),
        ]),
        Signal::ArithmeticFault { ctx, kind } => object(vec![
            ("context", fault_context(ctx)),
            ("fault_kind", arithmetic_kind(kind)),
            ("kind", json_string("ArithmeticFault")),
        ]),
        Signal::Diag { event } => object(vec![
            ("event", diag_event(event)),
            ("kind", json_string("Diag")),
        ]),
    }
}

pub fn canonical_signal_list_text(signals: &[Signal]) -> String {
    array(signals.iter().map(canonical_signal_text))
}

pub fn canonical_signal_list_hash(signals: &[Signal]) -> String {
    domain_hash_text(EMISSION_BUFFER_DOMAIN, &canonical_signal_list_text(signals))
}

fn patch_op_text(op: &PatchOp, local_seq: usize) -> String {
    let local_seq = local_seq.to_string();
    match op {
        PatchOp::SetComponentJson { entity, tag, json } => object(vec![
            ("kind", json_string("assign")),
            ("local_seq", local_seq),
            (
                "target",
                json_string(&format!("component/{}/{}", entity.0, tag.0)),
            ),
            (
                "value",
                object(vec![
                    ("kind", json_string("json")),
                    ("text", json_string(json)),
                ]),
            ),
        ]),
        PatchOp::RemoveComponent { entity, tag } => object(vec![
            ("kind", json_string("remove")),
            ("local_seq", local_seq),
            (
                "target",
                json_string(&format!("component/{}/{}", entity.0, tag.0)),
            ),
        ]),
        PatchOp::SetResourceJson { tag, json } => object(vec![
            ("kind", json_string("assign")),
            ("local_seq", local_seq),
            ("target", json_string(&format!("resource/{tag}"))),
            (
                "value",
                object(vec![
                    ("kind", json_string("json")),
                    ("text", json_string(json)),
                ]),
            ),
        ]),
        PatchOp::SetResourceFixed64 { tag, value } => object(vec![
            ("kind", json_string("assign")),
            ("local_seq", local_seq),
            ("target", json_string(&format!("resource/{tag}"))),
            ("value", fixed64_value(*value)),
        ]),
        PatchOp::SetResourceHandle { tag, handle } => object(vec![
            ("kind", json_string("assign")),
            ("local_seq", local_seq),
            ("target", json_string(&format!("resource/{tag}"))),
            (
                "value",
                object(vec![
                    ("kind", json_string("handle")),
                    ("raw", handle.raw().to_string()),
                ]),
            ),
        ]),
        PatchOp::SetResourceValue { tag, value } => object(vec![
            ("kind", json_string("assign")),
            ("local_seq", local_seq),
            ("target", json_string(&format!("resource/{tag}"))),
            ("value", resource_value(value)),
        ]),
        PatchOp::DivAssignResourceFixed64 {
            tag,
            rhs,
            tick_id,
            location,
            source_span: span,
            expr,
        } => object(vec![
            (
                "expr",
                expr.as_ref()
                    .map(expr_trace)
                    .unwrap_or_else(|| "null".to_string()),
            ),
            ("kind", json_string("div_assign_fixed64")),
            ("local_seq", local_seq),
            ("location", json_string(location)),
            ("rhs", fixed64_value(*rhs)),
            (
                "source_span",
                span.as_ref()
                    .map(source_span)
                    .unwrap_or_else(|| "null".to_string()),
            ),
            ("target", json_string(&format!("resource/{tag}"))),
            ("tick_id", tick_id.to_string()),
        ]),
        PatchOp::EmitSignal { signal, targets } => match signal {
            Signal::Alrim { name } => {
                let mut fields = vec![
                    ("kind", json_string("emit_alrim")),
                    ("local_seq", local_seq),
                    ("name", json_string(name)),
                ];
                if !targets.is_empty() {
                    fields.push(("targets", string_array(targets.clone())));
                }
                object(fields)
            }
            Signal::ArithmeticFault { .. } | Signal::Diag { .. } => {
                let mut fields = vec![
                    ("kind", json_string("emit_signal")),
                    ("local_seq", local_seq),
                    ("signal", canonical_signal_text(signal)),
                ];
                if !targets.is_empty() {
                    fields.push(("targets", string_array(targets.clone())));
                }
                object(fields)
            }
        },
        PatchOp::GuardViolation { entity, rule_id } => object(vec![
            ("entity", entity.0.to_string()),
            ("kind", json_string("guard_violation")),
            ("local_seq", local_seq),
            ("rule_id", json_string(rule_id)),
        ]),
    }
}

pub fn canonical_patch_ops_text(patch: &Patch) -> String {
    array(
        patch
            .ops
            .iter()
            .enumerate()
            .map(|(local_seq, op)| patch_op_text(op, local_seq)),
    )
}

pub fn canonical_patch_ops_hash(patch: &Patch) -> String {
    domain_hash_text(PATCH_OPS_DOMAIN, &canonical_patch_ops_text(patch))
}

fn emission_intents(candidate: &PatchCandidate) -> String {
    let intents = candidate.patch.ops.iter().filter_map(|op| match op {
        PatchOp::EmitSignal {
            signal: Signal::Alrim { name },
            targets,
        } => {
            let mut fields = vec![("kind", json_string("Alrim")), ("name", json_string(name))];
            if !targets.is_empty() {
                fields.push(("targets", string_array(targets.clone())));
            }
            Some(object(fields))
        }
        PatchOp::EmitSignal { signal, targets } => {
            let mut fields = vec![
                ("kind", json_string(signal.name())),
                ("signal", canonical_signal_text(signal)),
            ];
            if !targets.is_empty() {
                fields.push(("targets", string_array(targets.clone())));
            }
            Some(object(fields))
        }
        PatchOp::GuardViolation { entity, rule_id } => Some(object(vec![
            ("entity", entity.0.to_string()),
            ("kind", json_string("GuardViolation")),
            ("rule_id", json_string(rule_id)),
        ])),
        _ => None,
    });
    array(intents)
}

fn candidate_status(status: &PatchCandidateStatus) -> &'static str {
    match status {
        PatchCandidateStatus::Ok => "ok",
        PatchCandidateStatus::Failed => "failed",
        PatchCandidateStatus::Cancelled => "cancelled",
    }
}

pub fn canonicalize_candidate(candidate: &PatchCandidate) -> CanonicalCandidate {
    let ops_text = canonical_patch_ops_text(&candidate.patch);
    let ops_hash = domain_hash_text(PATCH_OPS_DOMAIN, &ops_text);
    let effects = analyze_patch_effects(&candidate.patch);
    let reads =
        normalized_sorted_strings(effects.reads.into_iter().map(|key| key.canonical_name()));
    let writes =
        normalized_sorted_strings(effects.writes.into_iter().map(|key| key.canonical_name()));
    let candidate_text = object(vec![
        ("candidate_id", json_string(&candidate.candidate_id)),
        ("emission_intents", emission_intents(candidate)),
        (
            "local_candidate_seq",
            candidate.local_candidate_seq.to_string(),
        ),
        ("madi", candidate.madi.to_string()),
        ("ops", ops_text.clone()),
        ("ops_canonical_hash", json_string(&ops_hash)),
        ("origin_id", json_string(&candidate.origin_id)),
        ("origin_kind", json_string(&candidate.origin_kind)),
        ("phase", candidate.phase.to_string()),
        ("read_set", string_array(reads)),
        ("status", json_string(candidate_status(&candidate.status))),
        ("task_group_id", json_string(&candidate.task_group_id)),
        ("task_id", json_string(&candidate.task_id)),
        ("write_set", string_array(writes)),
    ]);
    let candidate_hash = domain_hash_text(PATCH_CANDIDATE_DOMAIN, &candidate_text);
    CanonicalCandidate {
        ops_text,
        ops_hash,
        candidate_text,
        candidate_hash,
    }
}

pub(crate) fn compare_patch_candidates(
    left: &PatchCandidate,
    right: &PatchCandidate,
) -> std::cmp::Ordering {
    left.phase
        .cmp(&right.phase)
        .then_with(|| compare_nfc_text(&left.origin_kind, &right.origin_kind))
        .then_with(|| compare_nfc_text(&left.origin_id, &right.origin_id))
        .then_with(|| compare_nfc_text(&left.task_id, &right.task_id))
        .then_with(|| left.local_candidate_seq.cmp(&right.local_candidate_seq))
        .then_with(|| compare_nfc_text(&left.candidate_id, &right.candidate_id))
        .then_with(|| {
            let left_text = canonicalize_candidate(left).candidate_text;
            let right_text = canonicalize_candidate(right).candidate_text;
            left_text.as_bytes().cmp(right_text.as_bytes())
        })
}

pub fn canonicalize_candidate_set(
    candidates: &[PatchCandidate],
    madi: u64,
) -> CanonicalCandidateSet {
    let mut ordered = candidates.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| compare_patch_candidates(left, right));
    let canonical_candidates = ordered
        .iter()
        .map(|candidate| canonicalize_candidate(candidate))
        .collect::<Vec<_>>();
    let canonical_text = object(vec![
        ("madi", madi.to_string()),
        (
            "ordered_candidates",
            array(
                canonical_candidates
                    .iter()
                    .map(|candidate| candidate.candidate_text.clone()),
            ),
        ),
    ]);
    let hash = domain_hash_text(PATCH_CANDIDATE_SET_DOMAIN, &canonical_text);
    CanonicalCandidateSet {
        canonical_text,
        hash,
        ordered_candidate_ids: ordered
            .iter()
            .map(|candidate| candidate.candidate_id.clone())
            .collect(),
        candidates: canonical_candidates,
    }
}

pub fn candidate_disposition_name(disposition: CandidateDisposition) -> &'static str {
    match disposition {
        CandidateDisposition::Committed => "committed",
        CandidateDisposition::Rejected => "rejected",
        CandidateDisposition::Failed => "failed",
        CandidateDisposition::Cancelled => "cancelled",
    }
}

pub fn canonical_commit_receipt_text(record: &CanonicalCommitReceiptRecord) -> String {
    let dispositions = array(record.candidate_dispositions.iter().map(
        |(candidate_id, disposition)| {
            object(vec![
                ("candidate_id", json_string(candidate_id)),
                ("disposition", json_string(disposition)),
            ])
        },
    ));
    let diagnostic_range = record
        .diagnostic_sequence_range
        .map(|(start, end_exclusive)| {
            object(vec![
                ("end_exclusive", end_exclusive.to_string()),
                ("start", start.to_string()),
            ])
        })
        .unwrap_or_else(|| "null".to_string());
    object(vec![
        ("candidate_dispositions", dispositions),
        (
            "candidate_set_hash",
            json_string(&record.candidate_set_hash),
        ),
        ("commit_id", json_string(&record.commit_id)),
        (
            "committed_state_hash",
            json_string(&record.committed_state_hash),
        ),
        (
            "conflict_keys",
            string_array(normalized_sorted_strings(record.conflict_keys.clone())),
        ),
        ("diagnostic_sequence_range", diagnostic_range),
        ("emission_digest", json_string(&record.emission_digest)),
        ("error_code", optional_string(record.error_code.as_deref())),
        (
            "failure_consequence_ids",
            string_array(normalized_sorted_strings(
                record.failure_consequence_ids.clone(),
            )),
        ),
        (
            "initial_state_hash",
            json_string(&record.initial_state_hash),
        ),
        ("madi", record.madi.to_string()),
        (
            "ordered_candidate_ids",
            string_array(record.ordered_candidate_ids.clone()),
        ),
        ("status", json_string(&record.status)),
    ])
}

pub fn canonical_commit_receipt_hash(record: &CanonicalCommitReceiptRecord) -> String {
    domain_hash_text(
        COMMIT_RECEIPT_DOMAIN,
        &canonical_commit_receipt_text(record),
    )
}
