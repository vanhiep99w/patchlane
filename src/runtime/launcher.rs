use crate::cli::Runtime;
use std::fs::{self, File};
use std::io;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

#[derive(Debug, Clone)]
pub struct LaunchRequest {
    pub runtime: Runtime,
    pub shard_id: String,
    pub brief: String,
    pub workspace: PathBuf,
    pub logs_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct AgentLaunchRequest {
    pub runtime: Runtime,
    pub run_id: String,
    pub role: String,
    pub prompt: String,
    pub workspace: PathBuf,
    pub logs_dir: PathBuf,
    pub run_dir: PathBuf,
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

pub struct ManagedLaunchOutcome {
    pub child: Child,
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
    if let Ok(mode) = std::env::var("PATCHLANE_TEST_RUNTIME_MODE") {
        return match mode.as_str() {
            "success" => LaunchSpec {
                program: "sh",
                args: vec!["-c".to_owned(), "printf patchlane-worker".to_owned()],
            },
            "missing_binary" => LaunchSpec {
                program: "__patchlane_missing_binary__",
                args: vec!["simulate-worker".to_owned()],
            },
            _ => build_default_launch_spec(request),
        };
    }

    build_default_launch_spec(request)
}

fn build_default_launch_spec(request: &LaunchRequest) -> LaunchSpec {
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
    let managed = spawn_worker(request, program, args)?;
    Ok(LaunchOutcome {
        pid: managed.child.id(),
        stdout_log: managed.stdout_log,
        stderr_log: managed.stderr_log,
    })
}

pub fn launch_agent(request: &AgentLaunchRequest) -> Result<LaunchOutcome, RuntimeLaunchError> {
    let reporting_contract = format!(
        " Report state with `patchlane agent-event --run-dir {} --run-id {} --agent-id {} --message <payload> <event-type>`. Use artifact payload `<type>|<path>` and waiting-approval payload `<checkpoint-id>|<prompt>`.",
        request.run_dir.display(),
        request.run_id,
        request.role
    );
    let launch_request = LaunchRequest {
        runtime: request.runtime.clone(),
        shard_id: format!("agent-{}", request.role),
        brief: format!("{}{}", request.prompt, reporting_contract),
        workspace: request.workspace.clone(),
        logs_dir: request.logs_dir.clone(),
    };
    let spec = build_agent_launch_spec(request, &launch_request);
    let args = spec.args.iter().map(String::as_str).collect::<Vec<_>>();
    launch_worker(&launch_request, spec.program, &args)
}

fn build_agent_launch_spec(
    request: &AgentLaunchRequest,
    launch_request: &LaunchRequest,
) -> LaunchSpec {
    if let Ok(mode) = std::env::var("PATCHLANE_TEST_RUNTIME_MODE") {
        return match mode.as_str() {
            "success" => build_agent_success_spec(request),
            "missing_binary" => LaunchSpec {
                program: "__patchlane_missing_binary__",
                args: vec!["simulate-agent".to_owned()],
            },
            _ => build_default_launch_spec(launch_request),
        };
    }

    build_default_launch_spec(launch_request)
}

fn build_agent_success_spec(request: &AgentLaunchRequest) -> LaunchSpec {
    let phase_message = format!("launcher-contract:{}", request.run_id);
    let current_exe = std::env::current_exe()
        .unwrap_or_else(|_| PathBuf::from("patchlane"))
        .display()
        .to_string();
    let script = concat!(
        "printf 'launcher-contract run_id=%s agent=%s\\n' \"$3\" \"$4\"; ",
        "printf 'launcher-contract stderr agent=%s\\n' \"$4\" >&2; ",
        "\"$1\" agent-event phase --run-dir \"$2\" --run-id \"$3\" --agent-id \"$4\" --message \"$5\""
    );
    LaunchSpec {
        program: "sh",
        args: vec![
            "-c".to_owned(),
            script.to_owned(),
            "patchlane-agent-launch".to_owned(),
            current_exe,
            request.run_dir.display().to_string(),
            request.run_id.clone(),
            request.role.clone(),
            phase_message,
        ],
    }
}

pub fn spawn_worker(
    request: &LaunchRequest,
    program: &str,
    args: &[&str],
) -> Result<ManagedLaunchOutcome, RuntimeLaunchError> {
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

    Ok(ManagedLaunchOutcome {
        child,
        stdout_log,
        stderr_log,
    })
}
