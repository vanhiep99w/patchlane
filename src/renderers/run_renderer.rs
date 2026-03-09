pub struct RunOpeningBlock {
    objective: String,
}

impl RunOpeningBlock {
    pub fn new(objective: String) -> Self {
        Self { objective }
    }
}

pub fn render_opening_block(opening: &RunOpeningBlock) -> String {
    format!(
        "Run\n  queued\n\nObjective\n  {objective}\n\nPlan\n  1. Capture the requested objective.\n  2. Prepare a placeholder execution plan.\n\nPlacement\n  pending placement decision\n\nNext\n  waiting for planner and runtime integration",
        objective = opening.objective
    )
}
