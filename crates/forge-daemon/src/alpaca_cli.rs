//! Alpaca CLI subprocess bridge — the one seam where gate output becomes an
//! API call. Ported from F:\v3\forge-daemon-door\src\oracle_escalate.rs
//! (spawn/exists-check/typed-refusal shape); env names per CLI README.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::AlpacaCredentials;

/// Repo-relative default location of the checksum-verified CLI binary.
pub const DEFAULT_EXE: &str = r"tools\alpaca-cli\alpaca.exe";

#[derive(Debug, PartialEq, Eq)]
pub enum CliRefusal {
    ExeNotFound(String),
    SpawnFailed(String),
    NonZeroExit { code: Option<i32>, stderr: String },
    NotUtf8(String),
}

pub struct AlpacaCli {
    exe: PathBuf,
}

impl AlpacaCli {
    pub fn new(exe: impl Into<PathBuf>) -> Self {
        Self { exe: exe.into() }
    }

    /// `repo_root/tools/alpaca-cli/alpaca.exe`.
    pub fn at_repo_root(root: &Path) -> Self {
        Self::new(root.join(DEFAULT_EXE))
    }

    /// Run one CLI invocation to completion. Exists-check before spawn;
    /// stderr truncated to 200 chars in the refusal, never dropped.
    /// Credentials enter the child as `ALPACA_API_KEY`/`ALPACA_SECRET_KEY`
    /// env vars only — never argv (argv is world-readable on Windows).
    pub fn run(&self, creds: &AlpacaCredentials, args: &[&str]) -> Result<String, CliRefusal> {
        if !self.exe.exists() {
            return Err(CliRefusal::ExeNotFound(self.exe.display().to_string()));
        }

        let output = Command::new(&self.exe)
            .args(args)
            .env("ALPACA_API_KEY", String::from_utf8_lossy(creds.key_id.expose()).as_ref())
            .env("ALPACA_SECRET_KEY", String::from_utf8_lossy(creds.secret_key.expose()).as_ref())
            .output()
            .map_err(|e| CliRefusal::SpawnFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr: String = String::from_utf8_lossy(&output.stderr).chars().take(200).collect();
            return Err(CliRefusal::NonZeroExit { code: output.status.code(), stderr });
        }

        String::from_utf8(output.stdout)
            .map(|s| s.trim().to_string())
            .map_err(|e| CliRefusal::NotUtf8(e.to_string()))
    }

    /// Like [`Self::run`], but pipes `stdin_body` into the child — for JSON
    /// bodies (`--body @-`) that must never appear in argv.
    pub fn run_with_stdin(&self, creds: &AlpacaCredentials, args: &[&str], stdin_body: &[u8]) -> Result<String, CliRefusal> {
        use std::io::Write;
        use std::process::Stdio;

        if !self.exe.exists() {
            return Err(CliRefusal::ExeNotFound(self.exe.display().to_string()));
        }

        let mut child = Command::new(&self.exe)
            .args(args)
            .env("ALPACA_API_KEY", String::from_utf8_lossy(creds.key_id.expose()).as_ref())
            .env("ALPACA_SECRET_KEY", String::from_utf8_lossy(creds.secret_key.expose()).as_ref())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| CliRefusal::SpawnFailed(e.to_string()))?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(stdin_body).map_err(|e| CliRefusal::SpawnFailed(e.to_string()))?;
        }
        drop(child.stdin.take());

        let output = child.wait_with_output().map_err(|e| CliRefusal::SpawnFailed(e.to_string()))?;
        if !output.status.success() {
            let mut stderr: String = String::from_utf8_lossy(&output.stderr).chars().take(200).collect();
            if stderr.trim().is_empty() {
                stderr = String::from_utf8_lossy(&output.stdout).chars().take(200).collect();
            }
            return Err(CliRefusal::NonZeroExit { code: output.status.code(), stderr });
        }
        String::from_utf8(output.stdout)
            .map(|s| s.trim().to_string())
            .map_err(|e| CliRefusal::NotUtf8(e.to_string()))
    }

    pub fn account_get(&self, creds: &AlpacaCredentials) -> Result<String, CliRefusal> {
        self.run(creds, &["account", "get"])
    }

    pub fn clock(&self, creds: &AlpacaCredentials) -> Result<String, CliRefusal> {
        self.run(creds, &["clock"])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secrets::SecureSecret;

    fn fake_creds() -> AlpacaCredentials {
        AlpacaCredentials {
            key_id: SecureSecret::new(b"PKTEST".to_vec()),
            secret_key: SecureSecret::new(b"test-secret".to_vec()),
            base_url: crate::config::ALPACA_PAPER_BASE_URL.to_string(),
        }
    }

    #[test]
    fn a_missing_exe_is_refused_before_any_spawn() {
        let cli = AlpacaCli::new(r"Z:\does\not\exist\alpaca.exe");
        match cli.run(&fake_creds(), &["clock"]) {
            Err(CliRefusal::ExeNotFound(p)) => assert!(p.contains("does")),
            other => panic!("expected ExeNotFound, got {other:?}"),
        }
    }

    #[cfg(windows)]
    #[test]
    fn stdout_of_a_successful_child_comes_back_trimmed() {
        let cli = AlpacaCli::new(r"C:\Windows\System32\cmd.exe");
        let out = cli.run(&fake_creds(), &["/C", "echo hello"]).expect("echo succeeds");
        assert_eq!(out, "hello");
    }

    #[cfg(windows)]
    #[test]
    fn a_nonzero_exit_is_a_typed_refusal_with_stderr() {
        let cli = AlpacaCli::new(r"C:\Windows\System32\cmd.exe");
        match cli.run(&fake_creds(), &["/C", "echo boom 1>&2 & exit 7"]) {
            Err(CliRefusal::NonZeroExit { code, stderr }) => {
                assert_eq!(code, Some(7));
                assert!(stderr.contains("boom"));
            }
            other => panic!("expected NonZeroExit, got {other:?}"),
        }
    }

    #[cfg(windows)]
    #[test]
    fn credentials_reach_the_child_as_alpaca_env_vars_never_argv() {
        let cli = AlpacaCli::new(r"C:\Windows\System32\cmd.exe");
        let out = cli.run(&fake_creds(), &["/C", "echo %ALPACA_API_KEY%:%ALPACA_SECRET_KEY%"]).unwrap();
        assert_eq!(out, "PKTEST:test-secret");
    }
}
