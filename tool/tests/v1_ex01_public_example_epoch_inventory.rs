use std::fs;
use std::path::Path;

use ddonirang_tool::ddn_runtime::DdnProgram;

const CANDIDATES: &[(&str, &str, bool, bool, &str, &str)] = &[
    // WO03 D7 removes the blanket ':='-presence rejection. Explicit V25 now
    // reaches its parser: SPEC01/03/04 and VIEW cases retain rejection, but
    // expose the actual syntax error instead of a guessed migration category.
    // SPEC06 now has correct top-level source references after the root visitor
    // fix. RD-106 historical observation/result-table intent does not require
    // a missing source-map error. This parser census does NOT admit the old
    // observation spelling as the successor surface (RD-313), nor prove that
    // its unbound time expression can execute. PROJECTILE/RICH_TEXT reach
    // definition-vs-binding validation. HELLO/TENSOR contain ordinary binding
    // syntax accepted by V25 (not proof of full execution or product admission).
    // GRID_CURRENT is the individually migrated D8 deployment source.
    (
        "SPEC01",
        "publish/spec/v0.5.0/examples/01_result_and_optional.ddn",
        false,
        false,
        "UNCLASSIFIED",
        "UNCLASSIFIED",
    ),
    (
        "SPEC02",
        "publish/spec/v0.5.0/examples/02_generic_static_header.ddn",
        false,
        false,
        // Historical generic-header syntax is still unsupported. RD-090's
        // mixed-pack guard now identifies the first rejected parenthesis
        // shape explicitly; neither epoch is newly accepted. Keep the source.
        "E_PACK_MIXED_SHAPE",
        "E_PACK_MIXED_SHAPE",
    ),
    (
        "SPEC03",
        "publish/spec/v0.5.0/examples/03_bytes_and_exact_number.ddn",
        false,
        false,
        "UNCLASSIFIED",
        "UNCLASSIFIED",
    ),
    (
        "SPEC04",
        "publish/spec/v0.5.0/examples/04_function_and_optional_map.ddn",
        false,
        false,
        "UNCLASSIFIED",
        "UNCLASSIFIED",
    ),
    (
        "SPEC05",
        "publish/spec/v0.5.0/examples/05_pack_kinds.ddn",
        // RD-090: positional Tuple now remains its own parser category.
        // Parser acceptance is not storage support or public example promotion.
        true,
        true,
        "NONE",
        "NONE",
    ),
    (
        "SPEC06",
        "publish/spec/v0.5.0/examples/06_observation_and_mol.ddn",
        true,
        true,
        "NONE",
        "NONE",
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
        true,
        "NONE",
        "NONE",
    ),
    (
        "PROJECTILE",
        "pack/edu_seamgrim_rep_phys_projectile_xy_v1/lesson.ddn",
        true,
        false,
        "NONE",
        "E_V25_DEFINITION_BINDING_CATEGORY",
    ),
    (
        "GRID_CURRENT",
        "solutions/seamgrim_ui_mvp/lessons/rep_grid_game_state_drop_v1/lesson.ddn",
        true,
        true,
        "NONE",
        "NONE",
    ),
    (
        "CONSOLE_VIEW",
        "pack/bogae_backend_parity_console_web_v1/fixtures/input.ddn",
        false,
        false,
        "E_PACK_MIXED_SHAPE",
        "E_PACK_MIXED_SHAPE",
    ),
    (
        "GRID2D_VIEW",
        "pack/bogae_grid2d_smoke_v1/fixtures/input.ddn",
        false,
        false,
        "E_PACK_MIXED_SHAPE",
        "E_PACK_MIXED_SHAPE",
    ),
    (
        "RICH_TEXT",
        "pack/seamgrim_console_rich_markup_v1/input.ddn",
        true,
        false,
        "NONE",
        "E_V25_DEFINITION_BINDING_CATEGORY",
    ),
    (
        "TENSOR",
        "pack/tensor_stdlib_phase0/input.ddn",
        true,
        true,
        "NONE",
        "NONE",
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
    let mut mismatches = Vec::new();
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
        let actual = (pre.is_ok(), v25.is_ok(),
            pre.as_ref().err().map(|error| error_family(error)).unwrap_or("NONE"),
            v25.as_ref().err().map(|error| error_family(error)).unwrap_or("NONE"));
        let expected = (*expected_pre, *expected_v25, *expected_pre_error, *expected_v25_error);
        if actual != expected {
            mismatches.push(format!("{id}: actual={actual:?} expected={expected:?}"));
        }
    }
    assert_eq!(CANDIDATES.len(), 15);
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}
