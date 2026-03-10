use crate::cli::Runtime;
use std::fs::{self, File};
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
pub struct LaunchRequest {
    pub runtime: Runtime,
    pub shard_id: String,
    pub brief: String,
    pub workspace: PathBuf,
    pub logs_dir: PathBuf,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LaunchSpec {
    pub program: &'static str,
    pub args: Vec<String>,
}

#[derive(Debug)]
pub struct LaunchOutcome {
    pub pid: u32,
    pub stdout_log: PathBuf,
    pub stderr_log: PathBuf,
}

#[derive(Debug)]
pub enum RuntimeLaunchError {
    SpawnFailed {
        program: String,
        shard_id: String,
        source: io::Error,
    },
}

pub fn build_launch_spec(request: &LaunchRequest) -> LaunchSpec {
    match request.runtime {
        Runtime::Codex => LaunchSpec {
            program: "codex",
            args: vec![
                "exec".to_owned(),
                "--skip-git-repo-check".to_owned(),
                request.brief.clone(),
            ],
        },
        Runtime::Claude => LaunchSpec {
            program: "claude",
            args: vec!["-p".to_owned(), request.brief.clone()],
        },
    }
}

pub fn launch_worker(
    request: &LaunchRequest,
    program: &str,
    args: &[&str],
) -> Result<LaunchOutcome, RuntimeLaunchError> {
    fs::create_dir_all(&request.workspace).map_err(|source| RuntimeLaunchError::SpawnFailed {
        program: program.to_owned(),
        shard_id: request.shard_id.clone(),
        source,
    })?;
    fs::create_dir_all(&request.logs_dir).map_err(|source| RuntimeLaunchError::SpawnFailed {
        program: program.to_owned(),
        shard_id: request.shard_id.clone(),
        source,
    })?;

    let stdout_log = request
        .logs_dir
        .join(format!("shard-{}-stdout.log", request.shard_id));
    let stderr_log = request
        .logs_dir
        .join(format!("shard-{}-stderr.log", request.shard_id));
    let stdout = File::create(&stdout_log).map_err(|source| RuntimeLaunchError::SpawnFailed {
        program: program.to_owned(),
        shard_id: request.shard_id.clone(),
        source,
    })?;
    let stderr = File::create(&stderr_log).map_err(|source| RuntimeLaunchError::SpawnFailed {
        program: program.to_owned(),
        shard_id: request.shard_id.clone(),
        source,
    })?;

    let child = Command::new(program)
        .args(args)
        .current_dir(&request.workspace)
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .map_err(|source| RuntimeLaunchError::SpawnFailed {
            program: program.to_owned(),
            shard_id: request.shard_id.clone(),
            source,
        })?;

    Ok(LaunchOutcome {
        pid: child.id(),
        stdout_log,
        stderr_log,
    })
}
