use crate::events::run_events::EventLine;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuperpowersStage {
    ClarifyingObjective,
    DraftingDesign,
    WritingPlan,
    SplittingAssignments,
    DispatchingShards,
    ReviewingOutputs,
    MergingCleanShards,
}

pub fn cli_label_for_stage(stage: SuperpowersStage) -> &'static str {
    match stage {
        SuperpowersStage::ClarifyingObjective => "clarifying objective",
        SuperpowersStage::DraftingDesign => "drafting design",
        SuperpowersStage::WritingPlan => "writing plan",
        SuperpowersStage::SplittingAssignments => "splitting assignments",
        SuperpowersStage::DispatchingShards => "dispatching shards",
        SuperpowersStage::ReviewingOutputs => "reviewing outputs",
        SuperpowersStage::MergingCleanShards => "merging clean shards",
    }
}

pub fn fixture_stage_event_lines() -> Vec<EventLine> {
    [
        (
            "2026-03-09T10:00:00Z",
            SuperpowersStage::ClarifyingObjective,
        ),
        ("2026-03-09T10:02:00Z", SuperpowersStage::DraftingDesign),
        ("2026-03-09T10:05:00Z", SuperpowersStage::WritingPlan),
        (
            "2026-03-09T10:09:00Z",
            SuperpowersStage::SplittingAssignments,
        ),
        ("2026-03-09T10:15:00Z", SuperpowersStage::DispatchingShards),
        ("2026-03-09T10:18:00Z", SuperpowersStage::ReviewingOutputs),
        ("2026-03-09T10:21:00Z", SuperpowersStage::MergingCleanShards),
    ]
    .into_iter()
    .map(|(timestamp, stage)| EventLine {
        timestamp,
        message: format!("workflow stage: {}", cli_label_for_stage(stage)),
    })
    .collect()
}
