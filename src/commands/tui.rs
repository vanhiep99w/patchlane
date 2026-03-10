use crate::commands::CommandOutcome;
use crate::tui::app::TuiApp;
use crate::tui::render::{render_frame, render_to_string};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;
use std::io;
use std::io::IsTerminal;
use std::path::PathBuf;
use std::time::Duration;

pub fn execute() -> CommandOutcome {
    let state_root = state_root();
    match TuiApp::load_from_store(&state_root) {
        Ok(app) if !io::stdin().is_terminal() || !io::stdout().is_terminal() => {
            CommandOutcome::success(render_to_string(&app))
        }
        Ok(app) => match run_interactive(app) {
            Ok(()) => CommandOutcome::silent_success(),
            Err(error) => CommandOutcome::error(format!("error: {error}")),
        },
        Err(error) => CommandOutcome::error(format!("error: {error}")),
    }
}

fn state_root() -> PathBuf {
    std::env::var_os("PATCHLANE_STATE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".patchlane"))
}

fn run_interactive(mut app: TuiApp) -> io::Result<()> {
    let mut raw_mode = RawModeSession::enter()?;
    let mut stdout = io::stdout();
    let mut session = TerminalSession::enter(&mut stdout)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    let result = run_with_backend(terminal, &mut app, LiveEvents::new());
    let restore_result = session.restore();
    let raw_restore_result = raw_mode.restore();

    match (result, restore_result, raw_restore_result) {
        (Ok(()), Ok(()), Ok(())) => Ok(()),
        (Err(error), _, _) => Err(error),
        (Ok(()), Err(error), _) => Err(error),
        (Ok(()), Ok(()), Err(error)) => Err(error),
    }
}

fn restore_terminal() -> io::Result<()> {
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn run_with_backend<B, I>(mut terminal: Terminal<B>, app: &mut TuiApp, mut events: I) -> io::Result<()>
where
    B: Backend,
    I: Iterator<Item = io::Result<KeyCode>>,
{
    loop {
        terminal.draw(|frame| render_frame(frame, app))?;
        if app.should_quit() {
            return Ok(());
        }

        match events.next() {
            Some(Ok(KeyCode::Char('r'))) => app.refresh()?,
            Some(Ok(key)) => {
                app.handle_key(key);
                if app.should_quit() {
                    return Ok(());
                }
            }
            Some(Err(error)) => return Err(error),
            None => return Ok(()),
        }
    }
}

struct TerminalSession {
    restored: bool,
}

impl TerminalSession {
    fn enter(stdout: &mut io::Stdout) -> io::Result<Self> {
        stdout.execute(EnterAlternateScreen)?;
        Ok(Self { restored: false })
    }

    fn restore(&mut self) -> io::Result<()> {
        complete_restore(&mut self.restored, restore_terminal)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

struct RawModeSession {
    restored: bool,
}

impl RawModeSession {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        Ok(Self { restored: false })
    }

    fn restore(&mut self) -> io::Result<()> {
        complete_restore(&mut self.restored, disable_raw_mode)
    }
}

impl Drop for RawModeSession {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

fn complete_restore<F>(restored: &mut bool, restore: F) -> io::Result<()>
where
    F: FnOnce() -> io::Result<()>,
{
    if *restored {
        return Ok(());
    }
    restore()?;
    *restored = true;
    Ok(())
}

struct LiveEvents;

impl LiveEvents {
    fn new() -> Self {
        Self
    }
}

impl Iterator for LiveEvents {
    type Item = io::Result<KeyCode>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match event::poll(Duration::from_millis(250)) {
                Ok(false) => continue,
                Ok(true) => {}
                Err(error) => return Some(Err(error)),
            }
            match event::read() {
                Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => return Some(Ok(key.code)),
                Ok(_) => {}
                Err(error) => return Some(Err(error)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestration::model::{
        AgentEventType, OrchestratorState, PersistedAgent, PersistedTaskEvent,
    };
    use crate::orchestration::store::{append_task_event, create_task_run, write_agent};
    use crate::tui::store::load_runs;
    use crossterm::event::KeyCode;
    use ratatui::backend::TestBackend;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn interactive_loop_handles_refresh_navigation_and_quit() {
        let state_root = temp_state_root();
        let tasks_root = state_root.join("tasks");
        let run_dir = create_task_run(
            &tasks_root,
            &crate::orchestration::model::PersistedTaskRun {
                run_id: "run-task-010".to_owned(),
                objective: "Inspect live task state".to_owned(),
                runtime: "codex".to_owned(),
                current_phase: "writing-plans".to_owned(),
                overall_state: OrchestratorState::WaitingForApproval,
                blocking_reason: Some("checkpoint pending".to_owned()),
                workspace_root: "workspace".to_owned(),
                workspace_policy: "isolated_by_default".to_owned(),
                default_isolation: true,
                created_at: "2026-03-10T10:00:00Z".to_owned(),
                updated_at: "2026-03-10T10:06:00Z".to_owned(),
            },
        )
        .expect("task run should persist");
        write_agent(
            &run_dir,
            &PersistedAgent {
                agent_id: "agent-plan".to_owned(),
                run_id: "run-task-010".to_owned(),
                parent_agent_id: None,
                role: "writing-plans".to_owned(),
                current_phase: "writing-plans".to_owned(),
                current_state: OrchestratorState::WaitingForApproval,
                runtime: "codex".to_owned(),
                workspace_path: "workspace/plan".to_owned(),
                pid: None,
                related_artifact_ids: vec![],
                stdout_log: "logs/agent-plan-stdout.log".to_owned(),
                stderr_log: "logs/agent-plan-stderr.log".to_owned(),
                created_at: "2026-03-10T10:04:00Z".to_owned(),
                updated_at: "2026-03-10T10:06:00Z".to_owned(),
            },
        )
        .expect("agent should persist");
        fs::write(run_dir.join("logs/agent-plan-stdout.log"), "phase: writing-plans\n")
            .expect("stdout fixture should persist");
        fs::write(run_dir.join("logs/agent-plan-stderr.log"), "")
            .expect("stderr fixture should persist");

        let mut app = TuiApp::load_from_store(&state_root).expect("app should load");
        append_task_event(
            &run_dir,
            &PersistedTaskEvent {
                event_id: "event-plan-refresh".to_owned(),
                run_id: "run-task-010".to_owned(),
                agent_id: Some("agent-plan".to_owned()),
                event_type: AgentEventType::WaitingApproval,
                payload_summary: "Approve? [y/n]".to_owned(),
                timestamp: "2026-03-10T10:07:00Z".to_owned(),
            },
        )
        .expect("event should persist");

        let backend = TestBackend::new(120, 40);
        let terminal = Terminal::new(backend).expect("test terminal should initialize");
        let events = [
            Ok(KeyCode::Tab),
            Ok(KeyCode::Char('j')),
            Ok(KeyCode::Char('r')),
            Ok(KeyCode::Char('q')),
        ];
        let result = run_with_backend(terminal, &mut app, events.into_iter());

        assert!(result.is_ok(), "interactive loop should complete cleanly");
        assert_eq!(app.active_pane(), crate::tui::app::Pane::AgentList);
        assert!(app.should_quit(), "q should terminate the loop");
        assert!(
            app.selected_agent_detail()
                .expect("detail should exist after refresh")
                .blockers
                .iter()
                .any(|blocker| blocker.contains("Approve? [y/n]"))
        );
        assert_eq!(
            load_runs(&state_root)
                .expect("runs should reload")
                .first()
                .expect("run should exist")
                .events
                .len(),
            1
        );
    }

    #[test]
    fn interactive_loop_surfaces_refresh_errors() {
        let state_root = temp_state_root();
        let tasks_root = state_root.join("tasks");
        let run_dir = create_task_run(
            &tasks_root,
            &crate::orchestration::model::PersistedTaskRun {
                run_id: "run-task-020".to_owned(),
                objective: "Detect refresh corruption".to_owned(),
                runtime: "codex".to_owned(),
                current_phase: "writing-plans".to_owned(),
                overall_state: OrchestratorState::Running,
                blocking_reason: None,
                workspace_root: "workspace".to_owned(),
                workspace_policy: "isolated_by_default".to_owned(),
                default_isolation: true,
                created_at: "2026-03-10T10:00:00Z".to_owned(),
                updated_at: "2026-03-10T10:06:00Z".to_owned(),
            },
        )
        .expect("task run should persist");
        write_agent(
            &run_dir,
            &PersistedAgent {
                agent_id: "agent-plan".to_owned(),
                run_id: "run-task-020".to_owned(),
                parent_agent_id: None,
                role: "writing-plans".to_owned(),
                current_phase: "writing-plans".to_owned(),
                current_state: OrchestratorState::Running,
                runtime: "codex".to_owned(),
                workspace_path: "workspace/plan".to_owned(),
                pid: None,
                related_artifact_ids: vec![],
                stdout_log: "logs/agent-plan-stdout.log".to_owned(),
                stderr_log: "logs/agent-plan-stderr.log".to_owned(),
                created_at: "2026-03-10T10:04:00Z".to_owned(),
                updated_at: "2026-03-10T10:06:00Z".to_owned(),
            },
        )
        .expect("agent should persist");

        let mut app = TuiApp::load_from_store(&state_root).expect("app should load");
        let bad_run_dir = tasks_root.join("run-task-bad");
        fs::create_dir_all(&bad_run_dir).expect("bad run dir should persist");
        fs::write(bad_run_dir.join("run.json"), "{invalid").expect("bad run should persist");

        let backend = TestBackend::new(120, 40);
        let terminal = Terminal::new(backend).expect("test terminal should initialize");
        let result = run_with_backend(terminal, &mut app, [Ok(KeyCode::Char('r'))].into_iter());

        let error = result.expect_err("refresh should surface corrupt runs");
        assert!(
            error.to_string().contains("run-task-bad"),
            "refresh error should identify corrupt run, got: {error}"
        );
    }

    #[test]
    fn interactive_loop_surfaces_event_errors() {
        let mut app = TuiApp::from_snapshot(crate::orchestration::model::TaskSnapshot {
            run: crate::orchestration::model::PersistedTaskRun {
                run_id: "run-task-030".to_owned(),
                objective: "Surface event errors".to_owned(),
                runtime: "codex".to_owned(),
                current_phase: "writing-plans".to_owned(),
                overall_state: OrchestratorState::Running,
                blocking_reason: None,
                workspace_root: "workspace".to_owned(),
                workspace_policy: "isolated_by_default".to_owned(),
                default_isolation: true,
                created_at: "2026-03-10T10:00:00Z".to_owned(),
                updated_at: "2026-03-10T10:06:00Z".to_owned(),
            },
            agents: vec![],
            checkpoints: vec![],
            artifacts: vec![],
            events: vec![],
        });

        let backend = TestBackend::new(120, 40);
        let terminal = Terminal::new(backend).expect("test terminal should initialize");
        let result = run_with_backend(
            terminal,
            &mut app,
            [Err(io::Error::new(io::ErrorKind::Other, "poll failed"))].into_iter(),
        );

        let error = result.expect_err("event errors should surface");
        assert!(
            error.to_string().contains("poll failed"),
            "event error should be preserved, got: {error}"
        );
    }

    #[test]
    fn complete_restore_only_marks_restored_after_success() {
        let mut restored = false;
        let error = complete_restore(&mut restored, || {
            Err(io::Error::new(io::ErrorKind::Other, "leave failed"))
        })
        .expect_err("restore should fail");
        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert!(!restored, "failed restore should remain retryable");

        complete_restore(&mut restored, || Ok(())).expect("retry should succeed");
        assert!(restored, "successful restore should mark completion");
    }

    fn temp_state_root() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("patchlane-tui-command-{unique}"));
        fs::create_dir_all(root.join("tasks")).expect("temp root should be creatable");
        root
    }
}
