//! Program source admission and lifecycle; implementations moved unchanged.
use super::*;

// Project coordinates from the actual metadata-stripped buffer into the
// captured input. This uses the same preprocessing facts, never diagnostic text.
fn captured_source_offset(source:&str,meta_lines:usize,offset:usize)->usize {
    let mut original=0;let mut stripped=0;
    for chunk in source.split_inclusive('\n').take(meta_lines) {
        let newline=if chunk.ends_with("\r\n"){2}else if chunk.ends_with('\n'){1}else{0};
        if offset<stripped+newline {return original+chunk.len()-newline+offset-stripped;}
        original+=chunk.len();stripped+=newline;
    }
    original+offset-stripped
}


/// Saved definition owner still supplies permanent module/entry identities.
/// Recheck the actual source; no cached signature, bool or partition is a saved capability.
pub(crate) fn verify_piecewise_saved_definition(program:&CanonProgram,source:&str,path:&str)->Result<(),String>{
    if program.origin.piecewise_definitions.is_empty(){return Ok(());}
    let checked=DdnProgram::from_v25_source_with_mode(source,path,ParseMode::Strict)?;
    let normalized=ddonirang_lang::normalize(program,ddonirang_lang::NormalizationLevel::N1);
    if normalized!=ddonirang_lang::normalize(&checked.program,ddonirang_lang::NormalizationLevel::N1) {
        return Err("조건별 저장: 원문·실제 검사 프로그램 결속이 다릅니다".into());
    }
    if checked.piecewise_calculations.parents_len()!=program.origin.piecewise_definitions.len(){
        return Err("조건별 저장: 실제 검사된 부모 공급이 빠졌습니다".into());
    }
    Ok(())
}

#[cfg(test)]
mod wo30_piecewise_tests {
    use super::*;
    #[test]
    fn wo30_piecewise_source_requires_real_typed_private_calculation() {
        let source="요금 (사용량:나눔수~을 --> 나눔수) := 조건별 { 정의역: 0 <= 사용량 <= 6. 10 * 사용량 : 0 <= 사용량 <= 3. 30 + 20 * (사용량 - 3) : 3 < 사용량 <= 6. }.";
        let program=DdnProgram::from_v25_source(source,"요금.ddn").expect("checked private calculation");
        assert_eq!(program.piecewise_calculations.count(),1);
        let ordinary="두배하다 (값:바른수 --> 바른수) := 값 + 값.";
        assert!(DdnProgram::from_v25_source(ordinary,"기존.ddn").is_ok());
    }
}

