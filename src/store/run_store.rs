use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedRun {
    pub run_id: String,
    pub runtime: String,
    pub objective: String,
    pub shard_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedShard {
    pub shard_id: String,
    pub state: String,
    pub workspace: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedEvent {
    pub timestamp: String,
    pub shard_id: Option<String>,
    pub message: String,
}

pub fn create_run(
    root: &Path,
    run: &PersistedRun,
    shards: &[PersistedShard],
) -> io::Result<PathBuf> {
    let run_dir = root.join(&run.run_id);
    fs::create_dir_all(&run_dir)?;
    write_json(run_dir.join("run.json"), run)?;

    for shard in shards {
        write_json(run_dir.join(format!("shard-{}.json", shard.shard_id)), shard)?;
    }

    Ok(run_dir)
}

pub fn append_event(run_dir: &Path, event: &PersistedEvent) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(run_dir.join("events.jsonl"))?;
    serde_json::to_writer(&mut file, event)?;
    writeln!(file)?;
    Ok(())
}

pub fn load_run(run_dir: &Path) -> io::Result<PersistedRun> {
    read_json(run_dir.join("run.json"))
}

pub fn load_shards(run_dir: &Path) -> io::Result<Vec<PersistedShard>> {
    let mut shard_paths = fs::read_dir(run_dir)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name.starts_with("shard-") && name.ends_with(".json")
                })
        })
        .collect::<Vec<_>>();
    shard_paths.sort();

    shard_paths
        .into_iter()
        .map(read_json)
        .collect::<io::Result<Vec<_>>>()
}

pub fn load_events(run_dir: &Path) -> io::Result<Vec<PersistedEvent>> {
    let bytes = fs::read(run_dir.join("events.jsonl"))?;
    bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).map_err(io::Error::other))
        .collect::<io::Result<Vec<_>>>()
}

fn write_json<T: Serialize>(path: PathBuf, value: &T) -> io::Result<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(path, bytes)
}

fn read_json<T: for<'de> Deserialize<'de>>(path: PathBuf) -> io::Result<T> {
    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes).map_err(io::Error::other)
}
