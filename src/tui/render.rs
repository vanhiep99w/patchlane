use crate::tui::app::{SelectedAgentDetail, TuiApp};
use ratatui::backend::TestBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Terminal;

pub fn render_to_string(app: &TuiApp) -> String {
    let selected_run = app
        .selected_run()
        .map(|run| format!("{} {}", run.run.run_id, run.run.objective))
        .unwrap_or_else(|| "No runs loaded".to_owned());
    let selected_agent = app
        .selected_agent()
        .map(|agent| format!("{} {}", agent.agent_id, agent.state))
        .unwrap_or_else(|| "No agent selected".to_owned());

    format!(
        "Patchlane TUI\n\nSelected Run\n  {selected_run}\n\nSelected Agent\n  {selected_agent}\n\nKeys\n  q quit\n  j/k move selection\n  tab switch pane\n  r refresh"
    )
}

pub fn render_to_test_buffer(app: TuiApp) -> String {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| render_frame(frame, &app))
        .expect("test frame should render");

    let backend = terminal.backend();
    backend
        .buffer()
        .content
        .chunks(120)
        .map(|row| {
            row.iter()
                .map(|cell| cell.symbol())
                .collect::<String>()
                .trim_end()
                .to_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn render_frame(frame: &mut ratatui::Frame<'_>, app: &TuiApp) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(9), Constraint::Min(0)])
        .split(frame.area());
    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(root[0]);
    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(root[1]);

    let run_items = app
        .runs()
        .into_iter()
        .map(|run| {
            ListItem::new(format!(
                "{} {} [{} blockers]",
                run.run_id, run.current_state, run.blocker_count
            ))
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        List::new(run_items).block(Block::default().title("Runs").borders(Borders::ALL)),
        top[0],
    );

    let agent_items = app
        .agent_rows()
        .into_iter()
        .map(|agent| {
            let marker = if agent.has_blocker { "!" } else { "-" };
            ListItem::new(format!("{marker} {} {} {}", agent.agent_id, agent.phase, agent.state))
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        List::new(agent_items).block(Block::default().title("Agents").borders(Borders::ALL)),
        top[1],
    );

    let detail = app.selected_agent_detail();
    frame.render_widget(detail_summary(detail.as_ref()), top[2]);
    frame.render_widget(detail_body(detail.as_ref()), bottom[0]);
    frame.render_widget(log_panel(detail.as_ref()), bottom[1]);
}

fn detail_summary(detail: Option<&SelectedAgentDetail>) -> Paragraph<'static> {
    let lines = if let Some(detail) = detail {
        vec![
            Line::from(format!("phase: {}", detail.current_phase)),
            Line::from(format!("state: {}", detail.current_state)),
            Line::from(format!("stdout: {}", detail.stdout_log)),
            Line::from(format!("stderr: {}", detail.stderr_log)),
        ]
    } else {
        vec![Line::from("No agent selected")]
    };
    Paragraph::new(lines).block(Block::default().title("Selected Agent").borders(Borders::ALL))
}

fn detail_body(detail: Option<&SelectedAgentDetail>) -> Paragraph<'static> {
    let mut lines = Vec::new();
    lines.push(Line::styled(
        "Timeline",
        Style::default().add_modifier(Modifier::BOLD),
    ));
    if let Some(detail) = detail {
        for event in &detail.timeline {
            lines.push(Line::from(format!(
                "{} {} {}",
                event.timestamp, event.event_type, event.payload_summary
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::styled(
        "Artifacts",
        Style::default().add_modifier(Modifier::BOLD),
    ));
    if let Some(detail) = detail {
        for artifact in &detail.artifacts {
            lines.push(Line::from(format!("{} {}", artifact.artifact_type, artifact.path)));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::styled(
        "Blockers",
        Style::default().add_modifier(Modifier::BOLD),
    ));
    if let Some(detail) = detail {
        for blocker in &detail.blockers {
            lines.push(Line::from(format!("- {blocker}")));
        }
    }

    Paragraph::new(lines).block(Block::default().title("Detail").borders(Borders::ALL))
}

fn log_panel(detail: Option<&SelectedAgentDetail>) -> Paragraph<'static> {
    let mut lines = vec![Line::styled(
        "Logs",
        Style::default().add_modifier(Modifier::BOLD),
    )];

    if let Some(detail) = detail {
        lines.push(Line::from("stdout"));
        if let Some(error) = &detail.stdout_error {
            lines.push(Line::from(error.clone()));
        } else {
            for line in &detail.stdout_tail {
                lines.push(Line::from(line.clone()));
            }
        }
        lines.push(Line::from(""));
        lines.push(Line::from("stderr"));
        if let Some(error) = &detail.stderr_error {
            lines.push(Line::from(error.clone()));
        } else {
            for line in &detail.stderr_tail {
                lines.push(Line::from(line.clone()));
            }
        }
    }

    Paragraph::new(lines).block(Block::default().title("Logs").borders(Borders::ALL))
}
