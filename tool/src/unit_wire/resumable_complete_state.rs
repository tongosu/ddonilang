//! Owner-approved private successor implementation, R1 through R2.1a.
//! Layout and boundary checks are prerequisites, never full resume admission.
//! Source/input replay and confirmed checkpoint provenance must still succeed
//! before a typed capsule, storage I/O, runner or world publication is allowed.

use crate::snapshot_v2_durable_transaction::SnapshotV2DurabilityFailureV1 as Failure;
use crate::work_cost::{StandardResourceLimitsV1, WorkCostCounters, WorkCostReceipt};
use super::complete_state_frame::ResumableCompleteStateLayoutV2;

struct Cursor<'a> { bytes: &'a [u8], offset: usize }
impl<'a> Cursor<'a> {
    fn take(&mut self, length: usize) -> Result<&'a [u8], Failure> {
        let end=self.offset.checked_add(length).ok_or(Failure::LimitExceeded)?;
        let bytes=self.bytes.get(self.offset..end).ok_or(Failure::InvalidInput)?;
        self.offset=end;Ok(bytes)
    }
    fn byte(&mut self)->Result<u8,Failure>{Ok(self.take(1)?[0])}
    fn version(&mut self)->Result<(),Failure>{
        if self.take(2)?==[0,1]{Ok(())}else{Err(Failure::InvalidInput)}
    }
    fn u64(&mut self)->Result<u64,Failure>{
        Ok(u64::from_be_bytes(self.take(8)?.try_into().map_err(|_|Failure::InvalidInput)?))
    }
    fn bytes32(&mut self)->Result<&'a [u8],Failure>{
        let length=u32::from_be_bytes(self.take(4)?.try_into().map_err(|_|Failure::InvalidInput)?);
        self.take(usize::try_from(length).map_err(|_|Failure::LimitExceeded)?)
    }
    fn boolean(&mut self)->Result<bool,Failure>{match self.byte()?{
        0=>Ok(false),1=>Ok(true),_=>Err(Failure::InvalidInput),
    }}
    fn finish(self)->Result<(),Failure>{
        if self.offset==self.bytes.len(){Ok(())}else{Err(Failure::InvalidInput)}
    }
}

const CORE_INPUT_PROFILE: &[u8] = b"ddn.resume.core-input-frames.person.v1";
const LOGICAL_FIXED_SEED_PROFILE: &[u8] = b"ddn.resume.logical-fixed-seed.v1";
const GENERAL_INPUT_PROFILE: &[u8] = b"ddn.resume.core-input-frames.general-person.v1";
// Product-private discriminator of the observed ordinary runner strategy.
// This is not an RNG resource identity or a new execution-profile issuer.
pub(crate) const GENERAL_XOR_RNG_PROFILE:&[u8]=b"ddn.resume.rng.per-madi-xor.v1";

/// Owner-approved field0d v3. These rows are candidates, not provenance or
/// continuation admission. Preserve absent edges independently of held keys.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct GeneralProcessedInputRow {
    pub(crate) input: ddonirang_core::platform::InputSnapshot,
    pub(crate) edges: Option<(u64, u64)>,
}

pub(crate) fn encode_general_processed_inputs(
    rows: &[GeneralProcessedInputRow], initial_tick: u64, base_seed: u64,
) -> Result<Vec<u8>, Failure> {
    let mut out = vec![0, 3];
    append_bytes32(&mut out, GENERAL_INPUT_PROFILE)?;
    append_supply(&mut out, &u32::try_from(rows.len()).map_err(|_| Failure::LimitExceeded)?.to_be_bytes())?;
    initial_tick.checked_add(u64::try_from(rows.len()).map_err(|_| Failure::LimitExceeded)?)
        .ok_or(Failure::LimitExceeded)?;
    for (index, row) in rows.iter().enumerate() {
        let input = &row.input;
        let tick = initial_tick.checked_add(u64::try_from(index).map_err(|_| Failure::LimitExceeded)?)
            .ok_or(Failure::LimitExceeded)?;
        if input.tick_id != tick || input.rng_seed != base_seed ^ tick {
            return Err(Failure::CustodyConflict);
        }
        if input.frame_source != ddonirang_core::InputSource::Person
            || !input.ai_injections.is_empty() || !input.net_events.is_empty() {
            return Err(Failure::InvalidInput);
        }
        for bytes in [input.tick_id.to_be_bytes(), input.dt.raw_i64().to_be_bytes(), input.keys_pressed.to_be_bytes()] {
            append_supply(&mut out, &bytes)?;
        }
        append_bytes32(&mut out, input.last_key_name.as_bytes())?;
        append_supply(&mut out, &input.pointer_x_i32.to_be_bytes())?;
        append_supply(&mut out, &input.pointer_y_i32.to_be_bytes())?;
        append_supply(&mut out, &[0])?;
        append_supply(&mut out, &input.rng_seed.to_be_bytes())?;
        append_supply(&mut out, &[0; 8])?;
        append_supply(&mut out, &[u8::from(row.edges.is_some())])?;
        if let Some((pressed, released)) = row.edges {
            append_supply(&mut out, &pressed.to_be_bytes())?;
            append_supply(&mut out, &released.to_be_bytes())?;
        }
    }
    Ok(out)
}

pub(crate) fn decode_general_processed_inputs(
    bytes: &[u8], initial_tick: u64, base_seed: u64,
) -> Result<Vec<GeneralProcessedInputRow>, Failure> {
    if bytes.len() > 2_000_000 { return Err(Failure::LimitExceeded); }
    let mut c = Cursor { bytes, offset: 0 };
    if c.take(2)? != [0, 3] || c.bytes32()? != GENERAL_INPUT_PROFILE {
        return Err(Failure::InvalidInput);
    }
    let count = u32::from_be_bytes(c.take(4)?.try_into().map_err(|_| Failure::InvalidInput)?);
    let count = usize::try_from(count).map_err(|_| Failure::LimitExceeded)?;
    // Each row needs at least 54 bytes, before text or explicit edges.
    if count > (bytes.len() - c.offset) / 54 { return Err(Failure::InvalidInput); }
    let mut rows = Vec::new();
    rows.try_reserve_exact(count).map_err(|_| Failure::LimitExceeded)?;
    for _ in 0..count {
        let tick_id = c.u64()?;
        let dt = ddonirang_core::Fixed64::from_raw_i64(i64::from_be_bytes(c.take(8)?.try_into().map_err(|_| Failure::InvalidInput)?));
        let keys_pressed = c.u64()?;
        let last_key_name = std::str::from_utf8(c.bytes32()?).map_err(|_| Failure::InvalidInput)?.to_owned();
        let pointer_x_i32 = i32::from_be_bytes(c.take(4)?.try_into().map_err(|_| Failure::InvalidInput)?);
        let pointer_y_i32 = i32::from_be_bytes(c.take(4)?.try_into().map_err(|_| Failure::InvalidInput)?);
        if c.byte()? != 0 { return Err(Failure::InvalidInput); }
        let rng_seed = c.u64()?;
        if c.take(8)? != [0; 8] { return Err(Failure::InvalidInput); }
        let edges = if c.boolean()? { Some((c.u64()?, c.u64()?)) } else { None };
        rows.push(GeneralProcessedInputRow { input: ddonirang_core::platform::InputSnapshot {
            tick_id, dt, keys_pressed, last_key_name, pointer_x_i32, pointer_y_i32,
            ai_injections: Vec::new(), net_events: Vec::new(),
            frame_source: ddonirang_core::InputSource::Person, rng_seed,
        }, edges });
    }
    c.finish()?;
    if encode_general_processed_inputs(&rows, initial_tick, base_seed)? != bytes {
        return Err(Failure::CustodyConflict);
    }
    Ok(rows)
}

/// Shared existing V1 eight-field candidate writer. Unsupported world lanes
/// are rejected, never flattened into the typed resource hash lane.
pub(crate) fn encode_resource_only_world_candidate(
    world: &ddonirang_core::platform::NuriWorld, pesi: &[u8;32], next_tick: u64,
) -> Result<Vec<u8>, Failure> {
    if world.subject_world().is_some(){return Err(Failure::InvalidInput);}
    encode_owned_world_candidate_with_catalog(world,pesi,next_tick,None)
}
pub(crate) fn encode_owned_world_candidate_with_catalog(
    world:&ddonirang_core::platform::NuriWorld,pesi:&[u8;32],next_tick:u64,
    catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>,
)->Result<Vec<u8>,Failure>{
    encode_owned_world_candidate_with_evidence(world,pesi,next_tick,catalog,&[])
}
pub(crate) fn recorded_source_atom_evidence(owned:&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1)->Result<Vec<u8>,Failure>{
    let Some(registration)=owned.registration() else{return Ok(Vec::new())};
    let contracts=registration.verified_recorded_atom_contracts(owned).map_err(|_|Failure::AtomEvidenceInvalid)?;
    // Preserve the existing absent lane when this source has no contracts.
    if contracts.is_empty(){return Ok(Vec::new());}
    super::complete_state_tables::encode_registered_atom_evidence(contracts)
}
pub(crate) fn encode_owned_world_candidate_with_source(world:&ddonirang_core::platform::NuriWorld,
    pesi:&[u8;32],next_tick:u64,owned:&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
)->Result<Vec<u8>,Failure>{
    let catalog=owned.registration().map(|registration|registration.saved_nominal_catalog(owned)).transpose().map_err(|_|Failure::CustodyConflict)?;
    let evidence=recorded_source_atom_evidence(owned)?;
    encode_owned_world_candidate_with_evidence(world,pesi,next_tick,catalog.as_ref(),&evidence)
}
fn encode_owned_world_candidate_with_evidence(world:&ddonirang_core::platform::NuriWorld,pesi:&[u8;32],next_tick:u64,
    catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>,evidence:&[u8],
)->Result<Vec<u8>,Failure>{
    use ddonirang_core::platform::ResourceValue;
    if !world.resource_json_entries().is_empty() || !world.resource_handle_entries().is_empty()
        || !world.component_json_entries().is_empty() || world.next_entity_cursor()!=0 {
        return Err(Failure::InvalidInput);
    }
    let mut values = std::collections::BTreeMap::new();
    for (name,value) in world.resource_fixed64_entries() { values.insert(name,ResourceValue::Fixed64(value)); }
    for (name,value) in world.resource_value_entries() {
        if values.insert(name,value).is_some() { return Err(Failure::InvalidInput); }
    }
    let empty = [0;8];
    let subjects=world.subject_world().map(|state|state.subject_values.as_slice()).unwrap_or(&empty);
    let custody_bytes=world.subject_world().map(|state|state.subject_custody.as_slice()).unwrap_or(&[]);
    let custody=super::complete_state_tables::decode_saved_world_custody(custody_bytes,pesi)?;
    let references=custody.as_ref().map(crate::subject_reference_owner::SubjectReferenceContextV1::new).transpose()?;
    let contracts=super::complete_state_tables::decode_registered_atom_evidence(evidence)?;
    let resources = super::complete_state_tables::encode_resource_table_with_context(&values,catalog,references.as_ref(),&contracts)?;
    super::complete_state_frame::CompleteStateFrameV1 {pesi,resources:&resources,parameters:&empty,
        next_tick,subjects,atom_evidence:evidence,subject_custody:custody_bytes}.encode_canonical_with_catalog(catalog)
}

/// R2.1 checkpoint provenance belongs to this actual run, not to a saved
/// project revision, SQL query, host boolean, or accepted-madi count.
/// Deliberately non-Clone and inseparable from the owned execution session.
struct VerifiedRunCheckpointProvenance {
    source_owner: crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    initial_binding: crate::logical_input_tape::LogicalInputBinding,
    execution_instance: Vec<u8>,
    pesi: [u8;32],
    source_descriptor_prefix: Vec<u8>,
    prior_durable_receipts: Vec<Vec<u8>>,
    // Source/prefix reconstruction in browser storage is not a Native receipt.
    local_restored: bool,
}

