use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterventionResult {
    Queued,
    Acknowledged,
    Applied,
    Failed,
}

impl InterventionResult {
    pub fn acknowledge(self) -> Option<Self> {
        matches!(self, Self::Queued).then_some(Self::Acknowledged)
    }

    pub fn apply(self) -> Option<Self> {
        matches!(self, Self::Acknowledged).then_some(Self::Applied)
    }

    pub fn fail(self) -> Option<Self> {
        matches!(self, Self::Queued | Self::Acknowledged).then_some(Self::Failed)
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Applied | Self::Failed)
    }
}
