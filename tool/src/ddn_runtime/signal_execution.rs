//! Existing evaluator methods moved without changing their bodies.
use super::{
    BTreeMap,
    Body,
    DdnRunner,
    EndpointRoute,
    EndpointRouteSemantic,
    EvalContext,
    EvalError,
    Expr,
    ExprKind,
    Fixed64,
    HashMap,
    HashSet,
    InternalDeliveryEnvelopeSemantic,
    InternalEffectSinkObservation,
    InternalReactionProgram,
    Literal,
    PatchOp,
    PendingAsyncSignal,
    QueueAdmission,
    SeedDef,
    SeedKind,
    SenderIdentityPredicate,
    Stmt,
    TypeRef,
    Value,
    check_value_type,
    func,
    internal_reaction_failure_to_eval,
    is_truthy,
    normalize_expression_identity,
    receive_stmt_matches_rank,
    restore_local_binding,
    value_canon,
};

impl<'a> EvalContext<'a> {
    pub(super) fn dispatch_signal_send(
        &mut self,
        locals: &mut HashMap<String, Value>,
        sender_expr: Option<&Expr>,
        payload_expr: &Expr,
        receiver_expr: &Expr,
    ) -> Result<(), EvalError> {
        let receiver_name = self.resolve_entity_name(receiver_expr)?;
        let sender_name = sender_expr
            .map(|expr| self.resolve_entity_name(expr))
            .transpose()?
            .or_else(|| self.current_seed_name.clone())
            .unwrap_or_else(|| "누리".to_string());
        let (event_kind, event_payload) = self.build_signal_payload(locals, payload_expr)?;
        self.dispatch_signal_to_receiver(
            locals,
            &receiver_name,
            &event_kind,
            &sender_name,
            event_payload,
        )
    }

    pub(super) fn dispatch_async_send(
        &mut self,
        locals: &mut HashMap<String, Value>,
        sender_expr: Option<&Expr>,
        payload_expr: &Expr,
        receiver_expr: &Expr,
        send_span: ddonirang_lang::Span,
    ) -> Result<(), EvalError> {
        let receiver_name = self.resolve_entity_name(receiver_expr)?;
        let sender_name = sender_expr
            .map(|expr| self.resolve_entity_name(expr))
            .transpose()?
            .or_else(|| self.current_owner_identity.clone())
            .ok_or_else(|| {
                EvalError::async_candidate(
                    "E_ASYNC_SENDER_IDENTITY_UNRESOLVED_CANDIDATE",
                    "async_sender_identity",
                    "async send outside owner context requires an explicit sender identity",
                    send_span,
                )
            })?;
        let (event_kind, event_payload) = self.build_signal_payload(locals, payload_expr)?;
        let envelope = PendingAsyncSignal {
            receiver_name,
            event_kind,
            sender_name,
            event_payload,
            source_span: send_span,
        };
        if self.async_outbox.is_staging() {
            self.async_outbox.stage(envelope);
            return Ok(());
        }
        self.admit_async_envelopes(vec![envelope]);
        self.drain_async_queue(locals)
    }

    pub(super) fn admit_async_envelopes(&mut self, envelopes: Vec<PendingAsyncSignal>) {
        for envelope in envelopes {
            let operation_identity = format!(
                "madi:{}:{}>{}:{}",
                self.tick_id, envelope.sender_name, envelope.receiver_name, envelope.event_kind
            );
            if let QueueAdmission::Waiting(receipt) = self.pending_async_signals.enqueue_or_wait(
                "ddonirang.product.async.v1",
                &operation_identity,
                envelope,
            ) {
                self.async_wait_receipts.push(receipt);
            }
        }
    }

    pub(super) fn drain_async_queue(&mut self, locals: &mut HashMap<String, Value>) -> Result<(), EvalError> {
        if !self.pending_async_signals.begin_drain() {
            return Ok(());
        }
        let mut result = Ok(());
        while let Some(signal) = self.pending_async_signals.pop_front() {
            if let Err(err) = self.dispatch_async_to_endpoint(locals, signal) {
                self.pending_async_signals.clear();
                result = Err(err);
                break;
            }
        }
        self.async_wait_receipts
            .extend(self.pending_async_signals.drain_resumed_waits());
        self.pending_async_signals.finish_drain();
        result
    }