/// Existing field12 construction grammar, supplied from verified fresh source
/// and the actual runner. These comparison bytes do not issue fresh provenance;
/// the caller must be the retained fresh-run owner (never a restored capsule).
pub(crate) fn encode_general_fresh_source_descriptor(
    owned:&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    runner:&crate::ddn_runtime::DdnRunner,
    age:ddonirang_lang::AgeTarget,base_seed:u64,successful_madi_count:u64,
)->Result<Vec<u8>,Failure> {
    encode_execution_contract_supply(owned,runner)?;
    if runner.configured_saved_start()!=Some(owned.sources().entry_declaration().name()) {
        return Err(Failure::CustodyConflict);
    }
    let mut program=crate::ddn_runtime::DdnProgram::from_saved_v25_project_with_mode(
        owned.sources().clone(),ddonirang_lang::ParseMode::Strict,age,
    ).map_err(|_|Failure::InvalidInput)?;
    program.bind_registered_project_context(owned).map_err(|_|Failure::CustodyConflict)?;
    let preparation=owned.sources().preparation();
    crate::runtime_surface::prepare_supported_source_for_execution_with_program(
        &preparation.source,&preparation.target_path,&program,
    ).map_err(|_|Failure::InvalidInput)?;
    let canonical=program.canonical_ddn();let decision=program.execution_mode_decision();
    let issued=crate::project_snapshot_identity::current::issue_saved_local_project(owned)
        .map_err(|_|Failure::InvalidInput)?;
    let empty=[0;8];
    let subject_initial=runner.saved_initial_subject_world().map_err(|_|Failure::SubjectCustodyInvalid)?;
    let atom_evidence=recorded_source_atom_evidence(owned)?;
    let catalog=owned.registration().map(|registration|registration.saved_nominal_catalog(owned)).transpose().map_err(|_|Failure::CustodyConflict)?;
    let initial=super::complete_state_frame::CompleteStateFrameV1 {
        pesi:issued.pesi().as_raw(),resources:&empty,parameters:&empty,next_tick:0,
        subjects:subject_initial.as_ref().map(|world|world.subject_values.as_slice()).unwrap_or(&empty),
        atom_evidence:&atom_evidence,subject_custody:subject_initial.as_ref().map(|world|world.subject_custody.as_slice()).unwrap_or(&[]),
    }.encode_canonical_with_catalog(catalog.as_ref())?;
    let mut out=vec![0,2];
    append_bytes32(&mut out,b"ddn.resume.source-rebuild-descriptor.v1")?;
    append_bytes32(&mut out,b"v1-core-v25")?;
    let age_byte=match age {ddonirang_lang::AgeTarget::Age0=>0,ddonirang_lang::AgeTarget::Age1=>1,
        ddonirang_lang::AgeTarget::Age2=>2,ddonirang_lang::AgeTarget::Age3=>3,
        ddonirang_lang::AgeTarget::Age4=>4,ddonirang_lang::AgeTarget::Age5=>5,
        ddonirang_lang::AgeTarget::Age6=>6,ddonirang_lang::AgeTarget::Age7=>7};
    append_supply(&mut out,&[age_byte,1])?;append_bytes32(&mut out,canonical.as_bytes())?;
    append_supply(&mut out,&[match decision.mode {ddonirang_lang::ExecutionMode::OneShot=>1,
        ddonirang_lang::ExecutionMode::Live=>2}])?;
    for hook in [decision.live_hook_count,decision.every_madi_hook_count,decision.every_n_madi_hook_count] {
        append_supply(&mut out,&hook.to_be_bytes())?;
    }
    for _ in 0..2 {append_bytes32(&mut out,owned.definition_identity().as_bytes())?;}
    append_supply(&mut out,&[1])?;
    append_bytes32(&mut out,owned.project().save().map_err(|_|Failure::InvalidInput)?.as_bytes())?;
    append_bytes32(&mut out,&initial)?;append_supply(&mut out,&0u64.to_be_bytes())?;
    append_supply(&mut out,&base_seed.to_be_bytes())?;
    append_supply(&mut out,&[u8::from(successful_madi_count>0)])?;
    append_supply(&mut out,&successful_madi_count.to_be_bytes())?;
    append_bytes32(&mut out,runner.execution_instance_owner_bytes())?;
    append_supply(&mut out,&0u32.to_be_bytes())?;
    let rebuilt=rebuild_source_descriptor_candidate(&out,owned)?;
    if rebuilt.execution_instance!=runner.execution_instance_owner_bytes() {
        return Err(Failure::CustodyConflict);
    }
    Ok(out)
}

/// Exact existing fields 0e/0f/10, supplied from the actual retained general
/// owner. The caller must still stage replay/source/carrier/custody checks.
pub(crate) fn encode_general_boundary_supplies(
    owned:&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    runner:&crate::ddn_runtime::DdnRunner,rows:&[GeneralProcessedInputRow],base:u64,next:u64,
)->Result<[Vec<u8>;3],Failure> {
    encode_execution_contract_supply(owned,runner)?;
    encode_general_processed_inputs(rows,0,base)?;
    let receipt=runner.work_cost_receipt();
    if next!=u64::try_from(rows.len()).map_err(|_|Failure::LimitExceeded)?
        || receipt.counters.committed_madi!=next || receipt.counters.call_depth_current!=0
        || runner.configured_saved_start()!=Some(owned.sources().entry_declaration().name()) {
        return Err(Failure::CustodyConflict);
    }
    let mut rng=vec![0,1,1];append_bytes32(&mut rng,GENERAL_XOR_RNG_PROFILE)?;
    for value in [base,next,base^next,0] {append_supply(&mut rng,&value.to_be_bytes())?;}
    let definition=owned.definition_identity().as_bytes();
    let mut position=vec![0,1,1];append_supply(&mut position,&next.to_be_bytes())?;
    for _ in 0..2 {append_bytes32(&mut position,definition)?;}
    append_supply(&mut position,&[0])?;
    let work=encode_work_receipt_supply(&receipt)?;
    decode_boundary_sections(&rng,&position,&work,next,GENERAL_XOR_RNG_PROFILE,
        definition,definition,&receipt)?;
    Ok([rng,position,work])
}

/// Existing field12 receipt grammar. The caller must supply only checkpoint
/// receipts confirmed by its Native owner, never receipts read from a capsule.
pub(crate) fn encode_general_source_descriptor_with_receipts(
    owned:&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    runner:&crate::ddn_runtime::DdnRunner,age:ddonirang_lang::AgeTarget,
    base:u64,count:u64,receipts:&[Vec<u8>],
)->Result<Vec<u8>,Failure> {
    let mut out=encode_general_fresh_source_descriptor(owned,runner,age,base,count)?;
    // The original encoder ends in the existing empty receipt-count word.
    out.truncate(out.len().checked_sub(4).ok_or(Failure::InvalidInput)?);
    append_supply(&mut out,&u32::try_from(receipts.len()).map_err(|_|Failure::LimitExceeded)?.to_be_bytes())?;
    for receipt in receipts {append_bytes32(&mut out,receipt)?;}
    let rebuilt=rebuild_source_descriptor_candidate(&out,owned)?;
    if rebuilt.prior_durable_receipts.iter().copied().ne(receipts.iter().map(Vec::as_slice)) {
        return Err(Failure::CustodyConflict);
    }
    Ok(out)
}

pub(crate) fn encode_general_predecessor(receipts:&[Vec<u8>])->Result<Vec<u8>,Failure> {
    let mut out=vec![u8::from(!receipts.is_empty())];
    if let Some(last)=receipts.last(){append_bytes32(&mut out,last)?;}
    Ok(out)
}

pub(crate) struct CheckpointedLogicalRunV1 {
    session: crate::logical_input_tape::LogicalInputSession,
    provenance: VerifiedRunCheckpointProvenance,
}

impl CheckpointedLogicalRunV1 {
    /// Create a genuinely new execution from verified saved sources. No API
    /// accepts a live/restored session and relabels it as checkpoint-free.
    pub(crate) fn fresh(
        owned: &crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
        binding: crate::logical_input_tape::LogicalInputBinding,
        age: ddonirang_lang::AgeTarget,
    ) -> Result<Self,Failure> {
        let mut program = crate::ddn_runtime::DdnProgram::from_saved_v25_project_with_mode(
            owned.sources().clone(),ddonirang_lang::ParseMode::Strict,age,
        ).map_err(|_|Failure::InvalidInput)?;
        program.bind_registered_project_context(owned).map_err(|_|Failure::CustodyConflict)?;
        let preparation = owned.sources().preparation();
        crate::runtime_surface::prepare_supported_source_for_execution_with_program(
            &preparation.source,&preparation.target_path,&program,
        ).map_err(|_|Failure::InvalidInput)?;
        let canonical = program.canonical_ddn();
        let decision = program.execution_mode_decision();
        let execution_mode = match decision.mode {ddonirang_lang::ExecutionMode::OneShot=>1,
            ddonirang_lang::ExecutionMode::Live=>2};
        let hooks = [decision.live_hook_count,decision.every_madi_hook_count,decision.every_n_madi_hook_count];
        let start = owned.sources().entry_declaration().name();
        let mut runner = crate::ddn_runtime::DdnRunner::new(program,start);
        runner.configure_project_entry(start).map_err(|_|Failure::InvalidInput)?;
        encode_execution_contract_supply(owned,&runner)?;
        let execution_instance = runner.execution_instance_owner_bytes().to_vec();
        let issued = crate::project_snapshot_identity::current::issue_saved_local_project(owned)
            .map_err(|_|Failure::InvalidInput)?;
        let empty = [0;8];
        let initial = super::complete_state_frame::CompleteStateFrameV1 {
            pesi:issued.pesi().as_raw(),resources:&empty,parameters:&empty,next_tick:0,
            subjects:&empty,atom_evidence:&[],subject_custody:&[],
        }.encode_canonical()?;
        let mut prefix=vec![0,2];
        append_bytes32(&mut prefix,b"ddn.resume.source-rebuild-descriptor.v1")?;
        append_bytes32(&mut prefix,b"v1-core-v25")?;
        let age_byte=match age {ddonirang_lang::AgeTarget::Age0=>0,ddonirang_lang::AgeTarget::Age1=>1,
            ddonirang_lang::AgeTarget::Age2=>2,ddonirang_lang::AgeTarget::Age3=>3,
            ddonirang_lang::AgeTarget::Age4=>4,ddonirang_lang::AgeTarget::Age5=>5,
            ddonirang_lang::AgeTarget::Age6=>6,ddonirang_lang::AgeTarget::Age7=>7};
        append_supply(&mut prefix,&[age_byte,1])?;
        append_bytes32(&mut prefix,canonical.as_bytes())?;
        append_supply(&mut prefix,&[execution_mode])?;
        for hook in hooks {append_supply(&mut prefix,&hook.to_be_bytes())?;}
        for _ in 0..2 {append_bytes32(&mut prefix,owned.definition_identity().as_bytes())?;}
        append_supply(&mut prefix,&[1])?;
        let authored=owned.project().save().map_err(|_|Failure::InvalidInput)?;
        append_bytes32(&mut prefix,authored.as_bytes())?;append_bytes32(&mut prefix,&initial)?;
        append_supply(&mut prefix,&0u64.to_be_bytes())?;
        append_supply(&mut prefix,&binding.initial_rng.to_be_bytes())?;
        let session = crate::logical_input_tape::LogicalInputSession::from_saved_runner(
            owned,binding.clone(),runner,
        ).map_err(|_|Failure::InvalidInput)?;
        Ok(Self {session,provenance:VerifiedRunCheckpointProvenance {
            source_owner:owned.clone(),initial_binding:binding,execution_instance,
            pesi:*issued.pesi().as_raw(),source_descriptor_prefix:prefix,prior_durable_receipts:Vec::new(),local_restored:false,
        }})
    }

    pub(crate) fn step(&mut self,input:&crate::logical_input_tape::LogicalSnapshot)
        -> Result<serde_json::Value,String> {
        // Accepted madi changes execution, never durable checkpoint history.
        self.session.step(input)
    }
    pub(crate) fn replay_v3_frame(&mut self,frame:&crate::logical_input_tape::LogicalFrame)
        -> Result<serde_json::Value,String> {
        // Same authored execution owner and pre-commit frame comparison.
        // Replaying input does not create a durable checkpoint history.
        self.session.replay_v3_frame(frame)
    }

    pub(crate) fn reject_input(&mut self,reason:String) {
        self.session.reject_input(reason);
    }

    /// Construct supplies from this actual run rather than a caller's input,
    /// seed, counters or checkpoint labels. The supplied world frame remains
    /// untrusted until full replay/source/resource checks succeed. No I/O.
    pub(crate) fn check_checkpoint_channels(&self)->Result<(),String> {
        let world=self.session.committed_world();
        let legacy=world.resource_json_entries();
        if !legacy.is_empty() {
            let names:Vec<_>=legacy.iter().take(8).map(|(name,_)|name.as_str()).collect();
            return Err(format!("E_PROJECT_RUNTIME_LEGACY_CHANNEL_UNSUPPORTED: 기존 글·참거짓 저장 채널을 보존하는 규격이 아직 연결되지 않았습니다 ({}). 값을 누락하거나 바꾸어 저장하지 않습니다.",names.join(", ")));
        }
        if !world.resource_handle_entries().is_empty() || !world.component_json_entries().is_empty()
            || world.next_entity_cursor()!=0 {
            return Err("E_PROJECT_RUNTIME_WORLD_CHANNEL_UNSUPPORTED: 자원 핸들·개체·구성요소의 검증된 저장 공급이 필요합니다. 부분 저장은 하지 않습니다.".into());
        }
        Ok(())
    }