impl DdnProgram {
    pub(super) fn check_value_type(&self,value:&Value,type_ref:&TypeRef)->Result<(),ddonirang_lang::runtime::TypeMismatchDetail>{
        let mut pending=vec![(value,type_ref)];
        while let Some((value,type_ref))=pending.pop(){
            if let TypeRef::Applied{name,args}=type_ref {
                if name=="맺음"&&args.len()==2 {
                    let expected=ddonirang_lang::sum_owner::resolve_applied_sum_owner(type_ref,ddonirang_lang::Span{start:0,end:0})
                        .map_err(|_|ddonirang_lang::runtime::TypeMismatchDetail{expected:"맺음".into(),actual:ddonirang_lang::runtime::value_type_name(value)})?;
                    if let Value::SumBranch{owner,branch,payload}=value {
                        let selected=match branch.as_str(){"잘됨"=>Some(("값",0)),"안됨"=>Some(("이유",1)),_=>None};
                        if let Some((pin,index))=selected {
                            if owner==&expected.key&&payload.len()==1 {
                                if let Some(child)=payload.get(pin){pending.push((child,&args[index]));continue;}
                            }
                        }
                    }
                    return Err(ddonirang_lang::runtime::TypeMismatchDetail{expected:expected.key,actual:ddonirang_lang::runtime::value_type_name(value)});
                }
                if name=="가리킴"&&args.len()==1 {
                    let expected=match &args[0] {
                        TypeRef::Named(owner)=>self.products.get(owner).and_then(|seed|self.nominal_constructors.get(&seed.id)).filter(|schema|schema.category==1),
                        _=>None,
                    };
                    let valid=matches!((value,expected),(Value::IdentityReference(reference),Some(schema))
                        if reference.profile_version==1&&reference.generation==0&&reference.target_type_identity.as_slice()==schema.name_type
                        &&reference.owner_space_identity.len()==32&&reference.target_identity.len()==32);
                    if !valid{return Err(ddonirang_lang::runtime::TypeMismatchDetail{expected:"원 임자에 결속된 가리킴".into(),actual:ddonirang_lang::runtime::value_type_name(value)});}
                    continue;
                }
            }
            if let TypeRef::Tuple(kinds)=type_ref {
                let Value::Tuple(items)=value else {
                    return Err(ddonirang_lang::runtime::TypeMismatchDetail{expected:"위치형 씨".into(),actual:ddonirang_lang::runtime::value_type_name(value)});
                };
                if items.len()!=kinds.len(){
                    return Err(ddonirang_lang::runtime::TypeMismatchDetail{expected:format!("위치형 {}개",kinds.len()),actual:format!("위치형 {}개",items.len())});
                }
                pending.extend(items.iter().zip(kinds).rev());
                continue;
            }
            if let TypeRef::Named(name)=type_ref {
            if let Some(schema)=self.products.get(name).and_then(|seed|self.nominal_constructors.get(&seed.id)) {
                let matches=match value {
                    Value::NominalValueV1(value)=>schema.category==2&&value.name_type_identity()==&schema.name_type,
                    Value::SubjectHandleV1(value)=>schema.category==1&&value.name_type_identity==schema.name_type,
                    Value::FrozenSubjectViewV1(value)=>schema.category==1&&value.name_type_identity()==&schema.name_type,
                    _=>false,
                };
                if !matches{return Err(ddonirang_lang::runtime::TypeMismatchDetail{
                    expected:name.clone(),actual:ddonirang_lang::runtime::value_type_name(value)});}
                continue;
            }
            }
            ddonirang_lang::runtime::check_value_type_with_records(value,type_ref,&self.record_schemas)?;
        }
        Ok(())
    }
    pub(super) fn nominal_type_ids(&self)->BTreeMap<String,(u8,[u8;32])>{
        self.products.iter().filter_map(|(name,seed)|self.nominal_constructors.get(&seed.id)
            .map(|schema|(name.clone(),(schema.category,schema.name_type)))).collect()
    }
    pub(crate) fn bind_registered_project_context(&mut self,owned:&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1)->Result<(),String> {
        // Parsing/retaining a source is not VM admission. Native source custody
        // must still be able to reach its actual registration producer.
        if owned.sources().preparation().requires_registered_nominal_owner && owned.registration().is_none() {
            return Err("E_PROJECT_REGISTERED_NOMINAL_OWNER_REQUIRED: 동명 모듈 이름씨는 실제 등록자료를 Native에 명시 보관한 뒤 실행해 주세요".into());
        }
        if self.first_callable_context.is_some() {
            return Err("E_MULTILANG_DURABLE_CONTEXT_REQUIRED: DDL·프로필을 보존하는 실제 저장 문맥이 필요합니다".into());
        }
        if self.temporary_registration.is_some()||self.nominal_source_registration.is_some(){return Err("E_VC04_TEMPORARY_SAVED_ESCAPE: 임시 실행을 저장 작품 권한으로 바꿀 수 없습니다".into());}
        if !self.saved_project_sources.as_ref().is_some_and(|sources|Arc::ptr_eq(sources,owned.sources())) {
            return Err("E_SCHEMA_REGISTERED_CONTEXT_REQUIRED: 다른 원문 소유자의 등록 자료입니다".into());
        }
        if let Some(registration)=owned.registration() {
            registration.bind_resolved_program(owned,&mut self.program)?;
            self.refresh_registered_executable_copies()?;
            self.nominal_constructors=registration.bind_constructors(owned,&self.program)?;
            let (atoms,proof)=registration.bind_atom_values(owned,&self.program)?.into_parts();
            self.registered_atom_literals=atoms;
            self.result_output_type_plan=self.numeric_constraint_plan.as_deref().and_then(|original|
                crate::registered_atom_contract::output_evidence::OutputTypePlanV1::compile(&self.program,&self.top_level_decl_names,self.host_numeric_supply.as_ref(),original,proof));
            self.registered_nominal_catalog=Some(Arc::new(registration.saved_nominal_catalog(owned)?));
            self.registered_atom_contracts=Arc::new(registration.verified_recorded_atom_contracts(owned)?.to_vec());
        }
        Ok(())
    }
    pub(crate) fn prepare_temporary_source_execution(&mut self)->Result<(),String>{
        if self.saved_project_sources.is_some()||self.temporary_registration.is_some()||self.nominal_source_registration.is_some(){return Ok(())}
        let Some(context)=crate::authored_project::registration::temporary::TemporaryRegisteredSource::prepare(self)? else{return Ok(())};
        context.bind_resolved_program(&mut self.program)?;
        self.refresh_registered_executable_copies()?;
        let constructors=context.bind_constructors(&self.program)?;
        let (atoms,proof)=context.bind_atoms(&self.program)?.into_parts();
        let stage=crate::unit_wire::subject_execution::SubjectExecutionStage::fresh_for_temporary(&context,&self.program)?;
        let catalog=context.catalog(&self.program)?;let contracts=context.contracts().to_vec();
        self.nominal_constructors=constructors;self.registered_atom_literals=atoms;
        self.result_output_type_plan=self.numeric_constraint_plan.as_deref().and_then(|original|
            crate::registered_atom_contract::output_evidence::OutputTypePlanV1::compile(&self.program,&self.top_level_decl_names,self.host_numeric_supply.as_ref(),original,proof));
        self.registered_nominal_catalog=Some(Arc::new(catalog));self.registered_atom_contracts=Arc::new(contracts);
        self.temporary_subject_bootstrap=stage;self.temporary_registration=Some(Arc::new(context));Ok(())
    }
    pub(crate) fn has_temporary_registration(&self)->bool{self.temporary_registration.is_some()}
    pub(crate) fn from_nominal_source_context(context:Arc<crate::authored_project::registration::nominal_source::RegisteredNominalSource>)->Result<Self,String>{
        let mut program=Self::from_rootless_nominal_capture(context.capture(),context.path())?;
        context.bind_resolved_program(&mut program.program)?;program.refresh_registered_executable_copies()?;
        let constructors=context.bind_constructors(&program.program)?;let (atoms,proof)=context.bind_atoms(&program.program)?.into_parts();
        let catalog=context.catalog(&program.program)?;let contracts=context.contracts().to_vec();
        program.nominal_constructors=constructors;program.registered_atom_literals=atoms;
        program.result_output_type_plan=program.numeric_constraint_plan.as_deref().and_then(|original|
            crate::registered_atom_contract::output_evidence::OutputTypePlanV1::compile(&program.program,&program.top_level_decl_names,program.host_numeric_supply.as_ref(),original,proof));
        program.registered_nominal_catalog=Some(Arc::new(catalog));program.registered_atom_contracts=Arc::new(contracts);
        program.nominal_source_registration=Some(context);Ok(program)
    }
    pub(super) fn refresh_registered_executable_copies(&mut self)->Result<(),String> {
        #[cfg(test)]
        let _refresh_observation=saved_program_rebuild::RegisteredRefreshObservation::begin();
        let seeds=self.program.items.iter().map(|TopLevelItem::SeedDef(seed)|(seed.id,seed)).collect::<HashMap<_,_>>();
        for seed in self.functions.values_mut().chain(self.products.values_mut()).chain(self.ordinary_overloads.values_mut().flat_map(|seeds|seeds.iter_mut())) {
            *seed=(*seeds.get(&seed.id).ok_or("E_RESOLVED_SUPPLY_EXECUTABLE_COPY: 실행 선언의 소유 원문이 없습니다")?).clone();
        }
        // Registration refreshes executable copies, not static S4 declarations.
        // Preserve the same source-checked admission used by initial construction.
        self.root_stmts=self.program.root_stmts.iter().filter(|stmt|
            !(self.identity_calculations.active() && matches!(stmt,Stmt::Definition{..}))).cloned().collect();
        self.derived_plan=Arc::new(persistent_dependencies::DerivedPlan::compile(&self.program)?);
        self.numeric_constraint_plan=Some(Arc::new(crate::numeric_constraint_plan::NumericConstraintPlan::compile_with_host(
            &self.program,&self.top_level_decl_names,self.host_numeric_supply.as_ref())?));
        Ok(())
    }
    pub fn from_source(source: &str, file_path: &str) -> Result<Self, String> {
        Self::from_source_with_mode(source, file_path, default_parse_mode())
    }

    pub fn from_source_with_mode(
        source: &str,
        file_path: &str,
        mode: ParseMode,
    ) -> Result<Self, String> {
        Self::from_source_with_mode_and_age_target(source, file_path, mode, default_age_target())
    }

    pub fn from_source_with_mode_and_age_target(
        source: &str,
        file_path: &str,
        mode: ParseMode,
        age_target: AgeTarget,
    ) -> Result<Self, String> {
        Self::from_source_epoch_with_mode(source, file_path, mode, false, age_target,None)
    }

    pub fn from_v25_source(source: &str, file_path: &str) -> Result<Self, String> {
        Self::from_v25_source_with_mode(source, file_path, default_parse_mode())
    }

    /// Compilation-local registered entry. This reuses the common parser,
    /// type planner and evaluator; it does not register a source-profile
    /// version or provide a durable/CLI/WASM context transport.
    pub fn from_public_callable_authored_source(source:&str,path:&str,owner:Arc<crate::authored_project::PublicCallableAuthoredContext>)->Result<Self,String>{
        Self::from_public_callable_authored_source_diagnostic(source,path,owner,None)
    }
    pub(crate) fn from_public_callable_authored_source_diagnostic(
        source:&str,path:&str,owner:Arc<crate::authored_project::PublicCallableAuthoredContext>,
        diagnostic_out:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>,
    )->Result<Self,String>{
        owner.verify_runtime_activation(source,path)?;
        let capture=crate::source_snapshot_admission::capture_declared_v25(source.as_bytes())?;
        let mut program=Self::parse_captured_source_registered_with_diagnostic(source,path,ParseMode::Strict,true,default_age_target(),Some(capture),Some(crate::runtime_surface::bogae_host_numeric_supply()),None,Some(owner.compiler_context().clone()),None,diagnostic_out)?;
        validate_v1_core_no_named_callable_recursion(&program.program)?;
        program.public_callable_authored_owner=Some(owner);Ok(program)
    }
    pub fn public_callable_authored_owner(&self)->Option<&Arc<crate::authored_project::PublicCallableAuthoredContext>>{self.public_callable_authored_owner.as_ref()}

    pub fn from_first_callable_registered_source(
        source: &str, file_path: &str,
        context: Arc<ddonirang_lang::parser::RegisteredFirstCallableContext>,
    ) -> Result<Self, String> {
        let capture = crate::source_snapshot_admission::capture_declared_v25(source.as_bytes())?;
        let program = Self::parse_captured_source_registered(source, file_path, ParseMode::Strict, true,
            default_age_target(), Some(capture), Some(crate::runtime_surface::bogae_host_numeric_supply()), Some(context),None,None)?;
        validate_v1_core_no_named_callable_recursion(&program.program)?;
        Ok(program)
    }

