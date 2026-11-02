use crate::data::ResearchStatus;
use dbos::sysdb::types::WorkflowStatus;

#[test]
fn research_status_preserves_workflow_meaning() {
    for (stored, expected) in [
        (WorkflowStatus::Enqueued, "queued"),
        (WorkflowStatus::Delayed, "queued"),
        (WorkflowStatus::Pending, "running"),
        (WorkflowStatus::Success, "completed"),
        (WorkflowStatus::Error, "failed"),
        (WorkflowStatus::Cancelled, "cancelled"),
        (WorkflowStatus::MaxRecoveryAttemptsExceeded, "failed"),
    ] {
        let actual = serde_json::to_value(ResearchStatus::from_workflow_status(stored)).unwrap();
        assert_eq!(
            actual, expected,
            "API state misrepresents a recorded workflow: {stored:?}"
        );
    }
    assert_eq!(
        serde_json::to_value(ResearchStatus::NotSubmitted).unwrap(),
        "not_submitted",
        "Missing-workflow status changes its public spelling"
    );
}