    pub(super) fn dispatch_async_to_endpoint(
        &mut self,
        locals: &mut HashMap<String, Value>,
        signal: PendingAsyncSignal,
    ) -> Result<(), EvalError> {
        let Some(seed) = self.program.functions.get(&signal.receiver_name).cloned() else {
            return Err(EvalError::async_candidate(
                "E_ASYNC_ENDPOINT_RECEIVER_UNKNOWN_CANDIDATE",
                "async_route",
                format!("unknown receiver identity {}", signal.receiver_name),
                signal.source_span,
            ));
        };
        if !matches!(&seed.seed_kind, SeedKind::Named(kind) if kind == "임자") {
            return Err(EvalError::async_candidate(
                "E_ASYNC_ENDPOINT_RECEIVER_TYPE_CANDIDATE",
                "async_route",
                format!("receiver is not 임자: {}", signal.receiver_name),
                signal.source_span,
            ));
        }
        let Some(body) = seed.body.as_ref() else {
            return Err(EvalError::async_candidate(
                "E_ASYNC_ENDPOINT_UNHANDLED_CANDIDATE",
                "async_route",
                "receiver has no body",
                signal.source_span,
            ));
        };
        if body.stmts.iter().any(|stmt|matches!(stmt,Stmt::Definition {reaction:Some(_),..})) {
            return self.dispatch_authored_reaction_providers(locals,&signal,body);
        }
        let previous_seed = self.current_seed_name.clone();
        let previous_owner = self.current_owner_identity.clone();
        let frame_snapshot = self.capture_frame_snapshot(locals);
        let previous_aborted = self.aborted;
        let previous_rng_state = self.rng_state;
        let previous_flow_stack = self.flow_stack.clone();
        let previous_wait_receipts_len = self.async_wait_receipts.len();
        self.current_seed_name = Some(signal.receiver_name.clone());
        self.current_owner_identity = Some(signal.receiver_name.clone());
        self.async_outbox.begin_frame();
        let mut prepared_internal_reaction = None;
        let result = (|| {
            let mut routes = Vec::new();
            let mut route_stmt_indices = BTreeMap::new();
            for (index, stmt) in body.stmts.iter().enumerate() {
                let Stmt::Receive {
                    sender,
                    kind,
                    binding,
                    payload_type,
                    condition,
                    ..
                } = stmt
                else {
                    continue;
                };
                // Historical receive hooks remain migration evidence. Only
                // the Owner-approved W1-C1 typed receive surface is a successor
                // consuming EndpointRoute.
                let Some(payload_type) = payload_type.as_deref() else {
                    continue;
                };
                if kind
                    .as_deref()
                    .is_some_and(|name| name != signal.event_kind)
                {
                    continue;
                }
                if !self.async_payload_type_matches(
                    payload_type,
                    &signal.event_kind,
                    &signal.event_payload,
                ) {
                    continue;
                }
                let sender_predicate = sender
                    .as_ref()
                    .map(|expr| self.resolve_entity_name(expr))
                    .transpose()?
                    .map(SenderIdentityPredicate::exact)
                    .unwrap_or(SenderIdentityPredicate::AnySender);
                if !sender_predicate.matches(&signal.sender_name) {
                    continue;
                }
                if self.eval_receive_condition(
                    locals,
                    binding.as_deref(),
                    condition.as_ref(),
                    true,
                    &signal.event_kind,
                    &signal.sender_name,
                    &signal.event_payload,
                )? {
                    let message_type = payload_type;
                    let payload_predicate = condition
                        .as_ref()
                        .map(normalize_expression_identity)
                        .unwrap_or_else(|| "ANY_PAYLOAD".to_string());
                    let semantic = EndpointRouteSemantic::new(
                        &signal.receiver_name,
                        message_type,
                        sender_predicate,
                        payload_predicate,
                        format!("root::{}", signal.receiver_name),
                    );
                    let route =
                        EndpointRoute::new(semantic, InternalReactionProgram::new(Vec::new()));
                    route_stmt_indices.insert(route.identity.clone(), index);
                    routes.push(route);
                }
            }
            let envelope = InternalDeliveryEnvelopeSemantic::new(
                &signal.receiver_name,
                &signal.event_kind,
                &signal.sender_name,
                value_canon(&signal.event_payload),
            );
            let reached = self
                .internal_dispatch
                .reached_registration_override
                .clone()
                .unwrap_or_else(|| {
                    self.internal_dispatch
                        .provider_registry
                        .matching_registration_identities(&envelope)
                });
            let occurrence_number = self
                .internal_dispatch
                .occurrence_counter
                .checked_add(1)
                .expect("DdnRunner internal delivery occurrence overflow");
            self.internal_dispatch.occurrence_counter = occurrence_number;
            let prepared = self
                .internal_dispatch
                .runtime
                .prepare(
                    &routes,
                    &self.internal_dispatch.provider_registry,
                    reached,
                    &envelope,
                    self.internal_dispatch.execution_identity.clone(),
                    format!("tick:{}:delivery:{}", self.tick_id, occurrence_number),
                )
                .map_err(|failure| {
                    internal_reaction_failure_to_eval(failure, signal.source_span)
                })?;
            // F5-GA[SIGNAL_ROUTE]: routes와 이 표는 같은 Receive 열거에서 함께 만든다. prepare는 주어진 routes의 유일한 원소 정체를 반환하므로 표 조회와 원래 불변 body의 해당 인덱스가 존재한다.
            let route_index = route_stmt_indices
                .get(&prepared.route_identity)
                .copied()
                .expect("prepared DdnRunner route identity must map to one receive statement");
            let Stmt::Receive {
                kind,
                binding,
                payload_type,
                body,
                ..
            } = &body.stmts[route_index]
            else {
                unreachable!("route candidates contain only receive statements")
            };
            self.eval_receive_stmt(
                locals,
                binding.as_deref(),
                None,
                body,
                kind.is_some() || payload_type.is_some(),
                &signal.event_kind,
                &signal.sender_name,
                &signal.event_payload,
            )?;
            prepared_internal_reaction = Some(prepared);
            Ok(())
        })();
        let published = if result.is_ok() {
            self.async_outbox.commit_frame()
        } else {
            self.async_outbox.abort_frame();
            self.restore_frame_snapshot(locals, &frame_snapshot);
            self.aborted = previous_aborted;
            self.rng_state = previous_rng_state;
            self.flow_stack = previous_flow_stack;
            self.async_wait_receipts
                .truncate(previous_wait_receipts_len);
            Vec::new()
        };
        if result.is_ok() {
            // F5-GA[SIGNAL_PREPARED]: 위 클로저의 유일한 Ok 종료 직전에 Some(prepared)를 넣으며 그 뒤 이 take 전에는 Option을 바꾸지 않는다.
            let prepared = prepared_internal_reaction
                .take()
                .expect("successful DdnRunner delivery must have a prepared internal reaction");
            let mut effect_events = Vec::new();
            self.internal_dispatch
                .commit_events
                .push("ddn_route_frame_committed".to_string());
            let route_frame_committed = self
                .internal_dispatch
                .commit_events
                .last()
                .is_some_and(|event| event == "ddn_route_frame_committed");
            let mut effect_sink_observations = Vec::new();
            let success = self.internal_dispatch.runtime.commit_prepared(
                prepared,
                |effect, committed_world, committed_revision| {
                    effect_events.push(format!("effect_released:{}", effect.identity));
                    effect_sink_observations.push(InternalEffectSinkObservation {
                        identity: effect.identity.clone(),
                        committed_world: committed_world.clone(),
                        committed_revision,
                        route_frame_committed,
                    });
                    Ok(())
                },
            );
            self.internal_dispatch.commit_events.extend(effect_events);
            self.internal_dispatch
                .effect_sink_observations
                .borrow_mut()
                .extend(effect_sink_observations);
            self.internal_dispatch.last_success = Some(success);
        }
        self.current_seed_name = previous_seed;
        self.current_owner_identity = previous_owner;
        if result.is_ok() {
            self.admit_async_envelopes(published);
        }
        result
    }