    pub fn first_callable_registered_context(&self) -> Option<&Arc<ddonirang_lang::parser::RegisteredFirstCallableContext>> {
        self.first_callable_context.as_ref()
    }

    pub fn from_first_callable_authored_source(source: &str, path: &str,
        owner: Arc<crate::authored_project::FirstCallableAuthoredContext>) -> Result<Self, String> {
        owner.verify_runtime_activation(source,path)?;
        let mut program = Self::from_first_callable_registered_source(source, path, owner.compiler_context().clone())?;
        program.first_callable_authored_owner = Some(owner);
        Ok(program)
    }

    pub fn first_callable_authored_owner(&self) -> Option<&Arc<crate::authored_project::FirstCallableAuthoredContext>> {
        self.first_callable_authored_owner.as_ref()
    }

    pub fn from_v25_source_with_mode(
        source: &str,
        file_path: &str,
        mode: ParseMode,
    ) -> Result<Self, String> {
        Self::from_v25_source_with_mode_and_age_target(
            source,
            file_path,
            mode,
            default_age_target(),
        )
    }

    pub fn from_v25_source_with_mode_and_age_target(
        source: &str,
        file_path: &str,
        mode: ParseMode,
        age_target: AgeTarget,
    ) -> Result<Self, String> {
        Self::from_source_epoch_with_mode(source, file_path, mode, true, age_target,None)
    }

    pub fn from_supported_source_profile_with_mode(
        source: &str,
        file_path: &str,
        mode: ParseMode,
        profile: SupportedSourceProfile,
    ) -> Result<Self, String> {
        Self::from_supported_source_profile_with_mode_and_age_target(
            source,
            file_path,
            mode,
            profile,
            default_age_target(),
        )
    }

    pub fn from_supported_source_profile_with_mode_and_age_target(
        source: &str,
        file_path: &str,
        mode: ParseMode,
        profile: SupportedSourceProfile,
        age_target: AgeTarget,
    ) -> Result<Self, String> {
        match profile {
            SupportedSourceProfile::V1CoreV25JaBaseR1=>{
                let capture=crate::source_snapshot_admission::capture_declared_v25(source.as_bytes())?;
                let program=Self::parse_captured_source_registered(source,file_path,mode,true,age_target,Some(capture),
                    Some(crate::runtime_surface::bogae_host_numeric_supply()),None,None,
                    Some(ddonirang_lang::ddl::CheckedJ1Profile::resolve(ddonirang_lang::ddl::J1_SOURCE_PROFILE).map_err(|e|e.to_string())?))?;
                validate_v1_core_no_named_callable_recursion(&program.program)?;Ok(program)
            },
            SupportedSourceProfile::V1CoreV25 => {
                let program = Self::from_source_epoch_with_mode(
                    source, file_path, mode,true, age_target,Some(crate::runtime_surface::bogae_host_numeric_supply()),
                )?;
                validate_v1_core_no_named_callable_recursion(&program.program)?;
                Ok(program)
            }
        }
    }

    pub(crate) fn from_j1_source_with_mode_diagnostic(source:&str,path:&str,mode:ParseMode,age:AgeTarget)->Result<Self,crate::preprocess::ExecutionDiagnostic>{
        Self::from_supported_source_profile_diagnostic(source,path,mode,SupportedSourceProfile::V1CoreV25JaBaseR1,age)
    }

    pub(crate) fn from_supported_source_profile_diagnostic(source:&str,path:&str,mode:ParseMode,profile:SupportedSourceProfile,age:AgeTarget)->Result<Self,crate::preprocess::ExecutionDiagnostic>{
        Self::from_supported_source_profile_diagnostic_with_host(source,path,mode,profile,age,
            Some(crate::runtime_surface::bogae_host_numeric_supply()))
    }

    pub(crate) fn from_supported_source_profile_diagnostic_with_host(source:&str,path:&str,mode:ParseMode,profile:SupportedSourceProfile,age:AgeTarget,
        host:Option<crate::runtime_surface::BogaeHostNumericSupply>)->Result<Self,crate::preprocess::ExecutionDiagnostic>{
        fn failure(message:String,path:&str,phase:&'static str)->crate::preprocess::ExecutionDiagnostic{
            crate::preprocess::ExecutionDiagnostic{code:"E_PARSE",message,source_path:path.into(),span:None,phase,cause:None,offending_field:None,expected:None,actual:None}
        }
        if mode==ParseMode::HistoricalMigrationEvidence { return Err(failure("E_HISTORICAL_ASYNC_MIGRATION_EXECUTION_FORBIDDEN: migration evidence mode cannot enter product execution".into(),path,"source_capture")); }
        let capture=crate::source_snapshot_admission::capture_declared_v25(source.as_bytes()).map_err(|message|failure(message,path,"source_capture"))?;
        let syntax=match profile { SupportedSourceProfile::V1CoreV25=>None, SupportedSourceProfile::V1CoreV25JaBaseR1=>Some(ddonirang_lang::ddl::CheckedJ1Profile::resolve(ddonirang_lang::ddl::J1_SOURCE_PROFILE).map_err(|error|failure(error.to_string(),path,"source_profile"))?) };
        let mut actual_failure=None;
        let program=Self::parse_captured_source_registered_with_diagnostic(source,path,mode,true,age,Some(capture),
            host,None,None,syntax,Some(&mut actual_failure))
            .map_err(|message|actual_failure.unwrap_or_else(||failure(message,path,"execution_parse")))?;
        validate_v1_core_no_named_callable_recursion(&program.program).map_err(|message|{
            let mut error=failure(message,path,"execution_parse");error.code=E_V1_CORE_RECURSION_UNSUPPORTED;error
        })?;Ok(program)
    }

    pub(super) fn from_source_epoch_with_mode(
        source: &str,
        file_path: &str,
        mode: ParseMode,
        v25: bool,
        age_target: AgeTarget,
        host_numeric_supply:Option<crate::runtime_surface::BogaeHostNumericSupply>,
    ) -> Result<Self, String> {
        if source.starts_with('\u{feff}') {
            return Err("E_SOURCE_UTF8_BOM_FORBIDDEN: 원문은 BOM 없는 UTF-8이어야 합니다".to_string());
        }
        if mode == ParseMode::HistoricalMigrationEvidence {
            return Err(
                "E_HISTORICAL_ASYNC_MIGRATION_EXECUTION_FORBIDDEN: migration evidence mode cannot enter product execution"
                    .to_string(),
            );
        }
        // A3-P7 / RD-643: capture the unmodified bytes once at the common
        // declared-v25 frontdoor, before metadata stripping/canonicalization.
        // Content-only observation provenance remains separate and unchanged.
        let source_capture = if v25 {
            Some(crate::source_snapshot_admission::capture_declared_v25(source.as_bytes())?)
        } else { None };
        Self::parse_captured_source(source, file_path, mode, v25, age_target, source_capture,host_numeric_supply)
    }

    /// RD-643 replay frontdoor: a typed saved capture, not a live editor buffer.
    /// Revalidate the seal but never allocate a new admission ordinal.
    pub(crate) fn from_sealed_v25_source_with_mode(
        source_capture: crate::source_snapshot_admission::SourceCapture,
        file_path: &str,
        mode: ParseMode,
        age_target: AgeTarget,
    ) -> Result<Self, String> {
        Self::from_sealed_v25_source_with_host(source_capture,file_path,mode,age_target,None)
    }

