// 또니랑 결정성 증거 (Deterministic Proof) v0.1
// SSOT TOOLCHAIN v16.0.4 + PLATFORM §R 준수
//
// 핵심 기능:
// 1. state_hash: 누리 상태의 BLAKE3 해시
// 2. trace_hash: 실행 트레이스의 해시
// 3. replay_ok: 리플레이 무결성 검증

use blake3::Hasher;
use std::collections::BTreeMap;
use ddonirang_core::NuriWorld;

// ============================================================================
// 상태 해시 (state_hash)
// ============================================================================

/// 누리 상태의 결정적 해시
/// 
/// MUST 규칙:
/// - 동일한 입력 스냅샷 → 동일한 state_hash
/// - 플랫폼/OS/CPU 무관하게 비트 단위 동일
/// - BLAKE3 알고리즘 사용
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateHash([u8; 32]);

impl StateHash {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
    
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }
}

/// 누리 상태 스냅샷 (해시 계산용)
#[derive(Debug, Clone)]
pub struct NuriSnapshot {
    /// 틱 번호 (단조 증가)
    pub tick: u64,
    
    /// 개체(Entity) 상태들
    /// BTreeMap을 사용하여 순서 보장 (결정성)
    pub entities: BTreeMap<EntityId, EntityState>,
    
    /// 터살림씨(Resource) 상태들
    pub resources: BTreeMap<String, ResourceState>,
    
    /// 대기 중인 이벤트들
    pub pending_events: Vec<Event>,
}

pub type EntityId = u64;

/// 개체 상태
#[derive(Debug, Clone)]
pub struct EntityState {
    /// 개체 ID
    pub id: EntityId,
    
    /// 컴포넌트 데이터
    /// BTreeMap으로 순서 보장
    pub components: BTreeMap<String, ComponentData>,
}

/// 컴포넌트 데이터 (결정적 직렬화 필요)
#[derive(Debug, Clone)]
pub enum ComponentData {
    /// 정수
    Int(i64),
    /// 고정64 (Q32.32)
    Fixed64(i64), // raw value
    /// 문자열
    String(String),
    /// 불린
    Bool(bool),
    /// 원자
    Atom(String),
    /// 튜플
    Tuple(Vec<ComponentData>),
    /// 목록
    List(Vec<ComponentData>),
}

/// 터살림씨 상태
#[derive(Debug, Clone)]
pub struct ResourceState {
    pub name: String,
    pub data: ComponentData,
}

/// 이벤트
#[derive(Debug, Clone)]
pub struct Event {
    pub tick: u64,
    pub kind: EventKind,
}

#[derive(Debug, Clone)]
pub enum EventKind {
    Signal(String),
    Input(InputData),
}

#[derive(Debug, Clone)]
pub enum InputData {
    KeyPress(String),
    MouseMove { x: i64, y: i64 }, // Fixed64 raw values
}

impl NuriSnapshot {
    /// 상태 해시 계산
    /// 
    /// 결정성 보장:
    /// - BTreeMap 사용 (키 정렬 순서 보장)
    /// - 모든 수치는 Fixed64 (i64 raw value)로 정규화
    /// - 문자열은 UTF-8 바이트로 직렬화
    pub fn compute_state_hash(&self) -> StateHash {
        let mut hasher = Hasher::new();
        
        // 1. 틱 번호
        hasher.update(&self.tick.to_le_bytes());
        
        // 2. 개체 상태 (정렬된 순서로)
        for (entity_id, entity) in &self.entities {
            hasher.update(&entity_id.to_le_bytes());
            
            for (comp_name, comp_data) in &entity.components {
                hasher.update(comp_name.as_bytes());
                Self::hash_component_data(&mut hasher, comp_data);
            }
        }
        
        // 3. 터살림씨 상태 (정렬된 순서로)
        for (res_name, res_state) in &self.resources {
            hasher.update(res_name.as_bytes());
            Self::hash_component_data(&mut hasher, &res_state.data);
        }
        
        // 4. 대기 이벤트 (순서 유지)
        hasher.update(&self.pending_events.len().to_le_bytes());
        for event in &self.pending_events {
            hasher.update(&event.tick.to_le_bytes());
            // EventKind 해싱 (생략 - 실제 구현 필요)
        }
        
        let hash = hasher.finalize();
        StateHash::from_bytes(*hash.as_bytes())
    }
    
