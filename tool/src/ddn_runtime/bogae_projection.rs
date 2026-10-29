//! Existing evaluator methods moved without changing their bodies.
use super::{
    BOGAE_DEFAULT_CIRCLE_RADIUS,
    BOGAE_DEFAULT_POINT_SIZE,
    BOGAE_DEFAULT_RECT_HEIGHT,
    BOGAE_DEFAULT_RECT_WIDTH,
    BOGAE_DEFAULT_STROKE_WIDTH,
    BOGAE_DEFAULT_TEXT_SIZE,
    BTreeMap,
    BogaeShapeKind,
    EvalContext,
    EvalError,
    Fixed64,
    HashMap,
    MapEntry,
    PublicObservationDatasetTarget,
    PublicObservationEvent,
    Value,
    bogae_pick_arg,
    bogae_points_from_positional,
    bogae_read_arg,
    canonical_dataset_observation_name,
    dataset_public_observation_scalar,
    map_key_canon,
    parse_bogae_shape_call,
    public_observation_type,
    strip_wrapping_quotes,
    value_canon,
    value_to_string,
};

impl<'a> EvalContext<'a> {
    pub(super) fn apply_bogae_meta_block(
        &mut self,
        locals: &mut HashMap<String, Value>,
        entries: &[String],
    ) -> Result<(), EvalError> {
        let mut draw_items = Vec::new();
        for (entry_index, entry) in entries.iter().enumerate() {
            if let Some(mut draw_value) = self.bogae_entry_to_draw_item(locals, entry) {
                let Value::Map(draw_item) = &mut draw_value else { continue; };
                // A draw list is a frame-local ordered collection.  Its items
                // need a stable identity for the structured WASM space2d
                // boundary; otherwise several anonymous primitives collapse
                // into one ambiguous entity in the Workbench renderer.
                let id = Value::String(format!("bogae.{entry_index}"));
                draw_item.insert(
                    map_key_canon(&id),
                    MapEntry {
                        key: Value::String("id".to_string()),
                        value: id,
                    },
                );
                draw_items.push(draw_value);
            }
        }
        if draw_items.is_empty() {
            return Ok(());
        }
        self.set_resource("보개_그림판_목록", Value::List(draw_items))
    }

    pub(super) fn emit_public_observation(
        &mut self,
        locals: &mut HashMap<String, Value>,
        fields: &[ddonirang_lang::PublicObservationField],
    ) -> Result<(), EvalError> {
        let mut committed = Vec::with_capacity(fields.len());
        for (field_index, field) in fields.iter().enumerate() {
            let value = self.eval_expr(locals, &field.value)?;
            if value.contains_transient_subject_view(){return Err("E_VC04_READ_SNAPSHOT_ESCAPE: 임자 읽기값을 관찰 이력에 저장할 수 없습니다".to_string().into());}
            let dataset_scalar = dataset_public_observation_scalar(&value);
            let (name, dtype, unit, value_canon, value_text, dataset_target) =
                if let Some(dataset_scalar) = dataset_scalar {
                    let canonical_dataset_name =
                        canonical_dataset_observation_name(&dataset_scalar.name)?;
                    if canonical_dataset_name != field.name {
                        return Err(format!(
                        "E_DATASET_OBSERVATION_NAME_MISMATCH: DDN 필드 {}과 자료 열 {}이 다릅니다",
                        field.name, dataset_scalar.name
                    )
                        .into());
                    }
                    (
                        dataset_scalar.name,
                        dataset_scalar.dtype,
                        dataset_scalar.unit,
                        dataset_scalar.value_canon,
                        dataset_scalar.value_text,
                        Some(PublicObservationDatasetTarget {
                            dataset_content_sha256: dataset_scalar.dataset_content_sha256,
                            row_source_ordinal: dataset_scalar.row_source_ordinal,
                            column_name: field.name.clone(),
                        }),
                    )
                } else {
                    let (dtype, unit) = public_observation_type(&value);
                    (
                        field.name.clone(),
                        dtype.to_string(),
                        unit,
                        value_canon(&value),
                        value_to_string(&value),
                        None,
                    )
                };
            let original = self
                .program
                .public_observation_source_refs
                .get(&field.id)
                .ok_or_else(|| {
                    EvalError::from(
                        "E_BOIM_SOURCE_REF_MISMATCH: 공개 관찰 원문 위치가 없습니다".to_string(),
                    )
                })?;
            let source_ref = format!(
                "{}#bytes={}:{}",
                self.program.program.origin.file_path, original.start, original.end
            );
            committed.push(PublicObservationEvent {
                committed_madi: self.tick_id,
                starts_observation_event: field_index == 0,
                name,
                source_ref,
                source_start: original.start,
                source_end: original.end,
                dtype,
                unit,
                format: "ddn-canonical".to_string(),
                value_canon,
                value_text,
                dataset_target,
            });
        }
        self.work_cost
            .borrow_mut()
            .note_public_observation_samples(committed.len())?;
        self.public_observations.extend(committed);
        Ok(())
    }