    pub(crate) fn encode_world_frame(&self)->Result<Vec<u8>,Failure> {
        let tick=self.session.resume_boundary_next_tick(self.provenance.source_owner.sources().entry_declaration().name())
            .map_err(|_|Failure::InvalidInput)?;
        let world=self.session.committed_world();
        self.check_checkpoint_channels().map_err(|_|Failure::InvalidInput)?;
        let bytes=encode_resource_only_world_candidate(world,&self.provenance.pesi,tick)?;
        let frame=super::complete_state_frame::check_complete_state_frame_v1(&bytes,&self.provenance.pesi)?;
        check_replayed_world_candidate(&frame,&self.session)?;
        Ok(bytes)
    }

    /// Browser/local reconstruction cannot mint a Native retained-start right.
    pub(crate) fn require_native_checkpoint_provenance(&self)->Result<(),Failure> {
        if self.provenance.local_restored {Err(Failure::CustodyConflict)}else{Ok(())}
    }

    pub(crate) fn export_saved_artifact(&self,parent:&[u8],revision:&[u8])->Result<String,Failure> {
        // Validate the complete world before allocating its JSON projection;
        // unsupported channels never disappear into an empty saved carrier.
        self.encode_world_frame()?;
        let (runtime,resources)=self.session.saved_runtime_carrier().map_err(|_|Failure::InvalidInput)?;
        super::project_runtime_attachment::encode_saved_runtime_artifact(
            &self.provenance.source_owner,&runtime,&resources,parent,revision).map_err(|_|Failure::InvalidInput)
    }

    pub(crate) fn encode_checkpoint_request(
        &self,world_bytes:&[u8],
        context:&super::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        parent:&[u8],revision:&[u8],
    )->Result<Vec<u8>,Failure> {
        use sha2::Digest;
        let owned=&self.provenance.source_owner;
        let descriptor=self.source_rebuild_descriptor_supply()?;
        let rebuilt=rebuild_source_descriptor_candidate(&descriptor,owned)?;
        let supplies=[context.revision_envelope().to_vec(),encode_source_capture_supply(owned.sources())?,
            encode_constituent_supply(owned)?,encode_logical_execution_contract_supply(owned,&rebuilt.runner)?,
            encode_processed_input_supply(&self.session)?,self.logical_rng_supply()?,
            self.execution_position_supply()?,encode_replayed_work_supply(&self.session)?,
            self.checkpoint_predecessor_supply()?,descriptor];
        let world=super::complete_state_frame::check_complete_state_frame_v1(world_bytes,&self.provenance.pesi)?;
        let layout=ResumableCompleteStateLayoutV2 {world,supplies:std::array::from_fn(|i|supplies[i].as_slice())};
        let snapshot=layout.encode_canonical()?;
        let envelope:serde_json::Value=serde_json::from_slice(context.revision_envelope()).map_err(|_|Failure::InvalidInput)?;
        let integrity=envelope["revision_integrity_sha256"].as_str().ok_or(Failure::InvalidInput)?;
        let binding=crate::snapshot_v2_durable_transaction::TransactionBindingV1 {
            project:hex::decode(owned.project().authored_identity.logical_dependency_id.as_str()).map_err(|_|Failure::InvalidInput)?,
            expected_parent:parent.to_vec(),new_revision:revision.to_vec(),
            execution_profile:owned.project().source_profile.as_bytes().to_vec(),pesi:self.provenance.pesi,
            snapshot_sha256:sha2::Sha256::digest(&snapshot).into(),snapshot_length:snapshot.len() as u64,
            revision_integrity:hex::decode(integrity).map_err(|_|Failure::InvalidInput)?.try_into().map_err(|_|Failure::InvalidInput)?,
        };
        let candidate=crate::snapshot_v2_host_bridge::ResumableRequestCandidateV2 {
            binding,layout,revision_bytes:context.revision_envelope(),request_kind:1,snapshot_bytes:&snapshot,
        };
        self.check_active_request_execution_candidate(&candidate,context)?;
        candidate.encode_canonical()
    }

    /// Native storage is reachable only after source, actual input, RNG,
    /// runtime replay, work and checkpoint lineage have all been admitted.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn save_native_checkpoint(
        &mut self,raw:&[u8],
        context:&super::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        adapter:&crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,
        expected_project_head:Option<&crate::snapshot_v2_durable_transaction::native::StoredHeadV1>,
    ) -> Result<Vec<u8>,crate::snapshot_v2_durable_transaction::NativeTransactionFailureV1> {
        use crate::snapshot_v2_durable_transaction::{native::StoredPayloadV1,commit_native_evidence};
        use crate::snapshot_v2_host_bridge as bridge;
        self.require_native_checkpoint_provenance()?;
        let request=bridge::check_resumable_request_candidate_v2(raw,&self.provenance.pesi)?;
        if request.request_kind!=1 {return Err(Failure::InvalidInput.into());}
        self.check_active_request_execution_candidate(&request,context)?;
        // Project CAS and this execution's checkpoint predecessor are distinct
        // owners (R2.1/B01). A fresh run on an existing project has no runtime
        // checkpoint but may have a nonempty expected project head.
        if expected_project_head.map(|h|h.revision.as_slice()).unwrap_or(&[])!=request.binding.expected_parent
            || expected_project_head.is_some_and(|h|h.project!=request.binding.project) {
            return Err(Failure::CustodyConflict.into());
        }
        let profile=adapter.capability()?;
        let receipt=bridge::encode_committed_receipt_candidate(&request.binding,profile.as_bytes())?;
        let returned=receipt.clone();
        self.provenance.prior_durable_receipts.try_reserve(1).map_err(|_|Failure::LimitExceeded)?;
        let payload=StoredPayloadV1 {digest:request.binding.transaction_digest()?,binding:request.binding.clone(),
            snapshot:request.snapshot_bytes.to_vec(),envelope:request.revision_bytes.to_vec()};
        let evidence=commit_native_evidence(adapter,&payload,expected_project_head,profile.as_bytes())?;
        evidence.check_committed_receipt_evidence(&receipt,profile.as_bytes())?;
        self.provenance.prior_durable_receipts.push(receipt);
        Ok(returned)
    }

    pub(crate) fn session(&self)->&crate::logical_input_tape::LogicalInputSession {
        &self.session
    }

    /// Browser/local storage only: reconstruct the original source and full
    /// processed input prefix. The independently selected input contract is
    /// mandatory. This cannot issue or inherit a Native durable receipt.
    pub(crate) fn restore_local_checkpoint(
        raw:&[u8],
        context:&super::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        expected:&crate::logical_input_tape::LogicalInputBinding,
    )->Result<Self,Failure> {
        let request=crate::snapshot_v2_host_bridge::check_resumable_request_candidate_v2(
            raw,context.pesi().as_raw())?;
        if request.request_kind!=1 {return Err(Failure::InvalidInput);}
        let candidate=rebuild_complete_state_source_candidate(&request.layout,context)?;
        if !candidate.prior_durable_receipts.is_empty() || request.layout.supplies[8]!=[0] {
            return Err(Failure::CustodyConflict);
        }
        let session=replay_complete_state_candidate(&request.layout,context,expected)?;
        let restored=Self {session,provenance:VerifiedRunCheckpointProvenance {
            source_owner:context.owned().clone(),initial_binding:expected.clone(),
            execution_instance:candidate.execution_instance.to_vec(),pesi:*context.pesi().as_raw(),
            source_descriptor_prefix:candidate.source_descriptor_prefix.to_vec(),
            prior_durable_receipts:Vec::new(),local_restored:true,
        }};
        restored.check_active_request_execution_candidate(&request,context)?;
        Ok(restored)
    }

    /// Restore only from an actual freshly confirmed Native chain and an
    /// independently admitted source/input contract. Capsule binding is never
    /// recycled as its own expected input contract. No fresh-run constructor.
    #[cfg(not(target_arch="wasm32"))]
    pub(crate) fn restore_native_checkpoint(
        adapter:&crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,
        digest:&[u8;32],revision:&[u8],
        context:&super::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
        expected:&crate::logical_input_tape::LogicalInputBinding,
    ) -> Result<Self,Failure> {
        use crate::snapshot_v2_durable_transaction::TransactionEvidenceV1;
        use crate::snapshot_v2_host_bridge as bridge;
        let owned=context.owned();
        let project=hex::decode(owned.project().authored_identity.logical_dependency_id.as_str())
            .map_err(|_|Failure::InvalidInput)?;
        let profile=adapter.capability()?;
        let chain=adapter.reread_committed(digest,&project,revision)?;
        let payload=chain.payload.as_ref().ok_or(Failure::ArtifactMismatch)?;
        if payload.digest!=*digest || payload.binding.project!=project || payload.binding.new_revision!=revision {
            return Err(Failure::CustodyConflict);
        }
        let layout=super::complete_state_frame::check_resumable_complete_state_layout_v2(
            &payload.snapshot,context.pesi().as_raw())?;
        let request=bridge::ResumableRequestCandidateV2 {binding:payload.binding.clone(),layout,
            revision_bytes:&payload.envelope,request_kind:3,snapshot_bytes:&payload.snapshot};
        request.encode_canonical()?;
        let candidate=rebuild_complete_state_source_candidate(&request.layout,context)?;
        let session=replay_complete_state_candidate(&request.layout,context,expected)?;
        check_carrier_world(context,&session)?;
        let definition=owned.definition_identity().as_bytes();
        let boundary=check_boundary_supply_v1(&request.layout,LOGICAL_FIXED_SEED_PROFILE,
            definition,definition,&session.reconstructed_work_cost())?;
        if boundary.base_seed!=expected.initial_rng {return Err(Failure::CustodyConflict);}
        let prior=&candidate.prior_durable_receipts;
        let mut predecessor=vec![0];
        if let Some(last)=prior.last() {predecessor=vec![1];append_bytes32(&mut predecessor,last)?;}
        if request.layout.supplies[8]!=predecessor {return Err(Failure::CustodyConflict);}
        let mut receipts=Vec::new();receipts.try_reserve_exact(prior.len()+1).map_err(|_|Failure::LimitExceeded)?;
        for (i,raw) in prior.iter().enumerate() {
            let parsed=bridge::check_committed_receipt_payload_v1(raw)?;
            let head=parsed.native_head()?;
            if head.project!=project {return Err(Failure::CustodyConflict);}
            let old_chain=adapter.reread_committed(&head.digest,&project,&head.revision)?;
            let old=old_chain.payload.as_ref().ok_or(Failure::ArtifactMismatch)?;
            if old_chain.revision.as_ref()!=Some(&head) || old.digest!=head.digest
                || old.binding.transaction_digest()?!=head.digest {return Err(Failure::CustodyConflict);}
            parsed.check_binding(&old.binding,profile.as_bytes())?;
            let old_layout=super::complete_state_frame::check_resumable_complete_state_layout_v2(
                &old.snapshot,context.pesi().as_raw())?;
            let old_request=bridge::ResumableRequestCandidateV2 {binding:old.binding.clone(),layout:old_layout,
                revision_bytes:&old.envelope,request_kind:3,snapshot_bytes:&old.snapshot};
            old_request.encode_canonical()?;
            check_source_capture_supply(old_request.layout.supplies[1],owned.sources())?;
            check_constituent_supply(old_request.layout.supplies[2],context.pesi().as_raw(),owned)?;
            let old_candidate=rebuild_source_descriptor_candidate(old_request.layout.supplies[9],owned)?;
            if old_candidate.source_descriptor_prefix!=candidate.source_descriptor_prefix
                || old_candidate.execution_instance!=candidate.execution_instance
                || old_candidate.prior_durable_receipts!=prior[..i]
                || old_request.layout.supplies[3]!=encode_logical_execution_contract_supply(owned,&old_candidate.runner)? {
                return Err(Failure::CustodyConflict);
            }
            let old_session=replay_processed_input_candidate(old_request.layout.supplies[4],old_candidate,owned,expected)?;
            check_replayed_world_candidate(&old_request.layout.world,&old_session)?;
            let old_boundary=check_boundary_supply_v1(&old_request.layout,LOGICAL_FIXED_SEED_PROFILE,
                definition,definition,&old_session.reconstructed_work_cost())?;
            if old_boundary.base_seed!=expected.initial_rng {return Err(Failure::CustodyConflict);}
            let old_inputs=old_session.retained_core_inputs().map_err(|_|Failure::InvalidInput)?;
            if session.retained_core_inputs().map_err(|_|Failure::InvalidInput)?.get(..old_inputs.len())!=Some(old_inputs) {
                return Err(Failure::CustodyConflict);
            }
            let mut old_predecessor=vec![0];
            if i>0 {old_predecessor=vec![1];append_bytes32(&mut old_predecessor,prior[i-1])?;}
            if old_request.layout.supplies[8]!=old_predecessor {return Err(Failure::CustodyConflict);}
            receipts.push(raw.to_vec());
        }
        let receipt=bridge::encode_committed_receipt_candidate(&payload.binding,profile.as_bytes())?;
        let mut evidence=TransactionEvidenceV1::prepared(payload.binding.clone())?;
        evidence.synced()?;evidence.verified(&payload.snapshot,&payload.snapshot)?;evidence.head_changed()?;
        evidence.container_confirmation(adapter.confirm_container_durable())?;
        let fresh=adapter.reread_committed(digest,&project,revision)?;
        evidence.committed_native_chain(payload,&fresh)?;
        evidence.check_committed_receipt_evidence(&receipt,profile.as_bytes())?;
        receipts.push(receipt);
        Ok(Self {session,provenance:VerifiedRunCheckpointProvenance {
            source_owner:owned.clone(),initial_binding:expected.clone(),
            execution_instance:candidate.execution_instance.to_vec(),pesi:*context.pesi().as_raw(),
            source_descriptor_prefix:candidate.source_descriptor_prefix.to_vec(),prior_durable_receipts:receipts,local_restored:false,
        }})
    }

    /// Compare a byte-checked save request with this actual execution owner.
    /// This is only a prerequisite: RNG, non-resource runtime state,
    /// durable lineage and receipt publication are deliberately not admitted.
    /// In particular success cannot update checkpoint history or perform I/O.
    pub(crate) fn check_active_request_execution_candidate(
        &self,
        request: &crate::snapshot_v2_host_bridge::ResumableRequestCandidateV2<'_>,
        context: &super::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
    ) -> Result<(),Failure> {
        let owned=&self.provenance.source_owner;
        let project=hex::decode(owned.project().authored_identity.logical_dependency_id.as_str())
            .map_err(|_|Failure::InvalidInput)?;
        if request.binding.project!=project
            || request.binding.execution_profile!=owned.project().source_profile.as_bytes()
            || request.binding.pesi!=self.provenance.pesi
            || context.pesi().as_raw()!=&self.provenance.pesi {
            return Err(Failure::CustodyConflict);
        }
        // Compare independently retained owners, not the request with itself.
        if owned.project().save().map_err(|_|Failure::InvalidInput)?
                != context.owned().project().save().map_err(|_|Failure::InvalidInput)?
            || owned.definition_identity().as_bytes()!=context.owned().definition_identity().as_bytes() {
            return Err(Failure::CustodyConflict);
        }
        let source_supply=encode_source_capture_supply(owned.sources())?;
        check_source_capture_supply(&source_supply,context.owned().sources())?;
        check_revision_envelope_supply(request.revision_bytes,context)?;
        rebuild_complete_state_source_candidate(&request.layout,context)?;
        if request.layout.supplies[8]!=self.checkpoint_predecessor_supply()?
            || request.layout.supplies[9]!=self.source_rebuild_descriptor_supply()?
            || request.layout.supplies[6]!=self.execution_position_supply()? {
            return Err(Failure::CustodyConflict);
        }
        check_processed_input_supply(request.layout.supplies[4],&self.session)?;
        let definition=owned.definition_identity().as_bytes();
        let boundary=check_boundary_supply_v1(&request.layout,LOGICAL_FIXED_SEED_PROFILE,
            definition,definition,&self.session.reconstructed_work_cost())?;
        if boundary.base_seed!=self.provenance.initial_binding.initial_rng {
            return Err(Failure::CustodyConflict);
        }
        check_replayed_world_candidate(&request.layout.world,&self.session)?;
        // The source-bound carrier must not smuggle a different saved world
        // next to a matching complete-state request. Restore into a private
        // candidate and include ALL channels, including view-prefixed values;
        // the ordinary filtered state_hash is insufficient for this crosswalk.
        check_carrier_world(context,&self.session)?;
        check_replayed_work_supply(request.layout.supplies[7],&self.session.reconstructed_work_cost())?;
        // Independently replay the sealed prefix using the retained original
        // binding. Static source/layout checks and equal resources cannot prove
        // consumed keyboard state, full observation rows or past-view history.
        let replayed=replay_complete_state_candidate(&request.layout,context,&self.provenance.initial_binding)?;
        self.session.check_reconstructed_continuation(&replayed,owned.sources().entry_declaration().name())
            .map_err(|_|Failure::CustodyConflict)?;
        Ok(())
    }

    /// R1 section6 field0f, supplied by the actual run at a safe boundary.
    /// Definition bytes are retained by the original source owner; no host
    /// label, instruction offset, or inferred resume target is introduced.
    pub(crate) fn execution_position_supply(&self)->Result<Vec<u8>,Failure> {
        self.checkpoint_predecessor_supply()?;
        let owned=&self.provenance.source_owner;
        let tick=self.session.resume_boundary_next_tick(owned.sources().entry_declaration().name())
            .map_err(|_|Failure::InvalidInput)?;
        let definition=owned.definition_identity().as_bytes();
        let mut out=vec![0,1,1];append_supply(&mut out,&tick.to_be_bytes())?;
        append_bytes32(&mut out,definition)?;append_bytes32(&mut out,definition)?;
        append_supply(&mut out,&[0])?;
        check_execution_position_supply(&out,tick,definition,definition)?;
        Ok(out)
    }

    /// Approved R2 logical strategy; the seed comes from the retained original
    /// construction owner, never from a capsule or a host supplied expected value.
    pub(crate) fn logical_rng_supply(&self)->Result<Vec<u8>,Failure> {
        self.checkpoint_predecessor_supply()?;
        let next=self.session.resume_boundary_next_tick(
            self.provenance.source_owner.sources().entry_declaration().name())
            .map_err(|_|Failure::InvalidInput)?;
        let seed=self.provenance.initial_binding.initial_rng;
        let inputs=self.session.retained_core_inputs().map_err(|_|Failure::InvalidInput)?;
        if u64::try_from(inputs.len()).map_err(|_|Failure::LimitExceeded)?!=next {
            return Err(Failure::CustodyConflict);
        }
        for (i,input) in inputs.iter().enumerate() {
            if input.tick_id!=u64::try_from(i).map_err(|_|Failure::LimitExceeded)? || input.rng_seed!=seed {
                return Err(Failure::CustodyConflict);
            }
        }
        let mut out=vec![0,1,3];append_bytes32(&mut out,LOGICAL_FIXED_SEED_PROFILE)?;
        for value in [seed,next,seed,0] {append_supply(&mut out,&value.to_be_bytes())?;}
        Ok(out)
    }

    /// Field11 is supplied only by this private owner. A failed/aborted run
    /// cannot become checkpoint-eligible; missing restored provenance has no
    /// constructor here and cannot be replaced by this fresh-run factory.
    pub(crate) fn checkpoint_predecessor_supply(&self)->Result<Vec<u8>,Failure> {
        self.session.retained_core_inputs().map_err(|_|Failure::InvalidInput)?;
        let p=&self.provenance;
        // Retain original context, not a new capture, UUID, or host assertion.
        if p.execution_instance.is_empty() || p.initial_binding.source_profile != crate::logical_input_tape::LOGICAL_INPUT_PROFILE {
            return Err(Failure::CustodyConflict);
        }
        let issued=crate::project_snapshot_identity::current::issue_saved_local_project(&p.source_owner)
            .map_err(|_|Failure::InvalidInput)?;
        if issued.pesi().as_raw()!=&p.pesi {return Err(Failure::ForeignPesi);}
        match p.prior_durable_receipts.last() {
            None=>Ok(vec![0]),
            Some(receipt)=>{
                crate::snapshot_v2_host_bridge::check_committed_receipt_payload_v1(receipt)?;
                let mut out=vec![1];append_bytes32(&mut out,receipt)?;Ok(out)
            }
        }
    }

    /// R2 field12 producer retains the original source/entry contract and the
    /// actual accepted prefix count; checkpoint history comes only from owner.
    pub(crate) fn source_rebuild_descriptor_supply(&self)->Result<Vec<u8>,Failure> {
        self.checkpoint_predecessor_supply()?;
        let count=self.session.retained_core_inputs().map_err(|_|Failure::InvalidInput)?.len();
        let count=u64::try_from(count).map_err(|_|Failure::LimitExceeded)?;
        let mut out=self.provenance.source_descriptor_prefix.clone();
        append_supply(&mut out,&[u8::from(count>0)])?;
        append_supply(&mut out,&count.to_be_bytes())?;
        append_bytes32(&mut out,&self.provenance.execution_instance)?;
        let receipts=&self.provenance.prior_durable_receipts;
        let length=u32::try_from(receipts.len()).map_err(|_|Failure::LimitExceeded)?;
        append_supply(&mut out,&length.to_be_bytes())?;
        for receipt in receipts {append_bytes32(&mut out,receipt)?;}
        rebuild_source_descriptor_candidate(&out,&self.provenance.source_owner)?;
        Ok(out)
    }
}

