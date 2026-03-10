use patchlane::store::run_store::{
    append_event,
    create_run,
    load_run,
    load_shards,
    PersistedEvent,
    PersistedRun,
    PersistedShard,
};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("patchlane-run-store-{unique}"));
    fs::create_dir_all(&root).expect("temp root should be creatable");
    root
}

#[test]
fn run_store_persists_run_shards_and_events() {
    let root = temp_root();
    let run = PersistedRun {
        run_id: "run-001".to_owned(),
        runtime: "codex".to_owned(),
        objective: "demo objective".to_owned(),
        shard_count: 2,
    };
    let shards = vec![
        PersistedShard {
            shard_id: "01".to_owned(),
            state: "queued".to_owned(),
            workspace: "workspace-01".to_owned(),
        },
        PersistedShard {
            shard_id: "02".to_owned(),
            state: "queued".to_owned(),
            workspace: "workspace-02".to_owned(),
        },
    ];

    let run_dir = create_run(&root, &run, &shards).expect("run directory should be created");

    assert!(run_dir.is_dir(), "run path should be a directory");
    assert!(run_dir.join("run.json").is_file(), "run.json should exist");
    assert!(
        run_dir.join("shard-01.json").is_file(),
        "first shard metadata should exist"
    );
    assert!(
        run_dir.join("shard-02.json").is_file(),
        "second shard metadata should exist"
    );

    append_event(
        &run_dir,
        &PersistedEvent {
            timestamp: "2026-03-10T00:00:00Z".to_owned(),
            shard_id: Some("01".to_owned()),
            message: "worker launched".to_owned(),
        },
    )
    .expect("event append should succeed");

    let loaded_run = load_run(&run_dir).expect("run metadata should load");
    let loaded_shards = load_shards(&run_dir).expect("shards should load");
    let events = fs::read_to_string(run_dir.join("events.jsonl")).expect("events file should load");

    assert_eq!(loaded_run.run_id, "run-001");
    assert_eq!(loaded_run.runtime, "codex");
    assert_eq!(loaded_run.shard_count, 2);
    assert_eq!(loaded_shards.len(), 2);
    assert_eq!(loaded_shards[0].shard_id, "01");
    assert!(events.contains("worker launched"));

    fs::remove_dir_all(root).expect("temp root should be removable");
}
