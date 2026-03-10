#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedShard {
    pub id: &'static str,
    pub label: &'static str,
    pub brief: String,
}

pub fn plan_shards(objective: &str) -> Vec<PlannedShard> {
    [
        (
            "01",
            "analyze",
            "Analyze the objective and surface delivery risks",
        ),
        (
            "02",
            "implement",
            "Implement the primary code path for the objective",
        ),
        (
            "03",
            "verify",
            "Verify behavior with focused tests and execution checks",
        ),
        (
            "04",
            "integrate",
            "Integrate worker outputs into one operator-facing result",
        ),
    ]
    .into_iter()
    .map(|(id, label, summary)| PlannedShard {
        id,
        label,
        brief: format!("{summary}. Objective: {objective}"),
    })
    .collect()
}