    pub(super) fn dispatch_logical_input_reactions(&mut self)->Result<(),EvalError> {
        // A3-P4: source explicitly names every channel/nominal alert/receiver.
        let mut bindings=BTreeMap::<String,(String,String,ddonirang_lang::Span)>::new();
        for stmt in &self.program.root_stmts {
            let Stmt::Expr {expr,..}=stmt else {continue};
            let ExprKind::Call {func,args}=&expr.kind else {continue};
            if func!="입력사상.알림결속" {continue;}
            let mut slots: [Option<String>;3]=[None,None,None];
            for (index,arg) in args.iter().enumerate() {
                let ExprKind::Literal(Literal::String(name))=&arg.expr.kind else {
                    return Err("E_LOGICAL_REACTION_BINDING: 결속 이름은 최상위의 명시 글 값이어야 합니다".to_string().into());
                };
                let slot = match arg.resolved_pin.as_deref() {
                    Some("채널")=>0, Some("알림씨")=>1, Some("받는이")=>2,
                    None=>index,
                    _=>return Err("E_LOGICAL_REACTION_BINDING: 알림 결속의 받는자리 이름이 잘못되었습니다".to_string().into()),
                };
                // F5-GA[SIGNAL_SLOT]: 단락 평가로 slot<고정3칸일 때만 해당 원소를 쓴다.
                if slot>=slots.len() || slots[slot].replace(name.clone()).is_some() {
                    return Err("E_LOGICAL_REACTION_BINDING: 알림 결속의 받는자리가 중복되었습니다".to_string().into());
                }
            }
            let names=slots.into_iter().collect::<Option<Vec<_>>>().ok_or_else(|| EvalError::from("E_LOGICAL_REACTION_BINDING: 채널·알림씨·받는이를 명시하세요".to_string()))?;
            // F5-GA[SIGNAL_NAMES]: 고정3칸 모두 Some일 때만 collect가 성공하므로 아래 0/1/2번 이름은 존재한다.
            if bindings.insert(names[0].clone(),(names[1].clone(),names[2].clone(),expr.span)).is_some() {
                return Err("E_LOGICAL_REACTION_BINDING_DUPLICATE: 같은 채널의 알림 결속이 중복되었습니다".to_string().into());
            }
            let Some(alert)=self.program.functions.get(&names[1]) else {return Err("E_LOGICAL_REACTION_ALERT: 선언된 알림씨가 없습니다".to_string().into())};
            if !matches!(&alert.seed_kind,SeedKind::Named(kind) if kind=="알림씨") {
                return Err("E_LOGICAL_REACTION_ALERT: 결속 대상은 알림씨여야 합니다".to_string().into());
            }
            // These pins consume the canonical type owner. The source alias
            // `수` is normalized to `셈수` before this admission boundary.
            let required=[("채널","글"),("알림","글"),("순서","셈수"),("마디번호","셈수")];
            if alert.params.len()!=required.len() || required.iter().any(|(name,ty)|
                !alert.params.iter().any(|pin|pin.pin_name==*name&&pin.type_ref==TypeRef::Named((*ty).into())&&!pin.optional&&pin.default_value.is_none())) {
                return Err("E_LOGICAL_REACTION_ALERT: 알림씨는 채널:글·알림:글·순서:셈수·마디번호:셈수 받는자리를 명시해야 합니다".to_string().into());
            }
        }
        if bindings.is_empty() {return Ok(())}
        let input=self.logical_input.clone().ok_or_else(||EvalError::from("E_LOGICAL_INPUT_PROFILE_REQUIRED: 알림 결속에는 논리 입력 프로필이 필요합니다".to_string()))?;
        for channel in bindings.keys() {let _=input.events_for(channel).map_err(|_|EvalError::from("E_LOGICAL_REACTION_CHANNEL: 기록에 결속되지 않은 논리 채널입니다".to_string()))?;}
        self.input_reads.borrow_mut().insert("logical.reactions".into());
        let mut locals=HashMap::new();
        let result = (|| -> Result<(),EvalError> {
        for (index,event) in input.events().iter().enumerate() {
            let Some((alert,receiver,span))=bindings.get(&event.channel) else {continue};
            self.logical_input = Some(input.at_event_prefix(index+1).map_err(|_| EvalError::from("E_LOGICAL_INPUT_STATE: 반응에는 이전 확정 눌림 상태가 필요합니다".to_string()))?);
            let kind=match event.kind {
                ddonirang_core::input::LogicalInputEventKind::Press=>"누름",
                ddonirang_core::input::LogicalInputEventKind::Repeat=>"반복",
                ddonirang_core::input::LogicalInputEventKind::Release=>"뗌",
                _=>return Err("E_LOGICAL_INPUT_TEXT_UNSUPPORTED: 글 입력 반응은 아직 지원하지 않습니다".to_string().into()),
            };
            let ordinal=i64::try_from(event.receive_ordinal).map_err(|_|EvalError::from("E_LOGICAL_INPUT_ORDER: 사건 순서 범위 초과".to_string()))?;
            let madi=i64::try_from(input.madi()).map_err(|_|EvalError::from("E_LOGICAL_INPUT_ORDER: 마디 범위 초과".to_string()))?;
            if ordinal > i32::MAX as i64 || madi > i32::MAX as i64 {
                return Err("E_LOGICAL_INPUT_ORDER: 알림씨 수 받는자리의 정확한 정수 범위를 넘었습니다".to_string().into());
            }
            self.dispatch_async_to_endpoint(&mut locals,PendingAsyncSignal {
                receiver_name:receiver.clone(),event_kind:alert.clone(),sender_name:"입력사상".into(),source_span:*span,
                event_payload:Value::Pack(BTreeMap::from([
                    ("채널".into(),Value::String(event.channel.clone())),("알림".into(),Value::String(kind.into())),
                    ("순서".into(),Value::Fixed64(Fixed64::from_i64(ordinal))),
                    ("마디번호".into(),Value::Fixed64(Fixed64::from_i64(madi))),
                ])),
            })?;
            self.drain_async_queue(&mut locals)?;
        }
        Ok(())
        })();
        self.logical_input = Some(input);
        result
    }

