# Claim Status

## Current Target

- claim: `Core`
- binding: `HTTP Binding v1`
- protocol seed:
  - `/home/alan/projects/starla-protocol/conformance/v1/claims/core-http-claim-seed.md`
  - `/home/alan/projects/starla-protocol/conformance/v1/reports/core-http-report-seed.md`

## Current Decision

- local status: provisional pass
- basis:
  - route-level black-box integration tests in `tests/core_http_claim.rs`
  - daemon-level black-box claim test in `tests/core_http_blackbox.rs`
  - no standalone external conformance runner yet

## Vectors

- pass `agent-definition-listing-includes-visible-definition.md`
  - `agent_definition_routes_cover_listing_inspection_disable_and_enable`
- pass `agent-definition-inspection-exposes-state.md`
  - `agent_definition_routes_cover_listing_inspection_disable_and_enable`
- pass `disable-agent-definition-transitions-to-disabled.md`
  - `agent_definition_routes_cover_listing_inspection_disable_and_enable`
- pass `enable-agent-definition-transitions-to-enabled.md`
  - `agent_definition_routes_cover_listing_inspection_disable_and_enable`
- pass `agent-instance-listing-includes-visible-instance.md`
  - `agent_instance_routes_cover_listing_inspection_pause_resume_and_terminate`
- pass `agent-instance-inspection-exposes-definition-link-and-state.md`
  - `agent_instance_routes_cover_listing_inspection_pause_resume_and_terminate`
- pass `pause-agent-instance-transitions-to-paused.md`
  - `agent_instance_routes_cover_listing_inspection_pause_resume_and_terminate`
- pass `resume-agent-instance-transitions-to-ready.md`
  - `agent_instance_routes_cover_listing_inspection_pause_resume_and_terminate`
- pass `terminate-agent-instance-transitions-to-terminated.md`
  - `agent_instance_routes_cover_listing_inspection_pause_resume_and_terminate`
- pass `session-listing-includes-visible-session.md`
  - `session_routes_cover_listing_inspection_and_close`
- pass `session-inspection-exposes-state.md`
  - `session_routes_cover_listing_inspection_and_close`
- pass `close-session-transitions-to-closed.md`
  - `session_routes_cover_listing_inspection_and_close`
- pass `submit-work-success.md`
  - `submit_work_success_creates_pending_execution_and_visible_context`
- pass `execution-listing-includes-visible-execution.md`
  - `execution_listing_includes_visible_execution`
- pass `cancel-execution-transitions-to-canceled.md`
  - `cancel_execution_transitions_seeded_pending_execution`
- pass `cancel-execution-rejected-when-already-terminal.md`
  - `cancel_execution_rejected_when_already_terminal`
- pass `submit-work-rejected-when-instance-paused.md`
  - `submit_work_rejected_when_instance_paused`
- pass `delegate-execution-success.md`
  - `delegate_execution_success_preserves_parent_target_and_session`
- pass `delegate-execution-rejected-when-parent-missing.md`
  - `delegate_execution_rejects_missing_or_terminal_parent`
- pass `delegate-execution-rejected-when-parent-terminal.md`
  - `delegate_execution_rejects_missing_or_terminal_parent`
- pass `delegate-execution-rejected-when-target-instance-missing.md`
  - `delegate_execution_rejects_missing_not_ready_and_self_target`
- pass `delegate-execution-rejected-when-target-instance-not-ready.md`
  - `delegate_execution_rejects_missing_not_ready_and_self_target`
- pass `delegate-execution-rejected-when-target-instance-equals-parent-owner.md`
  - `delegate_execution_rejects_missing_not_ready_and_self_target`
- pass `missing-execution-inspection-returns-not-found.md`
  - `missing_execution_inspection_returns_not_found`
- pass `failed-execution-inspection-is-not-transport-error.md`
  - `failed_execution_inspection_remains_normal_resource_inspection`
- pass `context-snapshot-preserves-provenance.md`
  - `submit_work_with_session_exposes_context_buckets`
  - `delegated_child_context_preserves_session_and_lineage_buckets`
- pass `context-snapshot-omits-absent-contribution-sections.md`
  - `context_snapshot_omits_absent_buckets_without_lineage`
- pass `inherited-lineage-material-visible-on-child-execution.md`
  - `delegated_child_context_preserves_session_and_lineage_buckets`
- pass `inherited-lineage-material-omitted-without-visible-lineage.md`
  - `inherited_lineage_material_omitted_without_visible_lineage`
- pass `session-material-visible-on-session-attached-execution.md`
  - `session_material_visible_on_session_attached_execution`
- pass `execution-snapshot-separates-sections.md`
  - `execution_snapshot_separates_context_from_recent_events`

## Traces

- pass `execution-completion-terminal.md`
  - `execution_lifecycle_reaches_terminal_completion_in_order`
- pass `execution-cancel-terminal.md`
  - `cancel_execution_transitions_seeded_pending_execution`
- pass `execution-lifecycle-minimal.md`
  - `execution_lifecycle_reaches_terminal_completion_in_order`
- pass `execution-failure-terminal.md`
  - `execution_failure_terminal_for_failed_synthetic_outcome`
- pass `delegated-execution-minimal.md`
  - `delegate_execution_success_preserves_parent_target_and_session`

## Excluded Optional Surfaces

- not claimed:
  - approvals
  - tools
  - channels
  - stream binding
- no local evidence should be read as proof for excluded surfaces

## Remaining Work

- execute the same claim through a standalone external conformance runner
- record runner identity and run date
- convert provisional pass into a real implementation report
