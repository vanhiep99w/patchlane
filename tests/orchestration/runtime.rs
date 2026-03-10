use patchlane::cli::Runtime;
use patchlane::orchestration::runtime::{
    resolve_runtime, DetectionContext, RuntimeResolutionErrorKind, RuntimeResolutionState,
};

#[test]
fn explicit_runtime_wins_over_detection() {
    let resolved = resolve_runtime(Some(Runtime::Claude), DetectionContext::codex())
        .expect("runtime should resolve");
    assert_eq!(resolved.runtime_label, "claude");
    assert_eq!(resolved.state, RuntimeResolutionState::Resolved);
}

#[test]
fn ambiguous_detection_requires_confirmation() {
    let resolved = resolve_runtime(None, DetectionContext::ambiguous())
        .expect("ambiguous detection should yield a pending confirmation");
    assert_eq!(resolved.state, RuntimeResolutionState::WaitingForConfirmation);
    assert_eq!(
        resolved.confirmation_prompt.as_deref(),
        Some("Detected both codex and claude contexts. Use codex? [y/n]")
    );
}

#[test]
fn detection_failure_marks_run_blocked() {
    let error = resolve_runtime(None, DetectionContext::missing()).unwrap_err();
    assert_eq!(error.kind, RuntimeResolutionErrorKind::Blocked);
}