fn check_carrier_world(
    context:&super::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
    session:&crate::logical_input_tape::LogicalInputSession,
) -> Result<(),Failure> {
    let resources=super::parse_snapshot_json(context.resource_snapshot()).map_err(|_|Failure::InvalidInput)?;
    super::resource_snapshot::check_resource_snapshot_matches_world(&resources,session.committed_world())
        .map_err(|_|Failure::CustodyConflict)?;
    Ok(())
}

/// A staging candidate, never a continuation-admission or durability token.
/// The caller must replay and compare the prefix, boundary and checkpoint chain
/// before publishing either this runner or a restored world.
pub(crate) struct SourceRebuildCandidateV1<'a> {
    pub(crate) source_descriptor_prefix: &'a [u8],
    pub(crate) execution_instance: &'a [u8],
    pub(crate) runner: crate::ddn_runtime::DdnRunner,
    pub(crate) age: ddonirang_lang::AgeTarget,
    pub(crate) initial_world: super::complete_state_frame::CompleteStateFrameV1<'a>,
    pub(crate) initial_tick: u64,
    pub(crate) base_rng_seed: u64,
    pub(crate) phase: u8,
    pub(crate) successful_madi_count: u64,
    /// RD670 byte-checked comparison payloads, not confirmed provenance.
    pub(crate) prior_durable_receipts: Vec<&'a [u8]>,
}

/// Assemble all static source supplies before prefix replay. This still
/// deliberately cannot authorize durable storage or live-world publication.
pub(crate) fn rebuild_complete_state_source_candidate<'a>(
    layout: &ResumableCompleteStateLayoutV2<'a>,
    context: &super::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
) -> Result<SourceRebuildCandidateV1<'a>,Failure> {
    rebuild_source_supplies(layout,context,true)
}

/// General callers explicitly select the preserved ordinary field0c version2.
/// A version word or mode byte in untrusted input cannot choose the producer.
pub(crate) fn rebuild_general_complete_state_source_candidate<'a>(
    layout:&ResumableCompleteStateLayoutV2<'a>,
    context:&super::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
)->Result<SourceRebuildCandidateV1<'a>,Failure> {
    let candidate=rebuild_source_supplies(layout,context,false)?;
    let inputs=decode_general_processed_inputs(layout.supplies[4],candidate.initial_tick,candidate.base_rng_seed)?;
    if u64::try_from(inputs.len()).map_err(|_|Failure::LimitExceeded)?!=candidate.successful_madi_count {
        return Err(Failure::CustodyConflict);
    }
    Ok(candidate)
}

fn rebuild_source_supplies<'a>(
    layout:&ResumableCompleteStateLayoutV2<'a>,
    context:&super::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,logical:bool,
)->Result<SourceRebuildCandidateV1<'a>,Failure> {
    let owned = context.owned();
    if layout.world.pesi != context.pesi().as_raw() { return Err(Failure::ForeignPesi); }
    check_revision_envelope_supply(layout.supplies[0],context)?;
    check_source_capture_supply(layout.supplies[1],owned.sources())?;
    check_constituent_supply(layout.supplies[2],layout.world.pesi,owned)?;
    let candidate = rebuild_source_descriptor_candidate(layout.supplies[9],owned)?;
    let contract=if logical {encode_logical_execution_contract_supply(owned,&candidate.runner)?}
        else {encode_execution_contract_supply(owned,&candidate.runner)?};
    if layout.supplies[3]!=contract {
        return Err(Failure::CustodyConflict);
    }
    let expected_tick = candidate.initial_tick.checked_add(candidate.successful_madi_count)
        .ok_or(Failure::LimitExceeded)?;
    if layout.world.next_tick != expected_tick { return Err(Failure::CustodyConflict); }
    if candidate.phase == 0 && candidate.initial_world.encode_canonical()? != layout.world.encode_canonical()? {
        return Err(Failure::CustodyConflict);
    }
    Ok(candidate)
}

