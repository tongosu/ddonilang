use ddonirang_core::Fixed64;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt;

pub const SEALED_DOMAIN_DATASET_SCHEMA: &str = "ddn.sealed_domain_dataset.v1";
pub const DATASET_TRANSFORM_RECIPE_SCHEMA: &str = "ddn.sealed_domain_transform_recipe.v1";
pub const DERIVED_DATASET_ARTIFACT_SCHEMA: &str = "ddn.sealed_domain_dataset.derived.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DatasetError {
    pub code: &'static str,
    pub path: String,
    pub message: String,
}

impl DatasetError {
    fn new(code: &'static str, path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code,
            path: path.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for DatasetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}: {}", self.code, self.path, self.message)
    }
}

impl std::error::Error for DatasetError {}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SealedDomainDataset {
    pub schema: String,
    pub artifact_id: String,
    pub artifact_version: String,
    pub source: DatasetSource,
    pub payload: DatasetPayload,
    pub ordered_columns: Vec<DatasetColumn>,
    pub ordered_rows: Vec<DatasetRow>,
    pub content_sha256: String,
    pub schema_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DatasetSource {
    ProjectAuthored {
        project_revision_ref: String,
        source_ref: String,
        author_label: String,
    },
    ExternalSnapshot {
        provider: String,
        title: String,
        canonical_uri: String,
        published_or_version: String,
        retrieved_at: String,
        license_id: String,
        attribution: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetPayload {
    pub object_ref: String,
    pub media_type: String,
    pub byte_length: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetColumn {
    pub name: String,
    pub dtype: DatasetDtype,
    pub unit: Option<String>,
    pub source_locator: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetDtype {
    Integer,
    Fixed64,
    String,
    Boolean,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetRow {
    pub source_ordinal: u64,
    pub cells: Vec<DatasetCell>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum DatasetCell {
    Present { value: DatasetScalar },
    Gap,
    Missing { reason: MissingReason },
    Pruned,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DatasetScalar {
    Integer { value: i64 },
    Fixed64 { value: String },
    String { value: String },
    Boolean { value: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum MissingReason {
    #[serde(rename = "#계산실패")]
    CalculationFailed,
    #[serde(rename = "#원천없음")]
    SourceUnavailable,
    #[serde(rename = "#기록손상")]
    RecordDamaged,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetTransformRecipe {
    pub schema: String,
    pub derived_artifact_id: String,
    pub derived_artifact_version: String,
    pub operations: Vec<DatasetTransformOperation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum DatasetTransformOperation {
    SelectColumns {
        columns: Vec<String>,
    },
    FilterRows {
        predicate: DatasetScalarPredicate,
    },
    StableSort {
        keys: Vec<DatasetSortKey>,
    },
    Group {
        keys: Vec<String>,
        aggregates: Vec<DatasetAggregate>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetScalarPredicate {
    pub column: String,
    pub operator: DatasetComparisonOperator,
    pub value: DatasetScalar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetComparisonOperator {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetSortKey {
    pub column: String,
    pub direction: DatasetSortDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetSortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetAggregate {
    pub kind: DatasetAggregateKind,
    pub column: Option<String>,
    pub output: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetAggregateKind {
    Count,
    Sum,
    Min,
    Max,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DerivedDatasetArtifact {
    pub schema: String,
    pub parent_content_sha256: String,
    pub recipe_sha256: String,
    pub dataset: SealedDomainDataset,
    pub artifact_sha256: String,
}

pub fn sealed_domain_dataset_schema_sha256() -> String {
    sha256_hex(&canonical_json_bytes(&schema_descriptor()))
}

pub fn dataset_transform_recipe_sha256(
    recipe: &DatasetTransformRecipe,
) -> Result<String, DatasetError> {
    validate_transform_recipe(recipe)?;
    let value = serde_json::to_value(recipe).map_err(|err| {
        DatasetError::new(
            "E_DATASET_TRANSFORM_RECIPE_SERIALIZE",
            "$",
            format!("transform recipe를 직렬화할 수 없습니다: {err}"),
        )
    })?;
    Ok(sha256_hex(&canonical_json_bytes(&value)))
}

pub fn parse_and_validate_dataset_transform_recipe(
    input_json: &str,
) -> Result<DatasetTransformRecipe, DatasetError> {
    let raw_value: JsonValue = serde_json::from_str(input_json).map_err(|err| {
        DatasetError::new(
            "E_DATASET_TRANSFORM_RECIPE_JSON",
            "$",
            format!("transform recipe JSON을 읽을 수 없습니다: {err}"),
        )
    })?;
    scan_forbidden_material(&raw_value, "$")?;
    let recipe: DatasetTransformRecipe = serde_json::from_str(input_json).map_err(|err| {
        DatasetError::new(
            "E_DATASET_TRANSFORM_RECIPE_JSON",
            "$",
            format!("transform recipe가 strict contract와 다릅니다: {err}"),
        )
    })?;
    validate_transform_recipe(&recipe)?;
    Ok(recipe)
}

pub fn apply_dataset_transform_recipe(
    parent: &SealedDomainDataset,
    payload_bytes: &[u8],
    recipe: &DatasetTransformRecipe,
) -> Result<DerivedDatasetArtifact, DatasetError> {
    validate_sealed_domain_dataset(parent, payload_bytes)?;
    validate_transform_recipe(recipe)?;
    let recipe_sha256 = dataset_transform_recipe_sha256(recipe)?;
    let mut dataset = parent.clone();
    dataset.artifact_id = recipe.derived_artifact_id.clone();
    dataset.artifact_version = recipe.derived_artifact_version.clone();
    for (index, operation) in recipe.operations.iter().enumerate() {
        dataset = apply_transform_operation(dataset, operation, index)?;
    }
    dataset = seal_domain_dataset(dataset, payload_bytes)?;
    let mut artifact = DerivedDatasetArtifact {
        schema: DERIVED_DATASET_ARTIFACT_SCHEMA.to_string(),
        parent_content_sha256: parent.content_sha256.clone(),
        recipe_sha256,
        dataset,
        artifact_sha256: String::new(),
    };
    artifact.artifact_sha256 = derived_dataset_artifact_sha256(&artifact)?;
    Ok(artifact)
}

pub fn derived_dataset_artifact_sha256(
    artifact: &DerivedDatasetArtifact,
) -> Result<String, DatasetError> {
    if artifact.schema != DERIVED_DATASET_ARTIFACT_SCHEMA {
        return Err(DatasetError::new(
            "E_DATASET_DERIVED_SCHEMA",
            "$.schema",
            format!("schema는 {DERIVED_DATASET_ARTIFACT_SCHEMA}여야 합니다"),
        ));
    }
    validate_sha256_text(&artifact.parent_content_sha256, "$.parent_content_sha256")?;
    validate_sha256_text(&artifact.recipe_sha256, "$.recipe_sha256")?;
    let mut value = serde_json::to_value(artifact).map_err(|err| {
        DatasetError::new(
            "E_DATASET_DERIVED_SERIALIZE",
            "$",
            format!("derived dataset artifact를 직렬화할 수 없습니다: {err}"),
        )
    })?;
    value
        .as_object_mut()
        .ok_or_else(|| {
            DatasetError::new(
                "E_DATASET_DERIVED_SERIALIZE",
                "$",
                "derived dataset artifact가 JSON object가 아닙니다",
            )
        })?
        .remove("artifact_sha256");
    Ok(sha256_hex(&canonical_json_bytes(&value)))
}

pub fn validate_derived_dataset_artifact(
    artifact: &DerivedDatasetArtifact,
    payload_bytes: &[u8],
) -> Result<(), DatasetError> {
    validate_sealed_domain_dataset(&artifact.dataset, payload_bytes)?;
    validate_sha256_text(&artifact.artifact_sha256, "$.artifact_sha256")?;
    let expected = derived_dataset_artifact_sha256(artifact)?;
    if artifact.artifact_sha256 != expected {
        return Err(DatasetError::new(
            "E_DATASET_DERIVED_HASH_MISMATCH",
            "$.artifact_sha256",
            format!(
                "derived artifact hash 불일치: expected {expected}, actual {}",
                artifact.artifact_sha256
            ),
        ));
    }
    Ok(())
}

pub fn canonical_derived_dataset_artifact_json(
    artifact: &DerivedDatasetArtifact,
    payload_bytes: &[u8],
) -> Result<String, DatasetError> {
    validate_derived_dataset_artifact(artifact, payload_bytes)?;
    let value = serde_json::to_value(artifact).map_err(|err| {
        DatasetError::new(
            "E_DATASET_DERIVED_SERIALIZE",
            "$",
            format!("derived dataset artifact JSON 직렬화 실패: {err}"),
        )
    })?;
    String::from_utf8(canonical_json_bytes(&value)).map_err(|err| {
        DatasetError::new(
            "E_DATASET_DERIVED_SERIALIZE",
            "$",
            format!("derived dataset artifact canonical UTF-8 생성 실패: {err}"),
        )
    })
}

pub fn sealed_domain_dataset_content_sha256(
    dataset: &SealedDomainDataset,
) -> Result<String, DatasetError> {
    let mut value = serde_json::to_value(dataset).map_err(|err| {
        DatasetError::new(
            "E_DATASET_SERIALIZE",
            "$",
            format!("dataset content를 직렬화할 수 없습니다: {err}"),
        )
    })?;
    let object = value.as_object_mut().ok_or_else(|| {
        DatasetError::new(
            "E_DATASET_SERIALIZE",
            "$",
            "dataset content가 JSON object가 아닙니다",
        )
    })?;
    object.remove("content_sha256");
    object.remove("schema_sha256");
    Ok(sha256_hex(&canonical_json_bytes(&value)))
}

pub fn parse_and_validate_sealed_domain_dataset(
    input_json: &str,
    payload_bytes: &[u8],
) -> Result<SealedDomainDataset, DatasetError> {
    let raw_value: JsonValue = serde_json::from_str(input_json).map_err(|err| {
        DatasetError::new(
            "E_DATASET_JSON",
            "$",
            format!("dataset JSON을 읽을 수 없습니다: {err}"),
        )
    })?;
    scan_forbidden_material(&raw_value, "$")?;
    let dataset: SealedDomainDataset = serde_json::from_str(input_json).map_err(|err| {
        DatasetError::new(
            "E_DATASET_JSON",
            "$",
            format!("dataset schema가 strict contract와 다릅니다: {err}"),
        )
    })?;
    validate_sealed_domain_dataset(&dataset, payload_bytes)?;
    Ok(dataset)
}

pub fn validate_sealed_domain_dataset(
    dataset: &SealedDomainDataset,
    payload_bytes: &[u8],
) -> Result<(), DatasetError> {
    let raw = serde_json::to_value(dataset).map_err(|err| {
        DatasetError::new(
            "E_DATASET_SERIALIZE",
            "$",
            format!("dataset 검증 값을 직렬화할 수 없습니다: {err}"),
        )
    })?;
    scan_forbidden_material(&raw, "$")?;

    if dataset.schema != SEALED_DOMAIN_DATASET_SCHEMA {
        return Err(DatasetError::new(
            "E_DATASET_SCHEMA",
            "$.schema",
            format!("schema는 {SEALED_DOMAIN_DATASET_SCHEMA}여야 합니다"),
        ));
    }
    require_text(&dataset.artifact_id, "$.artifact_id")?;
    require_text(&dataset.artifact_version, "$.artifact_version")?;
    validate_source(&dataset.source)?;
    validate_payload(&dataset.payload, payload_bytes)?;
    validate_columns_and_rows(&dataset.ordered_columns, &dataset.ordered_rows)?;

    let expected_schema_sha256 = sealed_domain_dataset_schema_sha256();
    validate_sha256_text(&dataset.schema_sha256, "$.schema_sha256")?;
    if dataset.schema_sha256 != expected_schema_sha256 {
        return Err(DatasetError::new(
            "E_DATASET_SCHEMA_HASH_MISMATCH",
            "$.schema_sha256",
            format!(
                "schema hash 불일치: expected {expected_schema_sha256}, actual {}",
                dataset.schema_sha256
            ),
        ));
    }

    validate_sha256_text(&dataset.content_sha256, "$.content_sha256")?;
    let expected_content_sha256 = sealed_domain_dataset_content_sha256(dataset)?;
    if dataset.content_sha256 != expected_content_sha256 {
        return Err(DatasetError::new(
            "E_DATASET_CONTENT_HASH_MISMATCH",
            "$.content_sha256",
            format!(
                "content hash 불일치: expected {expected_content_sha256}, actual {}",
                dataset.content_sha256
            ),
        ));
    }
    Ok(())
}

pub fn seal_domain_dataset(
    mut dataset: SealedDomainDataset,
    payload_bytes: &[u8],
) -> Result<SealedDomainDataset, DatasetError> {
    dataset.payload.byte_length = payload_bytes.len() as u64;
    dataset.payload.sha256 = sha256_hex(payload_bytes);
    dataset.schema_sha256 = sealed_domain_dataset_schema_sha256();
    dataset.content_sha256 = sealed_domain_dataset_content_sha256(&dataset)?;
    validate_sealed_domain_dataset(&dataset, payload_bytes)?;
    Ok(dataset)
}

pub fn canonical_sealed_domain_dataset_json(
    dataset: &SealedDomainDataset,
) -> Result<String, DatasetError> {
    validate_sha256_text(&dataset.content_sha256, "$.content_sha256")?;
    let value = serde_json::to_value(dataset).map_err(|err| {
        DatasetError::new(
            "E_DATASET_SERIALIZE",
            "$",
            format!("dataset JSON 직렬화 실패: {err}"),
        )
    })?;
    String::from_utf8(canonical_json_bytes(&value)).map_err(|err| {
        DatasetError::new(
            "E_DATASET_SERIALIZE",
            "$",
            format!("dataset canonical UTF-8 생성 실패: {err}"),
        )
    })
}

fn validate_transform_recipe(recipe: &DatasetTransformRecipe) -> Result<(), DatasetError> {
    let raw = serde_json::to_value(recipe).map_err(|err| {
        DatasetError::new(
            "E_DATASET_TRANSFORM_RECIPE_SERIALIZE",
            "$",
            format!("transform recipe 검증 값을 직렬화할 수 없습니다: {err}"),
        )
    })?;
    scan_forbidden_material(&raw, "$")?;
    if recipe.schema != DATASET_TRANSFORM_RECIPE_SCHEMA {
        return Err(DatasetError::new(
            "E_DATASET_TRANSFORM_RECIPE_SCHEMA",
            "$.schema",
            format!("schema는 {DATASET_TRANSFORM_RECIPE_SCHEMA}여야 합니다"),
        ));
    }
    require_text(&recipe.derived_artifact_id, "$.derived_artifact_id")?;
    require_text(
        &recipe.derived_artifact_version,
        "$.derived_artifact_version",
    )?;
    if recipe.operations.is_empty() {
        return Err(DatasetError::new(
            "E_DATASET_TRANSFORM_RECIPE_EMPTY",
            "$.operations",
            "transform recipe에는 연산이 하나 이상 있어야 합니다",
        ));
    }
    for (index, operation) in recipe.operations.iter().enumerate() {
        let path = format!("$.operations[{index}]");
        match operation {
            DatasetTransformOperation::SelectColumns { columns } => {
                validate_name_list(columns, &format!("{path}.columns"))?;
            }
            DatasetTransformOperation::FilterRows { predicate } => {
                require_text(&predicate.column, format!("{path}.predicate.column"))?;
            }
            DatasetTransformOperation::StableSort { keys } => {
                if keys.is_empty() {
                    return Err(DatasetError::new(
                        "E_DATASET_TRANSFORM_SORT_KEYS_EMPTY",
                        format!("{path}.keys"),
                        "stable sort key는 하나 이상이어야 합니다",
                    ));
                }
                let names = keys
                    .iter()
                    .map(|key| key.column.clone())
                    .collect::<Vec<_>>();
                validate_name_list(&names, &format!("{path}.keys"))?;
            }
            DatasetTransformOperation::Group { keys, aggregates } => {
                validate_name_list(keys, &format!("{path}.keys"))?;
                if aggregates.is_empty() {
                    return Err(DatasetError::new(
                        "E_DATASET_TRANSFORM_AGGREGATES_EMPTY",
                        format!("{path}.aggregates"),
                        "group aggregate는 하나 이상이어야 합니다",
                    ));
                }
                let mut outputs = HashSet::new();
                for (aggregate_index, aggregate) in aggregates.iter().enumerate() {
                    let aggregate_path = format!("{path}.aggregates[{aggregate_index}]");
                    require_text(&aggregate.output, format!("{aggregate_path}.output"))?;
                    if keys.iter().any(|key| key == &aggregate.output)
                        || !outputs.insert(aggregate.output.as_str())
                    {
                        return Err(DatasetError::new(
                            "E_DATASET_TRANSFORM_DUPLICATE_OUTPUT",
                            format!("{aggregate_path}.output"),
                            format!("중복 aggregate output: {}", aggregate.output),
                        ));
                    }
                    match aggregate.kind {
                        DatasetAggregateKind::Count => {
                            if aggregate.column.is_some() {
                                return Err(DatasetError::new(
                                    "E_DATASET_TRANSFORM_COUNT_COLUMN",
                                    format!("{aggregate_path}.column"),
                                    "count는 column을 받지 않습니다",
                                ));
                            }
                        }
                        DatasetAggregateKind::Sum
                        | DatasetAggregateKind::Min
                        | DatasetAggregateKind::Max => {
                            let column = aggregate.column.as_deref().ok_or_else(|| {
                                DatasetError::new(
                                    "E_DATASET_TRANSFORM_AGGREGATE_COLUMN",
                                    format!("{aggregate_path}.column"),
                                    "sum/min/max는 명시적 column이 필요합니다",
                                )
                            })?;
                            require_text(column, format!("{aggregate_path}.column"))?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn validate_name_list(values: &[String], path: &str) -> Result<(), DatasetError> {
    if values.is_empty() {
        return Err(DatasetError::new(
            "E_DATASET_TRANSFORM_NAMES_EMPTY",
            path,
            "이름 목록은 하나 이상이어야 합니다",
        ));
    }
    let mut seen = HashSet::new();
    for (index, value) in values.iter().enumerate() {
        require_text(value, format!("{path}[{index}]"))?;
        if !seen.insert(value.as_str()) {
            return Err(DatasetError::new(
                "E_DATASET_TRANSFORM_DUPLICATE_NAME",
                format!("{path}[{index}]"),
                format!("중복 이름: {value}"),
            ));
        }
    }
    Ok(())
}

fn apply_transform_operation(
    mut dataset: SealedDomainDataset,
    operation: &DatasetTransformOperation,
    operation_index: usize,
) -> Result<SealedDomainDataset, DatasetError> {
    let path = format!("$.operations[{operation_index}]");
    match operation {
        DatasetTransformOperation::SelectColumns { columns } => {
            let indices = resolve_column_indices(&dataset.ordered_columns, columns, &path)?;
            dataset.ordered_columns = indices
                .iter()
                .map(|index| dataset.ordered_columns[*index].clone())
                .collect();
            for row in &mut dataset.ordered_rows {
                row.cells = indices
                    .iter()
                    .map(|index| row.cells[*index].clone())
                    .collect();
            }
        }
        DatasetTransformOperation::FilterRows { predicate } => {
            let column_index = resolve_column_index(
                &dataset.ordered_columns,
                &predicate.column,
                &format!("{path}.predicate.column"),
            )?;
            validate_scalar(
                dataset.ordered_columns[column_index].dtype,
                &predicate.value,
                format!("{path}.predicate.value"),
            )?;
            let mut filtered = Vec::new();
            for row in dataset.ordered_rows {
                let keep = match &row.cells[column_index] {
                    DatasetCell::Present { value } => compare_predicate_values(
                        value,
                        predicate.operator,
                        &predicate.value,
                        &path,
                    )?,
                    DatasetCell::Gap | DatasetCell::Missing { .. } | DatasetCell::Pruned => false,
                };
                if keep {
                    filtered.push(row);
                }
            }
            dataset.ordered_rows = filtered;
        }
        DatasetTransformOperation::StableSort { keys } => {
            let key_indices = keys
                .iter()
                .map(|key| {
                    resolve_column_index(
                        &dataset.ordered_columns,
                        &key.column,
                        &format!("{path}.keys"),
                    )
                    .map(|index| (index, key.direction))
                })
                .collect::<Result<Vec<_>, _>>()?;
            dataset.ordered_rows.sort_by(|left, right| {
                for (index, direction) in &key_indices {
                    let ordering = compare_cells(&left.cells[*index], &right.cells[*index]);
                    if ordering != Ordering::Equal {
                        return match direction {
                            DatasetSortDirection::Ascending => ordering,
                            DatasetSortDirection::Descending => ordering.reverse(),
                        };
                    }
                }
                left.source_ordinal.cmp(&right.source_ordinal)
            });
        }
        DatasetTransformOperation::Group { keys, aggregates } => {
            dataset = apply_group_operation(dataset, keys, aggregates, &path)?;
        }
    }
    validate_columns_and_rows(&dataset.ordered_columns, &dataset.ordered_rows)?;
    Ok(dataset)
}

fn resolve_column_indices(
    columns: &[DatasetColumn],
    names: &[String],
    path: &str,
) -> Result<Vec<usize>, DatasetError> {
    names
        .iter()
        .map(|name| resolve_column_index(columns, name, path))
        .collect()
}

fn resolve_column_index(
    columns: &[DatasetColumn],
    name: &str,
    path: &str,
) -> Result<usize, DatasetError> {
    columns
        .iter()
        .position(|column| column.name == name)
        .ok_or_else(|| {
            DatasetError::new(
                "E_DATASET_TRANSFORM_COLUMN_MISSING",
                path,
                format!("열을 찾을 수 없습니다: {name}"),
            )
        })
}

fn compare_predicate_values(
    left: &DatasetScalar,
    operator: DatasetComparisonOperator,
    right: &DatasetScalar,
    path: &str,
) -> Result<bool, DatasetError> {
    let ordering = compare_scalars(left, right).ok_or_else(|| {
        DatasetError::new(
            "E_DATASET_TRANSFORM_PREDICATE_TYPE",
            format!("{path}.predicate"),
            "predicate는 같은 갈래의 정수, Fixed64, 정본 문자열 또는 boolean equality만 지원합니다",
        )
    })?;
    if matches!(left, DatasetScalar::Boolean { .. })
        && !matches!(
            operator,
            DatasetComparisonOperator::Equal | DatasetComparisonOperator::NotEqual
        )
    {
        return Err(DatasetError::new(
            "E_DATASET_TRANSFORM_PREDICATE_TYPE",
            format!("{path}.predicate.operator"),
            "boolean은 equal/not_equal만 지원합니다",
        ));
    }
    Ok(match operator {
        DatasetComparisonOperator::Equal => ordering == Ordering::Equal,
        DatasetComparisonOperator::NotEqual => ordering != Ordering::Equal,
        DatasetComparisonOperator::LessThan => ordering == Ordering::Less,
        DatasetComparisonOperator::LessThanOrEqual => ordering != Ordering::Greater,
        DatasetComparisonOperator::GreaterThan => ordering == Ordering::Greater,
        DatasetComparisonOperator::GreaterThanOrEqual => ordering != Ordering::Less,
    })
}

fn compare_scalars(left: &DatasetScalar, right: &DatasetScalar) -> Option<Ordering> {
    match (left, right) {
        (DatasetScalar::Integer { value: left }, DatasetScalar::Integer { value: right }) => {
            Some(left.cmp(right))
        }
        (DatasetScalar::Fixed64 { value: left }, DatasetScalar::Fixed64 { value: right }) => {
            Some(Fixed64::parse_decimal(left)?.cmp(&Fixed64::parse_decimal(right)?))
        }
        (DatasetScalar::String { value: left }, DatasetScalar::String { value: right }) => {
            Some(left.as_bytes().cmp(right.as_bytes()))
        }
        (DatasetScalar::Boolean { value: left }, DatasetScalar::Boolean { value: right }) => {
            Some(left.cmp(right))
        }
        _ => None,
    }
}

fn compare_cells(left: &DatasetCell, right: &DatasetCell) -> Ordering {
    match (left, right) {
        (DatasetCell::Present { value: left }, DatasetCell::Present { value: right }) => {
            compare_scalars(left, right).unwrap_or(Ordering::Equal)
        }
        (DatasetCell::Missing { reason: left }, DatasetCell::Missing { reason: right }) => {
            missing_reason_rank(*left).cmp(&missing_reason_rank(*right))
        }
        _ => cell_status_rank(left).cmp(&cell_status_rank(right)),
    }
}

fn cell_status_rank(cell: &DatasetCell) -> u8 {
    match cell {
        DatasetCell::Present { .. } => 0,
        DatasetCell::Gap => 1,
        DatasetCell::Missing { .. } => 2,
        DatasetCell::Pruned => 3,
    }
}

fn missing_reason_rank(reason: MissingReason) -> u8 {
    match reason {
        MissingReason::CalculationFailed => 0,
        MissingReason::SourceUnavailable => 1,
        MissingReason::RecordDamaged => 2,
    }
}

fn apply_group_operation(
    dataset: SealedDomainDataset,
    keys: &[String],
    aggregates: &[DatasetAggregate],
    path: &str,
) -> Result<SealedDomainDataset, DatasetError> {
    let key_indices = resolve_column_indices(&dataset.ordered_columns, keys, path)?;
    let mut groups: Vec<(Vec<DatasetCell>, Vec<DatasetRow>)> = Vec::new();
    for row in &dataset.ordered_rows {
        let key_cells = key_indices
            .iter()
            .map(|index| row.cells[*index].clone())
            .collect::<Vec<_>>();
        if let Some((_, rows)) = groups.iter_mut().find(|(key, _)| key == &key_cells) {
            rows.push(row.clone());
        } else {
            groups.push((key_cells, vec![row.clone()]));
        }
    }

    let mut output_columns = key_indices
        .iter()
        .map(|index| dataset.ordered_columns[*index].clone())
        .collect::<Vec<_>>();
    for (aggregate_index, aggregate) in aggregates.iter().enumerate() {
        output_columns.push(aggregate_output_column(
            &dataset.ordered_columns,
            aggregate,
            &format!("{path}.aggregates[{aggregate_index}]"),
        )?);
    }

    let mut output_rows = Vec::with_capacity(groups.len());
    for (key_cells, rows) in groups {
        let source_ordinal = rows.first().map(|row| row.source_ordinal).ok_or_else(|| {
            DatasetError::new(
                "E_DATASET_TRANSFORM_GROUP_EMPTY",
                path,
                "빈 group은 만들 수 없습니다",
            )
        })?;
        let mut cells = key_cells;
        for (aggregate_index, aggregate) in aggregates.iter().enumerate() {
            cells.push(evaluate_aggregate(
                &dataset.ordered_columns,
                &rows,
                aggregate,
                &format!("{path}.aggregates[{aggregate_index}]"),
            )?);
        }
        output_rows.push(DatasetRow {
            source_ordinal,
            cells,
        });
    }

    Ok(SealedDomainDataset {
        ordered_columns: output_columns,
        ordered_rows: output_rows,
        ..dataset
    })
}

fn aggregate_output_column(
    columns: &[DatasetColumn],
    aggregate: &DatasetAggregate,
    path: &str,
) -> Result<DatasetColumn, DatasetError> {
    if aggregate.kind == DatasetAggregateKind::Count {
        return Ok(DatasetColumn {
            name: aggregate.output.clone(),
            dtype: DatasetDtype::Integer,
            unit: None,
            source_locator: "recipe:aggregate:count".to_string(),
        });
    }
    let source_name = aggregate.column.as_deref().ok_or_else(|| {
        DatasetError::new(
            "E_DATASET_TRANSFORM_AGGREGATE_COLUMN",
            format!("{path}.column"),
            "aggregate column이 없습니다",
        )
    })?;
    let source_index = resolve_column_index(columns, source_name, path)?;
    let source = &columns[source_index];
    match aggregate.kind {
        DatasetAggregateKind::Sum
            if !matches!(source.dtype, DatasetDtype::Integer | DatasetDtype::Fixed64) =>
        {
            return Err(DatasetError::new(
                "E_DATASET_TRANSFORM_SUM_TYPE",
                format!("{path}.column"),
                "sum은 integer 또는 Fixed64 열만 지원합니다",
            ));
        }
        DatasetAggregateKind::Min | DatasetAggregateKind::Max
            if matches!(source.dtype, DatasetDtype::Boolean) =>
        {
            return Err(DatasetError::new(
                "E_DATASET_TRANSFORM_ORDER_TYPE",
                format!("{path}.column"),
                "min/max는 integer, Fixed64 또는 정본 문자열 열만 지원합니다",
            ));
        }
        _ => {}
    }
    Ok(DatasetColumn {
        name: aggregate.output.clone(),
        dtype: source.dtype,
        unit: source.unit.clone(),
        source_locator: format!(
            "recipe:aggregate:{}:{}",
            aggregate_kind_name(aggregate.kind),
            source.source_locator
        ),
    })
}

fn evaluate_aggregate(
    columns: &[DatasetColumn],
    rows: &[DatasetRow],
    aggregate: &DatasetAggregate,
    path: &str,
) -> Result<DatasetCell, DatasetError> {
    if aggregate.kind == DatasetAggregateKind::Count {
        let value = i64::try_from(rows.len()).map_err(|_| {
            DatasetError::new(
                "E_DATASET_TRANSFORM_COUNT_OVERFLOW",
                path,
                "count가 i64 범위를 넘었습니다",
            )
        })?;
        return Ok(DatasetCell::Present {
            value: DatasetScalar::Integer { value },
        });
    }
    let source_name = aggregate.column.as_deref().ok_or_else(|| {
        DatasetError::new(
            "E_DATASET_TRANSFORM_AGGREGATE_COLUMN",
            format!("{path}.column"),
            "aggregate column이 없습니다",
        )
    })?;
    let source_index = resolve_column_index(columns, source_name, path)?;
    let mut values = Vec::with_capacity(rows.len());
    for row in rows {
        match &row.cells[source_index] {
            DatasetCell::Present { value } => values.push(value.clone()),
            DatasetCell::Gap | DatasetCell::Missing { .. } | DatasetCell::Pruned => {
                return Err(DatasetError::new(
                    "E_DATASET_TRANSFORM_SPARSE_AGGREGATE",
                    format!("{path}.column"),
                    "gap/missing/pruned를 집계값으로 숨길 수 없습니다",
                ));
            }
        }
    }
    let value = match aggregate.kind {
        DatasetAggregateKind::Count => unreachable!(),
        DatasetAggregateKind::Sum => sum_scalars(&values, path)?,
        DatasetAggregateKind::Min => select_extreme(&values, false, path)?,
        DatasetAggregateKind::Max => select_extreme(&values, true, path)?,
    };
    Ok(DatasetCell::Present { value })
}

fn sum_scalars(values: &[DatasetScalar], path: &str) -> Result<DatasetScalar, DatasetError> {
    match values.first() {
        Some(DatasetScalar::Integer { .. }) => {
            let mut total = 0i64;
            for value in values {
                let DatasetScalar::Integer { value } = value else {
                    return Err(DatasetError::new(
                        "E_DATASET_TRANSFORM_SUM_TYPE",
                        path,
                        "sum 입력 갈래가 일치하지 않습니다",
                    ));
                };
                total = total.checked_add(*value).ok_or_else(|| {
                    DatasetError::new(
                        "E_DATASET_TRANSFORM_SUM_OVERFLOW",
                        path,
                        "integer sum overflow",
                    )
                })?;
            }
            Ok(DatasetScalar::Integer { value: total })
        }
        Some(DatasetScalar::Fixed64 { .. }) => {
            let mut total = 0i64;
            for value in values {
                let DatasetScalar::Fixed64 { value } = value else {
                    return Err(DatasetError::new(
                        "E_DATASET_TRANSFORM_SUM_TYPE",
                        path,
                        "sum 입력 갈래가 일치하지 않습니다",
                    ));
                };
                let parsed = Fixed64::parse_decimal(value).ok_or_else(|| {
                    DatasetError::new(
                        "E_DATASET_FIXED64",
                        path,
                        format!("정본 Fixed64 decimal이 아닙니다: {value}"),
                    )
                })?;
                total = total.checked_add(parsed.raw_i64()).ok_or_else(|| {
                    DatasetError::new(
                        "E_DATASET_TRANSFORM_SUM_OVERFLOW",
                        path,
                        "Fixed64 sum overflow",
                    )
                })?;
            }
            Ok(DatasetScalar::Fixed64 {
                value: Fixed64::from_raw_i64(total).to_string(),
            })
        }
        _ => Err(DatasetError::new(
            "E_DATASET_TRANSFORM_SUM_TYPE",
            path,
            "sum은 비어 있지 않은 integer 또는 Fixed64 group만 지원합니다",
        )),
    }
}

fn select_extreme(
    values: &[DatasetScalar],
    select_max: bool,
    path: &str,
) -> Result<DatasetScalar, DatasetError> {
    let mut selected = values.first().cloned().ok_or_else(|| {
        DatasetError::new(
            "E_DATASET_TRANSFORM_EMPTY_AGGREGATE",
            path,
            "빈 group은 min/max할 수 없습니다",
        )
    })?;
    for value in values.iter().skip(1) {
        let ordering = compare_scalars(value, &selected).ok_or_else(|| {
            DatasetError::new(
                "E_DATASET_TRANSFORM_ORDER_TYPE",
                path,
                "min/max 입력 갈래가 일치하지 않습니다",
            )
        })?;
        if (select_max && ordering == Ordering::Greater)
            || (!select_max && ordering == Ordering::Less)
        {
            selected = value.clone();
        }
    }
    Ok(selected)
}

fn aggregate_kind_name(kind: DatasetAggregateKind) -> &'static str {
    match kind {
        DatasetAggregateKind::Count => "count",
        DatasetAggregateKind::Sum => "sum",
        DatasetAggregateKind::Min => "min",
        DatasetAggregateKind::Max => "max",
    }
}

fn validate_source(source: &DatasetSource) -> Result<(), DatasetError> {
    match source {
        DatasetSource::ProjectAuthored {
            project_revision_ref,
            source_ref,
            author_label,
        } => {
            require_text(project_revision_ref, "$.source.project_revision_ref")?;
            require_text(source_ref, "$.source.source_ref")?;
            require_text(author_label, "$.source.author_label")?;
        }
        DatasetSource::ExternalSnapshot {
            provider,
            title,
            canonical_uri,
            published_or_version,
            retrieved_at,
            license_id,
            attribution,
        } => {
            require_text(provider, "$.source.provider")?;
            require_text(title, "$.source.title")?;
            require_text(canonical_uri, "$.source.canonical_uri")?;
            require_text(published_or_version, "$.source.published_or_version")?;
            require_text(retrieved_at, "$.source.retrieved_at")?;
            require_text(license_id, "$.source.license_id")?;
            require_text(attribution, "$.source.attribution")?;
            if looks_like_host_path(canonical_uri) || canonical_uri.starts_with("file://") {
                return Err(DatasetError::new(
                    "E_DATASET_ABSOLUTE_PATH",
                    "$.source.canonical_uri",
                    "canonical_uri에 host 파일 경로를 넣을 수 없습니다",
                ));
            }
        }
    }
    Ok(())
}

fn validate_payload(payload: &DatasetPayload, payload_bytes: &[u8]) -> Result<(), DatasetError> {
    validate_relative_object_ref(&payload.object_ref)?;
    require_text(&payload.media_type, "$.payload.media_type")?;
    validate_sha256_text(&payload.sha256, "$.payload.sha256")?;
    if payload.byte_length != payload_bytes.len() as u64 {
        return Err(DatasetError::new(
            "E_DATASET_PAYLOAD_LENGTH_MISMATCH",
            "$.payload.byte_length",
            format!(
                "payload 길이 불일치: expected {}, actual {}",
                payload.byte_length,
                payload_bytes.len()
            ),
        ));
    }
    let actual = sha256_hex(payload_bytes);
    if payload.sha256 != actual {
        return Err(DatasetError::new(
            "E_DATASET_PAYLOAD_HASH_MISMATCH",
            "$.payload.sha256",
            format!(
                "payload hash 불일치: expected {}, actual {actual}",
                payload.sha256
            ),
        ));
    }
    Ok(())
}

fn validate_columns_and_rows(
    columns: &[DatasetColumn],
    rows: &[DatasetRow],
) -> Result<(), DatasetError> {
    if columns.is_empty() {
        return Err(DatasetError::new(
            "E_DATASET_COLUMNS_EMPTY",
            "$.ordered_columns",
            "열은 하나 이상이어야 합니다",
        ));
    }
    let mut names = HashSet::new();
    for (index, column) in columns.iter().enumerate() {
        let base = format!("$.ordered_columns[{index}]");
        require_text(&column.name, format!("{base}.name"))?;
        if !names.insert(column.name.as_str()) {
            return Err(DatasetError::new(
                "E_DATASET_DUPLICATE_COLUMN",
                format!("{base}.name"),
                format!("중복 열 이름: {}", column.name),
            ));
        }
        require_text(&column.source_locator, format!("{base}.source_locator"))?;
        if let Some(unit) = &column.unit {
            require_text(unit, format!("{base}.unit"))?;
        }
    }

    let mut ordinals = HashSet::new();
    for (row_index, row) in rows.iter().enumerate() {
        let row_path = format!("$.ordered_rows[{row_index}]");
        if !ordinals.insert(row.source_ordinal) {
            return Err(DatasetError::new(
                "E_DATASET_DUPLICATE_ROW_ORDINAL",
                format!("{row_path}.source_ordinal"),
                format!("중복 source ordinal: {}", row.source_ordinal),
            ));
        }
        if row.cells.len() != columns.len() {
            return Err(DatasetError::new(
                "E_DATASET_ROW_WIDTH",
                format!("{row_path}.cells"),
                format!(
                    "cell 수 {}가 열 수 {}와 다릅니다",
                    row.cells.len(),
                    columns.len()
                ),
            ));
        }
        for (column_index, (column, cell)) in columns.iter().zip(&row.cells).enumerate() {
            if let DatasetCell::Present { value } = cell {
                validate_scalar(
                    column.dtype,
                    value,
                    format!("{row_path}.cells[{column_index}].value"),
                )?;
            }
        }
    }
    Ok(())
}

fn validate_scalar(
    dtype: DatasetDtype,
    value: &DatasetScalar,
    path: String,
) -> Result<(), DatasetError> {
    let matches = matches!(
        (dtype, value),
        (DatasetDtype::Integer, DatasetScalar::Integer { .. })
            | (DatasetDtype::Fixed64, DatasetScalar::Fixed64 { .. })
            | (DatasetDtype::String, DatasetScalar::String { .. })
            | (DatasetDtype::Boolean, DatasetScalar::Boolean { .. })
    );
    if !matches {
        return Err(DatasetError::new(
            "E_DATASET_CELL_TYPE",
            path,
            "cell scalar kind가 열 dtype과 다릅니다",
        ));
    }
    if let DatasetScalar::Fixed64 { value } = value {
        if Fixed64::parse_decimal(value).is_none() {
            return Err(DatasetError::new(
                "E_DATASET_FIXED64",
                path,
                format!("정본 Fixed64 decimal이 아닙니다: {value}"),
            ));
        }
    }
    Ok(())
}

fn validate_relative_object_ref(value: &str) -> Result<(), DatasetError> {
    require_text(value, "$.payload.object_ref")?;
    if value.contains('\\') || value.starts_with('/') || looks_like_host_path(value) {
        return Err(DatasetError::new(
            "E_DATASET_ABSOLUTE_PATH",
            "$.payload.object_ref",
            "payload object_ref는 project-relative forward-slash 경로여야 합니다",
        ));
    }
    let parts: Vec<&str> = value.split('/').collect();
    if parts
        .iter()
        .any(|part| part.is_empty() || *part == "." || *part == "..")
    {
        return Err(DatasetError::new(
            "E_DATASET_OBJECT_REF",
            "$.payload.object_ref",
            "payload object_ref에 empty/dot/parent segment를 넣을 수 없습니다",
        ));
    }
    Ok(())
}

fn validate_sha256_text(value: &str, path: &str) -> Result<(), DatasetError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(DatasetError::new(
            "E_DATASET_SHA256",
            path,
            "SHA-256은 64자 lowercase hex여야 합니다",
        ));
    }
    Ok(())
}

fn require_text(value: &str, path: impl Into<String>) -> Result<(), DatasetError> {
    let path = path.into();
    if value.trim().is_empty() {
        return Err(DatasetError::new(
            "E_DATASET_REQUIRED_TEXT",
            path,
            "필수 글 값이 비어 있습니다",
        ));
    }
    Ok(())
}

fn scan_forbidden_material(value: &JsonValue, path: &str) -> Result<(), DatasetError> {
    match value {
        JsonValue::Object(map) => {
            for (key, child) in map {
                let child_path = format!("{path}.{key}");
                let lowered = key.to_ascii_lowercase();
                if [
                    "capability",
                    "authorization",
                    "access_token",
                    "refresh_token",
                    "password",
                    "credential",
                    "private_key",
                    "server_path",
                ]
                .iter()
                .any(|token| lowered.contains(token))
                {
                    return Err(DatasetError::new(
                        "E_DATASET_SECRET_FIELD",
                        child_path,
                        "secret 또는 authority field를 dataset에 넣을 수 없습니다",
                    ));
                }
                scan_forbidden_material(child, &child_path)?;
            }
        }
        JsonValue::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                scan_forbidden_material(child, &format!("{path}[{index}]"))?;
            }
        }
        JsonValue::String(text) => {
            let lowered = text.to_ascii_lowercase();
            if lowered.contains("bearer ")
                || lowered.contains("access_token=")
                || lowered.contains("refresh_token=")
                || lowered.contains("-----begin private key-----")
            {
                return Err(DatasetError::new(
                    "E_DATASET_SECRET_VALUE",
                    path,
                    "secret 또는 authority 값을 dataset에 넣을 수 없습니다",
                ));
            }
            if looks_like_host_path(text) || lowered.starts_with("file://") {
                return Err(DatasetError::new(
                    "E_DATASET_ABSOLUTE_PATH",
                    path,
                    "host absolute path를 dataset에 넣을 수 없습니다",
                ));
            }
        }
        _ => {}
    }
    Ok(())
}

fn looks_like_host_path(value: &str) -> bool {
    let bytes = value.as_bytes();
    (bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/'))
        || value.starts_with("\\\\")
        || ["/home/", "/Users/", "/tmp/", "/var/", "/etc/"]
            .iter()
            .any(|prefix| value.starts_with(prefix))
}

fn schema_descriptor() -> JsonValue {
    json!({
        "schema": SEALED_DOMAIN_DATASET_SCHEMA,
        "source_kinds": {
            "external_snapshot": [
                "provider",
                "title",
                "canonical_uri",
                "published_or_version",
                "retrieved_at",
                "license_id",
                "attribution"
            ],
            "project_authored": [
                "project_revision_ref",
                "source_ref",
                "author_label"
            ]
        },
        "payload_fields": ["object_ref", "media_type", "byte_length", "sha256"],
        "column_fields": ["name", "dtype", "unit", "source_locator"],
        "dtypes": ["integer", "fixed64", "string", "boolean"],
        "row_fields": ["source_ordinal", "cells"],
        "cell_statuses": ["present", "gap", "missing", "pruned"],
        "missing_reasons": ["#계산실패", "#원천없음", "#기록손상"],
        "hashes": {
            "content_sha256": "canonical_json_without_content_or_schema_hash",
            "payload_sha256": "raw_payload_bytes",
            "schema_sha256": "this_descriptor"
        },
        "ordering": {
            "columns": "array_order",
            "rows": "array_order",
            "cells": "column_position"
        }
    })
}

fn canonical_json_bytes(value: &JsonValue) -> Vec<u8> {
    serde_json::to_vec(&canonical_json(value)).unwrap_or_default()
}

fn canonical_json(value: &JsonValue) -> JsonValue {
    match value {
        JsonValue::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut out = serde_json::Map::new();
            for key in keys {
                out.insert(key.clone(), canonical_json(&map[key]));
            }
            JsonValue::Object(out)
        }
        JsonValue::Array(items) => JsonValue::Array(items.iter().map(canonical_json).collect()),
        other => other.clone(),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_dataset() -> SealedDomainDataset {
        SealedDomainDataset {
            schema: SEALED_DOMAIN_DATASET_SCHEMA.to_string(),
            artifact_id: "fixture.household-budget".to_string(),
            artifact_version: "v1".to_string(),
            source: DatasetSource::ProjectAuthored {
                project_revision_ref: "revision:test-1".to_string(),
                source_ref: "fixture/budget.csv".to_string(),
                author_label: "W1 fixture author".to_string(),
            },
            payload: DatasetPayload {
                object_ref: "objects/00/payload.bin".to_string(),
                media_type: "text/csv".to_string(),
                byte_length: 0,
                sha256: String::new(),
            },
            ordered_columns: vec![
                DatasetColumn {
                    name: "월".to_string(),
                    dtype: DatasetDtype::String,
                    unit: None,
                    source_locator: "csv:column:1".to_string(),
                },
                DatasetColumn {
                    name: "지출".to_string(),
                    dtype: DatasetDtype::Integer,
                    unit: Some("원".to_string()),
                    source_locator: "csv:column:3".to_string(),
                },
            ],
            ordered_rows: vec![
                DatasetRow {
                    source_ordinal: 0,
                    cells: vec![
                        DatasetCell::Present {
                            value: DatasetScalar::String {
                                value: "1월".to_string(),
                            },
                        },
                        DatasetCell::Present {
                            value: DatasetScalar::Integer { value: 700 },
                        },
                    ],
                },
                DatasetRow {
                    source_ordinal: 1,
                    cells: vec![
                        DatasetCell::Present {
                            value: DatasetScalar::String {
                                value: "2월".to_string(),
                            },
                        },
                        DatasetCell::Missing {
                            reason: MissingReason::SourceUnavailable,
                        },
                    ],
                },
            ],
            content_sha256: String::new(),
            schema_sha256: String::new(),
        }
    }

    fn sealed() -> (SealedDomainDataset, Vec<u8>) {
        let payload = b"month,expense\n1,700\n2,\n".to_vec();
        let dataset = seal_domain_dataset(base_dataset(), &payload).expect("seal");
        (dataset, payload)
    }

    #[test]
    fn seals_and_validates_ordered_dataset_without_unit_inference() {
        let (dataset, payload) = sealed();
        validate_sealed_domain_dataset(&dataset, &payload).expect("validate");
        assert_eq!(
            sealed_domain_dataset_schema_sha256(),
            "2a6ae70d28ec352aafdf6bfb6914b48903e03d4a880d430695f643c2537a3589"
        );
        assert_eq!(dataset.ordered_columns[0].unit, None);
        assert_eq!(dataset.ordered_columns[1].unit.as_deref(), Some("원"));
        assert_eq!(dataset.ordered_rows[0].source_ordinal, 0);
        assert_eq!(dataset.ordered_rows[1].source_ordinal, 1);
    }

    #[test]
    fn payload_tamper_is_exact_hash_failure() {
        let (dataset, mut payload) = sealed();
        payload[0] ^= 1;
        let error = validate_sealed_domain_dataset(&dataset, &payload).unwrap_err();
        assert_eq!(error.code, "E_DATASET_PAYLOAD_HASH_MISMATCH");
    }

    #[test]
    fn external_source_requires_license_and_attribution() {
        let (mut dataset, payload) = sealed();
        dataset.source = DatasetSource::ExternalSnapshot {
            provider: "fixture-provider".to_string(),
            title: "fixture".to_string(),
            canonical_uri: "https://example.invalid/fixture".to_string(),
            published_or_version: "v1".to_string(),
            retrieved_at: "2026-07-27".to_string(),
            license_id: String::new(),
            attribution: String::new(),
        };
        let error = seal_domain_dataset(dataset, &payload).unwrap_err();
        assert_eq!(error.code, "E_DATASET_REQUIRED_TEXT");
        assert_eq!(error.path, "$.source.license_id");
    }

    #[test]
    fn duplicate_column_is_rejected() {
        let (mut dataset, payload) = sealed();
        dataset.ordered_columns[1].name = "월".to_string();
        let error = seal_domain_dataset(dataset, &payload).unwrap_err();
        assert_eq!(error.code, "E_DATASET_DUPLICATE_COLUMN");
    }

    #[test]
    fn invalid_status_and_unknown_field_are_rejected() {
        let (dataset, payload) = sealed();
        let mut value = serde_json::to_value(dataset).expect("value");
        value["ordered_rows"][0]["cells"][0]["status"] = json!("unknown");
        value["graph_role"] = json!("x_axis");
        let input = serde_json::to_string(&value).expect("json");
        let error = parse_and_validate_sealed_domain_dataset(&input, &payload).unwrap_err();
        assert_eq!(error.code, "E_DATASET_JSON");
    }

    #[test]
    fn semantic_sidecar_and_project_identity_fields_are_rejected() {
        let (dataset, payload) = sealed();
        let mut unit_sidecar = serde_json::to_value(&dataset).expect("value");
        unit_sidecar["unit_sidecar"] = json!({"지출": "천원"});
        let input = serde_json::to_string(&unit_sidecar).expect("json");
        let error = parse_and_validate_sealed_domain_dataset(&input, &payload).unwrap_err();
        assert_eq!(error.code, "E_DATASET_JSON");

        let mut project_identity = serde_json::to_value(dataset).expect("value");
        project_identity["project_id"] = json!("project-that-claims-new-meaning");
        let input = serde_json::to_string(&project_identity).expect("json");
        let error = parse_and_validate_sealed_domain_dataset(&input, &payload).unwrap_err();
        assert_eq!(error.code, "E_DATASET_JSON");
    }

    #[test]
    fn duplicate_artifact_id_field_is_rejected() {
        let (dataset, payload) = sealed();
        let canonical = canonical_sealed_domain_dataset_json(&dataset).expect("canonical");
        let duplicate = canonical.replacen(
            "\"artifact_id\":",
            "\"artifact_id\":\"duplicate\",\"artifact_id\":",
            1,
        );
        let error = parse_and_validate_sealed_domain_dataset(&duplicate, &payload).unwrap_err();
        assert_eq!(error.code, "E_DATASET_JSON");
    }

    #[test]
    fn missing_reason_cannot_be_deleted() {
        let (dataset, payload) = sealed();
        let mut value = serde_json::to_value(dataset).expect("value");
        value["ordered_rows"][1]["cells"][1]
            .as_object_mut()
            .expect("cell")
            .remove("reason");
        let input = serde_json::to_string(&value).expect("json");
        let error = parse_and_validate_sealed_domain_dataset(&input, &payload).unwrap_err();
        assert_eq!(error.code, "E_DATASET_JSON");
    }

    #[test]
    fn gap_cannot_be_rewritten_as_numeric_zero() {
        let (mut dataset, payload) = sealed();
        dataset.ordered_rows[1].cells[1] = DatasetCell::Gap;
        dataset = seal_domain_dataset(dataset, &payload).expect("reseal gap fixture");
        dataset.ordered_rows[1].cells[1] = DatasetCell::Present {
            value: DatasetScalar::Integer { value: 0 },
        };
        let error = validate_sealed_domain_dataset(&dataset, &payload).unwrap_err();
        assert_eq!(error.code, "E_DATASET_CONTENT_HASH_MISMATCH");
    }

    #[test]
    fn content_and_schema_hashes_are_verified() {
        let (mut dataset, payload) = sealed();
        dataset.ordered_rows.swap(0, 1);
        let error = validate_sealed_domain_dataset(&dataset, &payload).unwrap_err();
        assert_eq!(error.code, "E_DATASET_CONTENT_HASH_MISMATCH");

        let (mut dataset, payload) = sealed();
        dataset.ordered_columns.swap(0, 1);
        for row in &mut dataset.ordered_rows {
            row.cells.swap(0, 1);
        }
        let error = validate_sealed_domain_dataset(&dataset, &payload).unwrap_err();
        assert_eq!(error.code, "E_DATASET_CONTENT_HASH_MISMATCH");

        let (mut dataset, payload) = sealed();
        dataset.source = DatasetSource::ExternalSnapshot {
            provider: "fixture-provider".to_string(),
            title: "fixture".to_string(),
            canonical_uri: "https://example.invalid/fixture-swapped".to_string(),
            published_or_version: "v1".to_string(),
            retrieved_at: "2026-07-27".to_string(),
            license_id: "LicenseRef-W1-fixture".to_string(),
            attribution: "W1 fixture".to_string(),
        };
        dataset = seal_domain_dataset(dataset, &payload).expect("reseal external source");
        let DatasetSource::ExternalSnapshot { canonical_uri, .. } = &mut dataset.source else {
            unreachable!("external source just assigned");
        };
        *canonical_uri = "https://example.invalid/fixture-uri-only-swap".to_string();
        let error = validate_sealed_domain_dataset(&dataset, &payload).unwrap_err();
        assert_eq!(error.code, "E_DATASET_CONTENT_HASH_MISMATCH");

        let (mut dataset, payload) = sealed();
        dataset.schema_sha256 = "0".repeat(64);
        let error = validate_sealed_domain_dataset(&dataset, &payload).unwrap_err();
        assert_eq!(error.code, "E_DATASET_SCHEMA_HASH_MISMATCH");
    }

    #[test]
    fn secret_and_absolute_path_are_rejected() {
        let (dataset, payload) = sealed();
        let mut value = serde_json::to_value(dataset).expect("value");
        value["access_token"] = json!("secret");
        let input = serde_json::to_string(&value).expect("json");
        let error = parse_and_validate_sealed_domain_dataset(&input, &payload).unwrap_err();
        assert_eq!(error.code, "E_DATASET_SECRET_FIELD");

        let (mut dataset, payload) = sealed();
        dataset.payload.object_ref = "C:\\private\\payload.bin".to_string();
        let error = seal_domain_dataset(dataset, &payload).unwrap_err();
        assert_eq!(error.code, "E_DATASET_ABSOLUTE_PATH");
    }

    #[test]
    fn fixed64_requires_canonical_decimal_input_kind() {
        let (mut dataset, payload) = sealed();
        dataset.ordered_columns[1].dtype = DatasetDtype::Fixed64;
        dataset.ordered_rows[0].cells[1] = DatasetCell::Present {
            value: DatasetScalar::Fixed64 {
                value: "not-a-number".to_string(),
            },
        };
        let error = seal_domain_dataset(dataset, &payload).unwrap_err();
        assert_eq!(error.code, "E_DATASET_FIXED64");
    }

    fn transform_fixture() -> (SealedDomainDataset, Vec<u8>) {
        let payload = b"group,amount,label\nB,7,z\nA,9,b\nA,5,a\nB,7,y\n".to_vec();
        let mut dataset = base_dataset();
        dataset.artifact_id = "fixture.transform".to_string();
        dataset.ordered_columns = vec![
            DatasetColumn {
                name: "갈래".to_string(),
                dtype: DatasetDtype::String,
                unit: None,
                source_locator: "csv:column:1".to_string(),
            },
            DatasetColumn {
                name: "양".to_string(),
                dtype: DatasetDtype::Integer,
                unit: Some("원".to_string()),
                source_locator: "csv:column:2".to_string(),
            },
            DatasetColumn {
                name: "표시".to_string(),
                dtype: DatasetDtype::String,
                unit: None,
                source_locator: "csv:column:3".to_string(),
            },
        ];
        dataset.ordered_rows = vec![
            ("B", 7, "z", 0),
            ("A", 9, "b", 1),
            ("A", 5, "a", 2),
            ("B", 7, "y", 3),
        ]
        .into_iter()
        .map(|(group, amount, label, source_ordinal)| DatasetRow {
            source_ordinal,
            cells: vec![
                DatasetCell::Present {
                    value: DatasetScalar::String {
                        value: group.to_string(),
                    },
                },
                DatasetCell::Present {
                    value: DatasetScalar::Integer { value: amount },
                },
                DatasetCell::Present {
                    value: DatasetScalar::String {
                        value: label.to_string(),
                    },
                },
            ],
        })
        .collect();
        let dataset = seal_domain_dataset(dataset, &payload).expect("seal transform fixture");
        (dataset, payload)
    }

    #[test]
    fn deterministic_recipe_preserves_order_and_records_lineage_hashes() {
        let (dataset, payload) = transform_fixture();
        let recipe = DatasetTransformRecipe {
            schema: DATASET_TRANSFORM_RECIPE_SCHEMA.to_string(),
            derived_artifact_id: "fixture.transform.derived".to_string(),
            derived_artifact_version: "v2".to_string(),
            operations: vec![
                DatasetTransformOperation::FilterRows {
                    predicate: DatasetScalarPredicate {
                        column: "양".to_string(),
                        operator: DatasetComparisonOperator::GreaterThanOrEqual,
                        value: DatasetScalar::Integer { value: 5 },
                    },
                },
                DatasetTransformOperation::StableSort {
                    keys: vec![DatasetSortKey {
                        column: "양".to_string(),
                        direction: DatasetSortDirection::Descending,
                    }],
                },
                DatasetTransformOperation::SelectColumns {
                    columns: vec!["갈래".to_string(), "양".to_string(), "표시".to_string()],
                },
                DatasetTransformOperation::Group {
                    keys: vec!["갈래".to_string()],
                    aggregates: vec![
                        DatasetAggregate {
                            kind: DatasetAggregateKind::Count,
                            column: None,
                            output: "개수".to_string(),
                        },
                        DatasetAggregate {
                            kind: DatasetAggregateKind::Sum,
                            column: Some("양".to_string()),
                            output: "합".to_string(),
                        },
                        DatasetAggregate {
                            kind: DatasetAggregateKind::Min,
                            column: Some("표시".to_string()),
                            output: "첫표시".to_string(),
                        },
                        DatasetAggregate {
                            kind: DatasetAggregateKind::Max,
                            column: Some("양".to_string()),
                            output: "최대".to_string(),
                        },
                    ],
                },
            ],
        };
        let first =
            apply_dataset_transform_recipe(&dataset, &payload, &recipe).expect("transform first");
        let second =
            apply_dataset_transform_recipe(&dataset, &payload, &recipe).expect("transform second");
        assert_eq!(first, second);
        validate_derived_dataset_artifact(&first, &payload).expect("derived validate");
        assert_eq!(first.parent_content_sha256, dataset.content_sha256);
        assert_eq!(
            first.recipe_sha256,
            dataset_transform_recipe_sha256(&recipe).expect("recipe hash")
        );
        assert_eq!(
            first
                .dataset
                .ordered_columns
                .iter()
                .map(|column| column.name.as_str())
                .collect::<Vec<_>>(),
            vec!["갈래", "개수", "합", "첫표시", "최대"]
        );
        assert_eq!(first.dataset.ordered_rows.len(), 2);
        assert_eq!(first.dataset.ordered_rows[0].source_ordinal, 1);
        assert_eq!(first.dataset.ordered_rows[1].source_ordinal, 0);
        assert_eq!(
            first.dataset.ordered_rows[0].cells,
            vec![
                DatasetCell::Present {
                    value: DatasetScalar::String {
                        value: "A".to_string()
                    }
                },
                DatasetCell::Present {
                    value: DatasetScalar::Integer { value: 2 }
                },
                DatasetCell::Present {
                    value: DatasetScalar::Integer { value: 14 }
                },
                DatasetCell::Present {
                    value: DatasetScalar::String {
                        value: "a".to_string()
                    }
                },
                DatasetCell::Present {
                    value: DatasetScalar::Integer { value: 9 }
                },
            ]
        );
        assert_eq!(first.dataset.ordered_columns[2].unit.as_deref(), Some("원"));
    }

    #[test]
    fn sparse_aggregate_and_overflow_fail_closed_without_partial_artifact() {
        let (dataset, payload) = sealed();
        let sparse_recipe = DatasetTransformRecipe {
            schema: DATASET_TRANSFORM_RECIPE_SCHEMA.to_string(),
            derived_artifact_id: "fixture.sparse.derived".to_string(),
            derived_artifact_version: "v2".to_string(),
            operations: vec![DatasetTransformOperation::Group {
                keys: vec!["월".to_string()],
                aggregates: vec![DatasetAggregate {
                    kind: DatasetAggregateKind::Sum,
                    column: Some("지출".to_string()),
                    output: "합".to_string(),
                }],
            }],
        };
        let error = apply_dataset_transform_recipe(&dataset, &payload, &sparse_recipe).unwrap_err();
        assert_eq!(error.code, "E_DATASET_TRANSFORM_SPARSE_AGGREGATE");

        let (mut overflow_dataset, overflow_payload) = transform_fixture();
        for row in &mut overflow_dataset.ordered_rows {
            row.cells[1] = DatasetCell::Present {
                value: DatasetScalar::Integer { value: i64::MAX },
            };
        }
        overflow_dataset =
            seal_domain_dataset(overflow_dataset, &overflow_payload).expect("reseal overflow");
        let overflow_recipe = DatasetTransformRecipe {
            schema: DATASET_TRANSFORM_RECIPE_SCHEMA.to_string(),
            derived_artifact_id: "fixture.overflow.derived".to_string(),
            derived_artifact_version: "v2".to_string(),
            operations: vec![DatasetTransformOperation::Group {
                keys: vec!["갈래".to_string()],
                aggregates: vec![DatasetAggregate {
                    kind: DatasetAggregateKind::Sum,
                    column: Some("양".to_string()),
                    output: "합".to_string(),
                }],
            }],
        };
        let error =
            apply_dataset_transform_recipe(&overflow_dataset, &overflow_payload, &overflow_recipe)
                .unwrap_err();
        assert_eq!(error.code, "E_DATASET_TRANSFORM_SUM_OVERFLOW");
    }

    #[test]
    fn recipe_and_derived_hash_tamper_are_exact_failures() {
        let (dataset, payload) = transform_fixture();
        let empty_recipe = DatasetTransformRecipe {
            schema: DATASET_TRANSFORM_RECIPE_SCHEMA.to_string(),
            derived_artifact_id: "fixture.empty".to_string(),
            derived_artifact_version: "v2".to_string(),
            operations: Vec::new(),
        };
        let error = apply_dataset_transform_recipe(&dataset, &payload, &empty_recipe).unwrap_err();
        assert_eq!(error.code, "E_DATASET_TRANSFORM_RECIPE_EMPTY");

        let unknown_recipe = r#"{
            "schema":"ddn.sealed_domain_transform_recipe.v1",
            "derived_artifact_id":"fixture.unknown",
            "derived_artifact_version":"v2",
            "operations":[{"op":"average","column":"양"}]
        }"#;
        let error = parse_and_validate_dataset_transform_recipe(unknown_recipe).unwrap_err();
        assert_eq!(error.code, "E_DATASET_TRANSFORM_RECIPE_JSON");

        let recipe = DatasetTransformRecipe {
            schema: DATASET_TRANSFORM_RECIPE_SCHEMA.to_string(),
            derived_artifact_id: "fixture.selected".to_string(),
            derived_artifact_version: "v2".to_string(),
            operations: vec![DatasetTransformOperation::SelectColumns {
                columns: vec!["양".to_string()],
            }],
        };
        let mut derived =
            apply_dataset_transform_recipe(&dataset, &payload, &recipe).expect("derived");
        let canonical =
            canonical_derived_dataset_artifact_json(&derived, &payload).expect("canonical");
        assert_eq!(
            serde_json::from_str::<DerivedDatasetArtifact>(&canonical).expect("parse canonical"),
            derived
        );
        derived.dataset.ordered_rows.swap(0, 1);
        let error = validate_derived_dataset_artifact(&derived, &payload).unwrap_err();
        assert_eq!(error.code, "E_DATASET_CONTENT_HASH_MISMATCH");
    }

    #[test]
    fn transform_mid_failure_returns_no_partial_dataset() {
        let (mut dataset, payload) = transform_fixture();
        for row in &mut dataset.ordered_rows {
            row.cells[1] = DatasetCell::Present {
                value: DatasetScalar::Integer { value: i64::MAX },
            };
        }
        dataset = seal_domain_dataset(dataset, &payload).expect("reseal overflow");
        let input_before = dataset.clone();
        let recipe = DatasetTransformRecipe {
            schema: DATASET_TRANSFORM_RECIPE_SCHEMA.to_string(),
            derived_artifact_id: "fixture.mid-failure".to_string(),
            derived_artifact_version: "v2".to_string(),
            operations: vec![
                DatasetTransformOperation::SelectColumns {
                    columns: vec!["갈래".to_string(), "양".to_string()],
                },
                DatasetTransformOperation::Group {
                    keys: vec!["갈래".to_string()],
                    aggregates: vec![DatasetAggregate {
                        kind: DatasetAggregateKind::Sum,
                        column: Some("양".to_string()),
                        output: "합".to_string(),
                    }],
                },
            ],
        };
        let error = apply_dataset_transform_recipe(&dataset, &payload, &recipe).unwrap_err();
        assert_eq!(error.code, "E_DATASET_TRANSFORM_SUM_OVERFLOW");
        assert_eq!(
            dataset, input_before,
            "failed transform mutated its sealed input"
        );
    }
}