    pub(super) fn from_sealed_v25_source_with_host(
        source_capture:crate::source_snapshot_admission::SourceCapture,file_path:&str,mode:ParseMode,age_target:AgeTarget,
        host_numeric_supply:Option<crate::runtime_surface::BogaeHostNumericSupply>,
    )->Result<Self,String> {
        crate::source_snapshot_admission::replay(
            &crate::source_snapshot_admission::EpochSupply::for_declared_epoch(ddonirang_lang::SourceEpoch::V25),
            &source_capture,
        ).map_err(|_| "E_SOURCE_ADMISSION_CUSTODY: 저장된 원문 봉인과 글판이 맞지 않습니다".to_string())?;
        Self::parse_captured_source("", file_path, mode, true, age_target, Some(source_capture),host_numeric_supply)
    }

    /// Sealing proves source custody, not permission to bypass profile limits.
    pub(crate) fn from_sealed_supported_source_profile_with_mode(
        source_capture: crate::source_snapshot_admission::SourceCapture,
        file_path: &str,
        mode: ParseMode,
        profile: SupportedSourceProfile,
        age_target: AgeTarget,
    ) -> Result<Self, String> {
        match profile {
            SupportedSourceProfile::V1CoreV25JaBaseR1=>{
                crate::source_snapshot_admission::replay(&crate::source_snapshot_admission::EpochSupply::for_declared_epoch(ddonirang_lang::SourceEpoch::V25),&source_capture)
                    .map_err(|_|"E_SOURCE_ADMISSION_CUSTODY: 저장된 J1 원문 봉인이 맞지 않습니다".to_string())?;
                let program=Self::parse_captured_source_registered("",file_path,mode,true,age_target,Some(source_capture),
                    Some(crate::runtime_surface::bogae_host_numeric_supply()),None,None,
                    Some(ddonirang_lang::ddl::CheckedJ1Profile::resolve(ddonirang_lang::ddl::J1_SOURCE_PROFILE).map_err(|e|e.to_string())?))?;
                validate_v1_core_no_named_callable_recursion(&program.program)?;Ok(program)
            },
            SupportedSourceProfile::V1CoreV25 => {
                let program = Self::from_sealed_v25_source_with_host(
                    source_capture, file_path, mode, age_target,Some(crate::runtime_surface::bogae_host_numeric_supply()),
                )?;
                validate_v1_core_no_named_callable_recursion(&program.program)?;
                Ok(program)
            }
        }
    }

    /// A linked execution unit is derived code, not a new original snapshot.
    /// Keep the admitted original closure alive and recheck its existing seals.
    pub(crate) fn from_saved_v25_project_with_mode(
        saved: Arc<crate::project_revision_envelope_supply::VerifiedSavedV25Sources>,
        mode: ParseMode,
        age_target: AgeTarget,
    ) -> Result<Self, String> {
        let epoch=crate::source_snapshot_admission::EpochSupply::for_declared_epoch(
            ddonirang_lang::SourceEpoch::V25,
        );
        for (_,capture) in saved.captures() {
            crate::source_snapshot_admission::replay(&epoch,capture)
                .map_err(|_|"E_SOURCE_ADMISSION_CUSTODY: 저장된 프로젝트 원문 봉인이 맞지 않습니다".to_string())?;
        }
        let preparation=saved.preparation();
        let mut program=if let Some(context)=saved.logic_callable_context(){
            let profile=saved.saved().j1_profile().ok_or("E_LOGIC_SOURCE_PROFILE_REQUIRED")?;
            Self::parse_captured_source_registered_contexts(&preparation.source,&preparation.target_path,mode,true,age_target,None,
                Some(crate::runtime_surface::bogae_host_numeric_supply()),None,None,Some(profile),Some(context.clone()),saved.linked_logic_program(),None)?
        }else if let Some(profile)=saved.saved().j1_profile(){
            // Derived linked code reuses the admitted original captures above;
            // it is neither a new source identity nor another entropy request.
            Self::parse_captured_source_registered(&preparation.source,&preparation.target_path,mode,true,age_target,None,
                Some(crate::runtime_surface::bogae_host_numeric_supply()),None,None,Some(profile))?
        }else{saved_program_rebuild::compile(&preparation.source,&preparation.target_path,mode,age_target)?};
        validate_v1_core_no_named_callable_recursion(&program.program)?;
        program.saved_project_sources=Some(saved);
        Ok(program)
    }

    /// Compile linked J1 source without starting a world. Both the editor's
    /// preparation check and explicit execution use this original-byte mapping.
    pub(crate) fn from_linked_j1_preparation(
        preparation: &crate::local_project::AuthoredPreparation,
        profile: ddonirang_lang::ddl::CheckedJ1Profile,
        mode: ParseMode,
        age_target: AgeTarget,
        diagnostic_out: Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>,
    ) -> Result<Self, String> {
    let mut linked = None;
    Self::parse_captured_source_registered_with_diagnostic(&preparation.source,&preparation.target_path,mode,true,age_target,None,
        Some(crate::runtime_surface::bogae_host_numeric_supply()),None,None,Some(profile),
        diagnostic_out.as_ref().map(|_| &mut linked))
        .map_err(|message| {
            if let (Some(out), Some(mut diagnostic)) = (diagnostic_out, linked) {
                if let Some((path, span)) = diagnostic.span.and_then(|(start,end)|
                    preparation.original_span(ddonirang_lang::Span::new(start,end)).ok()
                ) {
                    diagnostic = diagnostic.remap_source(path, (span.start, span.end));
                } else {
                    diagnostic.span = None;
                    diagnostic.source_path.clear();
                    diagnostic.cause = None;
                }
                *out = Some(diagnostic);
            }
            message
        })
    }

    pub(crate) fn bind_logic_source_scopes(&mut self,scopes:BTreeMap<ddonirang_lang::NodeId,crate::authored_project::registration::CheckedLogicSourceScope>)->Result<(),String>{
        let sites=self.program.logic_call_sites();
        if sites.len()!=scopes.len(){return Err("E_LOGIC_SOURCE_SCOPE_DENOMINATOR".into());}
        for (node,span) in sites{
            let scope=scopes.get(&node).ok_or("E_LOGIC_SOURCE_SCOPE_REQUIRED")?;
            if scope.site()!=span{return Err("E_LOGIC_SOURCE_SITE_MISMATCH".into());}
            scope.verify_source_generation(self)?;
        }
        self.logic_source_scopes=Some(Arc::new(scopes));Ok(())
    }
    pub(crate) fn from_rootless_nominal_capture(capture:crate::source_snapshot_admission::SourceCapture,path:&str)->Result<Self,String>{
        let text=capture.text().to_owned();
        Self::parse_captured_source(&text,path,ParseMode::Strict,true,default_age_target(),Some(capture),None)
    }
    pub(super) fn parse_captured_source(
        source: &str,
        file_path: &str,
        mode: ParseMode,
        v25: bool,
        age_target: AgeTarget,
        source_capture: Option<crate::source_snapshot_admission::SourceCapture>,
        host_numeric_supply:Option<crate::runtime_surface::BogaeHostNumericSupply>,
    ) -> Result<Self, String> {
        Self::parse_captured_source_registered(source, file_path, mode, v25, age_target,
            source_capture, host_numeric_supply, None,None,None)
    }