/// R2 field12: reconstruct static program and its real saved Start through the
/// ordinary product parser/linker. No source recapture or implicit ID issuance.
pub(crate) fn rebuild_source_descriptor_candidate<'a>(
    bytes: &'a [u8],
    owned: &crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
) -> Result<SourceRebuildCandidateV1<'a>, Failure> {
    use ddonirang_lang::{AgeTarget, ExecutionMode, ParseMode};
    if bytes.len() > 2_000_000 { return Err(Failure::LimitExceeded); }
    let mut c = Cursor {bytes, offset: 0};
    if c.take(2)? != [0,2] || c.bytes32()? != b"ddn.resume.source-rebuild-descriptor.v1"
        || c.bytes32()? != b"v1-core-v25" { return Err(Failure::InvalidInput); }
    let age = match c.byte()? {
        0=>AgeTarget::Age0,1=>AgeTarget::Age1,2=>AgeTarget::Age2,3=>AgeTarget::Age3,
        4=>AgeTarget::Age4,5=>AgeTarget::Age5,6=>AgeTarget::Age6,7=>AgeTarget::Age7,
        _=>return Err(Failure::InvalidInput),
    };
    // The current executable parser has only SuccessorDefault. Historical
    // migration evidence must not be turned into an executable fallback.
    if !c.boolean()? { return Err(Failure::InvalidInput); }
    let mode = ParseMode::Strict;
    let canonical = c.bytes32()?;
    let execution_mode = c.byte()?;
    let hooks = [c.u64()?,c.u64()?,c.u64()?];
    let entry = c.bytes32()?; let resume = c.bytes32()?;
    let strategy = c.byte()?;
    let authored = c.bytes32()?;
    let initial_bytes = c.bytes32()?;
    let initial_tick = c.u64()?; let base_rng_seed = c.u64()?;
    let prefix_end=c.offset;
    let phase = c.byte()?; let successful_madi_count = c.u64()?;
    let instance = c.bytes32()?;
    let receipt_count = u32::from_be_bytes(c.take(4)?.try_into().map_err(|_|Failure::InvalidInput)?);
    // Every receipt consumes at least its length word and one payload byte.
    // Validate physical availability before any count-derived allocation.
    let receipt_count = usize::try_from(receipt_count).map_err(|_|Failure::LimitExceeded)?;
    if receipt_count > (bytes.len()-c.offset)/5 { return Err(Failure::InvalidInput); }
    let mut prior_durable_receipts = Vec::new();
    prior_durable_receipts.try_reserve_exact(receipt_count).map_err(|_|Failure::LimitExceeded)?;
    for _ in 0..receipt_count {
        let receipt = c.bytes32()?;
        crate::snapshot_v2_host_bridge::check_committed_receipt_payload_v1(receipt)?;
        prior_durable_receipts.push(receipt);
    }
    c.finish()?;
    if !matches!((phase,successful_madi_count),(0,0)|(1,1..)) {
        return Err(Failure::InvalidInput);
    }
    if authored != owned.project().save().map_err(|_|Failure::InvalidInput)?.as_bytes()
        || entry != owned.definition_identity().as_bytes() || resume != entry {
        return Err(Failure::CustodyConflict);
    }
    // This admitted authored project has an explicit named Start. Strategy00
    // cannot silently replace it with a root or a selected-editor callable.
    if strategy != 1 { return Err(Failure::InvalidInput); }
    let issued = crate::project_snapshot_identity::current::issue_saved_local_project(owned)
        .map_err(|_|Failure::InvalidInput)?;
    let initial_world = super::complete_state_frame::check_complete_state_frame_v1(initial_bytes,issued.pesi().as_raw())?;
    let catalog=owned.registration().map(|registration|registration.saved_nominal_catalog(owned)).transpose().map_err(|_|Failure::CustodyConflict)?;
    initial_world.decode_value_tables_with_catalog(catalog.as_ref())?;
    if initial_world.next_tick != initial_tick { return Err(Failure::CustodyConflict); }
    let mut program = crate::ddn_runtime::DdnProgram::from_saved_v25_project_with_mode(
        owned.sources().clone(),mode,age,
    ).map_err(|_|Failure::InvalidInput)?;
    program.bind_registered_project_context(owned).map_err(|_|Failure::CustodyConflict)?;
    let preparation=owned.sources().preparation();
    crate::runtime_surface::prepare_supported_source_for_execution_with_program(
        &preparation.source,&preparation.target_path,&program,
    ).map_err(|_|Failure::InvalidInput)?;
    let decision = program.execution_mode_decision();
    let expected_mode = match decision.mode {ExecutionMode::OneShot=>1,ExecutionMode::Live=>2};
    if canonical != program.canonical_ddn().as_bytes() || execution_mode != expected_mode
        || hooks != [decision.live_hook_count,decision.every_madi_hook_count,decision.every_n_madi_hook_count] {
        return Err(Failure::CustodyConflict);
    }
    let mut runner = crate::ddn_runtime::DdnRunner::new(program,owned.sources().entry_declaration().name());
    if let Some(stage)=super::subject_execution::SubjectExecutionStage::from_recorded_initial(owned,&initial_world).map_err(|_|Failure::SubjectCustodyInvalid)? {
        runner.bind_subject_bootstrap(stage);
    }
    runner.configure_project_entry(owned.sources().entry_declaration().name()).map_err(|_|Failure::InvalidInput)?;
    if instance != runner.execution_instance_owner_bytes() { return Err(Failure::CustodyConflict); }
    // Ordinary source/profile/resource restrictions remain in the shared owner.
    encode_execution_contract_supply(owned,&runner)?;
    Ok(SourceRebuildCandidateV1 {runner,age,initial_world,initial_tick,base_rng_seed,phase,successful_madi_count,prior_durable_receipts,
        source_descriptor_prefix:&bytes[..prefix_end],execution_instance:instance})
}

/// Field09 must retain the exact canonical envelope already checked against
/// the saved project, revision lineage, source captures and runtime resources.
/// Bytes alone cannot construct this source-bound authority token.
pub(crate) fn check_revision_envelope_supply(
    bytes: &[u8], context: &super::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
) -> Result<(), Failure> {
    if bytes.len() > 1_000_000 { return Err(Failure::LimitExceeded); }
    if bytes != context.revision_envelope() { return Err(Failure::CustodyConflict); }
    Ok(())
}

const STANDARD_CONTRACT_FIELDS: [(&str, &str);7] = [
    ("결과", "정본값동일"), ("성능", "안전최적화"), ("외부유입", "봉인입력"),
    ("자원", "표준한도_v1"), ("재생", "검증재생"), ("재현범위", "이식가능"), ("증거", "최소증거"),
];

/// R2 field0c for the actual supported ordinary saved-project context.
/// The typed root must omit an explicit contract; unsupported contracts are
/// not downgraded. This projection does not issue an execution identity.
pub(crate) fn encode_execution_contract_supply(
    owned: &crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    runner: &crate::ddn_runtime::DdnRunner,
) -> Result<Vec<u8>, Failure> {
    let sources = owned.sources();
    let running_sources = runner.saved_program_sources().ok_or(Failure::InvalidInput)?;
    if encode_source_capture_supply(sources)? != encode_source_capture_supply(running_sources)? {
        return Err(Failure::CustodyConflict);
    }
    let root_path = &sources.saved().manifest().root_path;
    let root = sources.captures().iter().find(|(path, _)| path == root_path)
        .ok_or(Failure::InvalidInput)?;
    let directive = ddonirang_lang::work_root::parse_work_root(root.1.text())
        .map_err(|_| Failure::InvalidInput)?;
    if directive.field(ddonirang_lang::work_root::RootField::ExecutionContract).is_some() {
        return Err(Failure::InvalidInput);
    }
    // The actual project linker validates the ordinary, non-model context and
    // rejects unsupported explicit contracts, rather than ignoring their text.
    crate::local_project::build_project_graph(sources.saved().source().clone())
        .map_err(|_| Failure::InvalidInput)?;
    let receipt = runner.work_cost_receipt();
    if receipt.limits != StandardResourceLimitsV1::CANDIDATE || receipt.rd206_resource_capability {
        return Err(Failure::InvalidInput);
    }
    let mut out = Vec::new();
    append_supply(&mut out, &2u16.to_be_bytes())?;
    append_bytes32(&mut out, b"ddn.saved-execution-contract-supply.v1")?;
    append_bytes32(&mut out, "표준재현_v1".as_bytes())?;
    append_supply(&mut out, &1u32.to_be_bytes())?;
    append_supply(&mut out, &[7])?;
    for (name, value) in STANDARD_CONTRACT_FIELDS {
        append_bytes32(&mut out, name.as_bytes())?;
        append_bytes32(&mut out, value.as_bytes())?;
    }
    append_supply(&mut out, &[2])?; // source-omitted-expanded, proved above
    append_bytes32(&mut out, root_path.as_bytes())?;
    append_supply(&mut out, &[0;16])?; // no explicit source span
    let canonical = format!("({})", STANDARD_CONTRACT_FIELDS.iter()
        .map(|(name,value)| format!("{name}:#{value}")).collect::<Vec<_>>().join(","));
    append_bytes32(&mut out, canonical.as_bytes())?;
    append_supply(&mut out, &[10])?;
    let l = receipt.limits;
    for value in [l.slice_committed_madi,l.slice_work_cost,l.slice_solver_evaluations,l.slice_external_events,
        l.call_depth,l.created_identities,l.public_observation_samples,l.sealed_input_bytes,l.result_artifact_bytes,l.replay_capsule_bytes] {
        append_supply(&mut out, &value.to_be_bytes())?;
    }
    append_supply(&mut out, &[u8::from(receipt.rd206_resource_capability)])?;
    Ok(out)
}

pub(crate) fn check_execution_contract_supply(
    bytes: &[u8], owned: &crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    runner: &crate::ddn_runtime::DdnRunner,
) -> Result<(), Failure> {
    if bytes.len() > 2_000_000 { return Err(Failure::LimitExceeded); }
    if bytes != encode_execution_contract_supply(owned, runner)? { return Err(Failure::CustodyConflict); }
    Ok(())
}

/// Only the logical producer/consumer uses version3. Ordinary version2 is
/// byte-preserved, and cannot silently acquire a continuation strategy.
pub(crate) fn encode_logical_execution_contract_supply(
    owned: &crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    runner: &crate::ddn_runtime::DdnRunner,
) -> Result<Vec<u8>,Failure> {
    let mut out=encode_execution_contract_supply(owned,runner)?;
    out[..2].copy_from_slice(&3u16.to_be_bytes());
    append_supply(&mut out,&[3])?;
    append_bytes32(&mut out,LOGICAL_FIXED_SEED_PROFILE)?;
    Ok(out)
}

/// Field0a projects a verified, complete saved closure. Paths only locate
/// existing captures; this function neither captures source nor issues IDs.
pub(crate) fn encode_source_capture_supply(
    sources: &crate::project_revision_envelope_supply::VerifiedSavedV25Sources,
) -> Result<Vec<u8>, Failure> {
    let captures = sources.captures();
    let files = &sources.saved().source().files;
    if captures.len() != files.len() { return Err(Failure::CustodyConflict); }
    let mut out = Vec::new();
    append_supply(&mut out, &1u16.to_be_bytes())?;
    append_supply(&mut out, &u32::try_from(captures.len()).map_err(|_| Failure::LimitExceeded)?.to_be_bytes())?;
    let mut previous: Option<&str> = None;
    for ((path, capture), file) in captures.iter().zip(files) {
        if path != &file.path || capture.bytes() != file.utf8_text.as_bytes()
            || previous.is_some_and(|previous| previous.as_bytes() >= path.as_bytes()) {
            return Err(Failure::CustodyConflict);
        }
        let role = match file.role.as_str() {
            "root" => 1, "ddn_source" => 2, _ => return Err(Failure::InvalidInput),
        };
        append_bytes32(&mut out, path.as_bytes())?;
        append_supply(&mut out, &[role])?;
        append_bytes32(&mut out, capture.bytes())?;
        append_bytes32(&mut out, capture.epoch_owner_bytes())?;
        append_supply(&mut out, &capture.ordinal().to_be_bytes())?;
        append_supply(&mut out, capture.content_digest())?;
        append_supply(&mut out, capture.identity())?;
        previous = Some(path);
    }
    Ok(out)
}

/// Only compare against the existing source owner after its full verification.
/// A matching field is not durable provenance or full resume admission.
pub(crate) fn check_source_capture_supply(
    bytes: &[u8], sources: &crate::project_revision_envelope_supply::VerifiedSavedV25Sources,
) -> Result<(), Failure> {
    if bytes.len() > 2_000_000 { return Err(Failure::LimitExceeded); }
    let expected = encode_source_capture_supply(sources)?;
    if bytes != expected { return Err(Failure::CustodyConflict); }
    Ok(())
}

/// Field0b reuses the current RD689 issuer, including its unsupported external
/// supply gate. Adapters cannot supply self-consistent hashes as authority.
pub(crate) fn encode_constituent_supply(
    owned: &crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
) -> Result<Vec<u8>, Failure> {
    let issued = crate::project_snapshot_identity::current::issue_saved_local_project(owned)
        .map_err(|_| Failure::InvalidInput)?;
    let mut out = Vec::new(); append_supply(&mut out, &[4])?;
    for (index, (preimage, identity)) in issued.constituent_preimages().iter()
        .zip(issued.constituents()).enumerate() {
        append_supply(&mut out, &[(index + 1) as u8])?;
        append_bytes32(&mut out, preimage)?;
        append_supply(&mut out, identity.as_raw())?;
    }
    Ok(out)
}