    pub(super) fn dispatch_authored_reaction_providers(
        &mut self,locals:&mut HashMap<String,Value>,signal:&PendingAsyncSignal,body:&Body,
    )->Result<(),EvalError> {
        // RD-293/445: independent candidates against one settled snapshot.
        // This is actual DDN execution, not InternalReactionProgram lowering.
        let mut providers=BTreeMap::new();
        for stmt in &body.stmts {
            if let Stmt::Definition {name,reaction:Some(registration),value,..}=stmt {
                if registration.alert_type!=signal.event_kind {continue;}
                if !self.async_payload_type_matches(&registration.alert_type,&signal.event_kind,&signal.event_payload) {
                    return Err("E_REACTION_ALERT_TYPE: 등록한 알림씨와 전달 값의 씨가 다릅니다".to_string().into());
                }
                let ExprKind::Thunk(provider_body)=&value.kind else {return Err("E_REACTION_BODY: 반응 본문이 없습니다".to_string().into())};
                let identity=format!("{}::{}",signal.receiver_name,name);
                if providers.insert(identity,(Some(registration.payload.pin_name.as_str()),provider_body)).is_some() {
                    return Err("E_REACTION_PROVIDER_DUPLICATE: 같은 반응 정의 정체가 중복되었습니다".to_string().into());
                }
            }
        }
        // Existing consuming routes and non-consuming named providers are
        // separate participants. Route ambiguity never becomes a priority.
        let route_snapshot=self.capture_frame_snapshot(locals);
        let route_owner=self.current_owner_identity.clone();
        let route_seed=self.current_seed_name.clone();
        self.current_owner_identity=Some(signal.receiver_name.clone());
        self.current_seed_name=Some(signal.receiver_name.clone());
        let route_result:Result<Option<(Option<&str>,&Body)>,EvalError>=(|| {
            let mut route=None;
            for stmt in &body.stmts {
                let Stmt::Receive {sender,payload_type:Some(payload_type),binding,condition,body:route_body,..}=stmt else {continue};
                if !self.async_payload_type_matches(payload_type,&signal.event_kind,&signal.event_payload) {continue;}
                if let Some(sender)=sender {
                    if self.resolve_entity_name(sender)?!=signal.sender_name {continue;}
                }
                if !self.eval_receive_condition(locals,binding.as_deref(),condition.as_ref(),true,
                    &signal.event_kind,&signal.sender_name,&signal.event_payload)? {continue;}
                if route.is_some() {return Err("E_ASYNC_ENDPOINT_AMBIGUOUS_CANDIDATE: 한 전달에 맞는 수신 경로가 여러 개입니다".to_string().into());}
                route=Some((binding.as_deref(),route_body));
            }
            Ok(route)
        })();
        self.restore_frame_snapshot(locals,&route_snapshot);
        self.current_owner_identity=route_owner;self.current_seed_name=route_seed;
        if let Some(route)=route_result? {
            providers.insert(format!("route::{}",signal.receiver_name),route);
        }
        if providers.is_empty() {return Err("E_REACTION_UNHANDLED: 이 알림씨에 반응하는 정의가 없습니다".to_string().into());}
        let base=self.capture_frame_snapshot(locals);
        let previous_owner=self.current_owner_identity.clone();
        let previous_seed=self.current_seed_name.clone();
        let initial_rng=self.rng_state;
        let mut merged=BTreeMap::<String,PatchOp>::new();
        let mut resource_writes=BTreeMap::<String,Value>::new();
        let mut output_ops=Vec::new();let mut envelopes=Vec::new();
        let mut observations=Vec::new();let mut displays=Vec::new();
        let mut result_emissions=crate::replay_evidence::StagedResultEmissionsV1::default();
        let mut final_rng=None;
        let result=(|| {
            for (_identity,(binding,provider_body)) in providers {
                self.restore_frame_snapshot(locals,&base);self.rng_state=initial_rng;
                self.current_owner_identity=Some(signal.receiver_name.clone());
                self.current_seed_name=Some(signal.receiver_name.clone());
                locals.insert("받는이".into(),Value::String(signal.receiver_name.clone()));
                let mut fixed=HashSet::from(["받는이".to_string()]);
                if let Some(binding)=binding {fixed.insert(binding.to_string());}
                self.const_scopes.push(fixed);
                self.async_outbox.begin_frame();
                let evaluated=self.eval_receive_stmt(locals,binding,None,provider_body,
                    true,&signal.event_kind,&signal.sender_name,&signal.event_payload);
                if let Err(error)=evaluated {self.async_outbox.abort_frame();return Err(error);}
                if self.aborted || self.guard_rejected {
                    self.async_outbox.abort_frame();return Err("E_REACTION_ABORTED: 반응 실패로 마디를 게시하지 않습니다".to_string().into());
                }
                envelopes.extend(self.async_outbox.commit_frame());
                let written_tags:HashSet<&str>=self.patch_ops.iter().skip(base.patch_ops.len()).filter_map(|op|match op {
                    PatchOp::SetResourceJson{tag,..}|PatchOp::SetResourceFixed64{tag,..}
                    |PatchOp::SetResourceHandle{tag,..}|PatchOp::SetResourceValue{tag,..}=>Some(tag.as_str()),
                    _=>None,
                }).collect();
                for (key,shared) in self.resources.0.iter() {
                    // Lazy reads may cache defaults in resources; only an
                    // emitted write is a candidate, not a newly cached read.
                    if !written_tags.contains(key.as_str()) {continue;}
                    let value=shared.as_ref();
                    if resource_writes.get(key).is_some_and(|old|old!=value) {
                        return Err(format!("E_REACTION_PATCH_CONFLICT: 반응들이 같은 자원에 서로 다른 값을 썼습니다: {key}").into());
                    }
                    resource_writes.insert(key.clone(),value.clone());
                }
                let mut provider_ops=BTreeMap::new();
                for op in self.patch_ops.iter().skip(base.patch_ops.len()) {
                    let key=match op {
                        PatchOp::SetResourceJson{tag,..}|PatchOp::SetResourceFixed64{tag,..}
                        |PatchOp::SetResourceHandle{tag,..}|PatchOp::SetResourceValue{tag,..}=>Some(format!("resource:{tag}")),
                        PatchOp::SetComponentJson{entity,tag,..}|PatchOp::RemoveComponent{entity,tag}=>Some(format!("component:{entity:?}:{tag:?}")),
                        _=>None,
                    };
                    if let Some(key)=key {
                        provider_ops.insert(key,op.clone());
                    } else {output_ops.push(op.clone());}
                }
                for (key,op) in provider_ops {
                    if merged.get(&key).is_some_and(|old|old!=&op) {
                        return Err(format!("E_REACTION_PATCH_CONFLICT: 반응 결과가 충돌합니다: {key}").into());
                    }
                    merged.insert(key,op);
                }
                // F5-GA[SIGNAL_PREFIX]: 각 provider는 base의 출력/관찰/후보 prefix를 clone 복원한 뒤 append한다. 성공 분기만 합치고, 중첩 rollback/상태 전이도 prefix를 가진 로컬 snapshot/child를 되돌려 이 시작 길이보다 짧아지지 않는다.
                observations.extend_from_slice(&self.public_observations[base.public_observations.len()..]);
                displays.extend_from_slice(&self.display_outputs[base.display_outputs.len()..]);
                result_emissions.extend_since(&self.result_emissions,base.result_emissions.len());
                if self.rng_state!=initial_rng {
                    if final_rng.is_some_and(|old|old!=self.rng_state) {return Err("E_REACTION_PATCH_CONFLICT: 난수 상태가 충돌합니다".to_string().into());}
                    final_rng=Some(self.rng_state);
                }
            }
            Ok(())
        })();
        self.restore_frame_snapshot(locals,&base);
        self.current_owner_identity=previous_owner;self.current_seed_name=previous_seed;
        self.rng_state=initial_rng;
        result?;
        for (key,value) in resource_writes {self.resources.insert(key,value);}
        for op in merged.into_values().chain(output_ops) {self.patch_ops.push(op);}
        self.public_observations.extend(observations);self.display_outputs.extend(displays);
        self.result_emissions.extend_since(&result_emissions,0);
        self.rng_state=final_rng.unwrap_or(initial_rng);
        self.admit_async_envelopes(envelopes);
        Ok(())
    }

