use std::fs;
use std::path::Path;

use ddonirang_core::seulgi::observation::{observation_detjson, observation_from_detjson};

use crate::artifact_output::write_text_artifact_atomic;

pub fn run_canon(input: &Path, out: Option<&Path>) -> Result<(), String> {
    let raw = fs::read_to_string(input).map_err(|error| error.to_string())?;
    let observation = observation_from_detjson(&raw)?;
    let detjson = observation_detjson(&observation);
    if let Some(path) = out {
        require_file_target(path)?;
        write_text_artifact_atomic(path, &format!("{detjson}\n"))?;
    } else {
        println!("{detjson}");
    }
    Ok(())
}

fn require_file_target(path: &Path) -> Result<(), String> {
    if path.exists() && !path.is_file() {
        return Err(format!(
            "E_OBSERVATION_OUTPUT_TARGET_NOT_FILE {}",
            path.display()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::run_canon;
    use ddonirang_core::seulgi::observation::{
        observation_detjson, AgentState, Observation, WorldState,
    };
    use ddonirang_core::Fixed64;
    use std::fs;
    use std::path::PathBuf;
    use std::process;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NONCE: AtomicU64 = AtomicU64::new(0);

    fn test_dir(label: &str) -> PathBuf {
        let nonce = NONCE.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "ddn_observation_workflow_{label}_{}_{nonce}",
            process::id()
        ))
    }

    fn observation() -> Observation {
        Observation {
            agent_id: 7,
            madi: 11,
            timestamp_ms: 13,
            self_state: AgentState {
                position: (Fixed64::from_i64(1), Fixed64::from_i64(2)),
                velocity: (Fixed64::from_i64(0), Fixed64::from_i64(0)),
                health: 100,
                status: "idle".to_string(),
            },
            visible_objects: Vec::new(),
            visible_agents: Vec::new(),
            world_state: WorldState {
                gravity: Fixed64::from_i64(-9),
                time_of_day: 12,
                weather: "clear".to_string(),
            },
        }
    }

    #[test]
    fn canon_file_workflow_is_atomic_and_rejects_directory_target() {
        let dir = test_dir("canon");
        fs::create_dir_all(&dir).expect("create test dir");
        let input = dir.join("input.detjson");
        let output = dir.join("output.detjson");
        let blocked = dir.join("blocked.detjson");
        let expected = observation_detjson(&observation());
        fs::write(&input, format!("{expected}\n")).expect("write input");
        fs::create_dir_all(&blocked).expect("create blocked target");

        run_canon(&input, Some(&output)).expect("canonicalize observation");
        assert_eq!(
            fs::read_to_string(&output).expect("read output"),
            format!("{expected}\n")
        );
        let error =
            run_canon(&input, Some(&blocked)).expect_err("directory target must fail closed");
        assert!(error.contains("E_OBSERVATION_OUTPUT_TARGET_NOT_FILE"));
        assert_eq!(
            fs::read_dir(&blocked).expect("read blocked target").count(),
            0
        );

        fs::remove_dir_all(&dir).expect("remove test dir");
    }
}