pub(crate) fn check_constituent_supply(
    bytes: &[u8], pesi: &[u8;32],
    owned: &crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
) -> Result<(), Failure> {
    if bytes.len() > 2_000_000 { return Err(Failure::LimitExceeded); }
    let issued = crate::project_snapshot_identity::current::issue_saved_local_project(owned)
        .map_err(|_| Failure::InvalidInput)?;
    if issued.pesi().as_raw() != pesi || bytes != encode_constituent_supply(owned)? {
        return Err(Failure::CustodyConflict);
    }
    Ok(())
}

// Field0d is supplied by the actual successful producer, not host reconstructed rows.
pub(crate) fn encode_processed_input_supply(
    session: &crate::logical_input_tape::LogicalInputSession,
) -> Result<Vec<u8>, Failure> {
    let inputs = session.retained_core_inputs().map_err(|_| Failure::InvalidInput)?;
    let tape_bytes = session.seal().map_err(|_| Failure::InvalidInput)?;
    let tape = crate::logical_input_tape::decode(&tape_bytes).map_err(|_| Failure::InvalidInput)?;
    if tape.frames.len() != inputs.len() { return Err(Failure::CustodyConflict); }
    let mut out = Vec::new();
    append_supply(&mut out, &2u16.to_be_bytes())?;
    append_bytes32(&mut out, CORE_INPUT_PROFILE)?;
    append_bytes32(&mut out, &tape_bytes)?;
    append_supply(&mut out, &u32::try_from(inputs.len()).map_err(|_| Failure::LimitExceeded)?.to_be_bytes())?;
    for (input, frame) in inputs.iter().zip(&tape.frames) {
        if input.frame_source != ddonirang_core::InputSource::Person
            || !input.ai_injections.is_empty() || !input.net_events.is_empty() {
            return Err(Failure::InvalidInput);
        }
        append_supply(&mut out, &input.tick_id.to_be_bytes())?;
        append_supply(&mut out, &input.dt.raw_i64().to_be_bytes())?;
        append_supply(&mut out, &input.keys_pressed.to_be_bytes())?;
        append_bytes32(&mut out, input.last_key_name.as_bytes())?;
        append_supply(&mut out, &input.pointer_x_i32.to_be_bytes())?;
        append_supply(&mut out, &input.pointer_y_i32.to_be_bytes())?;
        append_supply(&mut out, &[0])?;
        append_supply(&mut out, &input.rng_seed.to_be_bytes())?;
        append_supply(&mut out, &[0; 8])?; // empty AI and network counts
        let row = serde_json::to_vec(&frame.input).map_err(|_| Failure::InvalidInput)?;
        append_bytes32(&mut out, &row)?;
    }
    Ok(out)
}

fn append_supply(out: &mut Vec<u8>, bytes: &[u8]) -> Result<(), Failure> {
    // Reuse the admitted enclosing frame budget; no invented section/count limit.
    if out.len().checked_add(bytes.len()).ok_or(Failure::LimitExceeded)? > 2_000_000 {
        return Err(Failure::LimitExceeded);
    }
    out.try_reserve(bytes.len()).map_err(|_| Failure::LimitExceeded)?;
    out.extend_from_slice(bytes); Ok(())
}
fn append_bytes32(out: &mut Vec<u8>, bytes: &[u8]) -> Result<(), Failure> {
    let length = u32::try_from(bytes.len()).map_err(|_| Failure::LimitExceeded)?;
    append_supply(out, &length.to_be_bytes())?;
    append_supply(out, bytes)
}

/// Comparison against a reconstructed producer is mandatory, not caller asserted rows.
/// This is an input-section check only, never authority to publish a restored world.
pub(crate) fn check_processed_input_supply(
    bytes: &[u8], reconstructed: &crate::logical_input_tape::LogicalInputSession,
) -> Result<(), Failure> {
    if bytes.len() > 2_000_000 { return Err(Failure::LimitExceeded); }
    let mut cursor = Cursor { bytes, offset: 0 };
    if cursor.take(2)? != [0, 2] || cursor.bytes32()? != CORE_INPUT_PROFILE {
        return Err(Failure::InvalidInput);
    }
    let tape_bytes = cursor.bytes32()?;
    let tape = crate::logical_input_tape::decode(tape_bytes).map_err(|_| Failure::InvalidInput)?;
    crate::logical_input_tape::admit_replay(tape_bytes, &tape.binding).map_err(|_| Failure::InvalidInput)?;
    // Exact reconstruction seals bind source, events, held state and each accepted world.
    let expected = encode_processed_input_supply(reconstructed)?;
    if bytes != expected { return Err(Failure::CustodyConflict); }
    Ok(())
}

/// Actual sealed-prefix execution in the staged saved-source runner. Expected
/// input contract must be the original admitted execution context, not the
/// binding read from these same untrusted bytes. RNG/boundary/provenance and
/// durable reread remain separate gates; no live session is published here.
pub(crate) fn replay_processed_input_candidate(
    bytes: &[u8], candidate: SourceRebuildCandidateV1<'_>,
    owned: &crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
    expected: &crate::logical_input_tape::LogicalInputBinding,
) -> Result<crate::logical_input_tape::LogicalInputSession,Failure> {
    if bytes.len() > 2_000_000 { return Err(Failure::LimitExceeded); }
    let mut c = Cursor {bytes,offset:0};
    if c.take(2)? != [0,2] || c.bytes32()? != CORE_INPUT_PROFILE { return Err(Failure::InvalidInput); }
    let tape_bytes = c.bytes32()?;
    let count = u32::from_be_bytes(c.take(4)?.try_into().map_err(|_|Failure::InvalidInput)?) as u64;
    if count != candidate.successful_madi_count || candidate.initial_tick != 0
        || candidate.base_rng_seed != expected.initial_rng { return Err(Failure::CustodyConflict); }
    let initial = candidate.initial_world.decode_value_tables()?;
    // Existing logical producer admits only empty initial worlds. Do not
    // silently erase nonempty V1 values, parameters, Subjects or custody.
    if !initial.resources.is_empty() || !initial.parameters.is_empty() || !initial.subjects.is_empty()
        || initial.subject_custody.is_some() || !candidate.initial_world.atom_evidence.is_empty() {
        return Err(Failure::InvalidInput);
    }
    let tape = crate::logical_input_tape::admit_replay(tape_bytes,expected).map_err(|_|Failure::InvalidInput)?;
    if tape.frames.len() as u64 != count { return Err(Failure::CustodyConflict); }
    let mut session = crate::logical_input_tape::LogicalInputSession::from_saved_runner(
        owned,expected.clone(),candidate.runner,
    ).map_err(|_|Failure::InvalidInput)?;
    session.replay_sealed_prefix(tape_bytes).map_err(|_|Failure::CustodyConflict)?;
    // Checks every supplied core row byte against actual consumed inputs; an
    // accepted tape alone cannot conceal altered dt/key/pointer/RNG rows.
    check_processed_input_supply(bytes,&session)?;
    Ok(session)
}

/// Compare the last replayed world with the saved eight-field payload through
/// its typed decoder. This logical profile has no parameter overrides or
/// native entity/legacy-resource producer; unsupported state is not erased.
pub(crate) fn check_replayed_world_candidate(
    saved: &super::complete_state_frame::CompleteStateFrameV1<'_>,
    session: &crate::logical_input_tape::LogicalInputSession,
) -> Result<(),Failure> {
    let values = saved.decode_value_tables()?;
    let inputs = session.retained_core_inputs().map_err(|_|Failure::InvalidInput)?;
    if saved.next_tick != inputs.len() as u64 { return Err(Failure::CustodyConflict); }
    let world = session.committed_world();
    if !values.parameters.is_empty() || !values.subjects.is_empty() || values.subject_custody.is_some()
        || !world.resource_json_entries().is_empty() || !world.resource_handle_entries().is_empty()
        || !world.component_json_entries().is_empty() || world.next_entity_cursor() != 0 {
        return Err(Failure::InvalidInput);
    }
    let mut actual = std::collections::BTreeMap::new();
    for (name,value) in world.resource_fixed64_entries() {
        actual.insert(name,ddonirang_core::platform::ResourceValue::Fixed64(value));
    }
    for (name,value) in world.resource_value_entries() {
        if actual.insert(name,value).is_some() { return Err(Failure::InvalidInput); }
    }
    if actual.len() != values.resources.len() { return Err(Failure::CustodyConflict); }
    for (name,value) in &actual {
        let expected = values.resources.get(name).ok_or(Failure::CustodyConflict)?;
        // Canonical encoding avoids recursive derived equality on deep values.
        let mut a=Vec::new();let mut b=Vec::new();value.encode_canon(&mut a);expected.encode_canon(&mut b);
        if a != b { return Err(Failure::CustodyConflict); }
    }
    Ok(())
}

pub(crate) fn replay_complete_state_candidate(
    layout: &ResumableCompleteStateLayoutV2<'_>,
    context: &super::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
    expected: &crate::logical_input_tape::LogicalInputBinding,
) -> Result<crate::logical_input_tape::LogicalInputSession,Failure> {
    let candidate = rebuild_complete_state_source_candidate(layout,context)?;
    let session = replay_processed_input_candidate(layout.supplies[4],candidate,context.owned(),expected)?;
    check_replayed_world_candidate(&layout.world,&session)?;
    check_replayed_work_supply(layout.supplies[7],&session.reconstructed_work_cost())?;
    // Receipt/provenance/RNG/boundary and durable reread still precede release.
    Ok(session)
}

#[derive(Debug)]
pub(crate) struct BoundarySupplyV1 {
    pub(crate) base_seed: u64,
    pub(crate) next_tick: u64,
    pub(crate) counters: WorkCostCounters,
    pub(crate) limits: StandardResourceLimitsV1,
    pub(crate) rd206_resource_capability: bool,
}

// Expected profile/definitions and receipt must come from verified program
// owners and the reconstructed shared runner, not a host assertion or name.
pub(crate) fn check_boundary_supply_v1(
    layout: &ResumableCompleteStateLayoutV2<'_>, rng_profile: &[u8],
    entry: &[u8], resume: &[u8], reconstructed_work: &WorkCostReceipt,
) -> Result<BoundarySupplyV1,Failure> {
    decode_boundary_sections(layout.supplies[5],layout.supplies[6],layout.supplies[7],
        layout.world.next_tick,rng_profile,entry,resume,reconstructed_work)
}

fn decode_boundary_sections(
    rng:&[u8],position:&[u8],work:&[u8],next_tick:u64,
    rng_profile:&[u8],entry:&[u8],resume:&[u8],expected:&WorkCostReceipt,
)->Result<BoundarySupplyV1,Failure>{
    if rng_profile.is_empty()||entry.is_empty()||resume.is_empty(){return Err(Failure::InvalidInput);}
    let mut rng=Cursor{bytes:rng,offset:0};rng.version()?;
    // The expected strategy belongs to the caller's trusted execution owner.
    // Mode02 remains unsupported; mode01 retains its original XOR meaning.
    let mode=rng.byte()?;
    let expected_mode=if rng_profile==LOGICAL_FIXED_SEED_PROFILE {3}else{1};
    if mode!=expected_mode{return Err(Failure::InvalidInput);}
    if rng.bytes32()?!=rng_profile{return Err(Failure::CustodyConflict);}
    let base_seed=rng.u64()?;
    let rng_next=rng.u64()?;let progress=rng.u64()?;let draws=rng.u64()?;
    rng.finish()?;
    let expected_progress=if mode==3 {base_seed}else{base_seed^next_tick};
    if rng_next!=next_tick||progress!=expected_progress||draws!=0{
        return Err(Failure::CustodyConflict);
    }
    check_execution_position_supply(position,next_tick,entry,resume)?;
    let (counters,limits,rd206_resource_capability)=check_replayed_work_supply(work,expected)?;
    Ok(BoundarySupplyV1{base_seed,next_tick,counters,limits,rd206_resource_capability})
}

pub(crate) fn check_execution_position_supply(
    bytes:&[u8],next_tick:u64,entry:&[u8],resume:&[u8],
)->Result<(),Failure>{
    if bytes.len()>2_000_000 {return Err(Failure::LimitExceeded);}
    if entry.is_empty()||resume.is_empty(){return Err(Failure::InvalidInput);}
    let mut position=Cursor{bytes,offset:0};position.version()?;
    if position.byte()?!=1{return Err(Failure::InvalidInput);}
    if position.u64()?!=next_tick||position.bytes32()?!=entry||position.bytes32()?!=resume{
        return Err(Failure::CustodyConflict);
    }
    if position.byte()?!=0{return Err(Failure::InvalidInput);}
    position.finish()
}

