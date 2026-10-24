use std::thread::available_parallelism;

use crate::{Fixed64, NuriWorld, ResourceHandle, StateHash};

#[derive(Clone, Debug)]
pub enum ThreadMode {
    Seq,
    Rayon(usize),
    Auto,
}

#[derive(Clone, Debug)]
pub struct Realm {
    pub id: u64,
    pub world: NuriWorld,
    pub rng: u64,
    pub madi: u64,
    pub state_hash: StateHash,
}

#[derive(Clone, Debug)]
pub struct RealmStepInput {
    pub realm_id: usize,
    pub delta: i64,
}

#[derive(Clone, Debug)]
pub struct RealmStepOutput {
    pub realm_id: usize,
    pub madi: u64,
    pub state_hash: StateHash,
}

#[derive(Clone, Debug)]
pub struct MultiRealmManager {
    pub realms: Vec<Realm>,
    pub master_seed: u64,
    pub thread_mode: ThreadMode,
}

impl ThreadMode {
    fn resolve(&self) -> ThreadMode {
        match self {
            ThreadMode::Seq => ThreadMode::Seq,
            ThreadMode::Rayon(n) => {
                if *n <= 1 {
                    ThreadMode::Seq
                } else {
                    ThreadMode::Rayon(*n)
                }
            }
            ThreadMode::Auto => {
                let threads = available_parallelism()
                    .map(|count| count.get())
                    .unwrap_or(1);
                if threads <= 1 {
                    ThreadMode::Seq
                } else {
                    ThreadMode::Rayon(threads)
                }
            }
        }
    }
}

impl RealmStepOutput {
    fn from_realm(realm: &Realm) -> Self {
        Self {
            realm_id: realm.id as usize,
            madi: realm.madi,
            state_hash: realm.state_hash,
        }
    }
}

impl Realm {
    pub fn new(id: u64, seed: u64) -> Self {
        let mut world = NuriWorld::new();
        world.set_resource_fixed64("realm.value".to_string(), Fixed64::ZERO);
        world.set_resource_handle("realm.rng".to_string(), ResourceHandle::from_raw(seed));
        let state_hash = world.state_hash();
        Self {
            id,
            world,
            rng: seed,
            madi: 0,
            state_hash,
        }
    }

    pub fn step_batch(&mut self, inputs: &[RealmStepInput]) -> Result<RealmStepOutput, String> {
        if inputs.is_empty() {
            return Ok(RealmStepOutput::from_realm(self));
        }

        // A batch is an atomic realm transition.  Build it on a clone so a
        // checked Fixed64 failure cannot publish a partial resource/rng/madi
        // update for an earlier item in the same batch.
        let mut candidate = self.clone();
        for input in inputs {
            let current = candidate
                .world
                .get_resource_fixed64("realm.value")
                .unwrap_or(Fixed64::ZERO);
            let updated = current
                .checked_add(Fixed64::from_i64(input.delta))
                .map_err(|_| "E_REALM_ARITH_OVERFLOW realm.value".to_string())?;
            candidate
                .world
                .set_resource_fixed64("realm.value".to_string(), updated);
            candidate.rng = splitmix64(candidate.rng);
            candidate.world.set_resource_handle(
                "realm.rng".to_string(),
                ResourceHandle::from_raw(candidate.rng),
            );
            candidate.madi = candidate.madi.wrapping_add(1);
        }
        candidate.state_hash = candidate.world.state_hash();
        let output = RealmStepOutput::from_realm(&candidate);
        *self = candidate;
        Ok(output)
    }
}

impl MultiRealmManager {
    pub fn new(realm_count: usize, master_seed: u64, thread_mode: ThreadMode) -> Self {
        let mut realms = Vec::with_capacity(realm_count);
        for id in 0..realm_count {
            let seed = mix64(master_seed, id as u64);
            realms.push(Realm::new(id as u64, seed));
        }
        Self {
            realms,
            master_seed,
            thread_mode,
        }
    }

    pub fn realm_count(&self) -> usize {
        self.realms.len()
    }

    pub fn state_hashes(&self) -> Vec<StateHash> {
        self.realms.iter().map(|realm| realm.state_hash).collect()
    }

