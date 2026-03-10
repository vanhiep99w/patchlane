use crate::orchestration::model::{
    PersistedAgent, PersistedArtifact, PersistedCheckpoint, PersistedTaskEvent, PersistedTaskRun,
    TaskSnapshot,
};
use serde::{de::DeserializeOwned, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn create_task_run(root: &Path, run: &PersistedTaskRun) -> io::Result<PathBuf> {
    let run_dir = root.join(&run.run_id);
    fs::create_dir_all(run_dir.join("agents"))?;
    fs::create_dir_all(run_dir.join("checkpoints"))?;
    fs::create_dir_all(run_dir.join("artifacts"))?;
    fs::create_dir_all(run_dir.join("logs"))?;
    write_json(run_dir.join("run.json"), run)?;
    fs::write(run_dir.join("events.jsonl"), b"")?;
    Ok(run_dir)
}

pub fn write_agent(run_dir: &Path, agent: &PersistedAgent) -> io::Result<()> {
    write_json(run_dir.join("agents").join(format!("{}.json", agent.agent_id)), agent)
}

pub fn write_checkpoint(run_dir: &Path, checkpoint: &PersistedCheckpoint) -> io::Result<()> {
    write_json(
        run_dir
            .join("checkpoints")
            .join(format!("{}.json", checkpoint.checkpoint_id)),
        checkpoint,
    )
}

pub fn write_artifact(run_dir: &Path, artifact: &PersistedArtifact) -> io::Result<()> {
    write_json(
        run_dir
            .join("artifacts")
            .join(format!("{}.json", artifact.artifact_id)),
        artifact,
    )
}

pub fn append_task_event(run_dir: &Path, event: &PersistedTaskEvent) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(run_dir.join("events.jsonl"))?;
    serde_json::to_writer(&mut file, event)?;
    writeln!(file)?;
    Ok(())
}

pub fn load_task_run(run_dir: &Path) -> io::Result<PersistedTaskRun> {
    read_json(run_dir.join("run.json"))
}

pub fn write_task_run(run_dir: &Path, run: &PersistedTaskRun) -> io::Result<()> {
    write_json(run_dir.join("run.json"), run)
}

pub fn load_task_snapshot(run_dir: &Path) -> io::Result<TaskSnapshot> {
    Ok(TaskSnapshot {
        run: load_task_run(run_dir)?,
        agents: read_json_dir(run_dir.join("agents"))?,
        checkpoints: read_json_dir(run_dir.join("checkpoints"))?,
        artifacts: read_json_dir(run_dir.join("artifacts"))?,
        events: read_jsonl(run_dir.join("events.jsonl"))?,
    })
}

fn write_json<T: Serialize + ?Sized>(path: PathBuf, value: &T) -> io::Result<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(path, bytes)
}

fn read_json<T: DeserializeOwned>(path: PathBuf) -> io::Result<T> {
    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes).map_err(io::Error::other)
}

fn read_json_dir<T: DeserializeOwned>(dir: PathBuf) -> io::Result<Vec<T>> {
    let mut paths = fs::read_dir(dir)?
        .collect::<io::Result<Vec<_>>>()?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort();
    paths.into_iter().map(read_json).collect()
}

fn read_jsonl<T: DeserializeOwned>(path: PathBuf) -> io::Result<Vec<T>> {
    let bytes = fs::read(path)?;
    bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).map_err(io::Error::other))
        .collect()
}
