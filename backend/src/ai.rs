use anyhow::{bail, Context};
use serde::de::DeserializeOwned;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;

#[derive(Clone, Copy, Debug)]
pub enum Model {
    Haiku,
    Sonnet,
}

impl Model {
    fn alias(self) -> &'static str {
        match self {
            Model::Haiku => "haiku",
            Model::Sonnet => "sonnet",
        }
    }
}

#[derive(Clone)]
pub struct Claude {
    bin: String,
    timeout: Duration,
}

impl Claude {
    pub fn from_env() -> Self {
        Self {
            bin: std::env::var("EMILE_CLAUDE_BIN").unwrap_or_else(|_| "claude".into()),
            timeout: Duration::from_secs(std::env::var("EMILE_CLAUDE_TIMEOUT_SECS").ok().and_then(|v| v.parse().ok()).unwrap_or(120)),
        }
    }

    pub async fn ask<T: DeserializeOwned>(&self, model: Model, system: &str, prompt: &str) -> anyhow::Result<T> {
        let child = Command::new(&self.bin)
            .arg("-p")
            .arg(prompt)
            .args(["--model", model.alias(), "--output-format", "json", "--tools", "", "--no-session-persistence"])
            .arg("--append-system-prompt")
            .arg(system)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .with_context(|| format!("cannot start {}", self.bin))?;
        let out = tokio::time::timeout(self.timeout, child.wait_with_output()).await.context("claude timed out")??;
        if !out.status.success() {
            bail!("claude exited with {}", out.status);
        }
        let envelope: serde_json::Value = serde_json::from_slice(&out.stdout).context("claude output is not JSON")?;
        if envelope.get("is_error").and_then(|v| v.as_bool()).unwrap_or(false) {
            bail!("claude returned an error result");
        }
        let text = envelope.get("result").and_then(|v| v.as_str()).context("claude output has no result")?;
        serde_json::from_str(strip_fences(text)).context("claude result is not the expected JSON")
    }
}

fn strip_fences(text: &str) -> &str {
    let t = text.trim();
    let t = t.strip_prefix("```json").or_else(|| t.strip_prefix("```")).unwrap_or(t);
    t.strip_suffix("```").unwrap_or(t).trim()
}

#[cfg(test)]
mod tests {
    use super::strip_fences;

    #[test]
    fn strips_code_fences() {
        assert_eq!(strip_fences("```json\n{\"a\":1}\n```"), "{\"a\":1}");
        assert_eq!(strip_fences(" {\"a\":1} "), "{\"a\":1}");
    }
}