    pub(super) fn bogae_entry_to_draw_item(
        &mut self,
        locals: &mut HashMap<String, Value>,
        entry: &str,
    ) -> Option<Value> {
        let shape = parse_bogae_shape_call(entry)?;
        let mut map = BTreeMap::new();
        let mut put = |key: &str, value: Value| {
            let key_value = Value::String(key.to_string());
            map.insert(
                map_key_canon(&key_value),
                MapEntry {
                    key: key_value,
                    value,
                },
            );
        };

        match shape.kind {
            BogaeShapeKind::Line => {
                put("kind", Value::String("line".to_string()));
                put(
                    "x1",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["x1"],
                        0,
                        Value::Fixed64(Fixed64::ZERO),
                    ),
                );
                put(
                    "y1",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["y1"],
                        1,
                        Value::Fixed64(Fixed64::ZERO),
                    ),
                );
                put(
                    "x2",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["x2"],
                        2,
                        Value::Fixed64(Fixed64::ZERO),
                    ),
                );
                put(
                    "y2",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["y2"],
                        3,
                        Value::Fixed64(Fixed64::ZERO),
                    ),
                );
                put(
                    "stroke",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["색", "선색", "stroke", "color"],
                        usize::MAX,
                        Value::String("#9ca3af".to_string()),
                    ),
                );
                put(
                    "width",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["굵기", "width"],
                        usize::MAX,
                        Value::Fixed64(BOGAE_DEFAULT_STROKE_WIDTH),
                    ),
                );
            }
            BogaeShapeKind::Circle => {
                put("kind", Value::String("circle".to_string()));
                put(
                    "x",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["x", "cx"],
                        0,
                        Value::Fixed64(Fixed64::ZERO),
                    ),
                );
                put(
                    "y",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["y", "cy"],
                        1,
                        Value::Fixed64(Fixed64::ZERO),
                    ),
                );
                put(
                    "r",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["r", "반지름"],
                        2,
                        Value::Fixed64(BOGAE_DEFAULT_CIRCLE_RADIUS),
                    ),
                );
                put(
                    "fill",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["색", "fill"],
                        usize::MAX,
                        Value::String("#38bdf8".to_string()),
                    ),
                );
                put(
                    "stroke",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["선색", "stroke", "color"],
                        usize::MAX,
                        Value::String("#0ea5e9".to_string()),
                    ),
                );
                put(
                    "width",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["굵기", "width"],
                        usize::MAX,
                        Value::Fixed64(BOGAE_DEFAULT_STROKE_WIDTH),
                    ),
                );
            }
            BogaeShapeKind::Point => {
                put("kind", Value::String("point".to_string()));
                put(
                    "x",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["x", "cx"],
                        0,
                        Value::Fixed64(Fixed64::ZERO),
                    ),
                );
                put(
                    "y",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["y", "cy"],
                        1,
                        Value::Fixed64(Fixed64::ZERO),
                    ),
                );
                put(
                    "size",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["크기", "size", "r"],
                        2,
                        Value::Fixed64(BOGAE_DEFAULT_POINT_SIZE),
                    ),
                );
                put(
                    "color",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["색", "color"],
                        usize::MAX,
                        Value::String("#f59e0b".to_string()),
                    ),
                );
            }
            BogaeShapeKind::Rect => {
                put("kind", Value::String("rect".to_string()));
                put(
                    "x",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["x"],
                        0,
                        Value::Fixed64(Fixed64::ZERO),
                    ),
                );
                put(
                    "y",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["y"],
                        1,
                        Value::Fixed64(Fixed64::ZERO),
                    ),
                );
                put(
                    "w",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["w", "너비"],
                        2,
                        Value::Fixed64(BOGAE_DEFAULT_RECT_WIDTH),
                    ),
                );
                put(
                    "h",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["h", "높이"],
                        3,
                        Value::Fixed64(BOGAE_DEFAULT_RECT_HEIGHT),
                    ),
                );
                put(
                    "fill",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["채움색", "채움", "fill", "색", "color"],
                        usize::MAX,
                        Value::String("#38bdf8".to_string()),
                    ),
                );
                put(
                    "stroke",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["선색", "stroke"],
                        usize::MAX,
                        Value::String("#0ea5e9".to_string()),
                    ),
                );
                put(
                    "width",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["굵기", "width"],
                        usize::MAX,
                        Value::Fixed64(BOGAE_DEFAULT_STROKE_WIDTH),
                    ),
                );
            }
            BogaeShapeKind::Text => {
                put("kind", Value::String("text".to_string()));
                put(
                    "x",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["x"],
                        0,
                        Value::Fixed64(Fixed64::ZERO),
                    ),
                );
                put(
                    "y",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["y"],
                        1,
                        Value::Fixed64(Fixed64::ZERO),
                    ),
                );
                put(
                    "text",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["글", "내용", "text"],
                        2,
                        Value::String(String::new()),
                    ),
                );
                put(
                    "size",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["크기", "size"],
                        usize::MAX,
                        Value::Fixed64(BOGAE_DEFAULT_TEXT_SIZE),
                    ),
                );
                put(
                    "color",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["색", "글색", "color"],
                        usize::MAX,
                        Value::String("#f8fafc".to_string()),
                    ),
                );
            }
            BogaeShapeKind::Polyline => {
                let points = bogae_pick_arg(&shape, &["점들", "좌표들", "points"], usize::MAX)
                    .and_then(|text| self.parse_bogae_value_token(locals, text))
                    .or_else(|| bogae_points_from_positional(&shape, locals, self))?;
                put("kind", Value::String("polyline".to_string()));
                put("points", points);
                put(
                    "stroke",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["색", "선색", "stroke", "color"],
                        usize::MAX,
                        Value::String("#22c55e".to_string()),
                    ),
                );
                put(
                    "width",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["굵기", "width"],
                        usize::MAX,
                        Value::Fixed64(BOGAE_DEFAULT_STROKE_WIDTH),
                    ),
                );
            }
            BogaeShapeKind::Polygon => {
                let points = bogae_pick_arg(&shape, &["점들", "좌표들", "points"], usize::MAX)
                    .and_then(|text| self.parse_bogae_value_token(locals, text))
                    .or_else(|| bogae_points_from_positional(&shape, locals, self))?;
                put("kind", Value::String("polygon".to_string()));
                put("points", points);
                put(
                    "fill",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["채움색", "채움", "fill", "색", "color"],
                        usize::MAX,
                        Value::String("#38bdf844".to_string()),
                    ),
                );
                put(
                    "stroke",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["선색", "stroke"],
                        usize::MAX,
                        Value::String("#0f172a".to_string()),
                    ),
                );
                put(
                    "width",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["굵기", "width"],
                        usize::MAX,
                        Value::Fixed64(BOGAE_DEFAULT_STROKE_WIDTH),
                    ),
                );
            }
        }
        if let Some(layer_index) = bogae_pick_arg(
            &shape,
            &["층", "레이어", "layer_index", "layer", "z", "z_index"],
            usize::MAX,
        )
        .and_then(|text| self.parse_bogae_value_token(locals, text))
        {
            put("layer_index", layer_index);
        }
        if let Some(group_id) = bogae_pick_arg(
            &shape,
            &["그룹", "묶음", "group_id", "group", "groupId"],
            usize::MAX,
        )
        .and_then(|text| self.parse_bogae_value_token(locals, text))
        {
            put("group_id", group_id);
        }
        Some(Value::Map(map))
    }

    pub(super) fn parse_bogae_value_token(
        &mut self,
        locals: &mut HashMap<String, Value>,
        token: &str,
    ) -> Option<Value> {
        let text = token.trim();
        if text.is_empty() {
            return None;
        }
        if let Some(unquoted) = strip_wrapping_quotes(text) {
            return Some(Value::String(unquoted));
        }
        if text == "참" {
            return Some(Value::Bool(true));
        }
        if text == "거짓" {
            return Some(Value::Bool(false));
        }
        let compact_numeric: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
        if let Some(num) = Fixed64::parse_decimal(&compact_numeric) {
            return Some(Value::Fixed64(num));
        }
        if text.starts_with('#') {
            return Some(Value::String(text.to_string()));
        }
        if let Some(value) = locals.get(text) {
            return Some(value.clone());
        }
        self.get_resource(text)
    }

}
