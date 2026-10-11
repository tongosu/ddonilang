use ddonirang_lang::{canonicalize, normalize, parse_v25, NormalizationLevel,Stmt,TopLevelItem};

// A3-P4 PROVISIONAL physical placement. RD-445 owns the named 움직씨,
// exact registration shape and declared payload binder; no source rewrite.
const SOURCE: &str = r#"
(값:수) 이동알림:알림씨 := { 없음. }.
조작받는이:임자 := {
    (받은알림:이동알림) 이동반영:움직씨 {이동알림을 받으면} := {
        합계 <- 합계 + 받은알림.값.
    }.
}.
"#;

#[test]
fn a3_rd445_named_owned_provider_roundtrips_exact_registration() {
    let mut program=parse_v25(SOURCE,"반응.ddn").expect("RD-445 named provider ingress");
    let TopLevelItem::SeedDef(owner)=&program.items[1];
    let Stmt::Definition {name,reaction:Some(registration),..}=&owner.body.as_ref().unwrap().stmts[0]
        else {panic!("owned provider identity was lost")};
    assert_eq!(name,"이동반영");
    assert_eq!(registration.alert_type,"이동알림");
    assert_eq!(registration.payload.pin_name,"받은알림");
    canonicalize(&mut program).unwrap();
    let text=normalize(&program,NormalizationLevel::N1);
    assert!(text.contains("이동반영"));
    assert!(text.contains("받으면"));
    let mut reparsed=parse_v25(&text,"반응정본.ddn").unwrap();
    canonicalize(&mut reparsed).unwrap();
    assert_eq!(normalize(&reparsed,NormalizationLevel::N1),text);
}

#[test]
fn a3_rd445_provider_cannot_escape_owner_or_change_payload_type() {
    let outside=SOURCE.replace("조작받는이:임자 := {", "실행:움직씨 := {");
    assert!(parse_v25(&outside,"잘못된소유.ddn").is_err());
    let mismatch=SOURCE.replace("{이동알림을 받으면}","{다른알림을 받으면}");
    assert!(parse_v25(&mismatch,"잘못된씨.ddn").is_err());
    let defaulted=SOURCE.replace("받은알림:이동알림)","받은알림:이동알림 = 없음)");
    assert!(parse_v25(&defaulted,"빈알림추측.ddn").is_err());
}