    pub(super) fn eval_receive_condition(
        &mut self,
        locals: &mut HashMap<String, Value>,
        binding: Option<&str>,
        condition: Option<&Expr>,
        bind_payload_direct: bool,
        event_kind: &str,
        sender_name: &str,
        event_payload: &Value,
    ) -> Result<bool, EvalError> {
        let bound_value = if bind_payload_direct {
            event_payload.clone()
        } else {
            let mut fields = BTreeMap::new();
            fields.insert("이름".to_string(), Value::String(event_kind.to_string()));
            fields.insert("보낸이".to_string(), Value::String(sender_name.to_string()));
            fields.insert("정보".to_string(), event_payload.clone());
            Value::Pack(fields)
        };
        let restore = binding.map(|name| {
            (
                name.to_string(),
                locals.insert(name.to_string(), bound_value),
            )
        });
        let result = condition
            .map(|expr| {
                self.eval_expr(locals, expr)
                    .and_then(|value| is_truthy(&value))
            })
            .transpose()
            .map(|value| value.unwrap_or(true));
        if let Some((name, previous)) = restore {
            restore_local_binding(locals, &name, previous);
        }
        result
    }

    pub(super) fn async_payload_type_matches(
        &self,
        payload_type: &str,
        event_kind: &str,
        event_payload: &Value,
    ) -> bool {
        if matches!(
            self.program.functions.get(payload_type),
            Some(SeedDef {
                seed_kind: SeedKind::Named(kind),
                ..
            }) if kind == "알림씨"
        ) {
            return payload_type == event_kind;
        }
        check_value_type(event_payload, &TypeRef::Named(payload_type.to_string())).is_ok()
    }