    fn parse_captured_source_registered(
        source: &str, file_path: &str, mode: ParseMode, v25: bool, age_target: AgeTarget,
        source_capture: Option<crate::source_snapshot_admission::SourceCapture>,
        host_numeric_supply: Option<crate::runtime_surface::BogaeHostNumericSupply>,
        first_callable_context: Option<Arc<ddonirang_lang::parser::RegisteredFirstCallableContext>>,
        public_callable_context: Option<Arc<ddonirang_lang::parser::RegisteredPublicCallableContext>>,
        j1_profile: Option<ddonirang_lang::ddl::CheckedJ1Profile>,
    ) -> Result<Self, String> {
        Self::parse_captured_source_registered_with_diagnostic(source,file_path,mode,v25,age_target,source_capture,
            host_numeric_supply,first_callable_context,public_callable_context,j1_profile,None)
    }

    fn parse_captured_source_registered_with_diagnostic(
        source: &str, file_path: &str, mode: ParseMode, v25: bool, age_target: AgeTarget,
        source_capture: Option<crate::source_snapshot_admission::SourceCapture>,
        host_numeric_supply: Option<crate::runtime_surface::BogaeHostNumericSupply>,
        first_callable_context: Option<Arc<ddonirang_lang::parser::RegisteredFirstCallableContext>>,
        public_callable_context: Option<Arc<ddonirang_lang::parser::RegisteredPublicCallableContext>>,
        j1_profile: Option<ddonirang_lang::ddl::CheckedJ1Profile>,
        diagnostic_out:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>,
    ) -> Result<Self, String> {
        Self::parse_captured_source_registered_contexts(source,file_path,mode,v25,age_target,source_capture,host_numeric_supply,
            first_callable_context,public_callable_context,j1_profile,None,None,diagnostic_out)
    }
    fn parse_captured_source_registered_contexts(
        source:&str,file_path:&str,mode:ParseMode,v25:bool,age_target:AgeTarget,
        source_capture:Option<crate::source_snapshot_admission::SourceCapture>,host_numeric_supply:Option<crate::runtime_surface::BogaeHostNumericSupply>,
        first_callable_context:Option<Arc<ddonirang_lang::parser::RegisteredFirstCallableContext>>,
        public_callable_context:Option<Arc<ddonirang_lang::parser::RegisteredPublicCallableContext>>,
        j1_profile:Option<ddonirang_lang::ddl::CheckedJ1Profile>,
        logic_context:Option<Arc<ddonirang_lang::parser::RegisteredLogicCallableContext>>,
        linked_logic:Option<&CanonProgram>,
        mut diagnostic_out:Option<&mut Option<crate::preprocess::ExecutionDiagnostic>>,
    )->Result<Self,String>{
        if logic_context.is_some()&&(j1_profile.is_none()||first_callable_context.is_some()||public_callable_context.is_some()){
            return Err("E_LOGIC_SOURCE_CONTEXT_CONFLICT".into());
        }
        if mode == ParseMode::HistoricalMigrationEvidence {
            return Err("E_HISTORICAL_ASYNC_MIGRATION_EXECUTION_FORBIDDEN: migration evidence mode cannot enter product execution".to_string());
        }
        let source = source_capture.as_ref().map_or(source, |sealed| sealed.text());
        if j1_profile.is_none() && first_callable_context.is_none() && public_callable_context.is_none() {
            if first_callable_header_requires_context(source) {
                return Err("E_MULTILANG_CONTEXT_REQUIRED: 日本語の原本・DDL・登録文脈が必要です".into());
            }
            validate_no_legacy_header(source)?;
        }
        validate_no_legacy_boim_surface(source)?;
        let meta_parse = split_file_meta(source);
        let mut top_level_decl_names = collect_top_level_decl_names(&meta_parse.stripped);
        let configured_madi = extract_setting_madi(&meta_parse.stripped)?;
        let cleaned = if v25 {
            meta_parse.stripped.clone()
        } else {
            preprocess_source_for_parse(&meta_parse.stripped)?
        };
        let prepared = if v25 {
            cleaned
        } else {
            ddonirang_lang::preprocess_frontdoor_source(&cleaned)
        };
        // Retain the actual parser/canonical failure before the legacy String
        // boundary. Caller-owned storage cannot outlive or leak into a next run.
        let mut parse_failure=|error:ddonirang_lang::ParseError|{
            let registration_unavailable =
                error.code() == ddonirang_lang::STRUCTURE_PROFILE_REGISTRY_UNAVAILABLE;
            let message = if registration_unavailable {
                error.message.clone()
            } else {
                format_parse_error(&prepared, &error)
            };
            if let Some(slot)=diagnostic_out.as_deref_mut(){
                let code=if j1_profile.is_some(){error.j1_source_code()}else{error.code()};
                let mut diagnostic=crate::preprocess::ExecutionDiagnostic::execution_mode_parse(error,file_path);
                diagnostic.code=code;if let Some(cause)=diagnostic.cause.as_mut(){cause.code=code;}
                // Metadata stripping retains lines but removes header bytes. Project
                // actual parser coordinates back to the captured original without reparsing.
                if v25 {
                    if let Some((start,end))=diagnostic.span {
                        diagnostic.span=Some((captured_source_offset(source,meta_parse.meta_lines,start),
                            captured_source_offset(source,meta_parse.meta_lines,end)));
                    }
                    if let Some(cause)=diagnostic.cause.as_mut(){cause.span=diagnostic.span;}
                }
                diagnostic.message = message.clone();
                if !registration_unavailable {
                    diagnostic.phase = "execution_parse";
                }
                *slot = Some(diagnostic);
            }
            message
        };
        let identity_prepared=if v25 && mode==ParseMode::Strict && j1_profile.is_none() && first_callable_context.is_none() && public_callable_context.is_none() {
            Some(ddonirang_lang::proof_authoring::prepare_source(&prepared,file_path)
                .map_err(&mut parse_failure)?)
        }else{None};
        let identity_calculations=Arc::new(if let Some(owner)=&identity_prepared {
            super::structure_identity_calculation::IdentityCalculations::compile(owner,&prepared,file_path)
                .map_err(&mut parse_failure)?
        }else{super::structure_identity_calculation::IdentityCalculations::default()});
        let mut program = if let Some(program)=linked_logic {
            if program.origin.source != prepared || program.origin.file_path != file_path {
                return Err("E_LOGIC_LINKED_SOURCE_SITE_MISMATCH".into());
            }
            Ok(program.clone())
        }else if let Some(context)=logic_context {
            ddonirang_lang::Parser::parse_with_logic_callable_context(&prepared,file_path,j1_profile.ok_or("E_LOGIC_SOURCE_PROFILE_REQUIRED")?,context)
        }else if let Some(profile)=j1_profile {
            ddonirang_lang::parser::Parser::parse_j1_v25_with_mode(&prepared,file_path,profile,mode)
        }else if let Some(owner)=identity_prepared { Ok(owner.into_program()) }
        else if let Some(context) = &first_callable_context {
            ddonirang_lang::parser::Parser::parse_with_first_callable_context(&prepared, file_path, context.clone())
        } else if let Some(context)=&public_callable_context {
            ddonirang_lang::parser::Parser::parse_with_public_callable_context(&prepared,file_path,context.clone())
        } else if v25 {
            parse_v25_with_mode(&prepared, file_path, mode)
        } else {
            parse_with_mode(&prepared, file_path, mode)
        }
        .map_err(&mut parse_failure)?;
        super::piecewise_calculation::verify_source_partitions(&program)
            .map_err(&mut parse_failure)?;
        let report = canonicalize(&mut program).map_err(&mut parse_failure)?;
        if v25 { top_level_decl_names = authored_top_level_decl_names(&program, &prepared); }
        if v25 { validate_v25_known_infix_operand_types(&program)?; }
        let mut parse_warnings: Vec<DdnParseWarning> = report
            .warnings
            .into_iter()
            .map(|warning| DdnParseWarning {
                code: warning.code.to_string(),
                message: warning.message,
                span_start: warning.span.start,
                span_end: warning.span.end,
            })
            .collect();
        parse_warnings.extend(phase_diagnostics::collect(&program));
        enforce_regex_age_gate(&program, age_target)?;
        enforce_assertion_age_gate(&program, age_target)?;
        enforce_state_machine_age_gate(&program, age_target)?;
        enforce_quantifier_age_gate(&program, age_target)?;
        let ast_fields = collect_public_observation_fields(&program);
        if (first_callable_context.is_some()||public_callable_context.is_some()) && !ast_fields.is_empty() {
            return Err("E_MULTILANG_OBSERVATION_SCOPE_UNSUPPORTED: 첫 함수의 관찰 의존 공급 범위를 벗어났습니다".into());
        }
        let original_fields = if v25 {
            source_validation::original_public_observation_fields_v25(source, &prepared, &ast_fields)?
        } else {
            scan_original_public_observation_fields(source)?
        };
        if original_fields.len() != ast_fields.len()
            || original_fields
                .iter()
                .zip(&ast_fields)
                .any(|(original, ast)| original.name != ast.name)
        {
            return Err(
                "E_BOIM_SOURCE_REF_MISMATCH: 공개 관찰 필드를 사용자 원문 위치와 연결할 수 없습니다"
                    .to_string(),
            );
        }
        let public_observation_source_refs = original_fields
            .into_iter()
            .zip(ast_fields)
            .map(|(original, ast)| (ast.id, original))
            .collect();
        let ordinary_callable_admission_evidence =
            ddonirang_lang::validate_ordinary_callable_admission(&program)
                .map_err(|failure| failure.diagnostic_message())?;
        let mut functions = HashMap::new();
        let mut products = HashMap::new();
        let mut record_schemas = BTreeMap::new();
        let mut ordinary_overloads = HashMap::<String, Vec<SeedDef>>::new();
        let root_stmts = program.root_stmts.iter().filter(|stmt|
            !(identity_calculations.active() && matches!(stmt,Stmt::Definition{..}))).cloned().collect();
        for item in &program.items {
            let ddonirang_lang::TopLevelItem::SeedDef(seed) = item;
            if matches!(seed.seed_kind,SeedKind::Product|SeedKind::Schema(_)) {
                let name = seed.canonical_name.clone();
                if seed.seed_kind==SeedKind::Product {
                    record_schemas.insert(name.clone(), seed.params.iter().map(|field|
                        (field.pin_name.clone(), field.type_ref.clone())).collect());
                }
                products.insert(name, seed.clone());
                continue;
            }
            for surface in seed.callable_surface_names() {
                functions.insert(surface.clone(), seed.clone());
                if ddonirang_lang::is_v1_ordinary_callable(seed) {
                    ordinary_overloads
                        .entry(surface)
                        .or_default()
                        .push(seed.clone());
                }
            }
        }
        let derived_plan=Arc::new(persistent_dependencies::DerivedPlan::compile(&program)?);
        let numeric_constraint_plan = if v25 {
            let mut numeric_failure=None;
            Some(Arc::new(crate::numeric_constraint_plan::NumericConstraintPlan::compile_with_host_diagnostic(&program,&top_level_decl_names,host_numeric_supply.as_ref(),&mut numeric_failure)
                .map_err(|error|{
                    if let (Some(slot),Some(mut diagnostic))=(diagnostic_out.as_deref_mut(),numeric_failure){
                        if let Some((start,end))=diagnostic.span{
                            diagnostic.span=Some((captured_source_offset(source,meta_parse.meta_lines,start),
                                captured_source_offset(source,meta_parse.meta_lines,end)));
                        }
                        let message=diagnostic.message.clone();*slot=Some(diagnostic);return message;
                    }
                    if error.starts_with("E_TYPE_MISMATCH:")
                        || error.starts_with("E_TUPLE_STATIC_SELECTION:")
                        || error.starts_with("E_SELECTION_BOUNDS:") {
                        // Preserve the established source diagnostic and exact
                        // selection span when that checker independently
                        // proves the same invalid source. Otherwise retain
                        // N1's fail-closed diagnostic; never turn it into Ok.
                        if let Some(slot)=diagnostic_out.as_deref_mut() {
                            if let Some(mut diagnostic)=crate::source_analysis::existing_static_rejection_diagnostic(&program,file_path) {
                                if let Some((start,end))=diagnostic.span {
                                    diagnostic.span=Some((captured_source_offset(source,meta_parse.meta_lines,start),
                                        captured_source_offset(source,meta_parse.meta_lines,end)));
                                }
                                let message=diagnostic.message.clone();*slot=Some(diagnostic);message
                            } else {error}
                        } else {crate::source_analysis::existing_static_rejection(&program).unwrap_or(error)}
                    } else {error}
                })?))
        } else { None };
        let source_identity=format!("sha256:{}", crate::runtime_surface::sha256_hex(source.as_bytes()));
        let piecewise_calculations=Arc::new(super::piecewise_calculation::CheckedCalculations::compile(
            &program,numeric_constraint_plan.as_deref(),&source_identity)
            .map_err(|error|format_parse_error(&prepared,&error))?);
        let first_callable_canonical_source = first_callable_context.as_ref().map(|context|
            ddonirang_lang::canonicalizer::canonicalize_first_callable_registered_source(&program, context)
                .map_err(|e| format!("{}", e))).transpose()?;
        let public_callable_canonical_source=public_callable_context.as_ref().map(|context|ddonirang_lang::canonicalizer::canonicalize_public_callable_registered_source(&program,context).map_err(|error|error.to_string())).transpose()?;
        Ok(Self {
            public_callable_context,public_callable_authored_owner:None,public_callable_canonical_source,
            source_identity,
            source_capture,
            first_callable_context,
            first_callable_authored_owner: None,
            first_callable_canonical_source,
            saved_project_sources: None,
            logic_source_scopes:None,
            nominal_constructors:BTreeMap::new(),
            registered_atom_literals:BTreeMap::new(),
            result_output_type_plan:None,
            temporary_registration:None,
            nominal_source_registration:None,
            temporary_subject_bootstrap:None,
            registered_nominal_catalog:None,
            registered_atom_contracts:Arc::new(Vec::new()),
            program,
            functions,
            ordinary_overloads,
            ordinary_callable_admission_evidence,
            products,
            record_schemas,
            root_stmts,
            derived_plan,
            top_level_decl_names,
            file_meta: meta_parse.meta,
            parse_warnings,
            configured_madi,
            public_observation_source_refs,
            numeric_constraint_plan,
            piecewise_calculations,
            identity_calculations,
            host_numeric_supply,
        })
    }

