use std::process::Command;

fn run_command(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_patchlane"))
        .args(args)
        .output()
        .expect("CLI should be executable")
}

#[test]
fn run_output_prints_opening_sections_in_contract_order() {
    let output = run_command(&["swarm", "run", "demo objective"]);

    assert!(
        output.status.success(),
        "expected swarm run with an objective to succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");

    let expected = [
        "Run",
        "Objective",
        "Plan",
        "Placement",
        "Next",
    ];

    let mut cursor = 0usize;
    for heading in expected {
        let relative_index = stderr[cursor..]
            .find(heading)
            .unwrap_or_else(|| panic!("expected `{heading}` after byte {cursor}, got:\n{stderr}"));
        cursor += relative_index + heading.len();
    }
}
