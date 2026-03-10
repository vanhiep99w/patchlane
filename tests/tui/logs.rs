use patchlane::tui::logs::tail_log;
use patchlane::tui::store::resolve_log_path;
use std::path::Path;

#[test]
fn tail_log_returns_latest_lines_for_selected_agent() {
    let lines = tail_log(
        Path::new("tests/fixtures/agent-plan-stdout.log"),
        2,
    )
    .expect("tail should succeed");

    assert_eq!(
        lines,
        vec!["phase: writing-plans".to_owned(), "Approve? [y/n]".to_owned()]
    );
}

#[test]
fn resolve_log_path_stays_within_run_logs_directory() {
    let state_root = Path::new("/tmp/patchlane-state");

    let path = resolve_log_path(state_root, "run-task-001", "../secrets.txt");
    assert_eq!(
        path,
        Path::new("/tmp/patchlane-state/tasks/run-task-001/logs/secrets.txt")
    );

    let absolute = resolve_log_path(state_root, "run-task-001", "/etc/passwd");
    assert_eq!(
        absolute,
        Path::new("/tmp/patchlane-state/tasks/run-task-001/logs/passwd")
    );
}