    pub fn parse_warnings(&self) -> &[DdnParseWarning] {
        &self.parse_warnings
    }

    pub(crate) fn sealed_source_capture(&self) -> Option<&crate::source_snapshot_admission::SourceCapture> {
        self.source_capture.as_ref()
    }

    pub(crate) fn sealed_project_sources(&self) -> Option<&crate::project_revision_envelope_supply::VerifiedSavedV25Sources> {
        self.saved_project_sources.as_deref()
    }

    pub(crate) fn nominal_source_registration(&self)->Option<&crate::authored_project::registration::nominal_source::RegisteredNominalSource>{
        self.nominal_source_registration.as_deref()
    }

    pub fn ordinary_callable_admission_evidence(
        &self,
    ) -> &[ddonirang_lang::OrdinaryCallableAdmissionEvidence] {
        &self.ordinary_callable_admission_evidence
    }

    pub(crate) fn canonical_program(&self) -> &CanonProgram {
        &self.program
    }

    pub fn configured_madi(&self) -> Option<u64> {
        self.configured_madi
    }

    pub fn has_root_program(&self) -> bool {
        !self.root_stmts.is_empty()
    }

    pub fn canonical_ddn(&self) -> String {
        if let Some(source)=&self.public_callable_canonical_source{return source.clone();}
        if let Some(source) = &self.first_callable_canonical_source { return source.clone(); }
        ddonirang_lang::normalize(&self.program, ddonirang_lang::NormalizationLevel::N1)
    }

