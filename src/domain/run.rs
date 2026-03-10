use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunState {
    Queued,
    Running,
    Paused,
    Succeeded,
    Failed,
    Stopped,
}

impl RunState {
    pub fn start(self) -> Option<Self> {
        matches!(self, Self::Queued).then_some(Self::Running)
    }

    pub fn pause(self) -> Option<Self> {
        matches!(self, Self::Running).then_some(Self::Paused)
    }

    pub fn resume(self) -> Option<Self> {
        matches!(self, Self::Paused).then_some(Self::Running)
    }

    pub fn succeed(self) -> Option<Self> {
        matches!(self, Self::Running).then_some(Self::Succeeded)
    }

    pub fn fail(self) -> Option<Self> {
        matches!(self, Self::Running).then_some(Self::Failed)
    }

    pub fn stop(self) -> Option<Self> {
        matches!(self, Self::Running | Self::Paused).then_some(Self::Stopped)
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Stopped)
    }
}
