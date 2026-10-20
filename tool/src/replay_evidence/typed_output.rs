//! Capture original evaluator values and declaration-owned canonical types.
//! No output_log, display text, JSON type guess or world resource is consumed.
use std::collections::BTreeMap;
use ddonirang_core::platform::RecordFieldType;
use ddonirang_lang::runtime::Value;

pub(super) const MAX_STAGED_BYTES: usize = 134_217_728;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StagedOutputV1 {
    pub(super) madi: u64,
    pub(super) type_bytes: Vec<u8>,
    pub(super) value_bytes: Vec<u8>,
}

/// Transaction-local candidates only. The actual commit owner must admit
/// every row and all result bounds before committing/publishing a result.
/// Unsupported rows do not change legacy run behavior or become strings.
#[derive(Clone, Debug, Default)]
pub(crate) struct StagedResultEmissionsV1 {
    rows: Vec<Result<StagedOutputV1, String>>,
}

impl StagedResultEmissionsV1 {
    pub(crate) fn stage(&mut self, madi:u64, declared:Option<&RecordFieldType>,
        nominals:&BTreeMap<String,(u8,[u8;32])>, value:&Value,
        source:Option<&crate::authored_project::registration::ResolvedRegisteredAtomResultTypeV1>,
        contracts:&[crate::registered_atom_contract::RecordedAtomContract]) {
        let candidate=(|| {
            let declared=declared.ok_or_else(||
                "E_UNSUPPORTED_RESULT_EVIDENCE: 출력의 정적 결과씨 공급이 없습니다".to_owned())?;
            let declared=crate::tuple_saved_type_plan::resolve_nominal_type(declared,nominals);
            let context=crate::registered_atom_contract::output_evidence::OutputValueContextV1{source,contracts};
            let saved=crate::ddn_runtime::resource_value_from_typed_value_with_output(value,&declared,Some(&context))
                .map_err(|error|format!("E_UNSUPPORTED_RESULT_EVIDENCE: {error}"))?;
            let mut type_bytes=Vec::new();declared.encode_canon(&mut type_bytes);
            // Output has its own strict type context; ordinary saved field
            // decoding must continue to reject the output-only 05 leaf.
            let decoded=crate::unit_wire::decode_result_output_field_type(&type_bytes)
                .map_err(|_|"E_UNSUPPORTED_RESULT_EVIDENCE: 출력 정본씨가 유효하지 않습니다".to_owned())?;
            if decoded!=declared {return Err("E_UNSUPPORTED_RESULT_EVIDENCE: 출력 정본씨가 일치하지 않습니다".into());}
            let mut value_bytes=Vec::new();saved.encode_canon(&mut value_bytes);
            if type_bytes.len().checked_add(value_bytes.len())
                .is_none_or(|length|length>MAX_STAGED_BYTES) {
                return Err("E_UNSUPPORTED_RESULT_EVIDENCE: 출력 증거 한도를 넘었습니다".into());
            }
            Ok(StagedOutputV1{madi,type_bytes,value_bytes})
        })();
        self.rows.push(candidate);
    }

    pub(crate) fn len(&self)->usize { self.rows.len() }

    /// Parallel branches merge in the evaluator's existing provider order,
    /// not callback order, display line numbers or map iteration at the host.
    pub(crate) fn extend_since(&mut self, other:&Self, start:usize) {
        self.rows.extend_from_slice(&other.rows[start..]);
    }

