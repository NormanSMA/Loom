use crate::process_group::{ProcessGuard, configure_command};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{mpsc, oneshot};

#[derive(Debug, Error)]
pub enum SupervisorError {
    #[error("no se encontro el ejecutable '{0}' en el PATH")]
    ProgramNotFound(String),
    #[error("fallo al iniciar proceso '{program}': {source}")]
    Spawn {
        program: String,
        #[source]
        source: std::io::Error,
    },
    #[error("fallo al escribir stdin en '{program}': {source}")]
    Stdin {
        program: String,
        #[source]
        source: std::io::Error,
    },
    #[error("fallo al vigilar grupo de procesos: {0}")]
    Guard(#[source] std::io::Error),
    #[error("error al esperar cierre de '{program}': {source}")]
    Wait {
        program: String,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminationReason {
    Completed,
    IdleTimeout,
    TotalTimeout,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct RunSpec {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub stdin_payload: Option<String>,
    pub env: Vec<(String, String)>,
    pub idle_timeout: Duration,
    pub total_timeout: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunResult {
    pub exit_code: Option<i32>,
    pub termination: TerminationReason,
    pub stdout_lines: Vec<String>,
    pub stderr_lines: Vec<String>,
    pub duration_ms: u64,
}

enum OutputLine {
    Stdout(String),
    Stderr(String),
}

pub fn resolve_program(program: &str) -> Result<PathBuf, SupervisorError> {
    let candidate = Path::new(program);
    if candidate.is_file() {
        return Ok(candidate.to_path_buf());
    }
    which::which(program).map_err(|_| SupervisorError::ProgramNotFound(program.to_string()))
}

pub async fn run_supervised(
    spec: RunSpec,
    cancel: Option<oneshot::Receiver<()>>,
) -> Result<RunResult, SupervisorError> {
    let started = Instant::now();
    let resolved = resolve_program(&spec.program)?;

    let mut cmd = tokio::process::Command::new(&resolved);
    configure_command(&mut cmd);
    cmd.args(&spec.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    if let Some(cwd) = &spec.cwd {
        cmd.current_dir(cwd);
    }
    for (k, v) in &spec.env {
        cmd.env(k, v);
    }

    let mut child = cmd.spawn().map_err(|source| SupervisorError::Spawn {
        program: spec.program.clone(),
        source,
    })?;

    let mut guard = match child.id() {
        Some(pid) => ProcessGuard::attach(pid).map_err(SupervisorError::Guard)?,
        None => {
            return Err(SupervisorError::Guard(std::io::Error::other(
                "proceso sin pid",
            )));
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        if let Some(payload) = &spec.stdin_payload {
            stdin
                .write_all(payload.as_bytes())
                .await
                .map_err(|source| SupervisorError::Stdin {
                    program: spec.program.clone(),
                    source,
                })?;
            let _ = stdin.shutdown().await;
        }
        drop(stdin);
    }

    let (tx, mut rx) = mpsc::channel::<OutputLine>(256);
    let stdout_task = child.stdout.take().map(|out| {
        let tx_out = tx.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(out).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if tx_out.send(OutputLine::Stdout(line)).await.is_err() {
                    break;
                }
            }
        })
    });
    let stderr_task = child.stderr.take().map(|err| {
        let tx_err = tx.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(err).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if tx_err.send(OutputLine::Stderr(line)).await.is_err() {
                    break;
                }
            }
        })
    });
    drop(tx);

    let mut stdout_lines = Vec::new();
    let mut stderr_lines = Vec::new();

    let total_sleep = tokio::time::sleep(spec.total_timeout);
    let idle_sleep = tokio::time::sleep(spec.idle_timeout);
    let cancel_fut = async move {
        match cancel {
            Some(rx) => {
                let _ = rx.await;
            }
            None => std::future::pending::<()>().await,
        }
    };

    tokio::pin!(total_sleep);
    tokio::pin!(idle_sleep);
    tokio::pin!(cancel_fut);

    let (termination, exit_code) = loop {
        tokio::select! {
            maybe_line = rx.recv() => {
                if let Some(line) = maybe_line {
                    idle_sleep.as_mut().reset(tokio::time::Instant::now() + spec.idle_timeout);
                    match line {
                        OutputLine::Stdout(s) => stdout_lines.push(s),
                        OutputLine::Stderr(s) => stderr_lines.push(s),
                    }
                } else {
                    let status = child.wait().await.map_err(|source| SupervisorError::Wait {
                        program: spec.program.clone(),
                        source,
                    })?;
                    guard.disarm();
                    break (TerminationReason::Completed, status.code());
                }
            }
            status_res = child.wait() => {
                let status = status_res.map_err(|source| SupervisorError::Wait {
                    program: spec.program.clone(),
                    source,
                })?;
                guard.disarm();
                drain_lines(&mut rx, &mut stdout_lines, &mut stderr_lines).await;
                break (TerminationReason::Completed, status.code());
            }
            () = &mut idle_sleep => {
                guard.kill_tree();
                let _ = child.wait().await;
                break (TerminationReason::IdleTimeout, None);
            }
            () = &mut total_sleep => {
                guard.kill_tree();
                let _ = child.wait().await;
                break (TerminationReason::TotalTimeout, None);
            }
            () = &mut cancel_fut => {
                guard.kill_tree();
                let _ = child.wait().await;
                break (TerminationReason::Cancelled, None);
            }
        }
    };

    if let Some(h) = stdout_task {
        h.abort();
    }
    if let Some(h) = stderr_task {
        h.abort();
    }

    Ok(RunResult {
        exit_code,
        termination,
        stdout_lines,
        stderr_lines,
        duration_ms: started.elapsed().as_millis() as u64,
    })
}

async fn drain_lines(
    rx: &mut mpsc::Receiver<OutputLine>,
    stdout_lines: &mut Vec<String>,
    stderr_lines: &mut Vec<String>,
) {
    let deadline = tokio::time::Instant::now() + Duration::from_millis(100);
    while let Ok(Some(line)) = tokio::time::timeout_at(deadline, rx.recv()).await {
        match line {
            OutputLine::Stdout(s) => stdout_lines.push(s),
            OutputLine::Stderr(s) => stderr_lines.push(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn echo_stdin_spec() -> RunSpec {
        #[cfg(windows)]
        let (program, args) = (
            "powershell".to_string(),
            vec![
                "-NoProfile".to_string(),
                "-Command".to_string(),
                "[Console]::In.ReadToEnd()".to_string(),
            ],
        );
        #[cfg(unix)]
        let (program, args) = ("cat".to_string(), Vec::new());

        RunSpec {
            program,
            args,
            cwd: None,
            stdin_payload: Some("hola-loom\n".to_string()),
            env: Vec::new(),
            idle_timeout: Duration::from_secs(5),
            total_timeout: Duration::from_secs(10),
        }
    }

    fn sleep_spec(secs: u64, idle_timeout: Duration) -> RunSpec {
        #[cfg(windows)]
        let (program, args) = (
            "powershell".to_string(),
            vec![
                "-NoProfile".to_string(),
                "-Command".to_string(),
                format!("Start-Sleep -Seconds {secs}"),
            ],
        );
        #[cfg(unix)]
        let (program, args) = ("sh".to_string(), vec!["-c".into(), format!("sleep {secs}")]);

        RunSpec {
            program,
            args,
            cwd: None,
            stdin_payload: None,
            env: Vec::new(),
            idle_timeout,
            total_timeout: Duration::from_secs(30),
        }
    }

    #[tokio::test]
    async fn echoes_stdin_and_completes() {
        let res = run_supervised(echo_stdin_spec(), None).await.unwrap();
        assert_eq!(res.termination, TerminationReason::Completed);
        assert_eq!(res.exit_code, Some(0));
        assert!(res.stdout_lines.iter().any(|l| l.contains("hola-loom")));
    }

    #[tokio::test]
    async fn kills_hung_process_on_idle_timeout() {
        let res = run_supervised(sleep_spec(30, Duration::from_millis(350)), None)
            .await
            .unwrap();
        assert_eq!(res.termination, TerminationReason::IdleTimeout);
        assert!(res.duration_ms < 4_000);
    }

    #[tokio::test]
    async fn kills_process_on_cancel_signal() {
        let (tx, rx) = oneshot::channel();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(150)).await;
            let _ = tx.send(());
        });

        let res = run_supervised(sleep_spec(30, Duration::from_secs(15)), Some(rx))
            .await
            .unwrap();
        assert_eq!(res.termination, TerminationReason::Cancelled);
        assert!(res.duration_ms < 3_000);
    }
}