    pub fn execution_mode_decision(&self) -> ddonirang_lang::ExecutionModeDecision {
        ddonirang_lang::decide_execution_mode(&self.program)
    }
}

#[cfg(test)]
mod multilang_first_runtime_tests {
    use super::*;
    use crate::runtime_surface::DEFAULT_UPDATE_NAME;
    use crate::authored_project::{ProjectArtifact, SourceFile, SourceSnapshot};
    const ORIGINAL: &str = "(총액합하다=총액합하다) 드러냄.\n(금액:나눔수~에, 세액:나눔수~을~를) 총액합하다{총액합하기}:나눔수:셈씨 := { 총액합하다 <- 금액 + 세액. }.\n";
    const DDL: &str = "#!日本語\n(金額:有理数~に, 税額:有理数~を) 合計する:有理数:関数 :={...}.\n";
    const SOURCE: &str = "#!日本語\n金額 <- (金額=(100,1) 有理数, 税額=(10,1) 有理数) 合計する.\n";

    fn original() -> ProjectArtifact {
        let created = ProjectArtifact::create("v1-core-v25".into(), SourceSnapshot { root_path: None,
            entry_path: "세금.ddn".into(), files: vec![SourceFile { path: "세금.ddn".into(), text: ORIGINAL.into() }] }, "세금".into()).unwrap();
        crate::authored_project::open(created.save().unwrap().as_bytes()).unwrap()
    }
    fn owner(original: &ProjectArtifact) -> Arc<crate::authored_project::FirstCallableAuthoredContext> {
        Arc::new(original.first_callable_registration_context(original.authored_identity.modules[0].id.as_str(),
            "세금.ddn", DDL, "세금.일본어.ddl", "language-profile:ja", 1).unwrap())
    }
    fn input() -> InputSnapshot {
        InputSnapshot { tick_id: 0, dt: Fixed64::from_i64(1), keys_pressed: 0, last_key_name: String::new(),
            pointer_x_i32: 0, pointer_y_i32: 0, ai_injections: Vec::new(), net_events: Vec::new(),
            frame_source: ddonirang_core::InputSource::Person, rng_seed: 0 }
    }
    fn value(source: &str, owner: Arc<crate::authored_project::FirstCallableAuthoredContext>) -> Value {
        let program = DdnProgram::from_first_callable_authored_source(source, "利用.ddn", owner.clone()).unwrap();
        assert!(Arc::ptr_eq(program.first_callable_authored_owner().unwrap(), &owner));
        assert_eq!(program.sealed_source_capture().unwrap().text(), source);
        assert_eq!(program.canonical_program().items.len(), 1);
        let canonical = program.canonical_ddn();
        assert!(canonical.starts_with("#!日本語\n") && !canonical.contains("총액합하다"));
        let again = DdnProgram::from_first_callable_authored_source(&canonical, "正本.ddn", owner.clone()).unwrap();
        assert_eq!(again.canonical_ddn(), canonical);
        let output = DdnRunner::new(program, DEFAULT_UPDATE_NAME).run_update(&NuriWorld::new(), &input(), &HashMap::new()).unwrap();
        assert!(!output.resources.contains_key("금액") && !output.resources.contains_key("세액"));
        output.resources["金額"].clone()
    }

    #[test]
    fn multilang_first_runtime_actual_registered_rational_call_and_required_rejections() {
        let original = original(); let owner = owner(&original);
        let actual = value(SOURCE, owner.clone());
        let exact = exact_numeric_from_value(&actual).unwrap().unwrap();
        assert_eq!(exact.value, BigRational::from_integer(BigInt::from(110)));
        let korean = format!("{}\n금액 <- (금액=(100,1) 나눔수, 세액=(10,1) 나눔수) 총액합하기.",
            owner.compiler_context().original_public().directives.declaration_source);
        let korean = DdnProgram::from_v25_source(&korean, "사용.ddn").unwrap();
        let korean = DdnRunner::new(korean, DEFAULT_UPDATE_NAME).run_update(&NuriWorld::new(), &input(), &HashMap::new()).unwrap();
        assert_eq!(actual, korean.resources["금액"]);
        let fractions = SOURCE.replace("(100,1)", "(1,3)").replace("(10,1)", "(1,6)");
        let actual = value(&fractions, owner.clone());
        assert_eq!(exact_numeric_from_value(&actual).unwrap().unwrap().value, BigRational::new(BigInt::from(1), BigInt::from(2)));
        for invalid in [SOURCE.replace(", 税額=(10,1) 有理数", ""),
            SOURCE.replace("税額=(10,1)", "金額=(10,1)"),
            SOURCE.replace("(100,1) 有理数", "\"100\""), SOURCE.replace("合計する", "total")] {
            assert!(DdnProgram::from_first_callable_authored_source(&invalid, "利用.ddn", owner.clone()).is_err(), "{invalid}");
        }
    }

    #[test]
    fn multilang_first_runtime_real_project_module_owners_reopen_and_same_spelling_stay_distinct() {
        let original = original(); let first = owner(&original);
        let pin = first.pin_source_span("金額").unwrap();
        assert_eq!(pin.source_path, "세금.ddn");
        assert!(pin.source_text[pin.span.start..pin.span.end].starts_with("금액:나눔수"));
        let definition = first.definition_source_span();
        assert!(definition.source_text[definition.span.start..definition.span.end].contains("총액합하다"));
        let reopened = crate::authored_project::open(original.save().unwrap().as_bytes()).unwrap();
        let restored = owner(&reopened);
        assert_eq!(first.project_identity(), restored.project_identity());
        assert_eq!(first.module_identity(), restored.module_identity());
        assert_eq!(value(SOURCE, first.clone()), value(SOURCE, restored));
        let duplicate = original.duplicate().unwrap();
        let duplicate = crate::authored_project::open(duplicate.save().unwrap().as_bytes()).unwrap();
        let other = owner(&duplicate);
        assert_ne!(first.module_identity(), other.module_identity());
        assert_ne!(first.project_identity(), other.project_identity());
        assert_eq!(value(SOURCE, first), value(SOURCE, other));
        assert!(original.first_callable_registration_context("not-an-issued-module", "세금.ddn", DDL,
            "세금.일본어.ddl", "language-profile:ja", 1).is_err());
        let mut stale = original.clone(); stale.source_snapshot.files[0].text.push(' ');
        assert!(stale.first_callable_registration_context(stale.authored_identity.modules[0].id.as_str(), "세금.ddn", DDL,
            "세금.일본어.ddl", "language-profile:ja", 1).err().unwrap().contains("E_MULTILANG_ORIGINAL_CHECKSUM_STALE"));
    }

