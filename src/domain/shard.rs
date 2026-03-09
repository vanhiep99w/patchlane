use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShardState {
    Queued,
    Assigned,
    Running,
    Succeeded,
    Failed,
    Blocked,
}

impl ShardState {
    pub fn assign(self) -> Option<Self> {
        matches!(self, Self::Queued).then_some(Self::Assigned)
    }

    pub fn start(self) -> Option<Self> {
        matches!(self, Self::Assigned).then_some(Self::Running)
    }

    pub fn succeed(self) -> Option<Self> {
        matches!(self, Self::Running).then_some(Self::Succeeded)
    }

    pub fn fail(self) -> Option<Self> {
        matches!(self, Self::Assigned | Self::Running).then_some(Self::Failed)
    }

    pub fn block(self) -> Option<Self> {
        matches!(self, Self::Assigned | Self::Running).then_some(Self::Blocked)
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Blocked)
    }
}
