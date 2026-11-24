//! Private observation only. Neither input locators nor output facts are permits.
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    root:String, grant_locator:String, authored_project_json_utf8:String,
    expected_head_revision_hex:String, expected_head_integrity_hex:String,
}
pub(crate) fn operate(text:&str)->Result<String,String>{
    if text.len()>8*1024*1024{return Err("E_NATIVE_SOURCE_OBSERVE: LimitExceeded".into());}
    let request:Request=serde_json::from_str(text).map_err(|_|"E_NATIVE_SOURCE_OBSERVE: InvalidInput")?;
    let source=crate::project_revision_envelope_supply::admit_fresh_authored_project(request.authored_project_json_utf8.as_bytes())?;
    let project=hex::decode(source.project().authored_identity.logical_dependency_id.as_str()).map_err(|_|"E_NATIVE_SOURCE_OBSERVE: InvalidInput")?;
    let revision=hex::decode(&request.expected_head_revision_hex).map_err(|_|"E_NATIVE_SOURCE_OBSERVE: InvalidInput")?;
    let integrity:[u8;32]=hex::decode(&request.expected_head_integrity_hex).map_err(|_|"E_NATIVE_SOURCE_OBSERVE: InvalidInput")?.try_into().map_err(|_|"E_NATIVE_SOURCE_OBSERVE: InvalidInput")?;
    #[cfg(windows)] {
        let canonical=source.project().save()?;
        let facts=crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1::observe_source_readonly(
            std::path::Path::new(&request.root),&request.grant_locator,&project,&revision,&integrity,&canonical)
            .map_err(|e|format!("E_NATIVE_SOURCE_OBSERVE: {e:?}"))?;
        return Ok(facts);
    }
    #[cfg(not(windows))] {let _=(request,source,project,revision,integrity);Err("E_NATIVE_SOURCE_OBSERVE: UnsupportedDurability".into())}
}