    pub fn step_batch(
        &mut self,
        inputs: &[RealmStepInput],
    ) -> Result<Vec<RealmStepOutput>, String> {
        let mut normalized: Vec<(usize, RealmStepInput)> =
            inputs.iter().cloned().enumerate().collect();
        normalized.sort_by_key(|(idx, input)| (input.realm_id, *idx));

        let mut buckets: Vec<Vec<RealmStepInput>> = vec![Vec::new(); self.realms.len()];
        for (_, input) in normalized {
            if input.realm_id >= self.realms.len() {
                return Err(format!(
                    "E_REALM_ID_OUT_OF_RANGE realm_id={} realm_count={}",
                    input.realm_id,
                    self.realms.len()
                ));
            }
            buckets[input.realm_id].push(input);
        }

        // All realms participate in one externally visible batch.  Keep the
        // manager unchanged unless every realm can produce its candidate.
        let mut candidate_realms = self.realms.clone();

        let outputs = match self.thread_mode.resolve() {
            ThreadMode::Seq => {
                candidate_realms
                    .iter_mut()
                    .enumerate()
                    .map(|(idx, realm)| realm.step_batch(&buckets[idx]))
                    .collect::<Result<Vec<_>, _>>()?
            }
            ThreadMode::Rayon(threads) => {
                use rayon::prelude::*;
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(threads)
                    .build()
                    .map_err(|err| format!("E_REALM_THREADPOOL {}", err))?;
                pool.install(|| {
                    candidate_realms
                        .par_iter_mut()
                        .enumerate()
                        .map(|(idx, realm)| realm.step_batch(&buckets[idx]))
                        .collect::<Result<Vec<_>, _>>()
                })?
            }
            ThreadMode::Auto => unreachable!("resolved thread mode"),
        };

        self.realms = candidate_realms;
        Ok(outputs)
    }
}

pub fn mix64(master_seed: u64, realm_id: u64) -> u64 {
    let mut x = master_seed ^ realm_id.wrapping_mul(0x9e3779b97f4a7c15);
    x = splitmix64(x);
    x
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::{MultiRealmManager, Realm, RealmStepInput, ThreadMode};
    use crate::Fixed64;

    #[test]
    fn realm_fixed64_overflow_is_atomic() {
        let mut realm = Realm::new(7, 19);
        realm
            .world
            .set_resource_fixed64("realm.value".to_string(), Fixed64::MAX);
        realm.state_hash = realm.world.state_hash();
        let before = realm.clone();

        let error = realm
            .step_batch(&[RealmStepInput {
                realm_id: 7,
                delta: 1,
            }])
            .expect_err("overflow must fail closed");

        assert_eq!(error, "E_REALM_ARITH_OVERFLOW realm.value");
        assert_eq!(realm.world.state_hash(), before.world.state_hash());
        assert_eq!(realm.state_hash, before.state_hash);
        assert_eq!(realm.rng, before.rng);
        assert_eq!(realm.madi, before.madi);
    }

    #[test]
    fn multi_realm_overflow_keeps_every_realm_unchanged() {
        let mut manager = MultiRealmManager::new(2, 19, ThreadMode::Seq);
        manager.realms[1]
            .world
            .set_resource_fixed64("realm.value".to_string(), Fixed64::MAX);
        manager.realms[1].state_hash = manager.realms[1].world.state_hash();
        let before = manager.state_hashes();
        let before_rng = manager.realms.iter().map(|realm| realm.rng).collect::<Vec<_>>();
        let before_madi = manager.realms.iter().map(|realm| realm.madi).collect::<Vec<_>>();

        let error = manager
            .step_batch(&[
                RealmStepInput {
                    realm_id: 0,
                    delta: 1,
                },
                RealmStepInput {
                    realm_id: 1,
                    delta: 1,
                },
            ])
            .expect_err("one realm overflow must abort the whole batch");

        assert_eq!(error, "E_REALM_ARITH_OVERFLOW realm.value");
        assert_eq!(manager.state_hashes(), before);
        assert_eq!(
            manager.realms.iter().map(|realm| realm.rng).collect::<Vec<_>>(),
            before_rng
        );
        assert_eq!(
            manager.realms.iter().map(|realm| realm.madi).collect::<Vec<_>>(),
            before_madi
        );
    }

    #[test]
    fn multi_realm_checked_batch_commits_all_candidates_after_success() {
        let mut manager = MultiRealmManager::new(2, 19, ThreadMode::Seq);
        let outputs = manager
            .step_batch(&[
                RealmStepInput {
                    realm_id: 0,
                    delta: 2,
                },
                RealmStepInput {
                    realm_id: 1,
                    delta: -1,
                },
            ])
            .expect("representable batch must commit");

        assert_eq!(outputs.len(), 2);
        assert_eq!(manager.realms[0].madi, 1);
        assert_eq!(manager.realms[1].madi, 1);
        assert_eq!(
            manager.realms[0].world.get_resource_fixed64("realm.value"),
            Some(Fixed64::from_i64(2))
        );
        assert_eq!(
            manager.realms[1].world.get_resource_fixed64("realm.value"),
            Some(Fixed64::from_i64(-1))
        );
    }
}