    fn hash_component_data(hasher: &mut Hasher, data: &ComponentData) {
        match data {
            ComponentData::Int(v) => {
                hasher.update(b"i");
                hasher.update(&v.to_le_bytes());
            }
            ComponentData::Fixed64(v) => {
                hasher.update(b"f");
                hasher.update(&v.to_le_bytes());
            }
            ComponentData::String(s) => {
                hasher.update(b"s");
                hasher.update(&s.len().to_le_bytes());
                hasher.update(s.as_bytes());
            }
            ComponentData::Bool(b) => {
                hasher.update(b"b");
                hasher.update(&[if *b { 1u8 } else { 0u8 }]);
            }
            ComponentData::Atom(a) => {
                hasher.update(b"a");
                hasher.update(&a.len().to_le_bytes());
                hasher.update(a.as_bytes());
            }
            ComponentData::Tuple(items) | ComponentData::List(items) => {
                hasher.update(if matches!(data, ComponentData::Tuple(_)) { b"t" } else { b"l" });
                hasher.update(&items.len().to_le_bytes());
                for item in items {
                    Self::hash_component_data(hasher, item);
                }
            }
        }
    }
}

// ============================================================================
// core NuriWorld 연동 (관문 0: 최소 연결)
// ============================================================================

/// core::NuriWorld의 state_hash를 그대로 사용 (결정적 해시 경로 연결)
pub fn state_hash_world(world: &NuriWorld) -> StateHash {
    let hash = world.state_hash();
    StateHash::from_bytes(*hash.as_bytes())
}

// ============================================================================
// 트레이스 해시 (trace_hash)
// ============================================================================

/// 실행 트레이스의 해시
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceHash([u8; 32]);

impl TraceHash {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }
}

/// 실행 트레이스 - 거울(Geoul)의 기록
#[derive(Debug, Clone)]
pub struct ExecutionTrace {
    /// 시작 틱
    pub start_tick: u64,
    /// 종료 틱
    pub end_tick: u64,
    
    /// 틱별 상태 해시
    pub tick_hashes: Vec<(u64, StateHash)>,
    
    /// 실행된 이야기(System) 목록
    pub executed_systems: Vec<String>,
    
    /// 입력 스냅샷 (재현용)
    pub input_snapshots: Vec<InputSnapshot>,
}

/// 입력 스냅샷 - 샘(Sam)의 동결된 입력
#[derive(Debug, Clone)]
pub struct InputSnapshot {
    pub tick: u64,
    pub inputs: Vec<InputData>,
    /// 랜덤 시드 (RNG 동결)
    pub rng_seed: u64,
}

impl ExecutionTrace {
    /// 트레이스 해시 계산
    pub fn compute_trace_hash(&self) -> TraceHash {
        let mut hasher = Hasher::new();
        
        // 1. 범위
        hasher.update(&self.start_tick.to_le_bytes());
        hasher.update(&self.end_tick.to_le_bytes());
        
        // 2. 틱 해시 시퀀스
        for (tick, state_hash) in &self.tick_hashes {
            hasher.update(&tick.to_le_bytes());
            hasher.update(state_hash.as_bytes());
        }
        
        // 3. 시스템 실행 순서
        for sys_name in &self.executed_systems {
            hasher.update(sys_name.as_bytes());
        }
        
        // 4. 입력 스냅샷 시퀀스
        for snapshot in &self.input_snapshots {
            hasher.update(&snapshot.tick.to_le_bytes());
            hasher.update(&snapshot.rng_seed.to_le_bytes());
            // InputData 해싱 (생략)
        }
        
        let hash = hasher.finalize();
        TraceHash::from_bytes(*hash.as_bytes())
    }
}

