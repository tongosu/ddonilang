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

    pub fn step_batch(&mut self, inputs: &[RealmStepInput]) -> RealmStepOutput {
        if inputs.is_empty() {
            return RealmStepOutput::from_realm(self);
        }

        for input in inputs {
            let current = self
                .world
                .get_resource_fixed64("realm.value")
                .unwrap_or(Fixed64::ZERO);
            let updated = current.saturating_add(Fixed64::from_i64(input.delta));
            self.world
                .set_resource_fixed64("realm.value".to_string(), updated);
            self.rng = splitmix64(self.rng);
            self.world
                .set_resource_handle("realm.rng".to_string(), ResourceHandle::from_raw(self.rng));
            self.madi = self.madi.wrapping_add(1);
        }
        self.state_hash = self.world.state_hash();
        RealmStepOutput::from_realm(self)
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

    pub fn step_batch(&mut self, inputs: &[RealmStepInput]) -> Result<Vec<RealmStepOutput>, String> {
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

        let mut outputs: Vec<RealmStepOutput> = self
            .realms
            .iter()
            .map(RealmStepOutput::from_realm)
            .collect();

        match self.thread_mode.resolve() {
            ThreadMode::Seq => {
                for (idx, realm) in self.realms.iter_mut().enumerate() {
                    outputs[idx] = realm.step_batch(&buckets[idx]);
                }
            }
            ThreadMode::Rayon(threads) => {
                use rayon::prelude::*;
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(threads)
                    .build()
                    .map_err(|err| format!("E_REALM_THREADPOOL {}", err))?;
                pool.install(|| {
                    self.realms
                        .par_iter_mut()
                        .zip(outputs.par_iter_mut())
                        .enumerate()
                        .for_each(|(idx, (realm, out))| {
                            *out = realm.step_batch(&buckets[idx]);
                        });
                });
            }
            ThreadMode::Auto => unreachable!("resolved thread mode"),
        }

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