pub(crate) fn check_replayed_work_supply(
    work:&[u8],expected:&WorkCostReceipt,
) -> Result<(WorkCostCounters,StandardResourceLimitsV1,bool),Failure> {
    if work.len()>2_000_000 {return Err(Failure::LimitExceeded);}
    let mut work=Cursor{bytes:work,offset:0};work.version()?;
    if work.byte()?!=15{return Err(Failure::InvalidInput);}
    let counters=WorkCostCounters{
        work_cost:work.u64()?,base_dispatches:work.u64()?,collection_tensor_element_visits:work.u64()?,
        canonical_utf8_byte_visits:work.u64()?,bigint_rational_signed_magnitude_byte_visits:work.u64()?,
        solver_evaluations:work.u64()?,external_events:work.u64()?,committed_madi:work.u64()?,
        call_depth_current:work.u64()?,call_depth_peak:work.u64()?,created_identities:work.u64()?,
        public_observation_samples:work.u64()?,sealed_input_bytes:work.u64()?,
        result_artifact_bytes:work.u64()?,replay_capsule_bytes:work.u64()?,
    };
    if work.byte()?!=10{return Err(Failure::InvalidInput);}
    let limits=StandardResourceLimitsV1{
        slice_committed_madi:work.u64()?,slice_work_cost:work.u64()?,slice_solver_evaluations:work.u64()?,
        slice_external_events:work.u64()?,call_depth:work.u64()?,created_identities:work.u64()?,
        public_observation_samples:work.u64()?,sealed_input_bytes:work.u64()?,
        result_artifact_bytes:work.u64()?,replay_capsule_bytes:work.u64()?,
    };
    let rd206_resource_capability=work.boolean()?;work.finish()?;
    if counters.call_depth_current!=0||counters!=expected.counters||limits!=expected.limits
        ||rd206_resource_capability!=expected.rd206_resource_capability{
        return Err(Failure::CustodyConflict);
    }
    Ok((counters,limits,rd206_resource_capability))
}

pub(crate) fn encode_replayed_work_supply(
    session:&crate::logical_input_tape::LogicalInputSession,
) -> Result<Vec<u8>,Failure> {
    session.retained_core_inputs().map_err(|_|Failure::InvalidInput)?;
    encode_work_receipt_supply(&session.reconstructed_work_cost())
}

/// General and logical executions share the existing exact field10 grammar.
/// A receipt is not an execution owner: callers must retain and replay that
/// owner before accepting these comparison bytes.
pub(crate) fn encode_work_receipt_supply(receipt:&WorkCostReceipt)->Result<Vec<u8>,Failure> {
    let c=&receipt.counters;let l=&receipt.limits;
    let mut out=vec![0,1,15];
    for value in [c.work_cost,c.base_dispatches,c.collection_tensor_element_visits,c.canonical_utf8_byte_visits,
        c.bigint_rational_signed_magnitude_byte_visits,c.solver_evaluations,c.external_events,c.committed_madi,
        c.call_depth_current,c.call_depth_peak,c.created_identities,c.public_observation_samples,
        c.sealed_input_bytes,c.result_artifact_bytes,c.replay_capsule_bytes] {
        append_supply(&mut out,&value.to_be_bytes())?;
    }
    append_supply(&mut out,&[10])?;
    for value in [l.slice_committed_madi,l.slice_work_cost,l.slice_solver_evaluations,l.slice_external_events,
        l.call_depth,l.created_identities,l.public_observation_samples,l.sealed_input_bytes,l.result_artifact_bytes,l.replay_capsule_bytes] {
        append_supply(&mut out,&value.to_be_bytes())?;
    }
    append_supply(&mut out,&[u8::from(receipt.rd206_resource_capability)])?;
    check_replayed_work_supply(&out,receipt)?;
    Ok(out)
}

