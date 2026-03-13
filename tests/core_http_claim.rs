use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use starla_rs::{http, store::AppState};
use tokio::time::{Duration, sleep};
use tower::ServiceExt;

async fn send(
    state: &AppState,
    method: Method,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut request = Request::builder().method(method).uri(uri);
    let body = match body {
        Some(body) => {
            request = request.header("content-type", "application/json");
            Body::from(body.to_string())
        }
        None => Body::empty(),
    };

    let response = http::router(state.clone())
        .oneshot(request.body(body).expect("build request"))
        .await
        .expect("send request");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();

    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).expect("decode json")
    };

    (status, value)
}

fn assert_error_code(body: &Value, code: &str) {
    assert_eq!(body["error"]["code"], code);
}

#[tokio::test]
async fn agent_definition_routes_cover_listing_inspection_disable_and_enable() {
    let state = AppState::seeded();

    let (status, body) = send(&state, Method::GET, "/v1/agent-definitions", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.as_array()
            .expect("list array")
            .iter()
            .any(|item| item["agent_definition_id"] == "agent-def-enabled")
    );

    let (status, body) = send(
        &state,
        Method::GET,
        "/v1/agent-definitions/agent-def-enabled",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "enabled");

    let (status, body) = send(
        &state,
        Method::POST,
        "/v1/agent-definitions/agent-def-enabled/disable",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "disabled");

    let (status, body) = send(
        &state,
        Method::POST,
        "/v1/agent-definitions/agent-def-enabled/enable",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "enabled");
}

#[tokio::test]
async fn agent_instance_routes_cover_listing_inspection_pause_resume_and_terminate() {
    let state = AppState::seeded();

    let (status, body) = send(&state, Method::GET, "/v1/agent-instances", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.as_array()
            .expect("list array")
            .iter()
            .any(|item| item["agent_instance_id"] == "agent-inst-primary")
    );

    let (status, body) = send(
        &state,
        Method::GET,
        "/v1/agent-instances/agent-inst-primary",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["agent_definition_id"], "agent-def-enabled");
    assert_eq!(body["state"], "ready");

    let (status, body) = send(
        &state,
        Method::POST,
        "/v1/agent-instances/agent-inst-primary/pause",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "paused");

    let (status, body) = send(
        &state,
        Method::POST,
        "/v1/agent-instances/agent-inst-primary/resume",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "ready");

    let (status, body) = send(
        &state,
        Method::POST,
        "/v1/agent-instances/agent-inst-primary/terminate",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "terminated");
}

#[tokio::test]
async fn session_routes_cover_listing_inspection_and_close() {
    let state = AppState::seeded();

    let (status, body) = send(&state, Method::GET, "/v1/sessions", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.as_array()
            .expect("list array")
            .iter()
            .any(|item| item["session_id"] == "session-open")
    );

    let (status, body) = send(&state, Method::GET, "/v1/sessions/session-open", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "open");

    let (status, body) = send(
        &state,
        Method::POST,
        "/v1/sessions/session-open/close",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "closed");
}

