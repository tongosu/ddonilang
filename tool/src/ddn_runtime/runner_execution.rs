//! Runner lifecycle and per-madi execution; implementations moved unchanged.
use super::*;

impl DdnRunner {
    pub(crate) fn saved_tuple_restore_plan(&self) -> Option<SavedTupleRestorePlan> {
        let saved = self.program.sealed_project_sources()?;
        if !self.named_entry_after_definitions || self.update_name != saved.entry_declaration().name() { return None; }
        Some(SavedTupleRestorePlan { plan: crate::tuple_saved_type_plan::TupleSavedTypePlan::compile(
            &self.program.program, Some(&self.update_name), false, &self.program.top_level_decl_names, &self.program.record_schemas,
            self.program.numeric_constraint_plan.as_deref(),&self.program.nominal_type_ids()) })
    }

    pub(crate) fn validate_persisted_records(&self, world: &NuriWorld) -> Result<(), String> {
        world.validate_typed_resource_lanes().map_err(|reason|
            format!("E_RESOURCE_SNAPSHOT_INVALID: {reason}"))?;
        self.validate_derived_world(world,false)?;
        // The shared parser places top-level chaebi declarations in the
        // update body's prefix. Local declarations of other callables do not
        // constrain world resources and must not be collected here.
        let update_stmts = self.program.functions.get(&self.update_name)
            .and_then(|seed| seed.body.as_ref())
            .map(|body| body.stmts.as_slice()).unwrap_or(&[]);
        for (name, value) in world.resource_value_entries() {
            if crate::unit_wire::saved_resource_contains_tuple(&value) {
                // Live madis use the current compiler-owned program. The
                // separate restore boundary requires SavedTupleRestorePlan.
                if self.tuple_type_plan.equation_result(&name) {
                    equation_saved_value::validate_saved(&value)?;
                } else {
                let kind = self.tuple_type_plan.expected(&name).ok_or("E_TUPLE_PERSISTENCE_UNBOUND: 현재 원문의 유일한 위치씨 결속이 필요합니다")?;
                crate::unit_wire::validate_saved_value_type(&value, kind)
                    .map_err(|_| "E_TUPLE_PERSISTENCE_UNBOUND: 현재 원문의 위치씨와 세계 값이 다릅니다".to_owned())?;
                }
            }
            // This public world boundary precedes candidate/input mutation.
            // Validate while converting, without decoding admitted Tensor
            // values twice or turning a malformed carrier into a missing value.
            let restored = super::value_bridge::try_value_from_resource_value(&value)?;
            ddonirang_lang::runtime::validate_record_tree(&restored).map_err(|detail|
                type_mismatch_error(&name, &detail.expected, &detail.actual).to_string())?;
            for stmt in self.program.root_stmts.iter().chain(update_stmts.iter()) {
                if let Stmt::DeclBlock { items, .. } = stmt {
                    for item in items.iter().filter(|item| item.name == name
                        && self.program.top_level_decl_names.contains(&item.name)) {
                        if matches!(&restored, Value::Record { .. }|Value::NominalValueV1(_)|Value::SubjectHandleV1(_)) ||
                            matches!(&item.type_ref, TypeRef::Named(owner) if self.program.record_schemas.contains_key(owner)
                                || owner=="글" || owner=="참거짓") {
                            self.program.check_value_type(&restored,&item.type_ref)
                                .map_err(|detail| type_mismatch_error(&name, &detail.expected, &detail.actual).to_string())?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn new(program: DdnProgram, update_name: &str) -> Self {
        let temporary_subject_bootstrap=program.temporary_subject_bootstrap.clone();
        let tuple_type_plan = Rc::new(crate::tuple_saved_type_plan::TupleSavedTypePlan::compile(
            &program.program, Some(update_name), true, &program.top_level_decl_names, &program.record_schemas, program.numeric_constraint_plan.as_deref(),&program.nominal_type_ids()));
        Self {
            subject_bootstrap:temporary_subject_bootstrap,
            tuple_type_plan,
            program,
            update_name: update_name.to_string(),
            named_entry_after_definitions: false,
            bogae_activation_requested: false,
            bogae_entry: None,
            authored_bogae: None,
            bogae_history: crate::bogae_time::SceneHistory::default(),
            prev_keys_pressed: 0,
            project_asset_registry: None,
            work_cost: Rc::new(RefCell::new(WorkCostMeter::default())),
            async_queue_capacity: ddonirang_lang::DEFAULT_ASYNC_QUEUE_CAPACITY,
            internal_dispatch: InternalDdnRunnerDispatchState::default(),
        }
    }

    /// Count only explicit RD-445 owned named providers. Historical receive
    /// hooks never become providers merely because they have matching bodies.
    pub fn public_provider_source_ingress_count(&self) -> usize {
        let mut identities = std::collections::BTreeSet::new();
        for seed in self.program.functions.values().filter(|seed|matches!(&seed.seed_kind,
            SeedKind::Named(kind) if kind=="임자")) {
            if let Some(body)=&seed.body {
                for stmt in &body.stmts {
                    if let Stmt::Definition {id,reaction:Some(_),..}=stmt {
                        identities.insert((seed.id,*id));
                    }
                }
            }
        }
        identities.len()
    }

    /// Public run frontdoors activate views; low-level constructor evaluation
    /// remains separate and does not manufacture an activation request.
    pub fn with_bogae_activation(mut self) -> Self {
        self.bogae_activation_requested = true;
        self
    }

    /// Product frontdoors supply an explicit authored definition, not a name
    /// heuristic. Value type is checked before commit by projection admission.
    pub fn configure_bogae_entry(&mut self, name: &str) -> Result<(), String> {
        if self.authored_bogae.as_ref().is_some_and(|activation| activation.entry_definition() != Some(name)) {
            return Err("E_AUTHORED_BOGAE_ENTRY: 저장된 시작 정체를 다른 이름으로 덮어쓸 수 없습니다".into());
        }
        if !self.program.root_stmts.iter().any(|stmt| matches!(stmt,
            Stmt::Definition {name:defined,..} if defined == name)) {
            return Err("E_BOGAE_ENTRY_UNRESOLVED: 명시 보개 시작 정의가 없습니다".into());
        }
        self.bogae_entry = Some(name.to_owned());
        Ok(())
    }

    /// Only an explicitly authored, sealed project can supply permanent IDs.
    /// Admission is atomic and never creates an identity while executing.
    pub fn configure_authored_bogae(&mut self, artifact_json: &str) -> Result<(), String> {
        let origin = &self.program.program.origin;
        let activation = crate::authored_bogae::Activation::from_artifact(
            artifact_json, &origin.source, &origin.file_path)?;
        let project = crate::authored_project::open(artifact_json.as_bytes())?;
        let (_, _, project_start) = project.execution_source()?;
        // Check both independent entry authorities before modifying the runner.
        // A supplied graph start must not be replaced by a host-chosen function.
        if let Some(start) = &project_start {
            let seed = self.program.functions.get(start)
                .ok_or_else(|| "E_PROJECT_START_DEFINITION: 저장된 시작 정의가 없습니다".to_string())?;
            if !seed.params.is_empty() || seed.body.is_none() || seed.seed_kind != SeedKind::Umjikssi {
                return Err("E_PROJECT_START_SIGNATURE: 저장된 시작은 인자 없는 움직씨여야 합니다".into());
            }
            if self.named_entry_after_definitions && self.update_name != *start {
                return Err("E_PROJECT_START_OVERRIDE: 저장된 작품 시작을 덮어쓸 수 없습니다".into());
            }
        }
        if let Some(entry) = activation.entry_definition() {
            let entry = entry.to_owned();
            self.configure_bogae_entry(&entry)?;
        } else if self.bogae_entry.is_some() {
            return Err("E_AUTHORED_BOGAE_ENTRY: 명시 저장 시작이 없는 자료에 임시 시작을 덧붙일 수 없습니다".into());
        }
        if let Some(start) = project_start {
            self.configure_project_entry(&start)?;
        }
        self.authored_bogae = Some(activation);
        Ok(())
    }

    /// Formal work startup evaluates authored root definitions before calling
    /// the explicitly resolved entry. This is not an implicit source wrapper.
    pub fn with_named_entry_after_definitions(mut self) -> Self {
        self.named_entry_after_definitions = true;
        for stmt in &self.program.root_stmts {
            if let Stmt::Definition { name, .. } = stmt {
                self.program.top_level_decl_names.insert(name.clone());
            }
        }
        self.refresh_tuple_type_plan();
        self
    }

    pub fn configure_project_entry(&mut self, name: &str) -> Result<(), String> {
        let entry = self.program.functions.get(name)
            .ok_or_else(|| format!("E_PROJECT_START_DEFINITION: 시작 정의가 없습니다: {name}"))?;
        if !entry.params.is_empty() || entry.body.is_none() || !matches!(entry.seed_kind, SeedKind::Umjikssi) {
            return Err("E_PROJECT_START_SIGNATURE: 시작은 인자 없는 실행 가능한 정의여야 합니다".into());
        }
        self.update_name = name.to_owned();
        self.named_entry_after_definitions = true;
        for stmt in &self.program.root_stmts {
            if let Stmt::Definition { name, .. } = stmt {
                self.program.top_level_decl_names.insert(name.clone());
            }
        }
        self.refresh_tuple_type_plan();
        Ok(())
    }

    pub(super) fn refresh_tuple_type_plan(&mut self) {
        self.tuple_type_plan = Rc::new(crate::tuple_saved_type_plan::TupleSavedTypePlan::compile(
            &self.program.program, Some(&self.update_name), !self.named_entry_after_definitions,
            &self.program.top_level_decl_names, &self.program.record_schemas, self.program.numeric_constraint_plan.as_deref(),&self.program.nominal_type_ids()));
    }

    /// Product-internal typed registry injection for W1-C2 integration
    /// evidence. No parser, CLI, environment, config, WASM, or Workbench path
    /// calls this method.
    pub(crate) fn with_internal_reaction_provider_registry(
        mut self,
        registry: ReactionProviderRegistry<InternalReactionProgram>,
        execution_identity: ExecutionInstanceIdentity,
    ) -> Self {
        self.internal_dispatch.provider_registry = registry;
        self.internal_dispatch.execution_identity = execution_identity;
        self
    }

    /// Overrides internal graph reachability only for typed Rust integration
    /// evidence (diamond paths and worker permutations). It is not public DDN
    /// input and is deliberately absent from every product frontdoor.
    pub(crate) fn with_internal_provider_reachability(
        mut self,
        reached: Vec<ProviderRegistrationIdentity>,
    ) -> Self {
        self.internal_dispatch.reached_registration_override = Some(reached);
        self
    }

    pub(crate) fn with_internal_reaction_runtime(
        mut self,
        runtime: InternalTypedReactionRuntime,
    ) -> Self {
        self.internal_dispatch.runtime = runtime;
        self
    }

    pub(crate) fn internal_reaction_runtime(&self) -> &InternalTypedReactionRuntime {
        &self.internal_dispatch.runtime
    }

    pub(crate) fn last_internal_reaction_success(&self) -> Option<&InternalReactionSuccess> {
        self.internal_dispatch.last_success.as_ref()
    }

    pub(crate) fn internal_reaction_commit_events(&self) -> &[String] {
        &self.internal_dispatch.commit_events
    }

    pub(crate) fn internal_effect_sink_observations(&self) -> Vec<InternalEffectSinkObservation> {
        self.internal_dispatch
            .effect_sink_observations
            .borrow()
            .clone()
    }

    pub(crate) fn internal_provider_registry_len(&self) -> usize {
        self.internal_dispatch.provider_registry.internal_len()
    }

    pub fn with_standard_resource_limits(mut self, limits: StandardResourceLimitsV1) -> Self {
        self.work_cost = Rc::new(RefCell::new(WorkCostMeter::new(limits)));
        self
    }

    /// Internal product capacity hook. Capacity is a runtime contract input;
    /// full admission remains logical WAIT and never becomes drop/success.
    pub fn with_async_queue_capacity(mut self, capacity: usize) -> Self {
        assert!(capacity > 0, "async queue capacity must be non-zero");
        self.async_queue_capacity = capacity;
        if self.internal_dispatch.runtime.revision() == 0 {
            self.internal_dispatch.runtime =
                InternalTypedReactionRuntime::with_world_and_queue_capacity(
                    BTreeMap::new(),
                    capacity,
                );
        }
        self
    }

    pub fn work_cost_receipt(&self) -> WorkCostReceipt {
        self.work_cost.borrow().receipt()
    }

    pub(crate) fn saved_program_sources(&self) -> Option<&crate::project_revision_envelope_supply::VerifiedSavedV25Sources> {
        self.program.sealed_project_sources()
    }

    /// Existing dispatch owner encoding, not a new resume identity issuer.
    pub(crate) fn execution_instance_owner_bytes(&self) -> &[u8] {
        self.internal_dispatch.execution_identity.as_str().as_bytes()
    }

    pub(crate) fn configured_saved_start(&self) -> Option<&str> {
        self.named_entry_after_definitions.then_some(self.update_name.as_str())
    }

    pub fn set_sealed_input_bytes(&mut self, bytes: usize) -> Result<(), WorkCostFailure> {
        self.work_cost.borrow_mut().set_sealed_input_bytes(bytes)
    }

    pub fn set_result_artifact_bytes(&mut self, bytes: usize) -> Result<(), WorkCostFailure> {
        self.work_cost.borrow_mut().set_result_artifact_bytes(bytes)
    }

    pub fn set_replay_capsule_bytes(&mut self, bytes: usize) -> Result<(), WorkCostFailure> {
        self.work_cost.borrow_mut().set_replay_capsule_bytes(bytes)
    }

    pub fn host_preflight_rejected(&self, fact: impl Into<String>) -> WorkCostFailure {
        self.work_cost.borrow().host_preflight_rejected(fact)
    }

    pub fn with_project_asset_registry(
        mut self,
        registry: Arc<gate0_registry::ProjectAssetRegistry>,
    ) -> Self {
        self.project_asset_registry = Some(registry);
        self
    }

    pub fn set_project_asset_registry(
        &mut self,
        registry: Option<Arc<gate0_registry::ProjectAssetRegistry>>,
    ) {
        self.project_asset_registry = registry;
    }

    pub fn run_update(
        &mut self,
        world: &NuriWorld,
        input: &InputSnapshot,
        defaults: &HashMap<String, Value>,
    ) -> Result<DdnRunOutput, String> {
        let update_name = self.update_name.clone();
        self.run_named_seed(world, input, defaults, None, &update_name, false)
            .map_err(|err| err.to_string())
    }

    /// Shared logical-profile execution. Physical key/device inputs are absent,
    /// not back-filled from actions. The caller owns admitted source/binding
    /// identity; the returned receipt binds the logical snapshot, not a V1 tape.
    pub fn run_update_logical(
        &mut self,
        world: &NuriWorld,
        binding: &crate::logical_input_tape::LogicalInputBinding,
        snapshot: &crate::logical_input_tape::LogicalSnapshot,
        defaults: &HashMap<String, Value>,
    ) -> Result<(DdnRunOutput, serde_json::Value), String> {
        self.run_update_logical_with_initial_held(world, binding, snapshot, defaults, None)
    }

    pub(crate) fn run_update_logical_with_initial_held(
        &mut self, world: &NuriWorld,
        binding: &crate::logical_input_tape::LogicalInputBinding,
        snapshot: &crate::logical_input_tape::LogicalSnapshot,
        defaults: &HashMap<String, Value>,
        initial_held: Option<&BTreeMap<String, bool>>,
    ) -> Result<(DdnRunOutput, serde_json::Value), String> {
        let (output, receipt, _) = self.run_update_logical_retaining_input(
            world, binding, snapshot, defaults, initial_held,
        )?;
        Ok((output, receipt))
    }

    /// Return the actual core input passed to execution, never a reconstructed history.
    pub(crate) fn run_update_logical_retaining_input(
        &mut self, world: &NuriWorld,
        binding: &crate::logical_input_tape::LogicalInputBinding,
        snapshot: &crate::logical_input_tape::LogicalSnapshot,
        defaults: &HashMap<String, Value>,
        initial_held: Option<&BTreeMap<String, bool>>,
    ) -> Result<(DdnRunOutput, serde_json::Value, InputSnapshot), String> {
        let query = crate::logical_input_tape::query(binding, snapshot)?;
        let query = if let Some(initial) = initial_held {
            query.with_initial_held(initial.clone()).map_err(|_| "E_LOGICAL_INPUT_STATE: 사건 전후 눌림 상태가 다릅니다".to_string())?
        } else { query };
        let input = InputSnapshot {
            tick_id: snapshot.madi,
            dt: Fixed64::from_raw_i64(binding.dt_raw),
            rng_seed: binding.initial_rng,
            keys_pressed: 0, last_key_name: String::new(),
            pointer_x_i32: 0, pointer_y_i32: 0,
            ai_injections: vec![], net_events: vec![],
            frame_source: ddonirang_core::InputSource::Person,
        };
        let update_name = self.update_name.clone();
        let output = self.run_named_seed_with_input(
            world, &input, defaults, None, &update_name, false, None, Some(&query),
        ).map_err(|error| error.to_string())?;
        let receipt = serde_json::json!({
            "input_profile": binding.source_profile,
            "input_binding": binding,
            "input_snapshot": snapshot,
            "input_snapshot_sha256": crate::logical_input_tape::snapshot_digest(snapshot)?,
            "input_reads": output.input_reads,
        });
        Ok((output, receipt, input))
    }

    // WO15 provisional typed manipulation intake. The common codec validates
    // exact payloads; DOM/renderers never write world resources directly.
    pub(crate) fn run_update_manipulation_with_initial_held(
        &mut self, world: &NuriWorld,
        binding: &crate::manipulation_input::ManipulationBinding,
        snapshot: &crate::manipulation_input::ManipulationSnapshot,
        defaults: &HashMap<String, Value>,
        initial_held: Option<&BTreeMap<String, bool>>,
    ) -> Result<(DdnRunOutput, serde_json::Value), String> {
        let query = crate::manipulation_input::query(binding, snapshot)?;
        let query = if let Some(initial) = initial_held {
            query.with_initial_held(initial.clone()).map_err(|_|
                "E_MANIPULATION_INPUT_STATE: 사건 전후 눌림 상태가 다릅니다".to_string())?
        } else { query };
        let input = InputSnapshot {
            tick_id: snapshot.madi, dt: Fixed64::from_raw_i64(binding.dt_raw),
            rng_seed: binding.initial_rng, keys_pressed: 0, last_key_name: String::new(),
            pointer_x_i32: 0, pointer_y_i32: 0, ai_injections: vec![], net_events: vec![],
            frame_source: ddonirang_core::InputSource::Person,
        };
        let update_name = self.update_name.clone();
        let output = self.run_named_seed_with_input(world, &input, defaults, None,
            &update_name, false, None, Some(&query)).map_err(|error| error.to_string())?;
        let receipt = serde_json::json!({
            "input_profile": binding.source_profile, "input_binding": binding,
            "input_snapshot": snapshot,
            "input_snapshot_sha256": crate::manipulation_input::snapshot_digest(snapshot)?,
            "input_reads": output.input_reads,
        });
        Ok((output, receipt))
    }

    pub(crate) fn run_update_diagnostic(
        &mut self,
        world: &NuriWorld,
        input: &InputSnapshot,
        defaults: &HashMap<String, Value>,
    ) -> Result<DdnRunOutput, EvalError> {
        let update_name = self.update_name.clone();
        self.run_named_seed(world, input, defaults, None, &update_name, false)
    }

    pub(crate) fn run_update_diagnostic_with_control_edges(
        &mut self,
        world: &NuriWorld,
        input: &InputSnapshot,
        defaults: &HashMap<String, Value>,
        edges: Option<(u64, u64)>,
        param_overrides: Option<&BTreeMap<String, Value>>,
    ) -> Result<DdnRunOutput, EvalError> {
        let update_name = self.update_name.clone();
        self.run_named_seed_with_control_edges(world, input, defaults, param_overrides, &update_name, false, edges)
    }

    pub fn run_update_with_param_overrides(
        &mut self,
        world: &NuriWorld,
        input: &InputSnapshot,
        defaults: &HashMap<String, Value>,
        param_overrides: &BTreeMap<String, Value>,
    ) -> Result<DdnRunOutput, String> {
        let update_name = self.update_name.clone();
        self.run_named_seed(
            world,
            input,
            defaults,
            Some(param_overrides),
            &update_name,
            false,
        )
        .map_err(|err| err.to_string())
    }

    pub fn run_finish(
        &mut self,
        world: &NuriWorld,
        input: &InputSnapshot,
        defaults: &HashMap<String, Value>,
    ) -> Result<DdnRunOutput, String> {
        self.run_named_seed(world, input, defaults, None, "끝", true)
            .map_err(|err| err.to_string())
    }

    pub fn run_finish_with_param_overrides(
        &mut self,
        world: &NuriWorld,
        input: &InputSnapshot,
        defaults: &HashMap<String, Value>,
        param_overrides: &BTreeMap<String, Value>,
    ) -> Result<DdnRunOutput, String> {
        self.run_named_seed(world, input, defaults, Some(param_overrides), "끝", true)
            .map_err(|err| err.to_string())
    }

    pub(super) fn run_named_seed(
        &mut self,
        world: &NuriWorld,
        input: &InputSnapshot,
        defaults: &HashMap<String, Value>,
        param_overrides: Option<&BTreeMap<String, Value>>,
        seed_name: &str,
        missing_is_noop: bool,
    ) -> Result<DdnRunOutput, EvalError> {
        self.run_named_seed_with_control_edges(world, input, defaults, param_overrides, seed_name, missing_is_noop, None)
    }

    pub(super) fn run_named_seed_with_control_edges(
        &mut self,
        world: &NuriWorld,
        input: &InputSnapshot,
        defaults: &HashMap<String, Value>,
        param_overrides: Option<&BTreeMap<String, Value>>,
        seed_name: &str,
        missing_is_noop: bool,
        control_edges: Option<(u64, u64)>,
    ) -> Result<DdnRunOutput, EvalError> {
        self.run_named_seed_with_input(world, input, defaults, param_overrides,
            seed_name, missing_is_noop, control_edges, None)
    }

    pub(super) fn run_named_seed_with_input(
        &mut self,
        world: &NuriWorld,
        input: &InputSnapshot,
        defaults: &HashMap<String, Value>,
        param_overrides: Option<&BTreeMap<String, Value>>,
        seed_name: &str,
        missing_is_noop: bool,
        control_edges: Option<(u64, u64)>,
        logical_input: Option<&ddonirang_core::input::LogicalInputQuery<String>>,
    ) -> Result<DdnRunOutput, EvalError> {
        verify_stdlib_registry_totality()?;
        // Validate committed record evidence before creating a candidate frame
        // or advancing input state. A malformed record is never a missing value.
        self.validate_persisted_records(world)?;
        self.work_cost
            .borrow_mut()
            .note_external_events(input.ai_injections.len())?;
        self.work_cost
            .borrow_mut()
            .note_external_events(input.net_events.len())?;
        let prev_keys_pressed = self.prev_keys_pressed;
        let input_state = InputState::new(input.keys_pressed, prev_keys_pressed);
        self.prev_keys_pressed = input.keys_pressed;

        let mut ctx = EvalContext::new_with_work_cost(
            &self.program,
            world,
            defaults,
            param_overrides,
            input_state,
            input.rng_seed,
            input.tick_id,
            self.project_asset_registry.as_deref(),
            Rc::clone(&self.work_cost),
            self.async_queue_capacity,
            self.internal_dispatch.clone(),
        );
        ctx.logical_input = logical_input.cloned();
        if let Some(bootstrap)=&self.subject_bootstrap {
            let staged=match world.subject_world().map(|snapshot|bootstrap.stage_committed(snapshot)).transpose() {
                Ok(stage)=>stage.unwrap_or_else(||bootstrap.clone()),
                Err(error)=>{self.prev_keys_pressed=prev_keys_pressed;return Err(error.into());}
            };
            ctx.subject_stage=Some(Rc::new(RefCell::new(staged)));
        } else if world.subject_world().is_some() {
            self.prev_keys_pressed=prev_keys_pressed;
            return Err("E_VC04_SUBJECT_CONTEXT_REQUIRED: 실제 실행 소유자 없는 대상 상태는 실행하지 않습니다".to_string().into());
        }
        ctx.tuple_type_plan = Rc::clone(&self.tuple_type_plan);
        if logical_input.is_none() {
        ctx.resources.insert(
            "입력키".to_string(),
            Value::String(input.last_key_name.clone()),
        );
        seed_keyboard_state_resources(&mut ctx.resources, prev_keys_pressed, input.keys_pressed);
        }
        if let Some((pressed, released)) = control_edges {
            ctx.control_pressed = Some(pressed);
            for entry in project_keyboard_state(
                runtime_keys_to_tape_mask(input.keys_pressed),
                runtime_keys_to_tape_mask(pressed),
                runtime_keys_to_tape_mask(released),
            ) {
                if let SnapshotStateValue::Number(value) = entry.value {
                    ctx.resources.insert(entry.key, Value::Fixed64(Fixed64::from_i64(value)));
                }
            }
        }
        let mut root_bogae_projection = None;
        if self.named_entry_after_definitions && seed_name == self.update_name {
            // The parser places authored top-level setup in the tick seed.
            // An explicit project start must initialize those declarations,
            // but must not implicitly invoke an unrelated tick function.
            let mut locals = HashMap::new();
            for item in &ctx.program.program.items {
                let ddonirang_lang::TopLevelItem::SeedDef(seed) = item;
                if seed.canonical_name == seed_name { continue; }
                let Some(body) = &seed.body else { continue; };
                let synthetic_setup = seed.span == body.span
                    && body.stmts.iter().all(|stmt| matches!(stmt, Stmt::DeclBlock { .. }));
                for stmt in &body.stmts {
                    let Stmt::DeclBlock { span, .. } = stmt else { continue; };
                    if synthetic_setup || span.start < seed.span.start || span.end > seed.span.end {
                        if let Err(error) = ctx.eval_stmt(&mut locals, stmt) {
                            self.prev_keys_pressed = prev_keys_pressed;
                            return Err(error);
                        }
                    }
                }
            }
        }
        if !ctx.program.root_stmts.is_empty()
            && (!self.named_entry_after_definitions || seed_name == self.update_name) {
            ctx.root_program_active = true;
            let mut locals = HashMap::new();
            // V25 canonicalization moves a top-level `채비` block into the
            // `매마디` seed while leaving lifecycle hooks at the root.  Run
            // that setup seed first so the root hooks see the same persistent
            // initial state as the author's source promised.
            if let Some(setup) = ctx.program.functions.get(seed_name).filter(|_| !self.named_entry_after_definitions) {
                if let Err(error) = ctx.eval_seed(setup, Vec::new()) {
                    self.prev_keys_pressed = prev_keys_pressed;
                    return Err(error);
                }
            }
            ctx.dispatch_logical_input_reactions()?;
            for stmt in &ctx.program.root_stmts {
                let post_settle_observation=crate::bogae_projection::is_observation_definition(stmt)
                    || (!ctx.program.derived_plan.bindings.is_empty() && matches!(stmt,Stmt::PublicObservation{..}));
                if (self.named_entry_after_definitions || !ctx.program.derived_plan.bindings.is_empty())
                    && post_settle_observation {
                    continue;
                }
                let snapshot = ctx.capture_frame_snapshot(&locals);
                if let Err(error) = ctx.eval_stmt(&mut locals, stmt) {
                    ctx.restore_frame_snapshot(&mut locals, &snapshot);
                    self.prev_keys_pressed = prev_keys_pressed;
                    return Err(error);
                }
                if ctx.aborted {
                    ctx.restore_frame_snapshot_preserving_abort_contract_diag(&mut locals, &snapshot);
                    self.prev_keys_pressed = prev_keys_pressed;
                    return Err("E_ROOT_PROGRAM_ABORTED: 실행이 중단되어 결과를 공개하지 않습니다".to_string().into());
                }
            }
            if self.named_entry_after_definitions {
                let entry = ctx.program.functions.get(seed_name).ok_or_else(|| {
                    EvalError::Message(format!("시작 정의를 찾을 수 없습니다: {seed_name}"))
                })?;
                if let Err(error) = ctx.eval_seed(entry, Vec::new()) {
                    self.prev_keys_pressed = prev_keys_pressed;
                    return Err(error);
                }
            }
            if let Err(error)=ctx.settle_derived(&mut locals){self.prev_keys_pressed=prev_keys_pressed;return Err(error);}
            if self.named_entry_after_definitions || !ctx.program.derived_plan.bindings.is_empty() {
                for stmt in &ctx.program.root_stmts {
                    if crate::bogae_projection::is_observation_definition(stmt)
                        || (!ctx.program.derived_plan.bindings.is_empty() && matches!(stmt,Stmt::PublicObservation{..})) {
                        if let Err(error) = ctx.eval_stmt(&mut locals, stmt) {
                            self.prev_keys_pressed = prev_keys_pressed;
                            return Err(error);
                        }
                    }
                }
            }
            root_bogae_projection = crate::bogae_projection::resolve_selected(
                &ctx.program.program, &ctx.program.source_identity, &locals, self.bogae_entry.as_deref());
        } else {
        let update = if let Some(update) = ctx.program.functions.get(seed_name) {
            update
        } else if seed_name == "매마디" {
            ctx.program.functions.get("매틱").ok_or_else(|| {
                EvalError::Message(format!("업데이트 함수 '{}'를 찾을 수 없습니다", seed_name))
            })?
        } else if missing_is_noop {
            return Ok(DdnRunOutput {
                result_emissions: Default::default(),
                display_outputs: Vec::new(),
                bogae_projection: None,
                patch: Patch {
                    ops: Vec::new(),
                    origin: Origin::system("ddn"),
                },
                resources: ctx.resources.into_owned(),
                input_reads: ctx.input_reads.borrow().clone(),
                public_observations: Vec::new(),
                explicit_nfc_provenance: Vec::new(),
                state_transitions: Vec::new(),
                work_cost: self.work_cost.borrow().receipt(),
            });
        } else {
            return Err(format!("업데이트 함수 '{}'를 찾을 수 없습니다", seed_name).into());
        };
        if let Err(error) = ctx.eval_seed(update, Vec::new()) {
            self.prev_keys_pressed = prev_keys_pressed;
            return Err(error);
        }
        }
        if !ctx.root_program_active {
            if let Err(error)=ctx.settle_derived(&mut HashMap::new()){self.prev_keys_pressed=prev_keys_pressed;return Err(error);}
        }
        if logical_input.is_some() && ctx.input_reads.borrow().contains("logical.forbidden_physical") {
            self.prev_keys_pressed = prev_keys_pressed;
            return Err("E_LOGICAL_INPUT_PHYSICAL_UNSUPPORTED: 논리 입력 원문은 물리 입력 자원을 읽지 않습니다".to_string().into());
        }
        ctx.flush_factor_route_metrics_resource();
        ctx.emit_factor_route_summary_diag();
        if let Some(stage)=&ctx.subject_stage {
            let snapshot=match stage.borrow().committed_snapshot() {
                Ok(value)=>value,Err(error)=>{self.prev_keys_pressed=prev_keys_pressed;return Err(error.into());}
            };
            if world.subject_world()!=Some(&snapshot) {ctx.patch_ops.push(PatchOp::SetSubjectWorld{value:snapshot});}
        }
        let patch = Patch {
            ops: ctx.patch_ops.into_owned(),
            origin: Origin::system("ddn"),
        };
        let state_transitions = collect_state_transition_records(&patch.ops);
        let input_reads = ctx.input_reads.borrow().clone();
        let resources = ctx.resources.into_owned();
        let bogae_projection = root_bogae_projection.or_else(|| crate::bogae_projection::resolve_selected(
            &ctx.program.program, &ctx.program.source_identity, &resources, self.bogae_entry.as_deref()));
        let bogae_projection = if let Some(activation) = &self.authored_bogae {
            match activation.apply(bogae_projection) {
                Ok(projection) => projection,
                Err(error) => {
                    self.prev_keys_pressed = prev_keys_pressed;
                    return Err(error.into());
                }
            }
        } else { bogae_projection };
        let authored_projection=bogae_projection.clone();
        let mut bogae_projection=bogae_projection;
        if self.bogae_activation_requested {
            if let Err(error) = crate::bogae_projection::admit_activation(bogae_projection.as_ref()) {
                self.prev_keys_pressed = prev_keys_pressed;
                return Err(error.into());
            }
            if let Some(projection)=bogae_projection.as_ref() {
                match self.bogae_history.preview(input.tick_id,projection) {
                    Ok(resolved)=>bogae_projection=Some(resolved),
                    Err(error)=>{self.prev_keys_pressed=prev_keys_pressed;return Err(error.into());}
                }
            }
        }
        if self.bogae_activation_requested {
            if let Some(projection)=authored_projection.as_ref() {
                let bytes=self.bogae_history.storage_bytes_after(input.tick_id,projection).map_err(EvalError::from)?;
                if let Err(error)=ctx.work_cost.borrow_mut().set_result_artifact_bytes(bytes) {
                    self.prev_keys_pressed=prev_keys_pressed;
                    return Err(error.into());
                }
            }
        }
        ctx.work_cost.borrow_mut().note_committed_madi()?;
        if self.bogae_activation_requested {
            if let Some(projection)=authored_projection {self.bogae_history.commit(input.tick_id,projection);}
        }
        let work_cost = ctx.work_cost.borrow().receipt();
        self.internal_dispatch = ctx.internal_dispatch.clone();
        Ok(DdnRunOutput {
            result_emissions: ctx.result_emissions,
            display_outputs: ctx.display_outputs,
            bogae_projection,
            patch,
            resources,
            input_reads,
            public_observations: ctx.public_observations,
            explicit_nfc_provenance: ctx.explicit_nfc_provenance,
            state_transitions,
            work_cost,
        })
    }

    #[allow(dead_code)]
    pub fn reset_transient_state(&mut self) {
        self.prev_keys_pressed = 0;
        self.bogae_history.clear();
    }

    pub(crate) fn bind_subject_bootstrap(&mut self,stage:crate::unit_wire::subject_execution::SubjectExecutionStage) {
        self.subject_bootstrap=Some(stage);
    }
    pub(crate) fn has_temporary_registration(&self)->bool{self.program.has_temporary_registration()}
    pub(crate) fn saved_initial_subject_world(&self)->Result<Option<ddonirang_core::platform::SubjectWorldStateV1>,String>{
        self.subject_bootstrap.as_ref().map(|stage|stage.committed_snapshot()).transpose()
    }

    pub fn scene_history_snapshot(&self) -> serde_json::Value { self.bogae_history.snapshot() }

    pub(crate) fn bogae_history_for_resume(&self)->&crate::bogae_time::SceneHistory { &self.bogae_history }

    // A3-P11: consumed input history is distinct from a newer host snapshot.
    pub(crate) fn consumed_keys_pressed(&self) -> u64 { self.prev_keys_pressed }

    pub(crate) fn restore_consumed_keys_pressed(&mut self, keys: u64) {
        self.prev_keys_pressed = keys;
    }

    pub fn restore_scene_history(&mut self, history: crate::bogae_time::SceneHistory) {
        self.bogae_history = history;
    }
}

#[cfg(test)]
mod malformed_resource_admission_tests {
    use super::*;

    fn input() -> InputSnapshot {
        InputSnapshot {tick_id:0,dt:Fixed64::from_i64(1),keys_pressed:0,last_key_name:String::new(),
            pointer_x_i32:0,pointer_y_i32:0,ai_injections:Vec::new(),net_events:Vec::new(),
            frame_source:ddonirang_core::InputSource::Person,rng_seed:0}
    }

    #[test]
    fn malformed_tensor_world_is_rejected_before_candidate_without_panic() {
        for value in [ResourceValue::BasicTensorV1(Vec::new()),
            ResourceValue::BasicTensorV1(vec![255]),
            ResourceValue::List(vec![ResourceValue::BasicTensorV1(vec![255])])] {
            let program=DdnProgram::from_source("매틱:움직씨 = { }","tensor-admission.ddn").unwrap();
            let mut runner=DdnRunner::new(program,"매틱");
            let mut world=NuriWorld::new();world.set_resource_value("표본".into(),value);
            let before=world.state_hash();
            let error=runner.run_update(&world,&input(),&HashMap::new()).err().expect("reject malformed carrier");
            assert!(error.to_string().starts_with("E_RESOURCE_SNAPSHOT_INVALID: Tensor 복원 검증 실패:"));
            assert_eq!(world.state_hash(),before);
        }
    }

    #[test]
    fn normal_resource_world_keeps_existing_runner_conversion() {
        for value in [None,Some(ResourceValue::String("정상".into()))] {
            let program=DdnProgram::from_source("매틱:움직씨 = { }","tensor-control.ddn").unwrap();
            let mut runner=DdnRunner::new(program,"매틱");let mut world=NuriWorld::new();
            if let Some(value)=value {world.set_resource_value("표본".into(),value);}
            let before=world.state_hash();runner.run_update(&world,&input(),&HashMap::new()).unwrap();
            assert_eq!(world.state_hash(),before);
        }
    }

    #[test]
    fn subject_payload_in_ordinary_resource_is_rejected_without_world_change() {
        let slots=ddonirang_core::nominal::NominalSlotTableV1::from_slots([1;32],&[],8192,100).unwrap();
        let payload=ResourceValue::SubjectPayloadV1(std::sync::Arc::new(slots));
        for value in [payload.clone(),ResourceValue::List(vec![payload])] {
            let program=DdnProgram::from_source("매틱:움직씨 = { }","subject-resource-admission.ddn").unwrap();
            let mut runner=DdnRunner::new(program,"매틱");let mut world=NuriWorld::new();
            world.set_resource_value("표본".into(),value);let before=world.state_hash();
            let error=runner.run_update(&world,&input(),&HashMap::new()).err().expect("reject ordinary-resource payload");
            assert!(error.to_string().starts_with("E_RESOURCE_SNAPSHOT_INVALID:"));
            assert_eq!(world.state_hash(),before);
        }
    }
}

