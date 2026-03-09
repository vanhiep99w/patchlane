use patchlane::workflow::superpowers_contract::{
    cli_label_for_stage, fixture_stage_event_lines, SuperpowersStage,
};

#[test]
fn superpowers_contract_maps_internal_stages_to_stable_cli_labels() {
    let cases = [
        (
            SuperpowersStage::ClarifyingObjective,
            "clarifying objective",
        ),
        (SuperpowersStage::DraftingDesign, "drafting design"),
        (SuperpowersStage::WritingPlan, "writing plan"),
        (
            SuperpowersStage::SplittingAssignments,
            "splitting assignments",
        ),
        (SuperpowersStage::DispatchingShards, "dispatching shards"),
        (SuperpowersStage::ReviewingOutputs, "reviewing outputs"),
        (SuperpowersStage::MergingCleanShards, "merging clean shards"),
    ];

    for (stage, expected) in cases {
        assert_eq!(cli_label_for_stage(stage), expected);
    }
}

#[test]
fn superpowers_contract_emits_cli_facing_watch_events_without_skill_names() {
    let events = fixture_stage_event_lines();

    let labels = events
        .iter()
        .map(|event| event.message.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        labels,
        vec![
            "workflow stage: clarifying objective",
            "workflow stage: drafting design",
            "workflow stage: writing plan",
            "workflow stage: splitting assignments",
            "workflow stage: dispatching shards",
            "workflow stage: reviewing outputs",
            "workflow stage: merging clean shards",
        ]
    );

    for event in events {
        assert!(
            !event.message.contains("skill"),
            "watch event should not expose raw skill internals"
        );
        assert!(
            !event.message.contains("brainstorming"),
            "watch event should not expose raw skill names"
        );
        assert!(
            !event.message.contains("superpowers"),
            "watch event should not expose adapter internals"
        );
    }
}