#[tokio::test]
async fn submit_work_success_creates_pending_execution_and_visible_context() {
    let state = AppState::seeded();
    let (status, body) = send(
        &state,
        Method::POST,
        "/v1/agent-instances/agent-inst-primary/submit-work",
        Some(json!({
            "input": {"message": "hello"},
            "references": [{"kind": "doc", "id": "doc-1"}]
        })),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["state"], "pending");
    assert!(body.get("approval_ids").is_none());
    let execution_id = body["execution_id"].as_str().expect("execution id");

    let (status, list_body) = send(&state, Method::GET, "/v1/executions", None).await;
    assert_eq!(status, StatusCode::OK);
    let matching: Vec<_> = list_body
        .as_array()
        .expect("execution list")
        .iter()
        .filter(|item| item["execution_id"] == execution_id)
        .collect();
    assert_eq!(matching.len(), 1);
    assert_eq!(matching[0]["state"], "pending");

    let (status, context_body) = send(
        &state,
        Method::GET,
        &format!("/v1/executions/{execution_id}/context"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(context_body["execution_id"], execution_id);
    assert_eq!(context_body["agent_instance_id"], "agent-inst-primary");
    assert_eq!(context_body["explicit_input"]["message"], "hello");
    assert_eq!(context_body["explicit_references"][0]["id"], "doc-1");
}

#[tokio::test]
async fn execution_listing_includes_visible_execution() {
    let state = AppState::seeded();
    let (status, body) = send(&state, Method::GET, "/v1/executions", None).await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        body.as_array()
            .expect("execution list")
            .iter()
            .any(|item| item["execution_id"] == "execution-running")
    );
}

#[tokio::test]
async fn submit_work_rejected_when_instance_paused() {
    let state = AppState::seeded();
    let (status, body) = send(
        &state,
        Method::POST,
        "/v1/agent-instances/agent-inst-paused/submit-work",
        Some(json!({
            "input": {"message": "hi"}
        })),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_error_code(&body, "invalid_state");
}

#[tokio::test]
async fn submit_work_with_session_exposes_context_buckets() {
    let state = AppState::seeded();
    let (status, body) = send(
        &state,
        Method::POST,
        "/v1/agent-instances/agent-inst-primary/submit-work",
        Some(json!({
            "input": {"message": "hello"},
            "session_id": "session-open",
            "references": [{"kind": "doc", "id": "doc-1"}]
        })),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let execution_id = body["execution_id"].as_str().expect("execution id");

    let (status, context_body) = send(
        &state,
        Method::GET,
        &format!("/v1/executions/{execution_id}/context"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(context_body["explicit_input"]["message"], "hello");
    assert_eq!(context_body["explicit_references"][0]["id"], "doc-1");
    assert_eq!(context_body["session_material"]["scope"], "session-open");
}

#[tokio::test]
async fn cancel_execution_rejected_when_already_terminal() {
    let state = AppState::seeded();
    let (status, body) = send(
        &state,
        Method::POST,
        "/v1/executions/execution-completed/cancel",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_error_code(&body, "invalid_state");
}

#[tokio::test]
async fn cancel_execution_transitions_seeded_pending_execution() {
    let state = AppState::seeded();
    let (status, body) = send(
        &state,
        Method::POST,
        "/v1/executions/execution-pending/cancel",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "canceled");

    let (status, snapshot_body) = send(
        &state,
        Method::GET,
        "/v1/executions/execution-pending",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(snapshot_body["state"], "canceled");
    assert_eq!(
        snapshot_body["recent_events"]
            .as_array()
            .and_then(|events| events.last())
            .and_then(|event| event.get("event"))
            .and_then(Value::as_str),
        Some("execution.canceled")
    );
}

#[tokio::test]
async fn delegate_execution_rejects_missing_or_terminal_parent() {
    let state = AppState::seeded();

    let (status, body) = send(
        &state,
        Method::POST,
        "/v1/executions/missing-parent/delegate",
        Some(json!({
            "target_agent_instance_id": "agent-inst-helper",
            "input": {"message": "child"}
        })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error_code(&body, "not_found");

    let (status, body) = send(
        &state,
        Method::POST,
        "/v1/executions/execution-completed/delegate",
        Some(json!({
            "target_agent_instance_id": "agent-inst-helper",
            "input": {"message": "child"}
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error_code(&body, "invalid_state");
}

#[tokio::test]
async fn delegate_execution_rejects_missing_not_ready_and_self_target() {
    let state = AppState::seeded();

    let (status, body) = send(
        &state,
        Method::POST,
        "/v1/executions/execution-running/delegate",
        Some(json!({
            "target_agent_instance_id": "missing-target",
            "input": {"message": "child"}
        })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error_code(&body, "not_found");

    let (status, body) = send(
        &state,
        Method::POST,
        "/v1/executions/execution-running/delegate",
        Some(json!({
            "target_agent_instance_id": "agent-inst-paused",
            "input": {"message": "child"}
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error_code(&body, "invalid_state");

    let (status, body) = send(
        &state,
        Method::POST,
        "/v1/executions/execution-running/delegate",
        Some(json!({
            "target_agent_instance_id": "agent-inst-primary",
            "input": {"message": "child"}
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error_code(&body, "invalid_state");
}

#[tokio::test]
async fn delegate_execution_success_preserves_parent_target_and_session() {
    let state = AppState::seeded();
    let (status, body) = send(
        &state,
        Method::POST,
        "/v1/executions/execution-running/delegate",
        Some(json!({
            "target_agent_instance_id": "agent-inst-helper",
            "input": {"message": "child", "synthetic_outcome": "failed"},
            "references": [{"kind": "doc", "id": "child-ref"}]
        })),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let execution_id = body["execution_id"].as_str().expect("execution id");
    assert_eq!(body["parent_execution_id"], "execution-running");
    assert_eq!(body["agent_instance_id"], "agent-inst-helper");
    assert_eq!(body["session_id"], "session-open");
    assert_eq!(body["state"], "pending");

    sleep(Duration::from_millis(80)).await;

    let (status, snapshot_body) = send(
        &state,
        Method::GET,
        &format!("/v1/executions/{execution_id}"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(snapshot_body["state"], "failed");
    assert_eq!(
        snapshot_body["recent_events"][0]["event"],
        "execution.created"
    );
    assert_eq!(
        snapshot_body["recent_events"][1]["event"],
        "execution.delegated"
    );
    assert_eq!(
        snapshot_body["recent_events"][2]["event"],
        "execution.state_changed"
    );
    assert_eq!(
        snapshot_body["recent_events"][3]["event"],
        "execution.failed"
    );
}

#[tokio::test]
async fn missing_execution_inspection_returns_not_found() {
    let state = AppState::seeded();
    let (status, body) = send(&state, Method::GET, "/v1/executions/missing", None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error_code(&body, "not_found");
}

#[tokio::test]
async fn failed_execution_inspection_remains_normal_resource_inspection() {
    let state = AppState::seeded();
    let (status, body) = send(&state, Method::GET, "/v1/executions/execution-failed", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "failed");
}

#[tokio::test]
async fn context_snapshot_omits_absent_buckets_without_lineage() {
    let state = AppState::seeded();
    let (status, body) = send(
        &state,
        Method::POST,
        "/v1/agent-instances/agent-inst-primary/submit-work",
        Some(json!({
            "input": {"message": "hello"}
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let execution_id = body["execution_id"].as_str().expect("execution id");

    let (status, context_body) = send(
        &state,
        Method::GET,
        &format!("/v1/executions/{execution_id}/context"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(context_body["explicit_input"]["message"], "hello");
    assert_eq!(context_body["explicit_references"], json!([]));
    assert!(context_body.get("session_material").is_none());
    assert!(context_body.get("inherited_lineage_material").is_none());
    assert!(context_body.get("implementation_supplied").is_none());
}

#[tokio::test]
async fn session_material_visible_on_session_attached_execution() {
    let state = AppState::seeded();
    let (status, body) = send(
        &state,
        Method::POST,
        "/v1/agent-instances/agent-inst-primary/submit-work",
        Some(json!({
            "input": {"message": "hello"},
            "session_id": "session-open"
        })),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let execution_id = body["execution_id"].as_str().expect("execution id");

    let (status, context_body) = send(
        &state,
        Method::GET,
        &format!("/v1/executions/{execution_id}/context"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(context_body["session_material"]["scope"], "session-open");
    assert!(context_body.get("inherited_lineage_material").is_none());
}

#[tokio::test]
async fn delegated_child_context_preserves_session_and_lineage_buckets() {
    let state = AppState::seeded();
    let (status, body) = send(
        &state,
        Method::POST,
        "/v1/executions/execution-running/delegate",
        Some(json!({
            "target_agent_instance_id": "agent-inst-helper",
            "input": {"message": "child"},
            "references": [{"kind": "doc", "id": "child-ref"}]
        })),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let execution_id = body["execution_id"].as_str().expect("execution id");
    assert_eq!(body["parent_execution_id"], "execution-running");
    assert_eq!(body["agent_instance_id"], "agent-inst-helper");
    assert_eq!(body["session_id"], "session-open");

    let (status, context_body) = send(
        &state,
        Method::GET,
        &format!("/v1/executions/{execution_id}/context"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(context_body["explicit_input"]["message"], "child");
    assert_eq!(context_body["explicit_references"][0]["id"], "child-ref");
    assert_eq!(context_body["session_material"]["scope"], "session-open");
    assert_eq!(
        context_body["inherited_lineage_material"]["parent_execution_id"],
        "execution-running"
    );
    assert_eq!(
        context_body["inherited_lineage_material"]["parent_explicit_input"]["seed"],
        "running"
    );
}

#[tokio::test]
async fn inherited_lineage_material_omitted_without_visible_lineage() {
    let state = AppState::seeded();
    let (status, body) = send(
        &state,
        Method::POST,
        "/v1/agent-instances/agent-inst-primary/submit-work",
        Some(json!({
            "input": {"message": "standalone"}
        })),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let execution_id = body["execution_id"].as_str().expect("execution id");

    let (status, context_body) = send(
        &state,
        Method::GET,
        &format!("/v1/executions/{execution_id}/context"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(context_body.get("inherited_lineage_material").is_none());
}

#[tokio::test]
async fn execution_snapshot_separates_context_from_recent_events() {
    let state = AppState::seeded();
    let (status, body) = send(
        &state,
        Method::GET,
        "/v1/executions/execution-running",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["execution_id"], "execution-running");
    assert_eq!(body["state"], "running");
    assert_eq!(body["agent_instance_id"], "agent-inst-primary");
    assert_eq!(body["context"]["execution_id"], "execution-running");
    assert!(body["recent_events"].is_array());
    assert!(body["context"].get("recent_events").is_none());
}

#[tokio::test]
async fn execution_failure_terminal_for_failed_synthetic_outcome() {
    let state = AppState::seeded();
    let (status, body) = send(
        &state,
        Method::POST,
        "/v1/agent-instances/agent-inst-primary/submit-work",
        Some(json!({
            "input": {"message": "run", "synthetic_outcome": "failed"}
        })),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let execution_id = body["execution_id"].as_str().expect("execution id");

    sleep(Duration::from_millis(80)).await;

    let (status, body) = send(
        &state,
        Method::GET,
        &format!("/v1/executions/{execution_id}"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "failed");
    assert_eq!(body["recent_events"][0]["event"], "execution.created");
    assert_eq!(body["recent_events"][1]["event"], "execution.state_changed");
    assert_eq!(body["recent_events"][2]["event"], "execution.failed");
    assert_eq!(body["recent_events"].as_array().expect("events").len(), 3);
}

#[tokio::test]
async fn execution_lifecycle_reaches_terminal_completion_in_order() {
    let state = AppState::seeded();
    let (status, body) = send(
        &state,
        Method::POST,
        "/v1/agent-instances/agent-inst-primary/submit-work",
        Some(json!({
            "input": {"message": "run"}
        })),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let execution_id = body["execution_id"].as_str().expect("execution id");

    sleep(Duration::from_millis(80)).await;

    let (status, body) = send(
        &state,
        Method::GET,
        &format!("/v1/executions/{execution_id}"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "completed");
    assert_eq!(body["recent_events"][0]["event"], "execution.created");
    assert_eq!(body["recent_events"][1]["event"], "execution.state_changed");
    assert_eq!(body["recent_events"][1]["lifecycle_state"], "running");
    assert_eq!(body["recent_events"][2]["event"], "execution.completed");
    assert_eq!(body["recent_events"].as_array().expect("events").len(), 3);
}