    pub(super) fn resolve_entity_name(&self, expr: &Expr) -> Result<String, EvalError> {
        match &expr.kind {
            ExprKind::Var(name) if name == "제" => {
                self.current_seed_name.clone().ok_or_else(|| {
                    "E_RUNTIME_JE_OUTSIDE_IMJA: `제`는 임자 안에서만 사용할 수 있습니다"
                        .to_string()
                        .into()
                })
            }
            ExprKind::Var(name) => Ok(name.clone()),
            ExprKind::FieldAccess { target, field, .. } => {
                let base = self.resolve_entity_name(target)?;
                Ok(format!("{base}.{field}"))
            }
            _ => Err("알림 대상은 임자 이름이어야 합니다".to_string().into()),
        }
    }

    pub(super) fn build_signal_payload(
        &mut self,
        locals: &mut HashMap<String, Value>,
        payload_expr: &Expr,
    ) -> Result<(String, Value), EvalError> {
        let ExprKind::Call { func, args } = &payload_expr.kind else {
            let value = self.eval_expr(locals, payload_expr)?;
            return Ok(("알림".to_string(), value));
        };
        let Some(seed) = self.program.functions.get(func) else {
            let value = self.eval_expr(locals, payload_expr)?;
            return Ok((func.clone(), value));
        };
        if !matches!(&seed.seed_kind, SeedKind::Named(kind) if kind == "알림씨") {
            let value = self.eval_expr(locals, payload_expr)?;
            return Ok((func.clone(), value));
        }
        let params = seed.params.clone();
        let mut fields = BTreeMap::new();
        for (index, arg) in args.iter().enumerate() {
            let name = arg
                .resolved_pin
                .clone()
                .or_else(|| params.get(index).map(|param| param.pin_name.clone()))
                .unwrap_or_else(|| format!("값{index}"));
            let value = self.eval_expr(locals, &arg.expr)?;
            let param = params.iter().find(|param| param.pin_name == name)
                .ok_or_else(|| EvalError::from(format!("알림씨 받는자리를 찾을 수 없습니다: {name}")))?;
            self.check_param_type(param, &value)?;
            fields.insert(name, value);
        }
        Ok((func.clone(), Value::Pack(fields)))
    }

