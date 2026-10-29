use crate::atomic_commit::LegacySinglePatchEnvelope;
use crate::platform::{Bogae, Geoul, Iyagi, Nuri, Sam, TickFrame};
use crate::signals::SignalSink;
use crate::signals::TickId;

pub struct EngineLoop<S, I, N, G, B>
where
    S: Sam,
    I: Iyagi,
    N: Nuri,
    G: Geoul,
    B: Bogae,
{
    pub sam: S,
    pub iyagi: I,
    pub nuri: N,
    pub geoul: G,
    pub bogae: B,
}

impl<S, I, N, G, B> EngineLoop<S, I, N, G, B>
where
    S: Sam,
    I: Iyagi,
    N: Nuri,
    G: Geoul,
    B: Bogae,
{
    pub fn new(sam: S, iyagi: I, nuri: N, geoul: G, bogae: B) -> Self {
        Self {
            sam,
            iyagi,
            nuri,
            geoul,
            bogae,
        }
    }

    /// ✅ 관문0: 한 틱 최소 루프
    /// Sam -> Iyagi -> Nuri -> Geoul -> Bogae
    pub fn tick_once(&mut self, tick_id: TickId, sink: &mut dyn SignalSink) -> TickFrame {
        // 1) Sam: 입력 동결
        let snapshot = self.sam.begin_tick(tick_id);

        // 2) Iyagi: Patch 생성 (world read-only)
        let patch = self.iyagi.run_update(self.nuri.world(), &snapshot);

        // 3) Nuri: 기존 단일 Patch producer를 명시적 compat envelope로 원자 확정
        let commit_envelope = LegacySinglePatchEnvelope::new(snapshot.tick_id, patch.clone());
        let commit_input = commit_envelope.input_record();
        let commit_receipt = commit_envelope.commit(self.nuri.world_mut(), sink);

        // 4) World hash 계산 (적용 후)
        let state_hash = self.nuri.world().state_hash();
        debug_assert_eq!(state_hash, commit_receipt.committed_state_hash);

        // 5) Geoul: 기록
        let frame = TickFrame {
            snapshot: snapshot.clone(),
            patch: patch.clone(),
            commit_input,
            commit_receipt,
            state_hash,
        };
        self.geoul.record(&frame);

        // 6) Bogae: 렌더(관찰)
        self.bogae.render(self.nuri.world(), snapshot.tick_id);

        frame
    }
}