// ============================================================================
// 리플레이 검증 (replay_ok)
// ============================================================================

/// 리플레이 결과
#[derive(Debug, Clone)]
pub struct ReplayResult {
    pub success: bool,
    pub original_trace: ExecutionTrace,
    pub replay_trace: ExecutionTrace,
    pub differences: Vec<ReplayDifference>,
}

/// 리플레이 차이점
#[derive(Debug, Clone)]
pub enum ReplayDifference {
    /// 틱의 상태 해시 불일치
    StateHashMismatch {
        tick: u64,
        expected: StateHash,
        actual: StateHash,
    },
    
    /// 트레이스 해시 불일치
    TraceHashMismatch {
        expected: TraceHash,
        actual: TraceHash,
    },
    
    /// 틱 범위 불일치
    TickRangeMismatch {
        expected_end: u64,
        actual_end: u64,
    },
}

/// 리플레이 검증기
pub struct ReplayVerifier;

impl ReplayVerifier {
    /// 리플레이 실행 및 검증
    /// 
    /// MUST 규칙:
    /// - 동일한 InputSnapshot 시퀀스 사용
    /// - 모든 틱의 state_hash가 일치해야 함
    /// - trace_hash가 일치해야 함
    pub fn verify_replay(
        original_trace: &ExecutionTrace,
        replay_trace: &ExecutionTrace,
    ) -> ReplayResult {
        let mut differences = Vec::new();
        
        // 1. 틱 범위 검증
        if original_trace.end_tick != replay_trace.end_tick {
            differences.push(ReplayDifference::TickRangeMismatch {
                expected_end: original_trace.end_tick,
                actual_end: replay_trace.end_tick,
            });
        }
        
        // 2. 틱별 상태 해시 검증
        for ((orig_tick, orig_hash), (replay_tick, replay_hash)) in 
            original_trace.tick_hashes.iter().zip(&replay_trace.tick_hashes) 
        {
            if orig_tick != replay_tick || orig_hash != replay_hash {
                differences.push(ReplayDifference::StateHashMismatch {
                    tick: *orig_tick,
                    expected: *orig_hash,
                    actual: *replay_hash,
                });
            }
        }
        
        // 3. 트레이스 해시 검증
        let orig_trace_hash = original_trace.compute_trace_hash();
        let replay_trace_hash = replay_trace.compute_trace_hash();
        
        if orig_trace_hash != replay_trace_hash {
            differences.push(ReplayDifference::TraceHashMismatch {
                expected: orig_trace_hash,
                actual: replay_trace_hash,
            });
        }
        
        ReplayResult {
            success: differences.is_empty(),
            original_trace: original_trace.clone(),
            replay_trace: replay_trace.clone(),
            differences,
        }
    }
}

// ============================================================================
// Det-Cert (결정성 인증) - §T6.2
// ============================================================================

/// 결정성 인증 보고서
#[derive(Debug, Clone)]
pub struct DetCert {
    pub program_id: String,
    pub test_date: String,
    
    /// 테스트 환경들
    pub environments: Vec<TestEnvironment>,
    
    /// 교차 플랫폼 해시 일치 여부
    pub cross_platform_match: bool,
    
    /// 모든 state_hash 기록
    pub state_hashes: Vec<StateHash>,
}

#[derive(Debug, Clone)]
pub struct TestEnvironment {
    pub os: String,
    pub arch: String,
    pub ddonirang_version: String,
}