    pub(super) fn dispatch_signal_to_receiver(
        &mut self,
        locals: &mut HashMap<String, Value>,
        receiver_name: &str,
        event_kind: &str,
        sender_name: &str,
        event_payload: Value,
    ) -> Result<(), EvalError> {
        let Some(seed) = self.program.functions.get(receiver_name).cloned() else {
            return Err(format!("알림 대상 임자를 찾을 수 없습니다: {receiver_name}").into());
        };
        if !matches!(&seed.seed_kind, SeedKind::Named(kind) if kind == "임자") {
            return Err(format!("알림 대상은 임자여야 합니다: {receiver_name}").into());
        }
        let Some(body) = seed.body.as_ref() else {
            return Ok(());
        };
        let prev_seed = self.current_seed_name.clone();
        self.current_seed_name = Some(receiver_name.to_string());
        let result = (|| {
            for rank in 0..4 {
                for stmt in &body.stmts {
                    let Stmt::Receive {
                        kind,
                        binding,
                        condition,
                        body,
                        ..
                    } = stmt
                    else {
                        continue;
                    };
                    if !receive_stmt_matches_rank(
                        rank,
                        kind.as_deref(),
                        binding.as_ref(),
                        condition.as_ref(),
                        event_kind,
                    ) {
                        continue;
                    }
                    self.eval_receive_stmt(
                        locals,
                        binding.as_deref(),
                        condition.as_ref(),
                        body,
                        kind.is_some(),
                        event_kind,
                        sender_name,
                        &event_payload,
                    )?;
                }
            }
            Ok(())
        })();
        self.current_seed_name = prev_seed;
        result
    }

