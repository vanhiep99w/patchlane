use std::process::Command;

fn run_command(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_patchlane"))
        .args(args)
        .output()
        .expect("CLI should be executable")
}

#[test]
fn watch_output_emits_operational_event_lines_without_transcript_noise() {
    let output = run_command(&["swarm", "watch"]);

    assert!(
        output.status.success(),
        "expected swarm watch to succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");
    let expected = "\
2026-03-09T10:00:00Z workflow stage: clarifying objective
2026-03-09T10:02:00Z workflow stage: drafting design
2026-03-09T10:05:00Z workflow stage: writing plan
2026-03-09T10:09:00Z workflow stage: splitting assignments
2026-03-09T10:15:00Z workflow stage: dispatching shards
2026-03-09T10:18:00Z workflow stage: reviewing outputs
2026-03-09T10:21:00Z workflow stage: merging clean shards
";

    assert_eq!(stdout, expected);
    assert!(
        stderr.is_empty(),
        "successful watch should not write to stderr"
    );
    assert!(
        !stdout.contains("transcript"),
        "watch output should avoid transcript framing"
    );
    assert!(
        !stdout.contains("assistant:"),
        "watch output should avoid raw speaker labels"
    );
    assert!(
        !stdout.contains("user:"),
        "watch output should avoid raw speaker labels"
    );
}