#[cfg(test)]
mod tests{
    #[test]
    fn local_logical_checkpoint_replays_prefix_and_rejects_foreign_seed() {
        use super::*;
        use crate::unit_wire::project_runtime_attachment as attachment;
        let request=serde_json::json!({"source_profile":"v1-core-v25","module_name":"",
            "source_snapshot":{"root_path":"root.ddn","entry_path":"main.ddn","files":[
                {"path":"root.ddn","text":include_str!("../../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/root.ddn")},
                {"path":"main.ddn","text":"채비 { 값:수 := 0. }. 실행:움직씨 := { (매마디)마다 { 값 <- 값 + 1. 값 보여주기. }. }."}]}});
        let authored=crate::authored_project::operate("create",&request.to_string()).unwrap();
        let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(authored.as_bytes()).unwrap();
        let mut binding:crate::logical_input_tape::LogicalInputBinding=serde_json::from_str(
            include_str!("../../../solutions/seamgrim_ui_mvp/samples/alpha2_logical_tetris.binding.json")).unwrap();
        binding.source_revision=crate::runtime_surface::sha256_identity(owned.sources().preparation().source.as_bytes());
        let mut original=CheckpointedLogicalRunV1::fresh(&owned,binding.clone(),crate::ddn_runtime::default_age_target()).unwrap();
        let input=|madi|serde_json::from_value(serde_json::json!({"madi":madi,"events":[],"held":binding.initial_held})).unwrap();
        for madi in 0..2 {original.step(&input(madi)).unwrap();}
        let saved=original.export_saved_artifact(&[],&[9]).unwrap();
        let checked=attachment::check_bytes(saved.as_bytes()).unwrap();
        let saved_owner=attachment::restore_saved_runtime_payload(&checked).unwrap();
        let context=attachment::check_saved_revision_context(&checked,&saved_owner,&[],&[9]).unwrap();
        let world=original.encode_world_frame().unwrap();
        let raw=original.encode_checkpoint_request(&world,&context,&[],&[9]).unwrap();
        let before=original.session().summary();
        let mut wrong=binding.clone();wrong.initial_rng^=1;
        assert!(CheckpointedLogicalRunV1::restore_local_checkpoint(&raw,&context,&wrong).is_err());
        let mut damaged=raw.clone();*damaged.last_mut().unwrap()^=1;
        assert!(CheckpointedLogicalRunV1::restore_local_checkpoint(&damaged,&context,&binding).is_err());
        assert_eq!(original.session().summary(),before);
        let mut restored=CheckpointedLogicalRunV1::restore_local_checkpoint(&raw,&context,&binding).unwrap();
        assert!(restored.provenance.local_restored);
        assert_eq!(restored.require_native_checkpoint_provenance(),Err(Failure::CustodyConflict));
        #[cfg(not(target_arch="wasm32"))]
        assert!(matches!(crate::snapshot_v2_durable_transaction::ActualPreparedRequestV1::from_logical(
            &restored,&context,&[],&[9]),Err(Failure::CustodyConflict)));
        assert!(original.require_native_checkpoint_provenance().is_ok());
        assert!(restored.provenance.prior_durable_receipts.is_empty());
        assert_eq!(restored.session().summary(),before);
        for madi in 2..4 {
            assert_eq!(restored.step(&input(madi)).unwrap(),original.step(&input(madi)).unwrap());
        }
        let second=restored.export_saved_artifact(&[9],&[10]).unwrap();
        let checked=attachment::check_bytes(second.as_bytes()).unwrap();
        let owner=attachment::restore_saved_runtime_payload(&checked).unwrap();
        let context=attachment::check_saved_revision_context(&checked,&owner,&[9],&[10]).unwrap();
        let raw=restored.encode_checkpoint_request(&restored.encode_world_frame().unwrap(),&context,&[9],&[10]).unwrap();
        let mut cold=CheckpointedLogicalRunV1::restore_local_checkpoint(&raw,&context,&binding).unwrap();
        assert_eq!(cold.step(&input(4)).unwrap(),original.step(&input(4)).unwrap());
    }
    use super::*;
    #[test]
    fn general_input_v3_preserves_edges_and_rejects_seed_tick_and_trailing_bytes() {
        let make = |tick, edges| GeneralProcessedInputRow { input: ddonirang_core::platform::InputSnapshot {
            tick_id: tick, dt: ddonirang_core::Fixed64::ONE, keys_pressed: 3,
            last_key_name: "왼쪽".into(), pointer_x_i32: -17, pointer_y_i32: 23,
            ai_injections: Vec::new(), net_events: Vec::new(),
            frame_source: ddonirang_core::InputSource::Person, rng_seed: 7 ^ tick,
        }, edges };
        for rows in [vec![], vec![make(0, None)], vec![make(0, Some((0,0))), make(1, Some((2,1)))]] {
            let bytes = encode_general_processed_inputs(&rows,0,7).unwrap();
            assert_eq!(decode_general_processed_inputs(&bytes,0,7).unwrap(), rows);
            let mut trailing = bytes.clone(); trailing.push(0);
            assert!(decode_general_processed_inputs(&trailing,0,7).is_err());
            if !rows.is_empty() { assert!(decode_general_processed_inputs(&bytes,0,8).is_err()); }
        }
        assert_ne!(encode_general_processed_inputs(&[make(0,None)],0,7).unwrap(),
            encode_general_processed_inputs(&[make(0,Some((0,0)))],0,7).unwrap());
        assert!(encode_general_processed_inputs(&[make(1,None)],0,7).is_err());
        assert!(encode_general_processed_inputs(&[make(u64::MAX,None)],u64::MAX,7).is_err());
    }
    #[test]
    fn authored_tetris_uses_real_named_start_and_source_owner() {
        // Match the existing Windows Native product's 16MB stack profile;
        // the test harness's default 2MB is not that product execution profile.
        fn check() {
        let request=serde_json::json!({"source_profile":"v1-core-v25","module_name":"",
            "source_snapshot":{"root_path":"root.ddn","entry_path":"main.ddn","files":[
                {"path":"root.ddn","text":include_str!("../../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/root.ddn")},
                {"path":"main.ddn","text":include_str!("../../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/main.ddn")}]}});
        let authored=crate::authored_project::operate("create",&request.to_string()).unwrap();
        let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(authored.as_bytes()).unwrap();
        let program=crate::ddn_runtime::DdnProgram::from_saved_v25_project_with_mode(
            owned.sources().clone(),ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target()).unwrap();
        let preparation=owned.sources().preparation();
        crate::runtime_surface::prepare_supported_source_for_execution_with_program(
            &preparation.source,&preparation.target_path,&program).unwrap();
        for (source,path) in [(format!("{}\n// changed",preparation.source),preparation.target_path.clone()),
            (preparation.source.clone(),"other.ddn".into())] {
            let error=crate::runtime_surface::prepare_supported_source_for_execution_with_program(
                &source,&path,&program).unwrap_err();
            assert_eq!(error.code,"E_SOURCE_ADMISSION_CUSTODY");
        }
        let mut runner=crate::ddn_runtime::DdnRunner::new(program,owned.sources().entry_declaration().name());
        runner.configure_project_entry(owned.sources().entry_declaration().name()).unwrap();
        encode_execution_contract_supply(&owned,&runner).unwrap();
        let mut binding:crate::logical_input_tape::LogicalInputBinding=serde_json::from_str(
            include_str!("../../../solutions/seamgrim_ui_mvp/samples/alpha2_logical_tetris.binding.json")).unwrap();
        binding.source_revision=crate::runtime_surface::sha256_identity(owned.sources().preparation().source.as_bytes());
        crate::logical_input_tape::LogicalInputSession::from_saved_runner(&owned,binding.clone(),runner).unwrap();
        let issued=crate::project_snapshot_identity::current::issue_saved_local_project(&owned).unwrap();
        let empty=[0;8];
        super::super::complete_state_frame::CompleteStateFrameV1 {pesi:issued.pesi().as_raw(),
            resources:&empty,parameters:&empty,next_tick:0,subjects:&empty,atom_evidence:&[],subject_custody:&[]}.encode_canonical().unwrap();
        owned.project().save().unwrap();
        let mut run=CheckpointedLogicalRunV1::fresh(&owned,binding.clone(),crate::ddn_runtime::default_age_target()).unwrap();
        let original=include_str!("../../../solutions/seamgrim_ui_mvp/samples/alpha2_logical_tetris.ddn");
        let mut old_binding=binding.clone();
        old_binding.source_revision=crate::runtime_surface::sha256_identity(original.as_bytes());
        let mut old=crate::logical_input_tape::LogicalInputSession::new(original,old_binding).unwrap();
        let mut held=binding.initial_held.clone();
        let moves:Vec<Vec<(&str,&str)>>=vec![vec![],vec![("왼쪽","Press")],
            vec![("왼쪽","Release"),("회전","Press")],vec![("회전","Release"),("떨어뜨리기","Press")],
            vec![("떨어뜨리기","Release")],vec![("오른쪽","Press")],vec![("오른쪽","Release")]];
        let impact=std::env::var("DDN_TETRIS_HASH_IMPACT_CAPTURE").ok().map(|capture|{
            let captured:serde_json::Value=serde_json::from_slice(&std::fs::read(capture).unwrap()).unwrap();
            let output=std::path::PathBuf::from(std::env::var("DDN_TETRIS_HASH_PREIMAGE_OUTPUT").unwrap());
            std::fs::create_dir(&output).unwrap();(captured,output)
        });
        let mut preimage_rows=Vec::new();
        for (madi,events) in moves.into_iter().enumerate() {
            let events:Vec<_>=events.into_iter().enumerate().map(|(ordinal,(channel,kind))| {
                held.insert(channel.into(),kind!="Release");
                serde_json::json!({"ordinal":ordinal,"channel":channel,"event":{"kind":kind}})
            }).collect();
            let input=serde_json::from_value(serde_json::json!({"madi":madi,"events":events,"held":held})).unwrap();
            let actual=run.step(&input).unwrap();let previous=old.step(&input).unwrap();
            assert_eq!(actual["frame"]["accepted_world_digest"],previous["frame"]["accepted_world_digest"],"madi {madi}");
            if let Some((capture,output))=&impact {
                use ddonirang_core::platform::{NuriWorld,ResourceValue};
                use sha2::{Digest,Sha256};
                let world=run.session().committed_world();
                assert_eq!(world.next_entity_cursor(),0);assert!(world.component_json_entries().is_empty());
                assert!(world.resource_handle_entries().is_empty());
                let mut historical=NuriWorld::new();
                for (name,value) in world.resource_fixed64_entries(){historical.set_resource_fixed64(name,value);}
                for (name,value) in world.resource_value_entries(){
                    match (name.as_str(),&value){
                        ("고정",ResourceValue::Bool(value))=>historical.set_resource_json(name,if *value{"참"}else{"거짓"}.into()),
                        ("행동",ResourceValue::String(value))=>historical.set_resource_json(name.clone(),value.clone()),
                        _=>historical.set_resource_value(name,value),
                    }
                }
                // Diagnostic historical reconstruction ONLY. Bind both full
                // hashes to the actual fixed r17/new WASM execution capture;
                // this must never become a product old-state admission path.
                let old_hash=crate::runtime_surface::current_state_hash(&historical);
                let new_hash=crate::runtime_surface::current_state_hash(world);
                assert_eq!(old_hash,capture["rows"][madi]["old_state_hash"].as_str().unwrap());
                assert_eq!(new_hash,capture["rows"][madi]["new_state_hash"].as_str().unwrap());
                let old_bytes=historical.encode_canonical_with_filter(crate::runtime_surface::DEFAULT_VIEW_PREFIXES);
                let new_bytes=world.encode_canonical_with_filter(crate::runtime_surface::DEFAULT_VIEW_PREFIXES);
                let first_difference=old_bytes.iter().zip(&new_bytes).position(|(a,b)|a!=b)
                    .or_else(||(old_bytes.len()!=new_bytes.len()).then_some(old_bytes.len().min(new_bytes.len())));
                std::fs::write(output.join(format!("madi-{madi}-r17.detbin")),&old_bytes).unwrap();
                std::fs::write(output.join(format!("madi-{madi}-typed.detbin")),&new_bytes).unwrap();
                preimage_rows.push(serde_json::json!({"madi":madi,"old_state_hash":old_hash,"new_state_hash":new_hash,
                    "old_bytes":old_bytes.len(),"new_bytes":new_bytes.len(),"first_difference":first_difference,
                    "old_preimage_sha256":hex::encode(Sha256::digest(&old_bytes)),
                    "new_preimage_sha256":hex::encode(Sha256::digest(&new_bytes)),"actual_wasm_hashes_matched":true}));
            }
        }
        if let Some((_,output))=&impact{
            std::fs::write(output.join("RESULT.json"),serde_json::to_vec_pretty(&serde_json::json!({
                "scope":"EXACT_TETRIS_HASH_PREIMAGE_IMPACT_NOT_LEGACY_RESTORE","rows":preimage_rows})).unwrap()).unwrap();
        }
        let world=run.session().committed_world();
        assert!(world.resource_json_entries().is_empty());
        assert!(matches!(world.get_resource_value("고정"),Some(ddonirang_core::platform::ResourceValue::Bool(_))));
        assert!(matches!(world.get_resource_value("행동"),Some(ddonirang_core::platform::ResourceValue::String(_))));
        let before=run.session().summary()["state_hash"].clone();
        if let Some(path)=std::env::var_os("DDN_N1_TETRIS_SAMPLE_EVIDENCE") {
            // Fixture assembly consumes the real source-bound session. This
            // does not alter source, invent a binding, or replace CLI replay.
            use std::io::Write;
            let bytes=serde_json::to_vec_pretty(&serde_json::json!({
                "scope":"ACTUAL_AUTHORED_TETRIS_SESSION_FIXTURE_SUPPLY_NOT_CLI_OR_WASM_PASS",
                "binding":binding,"seven_madi_state_hash":before,
                "single_source_revision":crate::runtime_surface::sha256_identity(original.as_bytes())
            })).unwrap();
            std::fs::OpenOptions::new().create_new(true).write(true).open(path).unwrap().write_all(&bytes).unwrap();
        }
        run.check_checkpoint_channels().unwrap();
        // R1.1 existing typed leaves/table, not JSON embedded in a binary frame.
        assert!(run.encode_world_frame().is_ok());
        assert_eq!(run.session().summary()["state_hash"],before);
        #[cfg(not(target_arch="wasm32"))]
        {
            use crate::snapshot_v2_durable_transaction::{TransactionEvidenceV1,
                SnapshotV2TransactionStateV1 as State,native::SqliteAdapterV1};
            let artifact=run.export_saved_artifact(&[],&[2]).unwrap();
            let checked=super::super::project_runtime_attachment::check_bytes(artifact.as_bytes()).unwrap();
            let context=super::super::project_runtime_attachment::check_saved_revision_context(&checked,&owned,&[],&[2]).unwrap();
            let frame=run.encode_world_frame().unwrap();
            let raw=run.encode_checkpoint_request(&frame,&context,&[],&[2]).unwrap();
            let root=std::env::temp_dir().join(format!("ddn-typed-tetris-post-cas-{}-{}",std::process::id(),
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
            std::fs::create_dir(&root).unwrap();
            let adapter=SqliteAdapterV1::provision(&root).unwrap();
            let receipt=run.save_native_checkpoint(&raw,&context,&adapter,None).unwrap();
            let request=crate::snapshot_v2_host_bridge::check_resumable_request_candidate_v2(&raw,issued.pesi().as_raw()).unwrap();
            let digest=request.binding.transaction_digest().unwrap();
            let exact=adapter.reread_committed(&digest,&request.binding.project,&[2]).unwrap();
            let payload=exact.payload.as_ref().unwrap();
            // Actual admitted Tetris 19-field payload and real SQLite head.
            // Failure injection is at the shared post-CAS comparison gate,
            // never a production failpoint or an altered snapshot encoder.
            let mut evidence=TransactionEvidenceV1::prepared(payload.binding.clone()).unwrap();
            evidence.synced().unwrap();evidence.verified(&payload.snapshot,&payload.snapshot).unwrap();
            evidence.head_changed().unwrap();
            evidence.container_confirmation(adapter.confirm_container_durable()).unwrap();
            let mut bad=adapter.reread_committed(&digest,&request.binding.project,&[2]).unwrap();
            bad.payload.as_mut().unwrap().snapshot.last_mut().map(|byte|*byte^=1);
            assert!(evidence.committed_native_chain(payload,&bad).is_err());
            assert_eq!(evidence.state(),State::RecoveryRequired);
            assert!(evidence.check_committed_receipt_evidence(&receipt,adapter.capability().unwrap().as_bytes()).is_err());
            assert_eq!(run.session().summary()["state_hash"],before);
            let reread=adapter.reread_committed(&digest,&request.binding.project,&[2]).unwrap();
            assert_eq!(reread.head,exact.head,"post-CAS failure does not invent a rollback");
            assert!(evidence.committed_native_chain(payload,&reread).is_err(),"exact reread alone is not fresh durability confirmation");
            evidence.container_confirmation(adapter.confirm_container_durable()).unwrap();
            evidence.committed_native_chain(payload,&reread).unwrap();
            assert_eq!(evidence.state(),State::Committed);
        }
        }
        std::thread::Builder::new().stack_size(16*1024*1024).spawn(check).unwrap().join().unwrap();
    }
    fn bytes32(out:&mut Vec<u8>,bytes:&[u8]){out.extend((bytes.len() as u32).to_be_bytes());out.extend(bytes);}
    #[test]
    fn exact_boundary_sections_preserve_progress_and_reject_reset_or_hidden_state(){
        let expected=crate::work_cost::WorkCostMeter::default().receipt();
        let mut rng=vec![0,1,1];bytes32(&mut rng,b"verified-profile");
        for value in [9u64,3,9^3,0]{rng.extend(value.to_be_bytes());}
        let mut position=vec![0,1,1];position.extend(3u64.to_be_bytes());
        bytes32(&mut position,b"entry");bytes32(&mut position,b"resume");position.push(0);
        let mut work=vec![0,1,15];work.extend([0;15*8]);work.push(10);
        let l=expected.limits;
        for value in [l.slice_committed_madi,l.slice_work_cost,l.slice_solver_evaluations,l.slice_external_events,
            l.call_depth,l.created_identities,l.public_observation_samples,l.sealed_input_bytes,l.result_artifact_bytes,l.replay_capsule_bytes]{work.extend(value.to_be_bytes());}
        work.push(0);
        let check=|r:&[u8],p:&[u8],w:&[u8]|decode_boundary_sections(r,p,w,3,b"verified-profile",b"entry",b"resume",&expected);
        let result=check(&rng,&position,&work).unwrap();
        // R2 mode03 has identical framing but a distinct trusted strategy.
        let mut logical=vec![0,1,3];bytes32(&mut logical,LOGICAL_FIXED_SEED_PROFILE);
        for value in [7u64,3,7,0] {logical.extend(value.to_be_bytes());}
        let logical_check=|r:&[u8]|decode_boundary_sections(r,&position,&work,3,
            LOGICAL_FIXED_SEED_PROFILE,b"entry",b"resume",&expected);
        assert_eq!(logical_check(&logical).unwrap().base_seed,7);
        assert!(check(&logical,&position,&work).is_err());
        assert!(logical_check(&rng).is_err());
        for index in [2,logical.len()-24,logical.len()-16,logical.len()-8] {
            let mut changed=logical.clone();changed[index]^=1;
            assert!(logical_check(&changed).is_err());
        }
        let mut trailing=logical.clone();trailing.push(0);
        assert!(logical_check(&trailing).is_err());
        let mut vector=vec![0,1,3];bytes32(&mut vector,LOGICAL_FIXED_SEED_PROFILE);
        for value in [7u64,2,7,0] {vector.extend(value.to_be_bytes());}
        assert_eq!(vector.len(),71);
        use sha2::Digest;
        assert_eq!(hex::encode(sha2::Sha256::digest(&vector)),
            "46736a9211b9ddefbb400a20a2ec5756d0ef0fecd7fe73aea42d5e1e43757382");
        assert_eq!(result.base_seed,9);assert_eq!(result.next_tick,3);
        assert_eq!(result.counters,expected.counters);assert_eq!(result.limits,expected.limits);
        assert!(!result.rd206_resource_capability);
        let mut reset=rng.clone();let offset=reset.len()-16;reset[offset..offset+8].copy_from_slice(&9u64.to_be_bytes());
        assert!(matches!(check(&reset,&position,&work),Err(Failure::CustodyConflict)));
        let mut stream=rng.clone();stream[2]=2;
        assert!(matches!(check(&stream,&position,&work),Err(Failure::InvalidInput)));
        let mut suspended=position.clone();*suspended.last_mut().unwrap()=1;
        assert!(check(&rng,&suspended,&work).is_err());
        assert!(check_execution_position_supply(&position,2,b"entry",b"resume").is_err());
        assert!(check_execution_position_supply(&position,3,b"foreign-entry",b"resume").is_err());
        assert!(check_execution_position_supply(&position,3,b"entry",b"foreign-resume").is_err());
        let mut position_suffix=position.clone();position_suffix.push(0);
        assert!(check_execution_position_supply(&position_suffix,3,b"entry",b"resume").is_err());
        let mut changed=work.clone();changed[10]=1;
        assert!(matches!(check(&rng,&position,&changed),Err(Failure::CustodyConflict)));
        let mut capability=work.clone();*capability.last_mut().unwrap()=1;
        assert!(check(&rng,&position,&capability).is_err());
        let mut suffix=work.clone();suffix.push(0);assert!(check(&rng,&position,&suffix).is_err());
        for prefix in [0,1,2,work.len()-1]{assert!(check(&rng,&position,&work[..prefix]).is_err());}
    }
}