    pub(super) fn into_checked_madi(self,madi:u64)->Result<Vec<StagedOutputV1>,String> {
        let mut checked=Vec::new();
        checked.try_reserve(self.rows.len()).map_err(|_|
            "E_UNSUPPORTED_RESULT_EVIDENCE: 출력 증거 메모리 한도를 확인할 수 없습니다".to_owned())?;
        for row in self.rows {
            let row=row?;
            if row.madi!=madi {
                return Err("E_RESULT_EMISSION_ORDER: 다른 마디의 출력 후보입니다".into());
            }
            checked.push(row);
        }
        Ok(checked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ddn_runtime::{DdnProgram,DdnRunner};
    use crate::runtime_surface::DEFAULT_UPDATE_NAME;
    use ddonirang_core::{Fixed64,InputSnapshot,InputSource,platform::NuriWorld};
    use std::collections::HashMap;

    fn run(source:&str)->crate::ddn_runtime::DdnRunOutput {
        let mut program=DdnProgram::from_v25_source(source,"typed-result.ddn").unwrap();
        program.prepare_temporary_source_execution().unwrap();
        let input=InputSnapshot{tick_id:0,dt:Fixed64::from_i64(1),keys_pressed:0,
            last_key_name:String::new(),pointer_x_i32:0,pointer_y_i32:0,
            ai_injections:vec![],net_events:vec![],frame_source:InputSource::Person,rng_seed:0};
        DdnRunner::new(program,DEFAULT_UPDATE_NAME)
            .run_update(&NuriWorld::new(),&input,&HashMap::new()).unwrap()
    }

    #[test]
    fn evaluator_preserves_different_numeric_types_with_same_value_shape() {
        let output=run("1 보여주기. 1.0 보여주기.");
        let rows=&output.result_emissions.rows;
        assert_eq!(rows.len(),2);
        let a=rows[0].as_ref().unwrap();let b=rows[1].as_ref().unwrap();
        assert_ne!(a.type_bytes,b.type_bytes);
        assert_eq!(a.value_bytes,b.value_bytes);
        assert_eq!(output.display_outputs[0].text,output.display_outputs[1].text);
    }

    #[test]
    fn evaluator_preserves_exact_values_even_when_display_rounds_equal() {
        let output=run("1.0000001 보여주기. 1.0000002 보여주기.");
        let a=output.result_emissions.rows[0].as_ref().unwrap();
        let b=output.result_emissions.rows[1].as_ref().unwrap();
        assert_eq!(a.type_bytes,b.type_bytes);assert_ne!(a.value_bytes,b.value_bytes);
        assert_eq!(output.display_outputs[0].text,output.display_outputs[1].text);
    }

    #[test]
    fn tuple_positions_and_fraction_are_supplied_by_the_compilation_owner() {
        let output=run("(1, 1 / 3) 보여주기.");
        let row=output.result_emissions.rows[0].as_ref().unwrap();
        let kind=crate::unit_wire::decode_recorded_field_type(&row.type_bytes).unwrap();
        assert!(matches!(kind,RecordFieldType::Tuple{..}));
        assert!(!row.value_bytes.is_empty());
    }

    #[test]
    fn missing_type_never_uses_the_runtime_value_to_issue_an_identity() {
        let mut staged=StagedResultEmissionsV1::default();
        staged.stage(0,None,&BTreeMap::new(),&Value::Bool(true),None,&[]);
        assert!(staged.rows[0].as_ref().unwrap_err().starts_with("E_UNSUPPORTED_RESULT_EVIDENCE"));
    }

    #[test]
    fn b3_atom_output_type_codec_is_strict_and_not_saved_admission() {
        let kind=RecordFieldType::RegisteredAtomV1;
        let mut bytes=Vec::new();kind.encode_canon(&mut bytes);
        assert_eq!(bytes,[5,1,0]);
        assert_eq!(crate::unit_wire::decode_result_output_field_type(&bytes).unwrap(),kind);
        assert!(crate::unit_wire::decode_recorded_field_type(&bytes).is_err());
        for bad in [&[][..],&[5][..],&[5,1][..],&[5,2,0][..],&[5,1,0,0][..],&[13,1,0][..]] {
            assert!(crate::unit_wire::decode_result_output_field_type(bad).is_err(),"{bad:?}");
        }
        let nested=RecordFieldType::Tuple{items:vec![kind]};
        let mut bytes=Vec::new();nested.encode_canon(&mut bytes);
        assert_eq!(crate::unit_wire::decode_result_output_field_type(&bytes).unwrap(),nested);
        assert!(crate::unit_wire::decode_recorded_field_type(&bytes).is_err());
    }

    #[test]
    fn b3_atom_output_type_without_source_proof_rejects_non_atom_carrier() {
        let mut staged=StagedResultEmissionsV1::default();
        staged.stage(0,Some(&RecordFieldType::RegisteredAtomV1),&BTreeMap::new(),&Value::Bool(true),None,&[]);
        assert!(staged.into_checked_madi(0).is_err());
    }

    #[test]
    fn b3_atom_output_direct_alias_and_tuple_use_actual_source_owner() {
        let output=run("표지 := #표지. 표지 보여주기. #표지 보여주기. (표지,1 / 3) 보여주기.");
        let rows=output.result_emissions.into_checked_madi(0).unwrap();
        assert_eq!(rows.len(),3);
        assert_eq!(rows[0].type_bytes,[5,1,0]);
        assert_eq!(rows[0].value_bytes[0],0x0d);
        assert_eq!(rows[0].type_bytes,rows[1].type_bytes);
        assert_eq!(rows[0].value_bytes,rows[1].value_bytes);
        let kind=crate::unit_wire::decode_result_output_field_type(&rows[2].type_bytes).unwrap();
        assert!(matches!(&kind,RecordFieldType::Tuple{items} if items[0]==RecordFieldType::RegisteredAtomV1));
    }
}
