#[derive(Debug)]
pub struct ReplSession {
    source_profile_identity: String,
    context_json: String,
    state_hash: String,
}

impl ReplSession {
    pub fn new(source_profile_identity: &str) -> Result<Self, String> {
        let initial = crate::currentline_workflow::execute_currentline_with_supported_profile(
            "",
            "<repl>",
            None,
            source_profile_identity,
        )
        .map_err(|error| error.message)?;
        Ok(Self {
            source_profile_identity: source_profile_identity.to_string(),
            context_json: initial.context_json,
            state_hash: summary_state_hash(&initial.summary)?,
        })
    }

    pub fn execute_line(&mut self, line: &str) -> Result<String, String> {
        let result = crate::currentline_workflow::execute_currentline_with_supported_profile(
            line,
            "<repl>",
            Some(&self.context_json),
            &self.source_profile_identity,
        )
        .map_err(|error| error.message)?;
        let next_state_hash = summary_state_hash(&result.summary)?;
        self.context_json = result.context_json;
        self.state_hash = next_state_hash;
        Ok(result.stdout)
    }

    pub fn reset(&mut self) -> Result<(), String> {
        let replacement = Self::new(&self.source_profile_identity)?;
        *self = replacement;
        Ok(())
    }

    pub fn state_hash(&self) -> &str {
        &self.state_hash
    }

    pub fn context_json(&self) -> &str {
        &self.context_json
    }
}

fn summary_state_hash(summary: &serde_json::Value) -> Result<String, String> {
    summary
        .get("state_hash")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "E_REPL_SHARED_STATE_HASH_MISSING".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_uses_shared_v25_context_and_fails_unknown_profile_closed() {
        let mut session = ReplSession::new("v1-core-v25").expect("supported profile");
        session.execute_line("x <- 15.").expect("first cell");
        session.execute_line("y <- x + 8.").expect("dependent cell");
        let context_before_failure = session.context_json().to_string();
        session
            .execute_line("z = 1.")
            .expect_err("invalid cell must fail closed");
        assert_eq!(session.context_json(), context_before_failure);
        let stdout = session
            .execute_line("x 보여주기.")
            .expect("context-backed observation");
        assert_eq!(stdout, "15\n");
        assert!(session.state_hash().starts_with("blake3:"));
        assert!(session.context_json().contains("x <- 15."));
        assert!(session.context_json().contains("y <- x + 8."));

        let error = ReplSession::new("unknown-profile").expect_err("unknown profile");
        assert!(error.contains("E_SOURCE_PROFILE_UNSUPPORTED"));
    }

    #[test]
    fn reset_discards_prior_context_and_returns_initial_state() {
        let mut session = ReplSession::new("v1-core-v25").expect("supported profile");
        let initial_hash = session.state_hash().to_string();
        session.execute_line("x <- 15.").expect("first cell");
        assert_ne!(session.state_hash(), initial_hash);
        session.reset().expect("reset");
        assert_eq!(session.state_hash(), initial_hash);
        assert!(!session.context_json().contains("x <- 15."));
    }
}