    pub(super) fn eval_receive_stmt(
        &mut self,
        locals: &mut HashMap<String, Value>,
        binding: Option<&str>,
        condition: Option<&Expr>,
        body: &Body,
        bind_payload_direct: bool,
        event_kind: &str,
        sender_name: &str,
        event_payload: &Value,
    ) -> Result<(), EvalError> {
        let bound_value = if bind_payload_direct {
            event_payload.clone()
        } else {
            let mut fields = BTreeMap::new();
            fields.insert("이름".to_string(), Value::String(event_kind.to_string()));
            fields.insert("보낸이".to_string(), Value::String(sender_name.to_string()));
            fields.insert("정보".to_string(), event_payload.clone());
            Value::Pack(fields)
        };
        let restore = if let Some(name) = binding {
            Some((
                name.to_string(),
                locals.insert(name.to_string(), bound_value),
            ))
        } else {
            None
        };
        let should_run = condition
            .map(|expr| {
                self.eval_expr(locals, expr)
                    .and_then(|value| is_truthy(&value))
            })
            .transpose()?
            .unwrap_or(true);
        if should_run {
            let _ = self.eval_body(locals, body)?;
        }
        if let Some((name, previous)) = restore {
            if let Some(value) = previous {
                locals.insert(name, value);
            } else {
                locals.remove(&name);
            }
        }
        Ok(())
    }

}