impl DetCert {
    /// 교차 플랫폼 결정성 검증
    /// 
    /// MUST 규칙:
    /// - Windows/macOS/Linux에서 동일 입력 → 동일 state_hash
    /// - x86_64/ARM64에서 동일 state_hash
    pub fn verify_cross_platform(test_results: Vec<(TestEnvironment, StateHash)>) -> bool {
        if test_results.is_empty() {
            return false;
        }
        
        let first_hash = test_results[0].1;
        test_results.iter().all(|(_, hash)| *hash == first_hash)
    }
}

// ============================================================================
// ddonirang-tool CLI 인터페이스
// ============================================================================

/// ddonirang-tool의 증거 관련 명령
pub enum ProofCommand {
    /// 상태 해시 출력
    StateHash { _input_file: String },
    
    /// 리플레이 검증
    Replay {
        trace_file: String,
        program_file: String,
    },
    
    /// Det-Cert 생성
    Certify {
        program_file: String,
        output: String,
    },
}

pub struct ProofTool;

impl ProofTool {
    pub fn execute(cmd: ProofCommand) -> Result<String, String> {
        match cmd {
            ProofCommand::StateHash { _input_file } => {
                // TODO: 파일 로드 후 해시 계산
                Ok("state_hash: 0x...".to_string())
            }
            ProofCommand::Replay { .. } => { 
                // 현재 구현되지 않은 기능이므로 아무것도 하지 않음
                Ok("Replay 기능은 현재 준비 중입니다.".to_string())            
            }
            
            ProofCommand::Certify { .. } => {
                // 현재 구현되지 않은 기능이므로 아무것도 하지 않음
                Ok("Certify 기능은 현재 준비 중입니다.".to_string())            
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_state_hash_determinism() {
        let snapshot1 = create_test_snapshot();
        let snapshot2 = create_test_snapshot();
        
        let hash1 = snapshot1.compute_state_hash();
        let hash2 = snapshot2.compute_state_hash();
        
        assert_eq!(hash1, hash2, "동일한 스냅샷은 동일한 해시를 생성해야 함");
    }
    
    #[test]
    fn test_state_hash_sensitivity() {
        let mut snapshot1 = create_test_snapshot();
        let snapshot2 = create_test_snapshot();
        
        // 틱 번호 변경
        snapshot1.tick += 1;
        
        let hash1 = snapshot1.compute_state_hash();
        let hash2 = snapshot2.compute_state_hash();
        
        assert_ne!(hash1, hash2, "다른 틱은 다른 해시를 생성해야 함");
    }
    
    #[test]
    fn test_replay_verification() {
        let original = create_test_trace();
        let replay = create_test_trace();
        
        let result = ReplayVerifier::verify_replay(&original, &replay);
        
        assert!(result.success, "동일한 트레이스는 검증에 성공해야 함");
        assert_eq!(result.differences.len(), 0);
    }
    
    fn create_test_snapshot() -> NuriSnapshot {
        let mut entities = BTreeMap::new();
        entities.insert(1, EntityState {
            id: 1,
            components: BTreeMap::from([
                ("위치".to_string(), ComponentData::Tuple(vec![
                    ComponentData::Fixed64(100 << 32), // x = 100.0
                    ComponentData::Fixed64(200 << 32), // y = 200.0
                ])),
            ]),
        });
        
        NuriSnapshot {
            tick: 42,
            entities,
            resources: BTreeMap::new(),
            pending_events: vec![],
        }
    }
    
    fn create_test_trace() -> ExecutionTrace {
        ExecutionTrace {
            start_tick: 0,
            end_tick: 100,
            tick_hashes: vec![
                (0, StateHash::from_bytes([0u8; 32])),
                (100, StateHash::from_bytes([1u8; 32])),
            ],
            executed_systems: vec!["물리".to_string()],
            input_snapshots: vec![],
        }
    }

    #[test]
    fn test_state_hash_world_matches_core() {
        let world = NuriWorld::new();
        let hash = state_hash_world(&world);
        let core_hash = world.state_hash();
        assert_eq!(hash.as_bytes(), core_hash.as_bytes());
    }
}
