use crate::cli::Runtime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeResolutionState {
    Resolved,
    WaitingForConfirmation,
}

#[derive(Debug, Clone)]
pub struct ResolvedRuntime {
    pub runtime: Runtime,
    pub runtime_label: String,
    pub state: RuntimeResolutionState,
    pub confirmation_prompt: Option<String>,
}

impl ResolvedRuntime {
    pub fn resolved(runtime: Runtime) -> Self {
        let runtime_label = runtime_label(&runtime).to_owned();
        Self {
            runtime,
            runtime_label,
            state: RuntimeResolutionState::Resolved,
            confirmation_prompt: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeResolutionError {
    pub kind: RuntimeResolutionErrorKind,
    pub message: String,
}

impl RuntimeResolutionError {
    pub fn blocked(message: impl Into<String>) -> Self {
        let message = message.into();
        Self {
            kind: RuntimeResolutionErrorKind::Blocked,
            message,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeResolutionErrorKind {
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionContext {
    Codex,
    Claude,
    Ambiguous,
    Missing,
}

impl DetectionContext {
    pub fn codex() -> Self {
        Self::Codex
    }

    pub fn ambiguous() -> Self {
        Self::Ambiguous
    }

    pub fn missing() -> Self {
        Self::Missing
    }

    pub fn from_env() -> Self {
        match std::env::var("PATCHLANE_TEST_RUNTIME_CONTEXT").ok().as_deref() {
            Some("ambiguous") => Self::Ambiguous,
            Some("missing") => Self::Missing,
            Some("claude") => Self::Claude,
            _ => Self::Codex,
        }
    }

    fn resolve(self) -> Result<DetectionResult, RuntimeResolutionError> {
        match self {
            Self::Codex => Ok(DetectionResult::Resolved(Runtime::Codex)),
            Self::Claude => Ok(DetectionResult::Resolved(Runtime::Claude)),
            Self::Ambiguous => Ok(DetectionResult::Ambiguous {
                preferred: Runtime::Codex,
                prompt: "Detected both codex and claude contexts. Use codex? [y/n]".to_owned(),
            }),
            Self::Missing => Ok(DetectionResult::Unavailable(
                "unable to detect runtime context".to_owned(),
            )),
        }
    }
}

enum DetectionResult {
    Resolved(Runtime),
    Ambiguous { preferred: Runtime, prompt: String },
    Unavailable(String),
}

pub fn resolve_runtime(
    explicit: Option<Runtime>,
    detection: DetectionContext,
) -> Result<ResolvedRuntime, RuntimeResolutionError> {
    if let Some(runtime) = explicit {
        return Ok(ResolvedRuntime::resolved(runtime));
    }

    match detection.resolve()? {
        DetectionResult::Resolved(runtime) => Ok(ResolvedRuntime::resolved(runtime)),
        DetectionResult::Ambiguous { preferred, prompt } => {
            let runtime_label = runtime_label(&preferred).to_owned();
            Ok(ResolvedRuntime {
            runtime: preferred,
            runtime_label,
            state: RuntimeResolutionState::WaitingForConfirmation,
            confirmation_prompt: Some(prompt),
        })
        }
        DetectionResult::Unavailable(reason) => Err(RuntimeResolutionError::blocked(reason)),
    }
}

pub fn apply_runtime_confirmation(
    input: &str,
    resolution: ResolvedRuntime,
) -> Result<ResolvedRuntime, RuntimeResolutionError> {
    match input.trim() {
        "y" | "Y" => Ok(ResolvedRuntime::resolved(resolution.runtime)),
        "n" | "N" => Err(RuntimeResolutionError::blocked(
            "runtime confirmation rejected by user",
        )),
        _ => Err(RuntimeResolutionError::blocked(
            "runtime confirmation requires y or n",
        )),
    }
}

fn runtime_label(runtime: &Runtime) -> &'static str {
    match runtime {
        Runtime::Codex => "codex",
        Runtime::Claude => "claude",
    }
}
