use std::fs;
use std::path::Path;

use ddonirang_tool::ddn_runtime::DdnProgram;

const CANDIDATES: &[(&str, &str, bool, bool, &str, &str)] = &[
    (
        "SPEC01",
        "publish/spec/v0.5.0/examples/01_result_and_optional.ddn",
        false,
        false,
        "UNCLASSIFIED",
        "E_V25_DEFINITION_MIGRATION_REQUIRED",
    ),
    (
        "SPEC02",
        "publish/spec/v0.5.0/examples/02_generic_static_header.ddn",
        false,
        false,
        "UNCLASSIFIED",
        "UNCLASSIFIED",
    ),
    (
        "SPEC03",
        "publish/spec/v0.5.0/examples/03_bytes_and_exact_number.ddn",
        false,
        false,
        "UNCLASSIFIED",
        "E_V25_DEFINITION_MIGRATION_REQUIRED",
    ),
    (
        "SPEC04",
        "publish/spec/v0.5.0/examples/04_function_and_optional_map.ddn",
        false,
        false,
        "UNCLASSIFIED",
        "E_V25_DEFINITION_MIGRATION_REQUIRED",
    ),
    (
        "SPEC05",
        "publish/spec/v0.5.0/examples/05_pack_kinds.ddn",
        false,
        false,
        "UNCLASSIFIED",
        "E_V25_DEFINITION_MIGRATION_REQUIRED",
    ),
    (
        "SPEC06",
        "publish/spec/v0.5.0/examples/06_observation_and_mol.ddn",
        true,
        false,
        "NONE",
        "E_V25_DEFINITION_MIGRATION_REQUIRED",
    ),
    (
        "SPEC07",
        "publish/spec/v0.5.0/examples/07_common_model.ddn",
        false,
        false,
        "UNCLASSIFIED",
        "E_V25_DEFINITION_MIGRATION_REQUIRED",
    ),
    (
        "PC01",
        "pack/ddn_v25_pc01_definition_binding_v1/positive_definition_binding.ddn",
        false,
        true,
        "UNCLASSIFIED",
        "NONE",
    ),
    (
        "HELLO",
        "pack/edu_ddn_mc_l01_hello_det/lesson.ddn",
        true,
        false,
        "NONE",
        "E_V25_DEFINITION_MIGRATION_REQUIRED",
    ),
    (
        "PROJECTILE",
        "pack/edu_seamgrim_rep_phys_projectile_xy_v1/lesson.ddn",
        true,
        false,
        "NONE",
        "E_V25_DEFINITION_MIGRATION_REQUIRED",
    ),
    (
        "GRID_CURRENT",
        "solutions/seamgrim_ui_mvp/lessons/rep_grid_game_state_drop_v1/lesson.ddn",
        true,
        false,
        "NONE",
        "E_V25_DEFINITION_MIGRATION_REQUIRED",
    ),
    (
        "CONSOLE_VIEW",
        "pack/bogae_backend_parity_console_web_v1/fixtures/input.ddn",
        false,
        false,
        "UNCLASSIFIED",
        "E_V25_DEFINITION_MIGRATION_REQUIRED",
    ),
    (
        "GRID2D_VIEW",
        "pack/bogae_grid2d_smoke_v1/fixtures/input.ddn",
        false,
        false,
        "UNCLASSIFIED",
        "E_V25_DEFINITION_MIGRATION_REQUIRED",
    ),
    (
        "RICH_TEXT",
        "pack/seamgrim_console_rich_markup_v1/input.ddn",
        true,
        false,
        "NONE",
        "E_V25_DEFINITION_MIGRATION_REQUIRED",
    ),
    (
        "TENSOR",
        "pack/tensor_stdlib_phase0/input.ddn",
        true,
        false,
        "NONE",
        "E_V25_DEFINITION_MIGRATION_REQUIRED",
    ),
];

fn error_family(message: &str) -> &str {
    message
        .split(|ch: char| ch == ':' || ch.is_whitespace())
        .find(|part| part.starts_with("E_"))
        .unwrap_or("UNCLASSIFIED")
}

#[test]
fn inventory_candidate_examples_by_real_product_parser() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    for (id, relative, expected_pre, expected_v25, expected_pre_error, expected_v25_error) in
        CANDIDATES
    {
        let path = repository.join(relative);
        let source =
            fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let pre = DdnProgram::from_source(&source, relative);
        let v25 = DdnProgram::from_v25_source(&source, relative);
        println!(
            "EX01|{id}|pre={}|v25={}|pre_error={}|v25_error={}",
            pre.is_ok(),
            v25.is_ok(),
            pre.as_ref()
                .err()
                .map(|error| error_family(error))
                .unwrap_or("NONE"),
            v25.as_ref()
                .err()
                .map(|error| error_family(error))
                .unwrap_or("NONE"),
        );
        assert_eq!(pre.is_ok(), *expected_pre, "{id}: pre-V25 acceptance drift");
        assert_eq!(v25.is_ok(), *expected_v25, "{id}: V25 acceptance drift");
        assert_eq!(
            pre.as_ref()
                .err()
                .map(|error| error_family(error))
                .unwrap_or("NONE"),
            *expected_pre_error,
            "{id}: pre-V25 diagnostic family drift",
        );
        assert_eq!(
            v25.as_ref()
                .err()
                .map(|error| error_family(error))
                .unwrap_or("NONE"),
            *expected_v25_error,
            "{id}: V25 diagnostic family drift",
        );
    }
    assert_eq!(CANDIDATES.len(), 15);
}
