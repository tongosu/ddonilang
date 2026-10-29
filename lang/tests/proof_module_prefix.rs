use ddonirang_lang::module_directive::{parse_module_directive_prefix,ModuleDirectiveContext};
#[test]
fn static_type_header_after_exports_is_owned_by_the_common_declaration_parser(){
    let raw=include_str!("structure_law_ko_surface_r1.ddn");
    let source=format!("(법칙전달=법칙전달) 드러냄.\n{raw}");
    let context=ModuleDirectiveContext::default().with_public_definitions(["법칙전달"]);
    let prefix=parse_module_directive_prefix(&source,"lib/proof.ddn",&context).unwrap();
    assert_eq!(prefix.declaration_source.trim(),raw.trim());
    ddonirang_lang::parse_v25(&prefix.declaration_source,"lib/proof.ddn").unwrap();
}
#[test]
fn malformed_module_record_is_still_a_directive_error(){
    let context=ModuleDirectiveContext::default().with_public_definitions(["법칙전달"]);
    let error=parse_module_directive_prefix("(법칙전달=법칙전달) 드러냄. (앱:앱) 쓰임.","lib.ddn",&context).unwrap_err();
    assert!(error.message.contains("E_RD457_ENTRY_EQUALS"));
}