    #[test]
    fn multilang_first_runtime_analysis_stale_context_is_not_a_current_binding_or_profile() {
        use crate::source_analysis::*;
        let original = original(); let first = owner(&original); let other = owner(&original);
        let request = || SourceAnalysisRequest { source_text: SOURCE.into(), path: "利用.ddn".into(),
            analysis_generation: 7, source_profile_identity: "v1-core-v25".into() };
        let analysis = analyze_source_with_first_callable_context(request(), first.compiler_context().clone());
        assert_eq!(analysis.snapshot().outcome, "ACCEPTED", "{:?}", analysis.snapshot());
        assert!(analysis.is_current_for(SOURCE, 7, first.compiler_context()));
        assert!(!analysis.is_current_for(SOURCE, 8, first.compiler_context()));
        assert!(!analysis.is_current_for(&(SOURCE.to_owned()+" "), 7, first.compiler_context()));
        assert!(!analysis.is_current_for(SOURCE, 7, other.compiler_context()));
        let declaration = analysis.original_declaration().unwrap();
        assert_eq!(declaration.pin("金額").unwrap().pin_name, "금액");
        let missing = analyze_source(request());
        assert_eq!(missing.outcome, "UNSUPPORTED");
        assert_eq!(missing.diagnostics[0].code, "E_MULTILANG_CONTEXT_REQUIRED");
        assert!(resolve_supported_source_profile(Some("1")).is_err());
        assert!(DdnProgram::from_v25_source(SOURCE, "利用.ddn").err().unwrap().contains("E_MULTILANG_CONTEXT_REQUIRED"));
    }
}


#[cfg(test)]
mod r31_j1_runtime_tests{
    use super::*;
    fn input()->ddonirang_core::InputSnapshot{ddonirang_core::InputSnapshot{tick_id:0,dt:ddonirang_core::Fixed64::from_i64(1),keys_pressed:0,last_key_name:String::new(),pointer_x_i32:0,pointer_y_i32:0,ai_injections:vec![],net_events:vec![],frame_source:ddonirang_core::InputSource::Person,rng_seed:0}}
    #[test]fn r31_j1_actual_common_runtime_and_explicit_profile(){
        let raw="#!日本語\n(金額:有理数~に, 税額:有理数~を) 合計:有理数:関数 := { 合計 <- 金額 + 税額. }.\n実行:手続き := { ((税額=(3,1) 有理数, 金額=(12,1) 有理数) 合計) 表示. 真 表示. 偽 表示. }.";
        let profile=resolve_supported_source_profile(Some(ddonirang_lang::ddl::J1_SOURCE_PROFILE)).unwrap();
        let program=DdnProgram::from_supported_source_profile_with_mode(raw,"main.ddn",ParseMode::Strict,profile).unwrap();
        let result=DdnRunner::new(program,"実行").run_update(&ddonirang_core::NuriWorld::new(),&input(),&std::collections::HashMap::new()).unwrap();
        assert_eq!(result.display_outputs.iter().map(|r|r.text.as_str()).collect::<Vec<_>>(),vec!["15/1","참","거짓"]);
        assert!(DdnProgram::from_supported_source_profile_with_mode(raw,"main.ddn",ParseMode::Strict,SupportedSourceProfile::V1CoreV25).is_err());
        assert!(DdnProgram::from_supported_source_profile_with_mode("実行:手続き := {}.","main.ddn",ParseMode::Strict,profile).is_err());
        assert!(resolve_supported_source_profile(Some(" v1-core-v25-ja-base-r1 ")).is_err());
        assert!(resolve_supported_source_profile(Some(" v1-core-v25 ")).is_ok());
    }
}

#[cfg(test)]
mod r31_j1_same_attempt_diagnostic_tests{
    use super::*;
    #[test]fn r31_j1_actual_parse_error_keeps_code_span_path_and_cause(){
        let raw="#!日本語\n((金額=(4,1) 有理数） 倍) 表示.\n";
        let error=match DdnProgram::from_j1_source_with_mode_diagnostic(raw,"same-attempt.ddn",ParseMode::Strict,default_age_target()){
            Ok(_)=>panic!("fullwidth accepted"),Err(error)=>error,
        };
        let start=raw.find('）').unwrap();assert_eq!(error.code,"E_MULTILANG_FULLWIDTH_SYNTAX");assert_eq!(error.span,Some((start,start+3)));
        assert_eq!(error.source_path,"same-attempt.ddn");let cause=error.cause.unwrap();assert_eq!(cause.code,error.code);assert_eq!(cause.span,error.span);
        assert!(DdnProgram::from_j1_source_with_mode_diagnostic("#!日本語\n実行:手続き := { 真 表示. }.","valid.ddn",ParseMode::Strict,default_age_target()).is_ok());
    }
}

#[cfg(test)]
mod r31_j1_static_diagnostic_tests {
    use super::*;
    #[test]
    fn r31_j1_static_type_rejection_keeps_same_attempt_utf8_span() {
        let raw="#!日本語\r\n実行:手続き := { (真 + 1) 表示. }.";
        let typed=DdnProgram::from_j1_source_with_mode_diagnostic(raw,"edited.ddn",ParseMode::Strict,default_age_target()).err().expect("type mismatch");
        let original=DdnProgram::from_supported_source_profile_with_mode(raw,"edited.ddn",ParseMode::Strict,SupportedSourceProfile::V1CoreV25JaBaseR1).err().expect("same rejection");
        assert_eq!(typed.code,"E_CHECK_TYPE_MISMATCH");assert_eq!(typed.message,original);
        let start=raw.find('真').unwrap();assert_eq!(typed.span,Some((start,start+"真 + 1".len())));
        assert_eq!(typed.source_path,"edited.ddn");assert_eq!(typed.phase,"execution_parse");
        let valid="#!日本語\r\n実行:手続き := { 真 表示. }.";
        assert!(DdnProgram::from_j1_source_with_mode_diagnostic(valid,"valid.ddn",ParseMode::Strict,default_age_target()).is_ok());
    }
}

#[cfg(test)]
mod ko_same_attempt_diagnostic_tests {
    use super::*;
    #[test]
    fn ko_admission_failure_is_not_fabricated_as_a_parser_span() {
        let source="#설명: 금지된 과거 머리\n4 보여주기.";
        let error=DdnProgram::from_supported_source_profile_diagnostic(source,"머리.또니",ParseMode::Strict,SupportedSourceProfile::V1CoreV25,default_age_target()).err().unwrap();
        assert!(error.message.contains("E_FRONTDOOR_LEGACY_HEADER_FORBIDDEN"));
        assert_eq!(error.span,None);assert!(error.cause.is_none());
    }
    #[test]
    fn ko_same_attempt_error_preserves_original_utf8_metadata_and_new_attempt() {
        let raw="기준값 := 4.\n결과 := 기준값 * 2.\n계산 결과 보여주기.\n";
        for prefix in ["", "#제목: 한글 머리\r\n", "#제목: 다른 판\n#메모: 두 머리\n"] {
            let source=format!("{prefix}{raw}");
            let error=DdnProgram::from_supported_source_profile_diagnostic(&source,"값과글.또니",ParseMode::Strict,SupportedSourceProfile::V1CoreV25,default_age_target()).err().expect("invalid sentence");
            let plain=DdnProgram::from_supported_source_profile_with_mode(&source,"값과글.또니",ParseMode::Strict,SupportedSourceProfile::V1CoreV25).err().unwrap();
            assert_eq!(error.message,plain);assert_eq!(error.code,"E_PARSE");
            let start=source.find("결과 보여주기").unwrap();assert_eq!(error.span,Some((start,start+"결과".len())));
            assert_eq!(error.source_path,"값과글.또니");assert_eq!(error.phase,"execution_parse");
            let cause=error.cause.unwrap();assert_eq!(cause.span,error.span);assert_eq!(cause.code,error.code);
        }
        assert!(DdnProgram::from_supported_source_profile_diagnostic("4 보여주기.","정상.또니",ParseMode::Strict,SupportedSourceProfile::V1CoreV25,default_age_target()).is_ok());
    }
}
