use crate::{
    domain::{DelegateExecutionCommand, ProtocolError, SubmitWorkCommand},
    runtime,
    store::AppState,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize)]
struct ProtocolErrorEnvelope {
    error: ProtocolErrorBody,
}

#[derive(Clone, Debug, Serialize)]
struct ProtocolErrorBody {
    code: &'static str,
}

#[derive(Clone, Debug, Serialize)]
struct RootResponse {
    implementation: &'static str,
    state: &'static str,
    target_protocol_version: &'static str,
    target_binding: &'static str,
    target_profile: &'static str,
}

impl IntoResponse for ProtocolError {
    fn into_response(self) -> Response {
        let (status, code) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            Self::InvalidState => (StatusCode::CONFLICT, "invalid_state"),
        };

        (
            status,
            Json(ProtocolErrorEnvelope {
                error: ProtocolErrorBody { code },
            }),
        )
            .into_response()
    }
}

#[derive(Clone, Debug, Deserialize)]
struct SubmitWorkRequest {
    input: Value,
    #[serde(default)]
    session_id: Option<String>,
    #[serde(default)]
    references: Vec<Value>,
}

#[derive(Clone, Debug, Deserialize)]
struct DelegateExecutionRequest {
    target_agent_instance_id: String,
    input: Value,
    #[serde(default)]
    references: Vec<Value>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/v1/agent-definitions", get(list_agent_definitions))
        .route(
            "/v1/agent-definitions/{agent_definition_id}",
            get(get_agent_definition),
        )
        .route(
            "/v1/agent-definitions/{agent_definition_id}/disable",
            post(disable_agent_definition),
        )
        .route(
            "/v1/agent-definitions/{agent_definition_id}/enable",
            post(enable_agent_definition),
        )
        .route("/v1/agent-instances", get(list_agent_instances))
        .route(
            "/v1/agent-instances/{agent_instance_id}",
            get(get_agent_instance),
        )
        .route(
            "/v1/agent-instances/{agent_instance_id}/pause",
            post(pause_agent_instance),
        )
        .route(
            "/v1/agent-instances/{agent_instance_id}/resume",
            post(resume_agent_instance),
        )
        .route(
            "/v1/agent-instances/{agent_instance_id}/terminate",
            post(terminate_agent_instance),
        )
        .route(
            "/v1/agent-instances/{agent_instance_id}/submit-work",
            post(submit_work),
        )
        .route("/v1/sessions", get(list_sessions))
        .route("/v1/sessions/{session_id}", get(get_session))
        .route("/v1/sessions/{session_id}/close", post(close_session))
        .route("/v1/executions", get(list_executions))
        .route("/v1/executions/{execution_id}", get(get_execution))
        .route(
            "/v1/executions/{execution_id}/context",
            get(get_execution_context),
        )
        .route(
            "/v1/executions/{execution_id}/cancel",
            post(cancel_execution),
        )
        .route(
            "/v1/executions/{execution_id}/delegate",
            post(delegate_execution),
        )
        .with_state(state)
}

async fn root() -> Json<RootResponse> {
    Json(RootResponse {
        implementation: "starla-rs",
        state: "bootstrap",
        target_protocol_version: "v1",
        target_binding: "HTTP Binding v1",
        target_profile: "Core",
    })
}

async fn list_agent_definitions(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::domain::AgentDefinitionView>>, ProtocolError> {
    Ok(Json(state.list_agent_definitions().await))
}

async fn get_agent_definition(
    Path(agent_definition_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<crate::domain::AgentDefinitionView>, ProtocolError> {
    Ok(Json(
        state.get_agent_definition(&agent_definition_id).await?,
    ))
}

async fn disable_agent_definition(
    Path(agent_definition_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<crate::domain::AgentDefinitionView>, ProtocolError> {
    Ok(Json(
        state.disable_agent_definition(&agent_definition_id).await?,
    ))
}

async fn enable_agent_definition(
    Path(agent_definition_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<crate::domain::AgentDefinitionView>, ProtocolError> {
    Ok(Json(
        state.enable_agent_definition(&agent_definition_id).await?,
    ))
}

async fn list_agent_instances(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::domain::AgentInstanceView>>, ProtocolError> {
    Ok(Json(state.list_agent_instances().await))
}

async fn get_agent_instance(
    Path(agent_instance_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<crate::domain::AgentInstanceView>, ProtocolError> {
    Ok(Json(state.get_agent_instance(&agent_instance_id).await?))
}

async fn pause_agent_instance(
    Path(agent_instance_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<crate::domain::AgentInstanceView>, ProtocolError> {
    Ok(Json(state.pause_agent_instance(&agent_instance_id).await?))
}

async fn resume_agent_instance(
    Path(agent_instance_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<crate::domain::AgentInstanceView>, ProtocolError> {
    Ok(Json(state.resume_agent_instance(&agent_instance_id).await?))
}

async fn terminate_agent_instance(
    Path(agent_instance_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<crate::domain::AgentInstanceView>, ProtocolError> {
    Ok(Json(
        state.terminate_agent_instance(&agent_instance_id).await?,
    ))
}

async fn submit_work(
    Path(agent_instance_id): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<SubmitWorkRequest>,
) -> Result<(StatusCode, Json<crate::domain::SubmitWorkView>), ProtocolError> {
    let (execution_id, view) = state
        .submit_work(
            &agent_instance_id,
            SubmitWorkCommand {
                input: request.input,
                session_id: request.session_id,
                references: request.references,
            },
        )
        .await?;

    runtime::spawn_execution_progress(state, execution_id);
    Ok((StatusCode::CREATED, Json(view)))
}

async fn list_sessions(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::domain::SessionView>>, ProtocolError> {
    Ok(Json(state.list_sessions().await))
}

async fn get_session(
    Path(session_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<crate::domain::SessionView>, ProtocolError> {
    Ok(Json(state.get_session(&session_id).await?))
}

async fn close_session(
    Path(session_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<crate::domain::SessionView>, ProtocolError> {
    Ok(Json(state.close_session(&session_id).await?))
}

async fn list_executions(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::domain::ExecutionListItem>>, ProtocolError> {
    Ok(Json(state.list_executions().await))
}

async fn get_execution(
    Path(execution_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<crate::domain::ExecutionSnapshot>, ProtocolError> {
    Ok(Json(state.get_execution(&execution_id).await?))
}

async fn get_execution_context(
    Path(execution_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<crate::domain::ContextSnapshot>, ProtocolError> {
    Ok(Json(state.get_execution_context(&execution_id).await?))
}

async fn cancel_execution(
    Path(execution_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<crate::domain::ExecutionListItem>, ProtocolError> {
    Ok(Json(state.cancel_execution(&execution_id).await?))
}

async fn delegate_execution(
    Path(execution_id): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<DelegateExecutionRequest>,
) -> Result<(StatusCode, Json<crate::domain::DelegateExecutionView>), ProtocolError> {
    let (child_execution_id, view) = state
        .delegate_execution(
            &execution_id,
            DelegateExecutionCommand {
                target_agent_instance_id: request.target_agent_instance_id,
                input: request.input,
                references: request.references,
            },
        )
        .await?;

    runtime::spawn_execution_progress(state, child_execution_id);
    Ok((StatusCode::CREATED, Json(view)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tokio::time::{Duration, sleep};
    use tower::ServiceExt;

    #[tokio::test]
    async fn submit_work_rejects_paused_instance() {
        let app = router(AppState::seeded());
        let request = Request::builder()
            .method("POST")
            .uri("/v1/agent-instances/agent-inst-paused/submit-work")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"input":{"message":"hi"}}"#))
            .expect("build request");

        let response = app.oneshot(request).await.expect("send request");
        assert_eq!(response.status(), StatusCode::CONFLICT);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("collect body")
            .to_bytes();
        let value: Value = serde_json::from_slice(&body).expect("decode json");
        assert_eq!(value["error"]["code"], "invalid_state");
    }

    #[tokio::test]
    async fn submit_work_with_session_exposes_context_buckets() {
        let state = AppState::seeded();
        let app = router(state.clone());
        let request = Request::builder()
            .method("POST")
            .uri("/v1/agent-instances/agent-inst-primary/submit-work")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{
                    "input":{"message":"hello"},
                    "session_id":"session-open",
                    "references":[{"kind":"doc","id":"doc-1"}]
                }"#,
            ))
            .expect("build request");

        let response = app.oneshot(request).await.expect("send request");
        assert_eq!(response.status(), StatusCode::CREATED);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("collect body")
            .to_bytes();
        let created: Value = serde_json::from_slice(&body).expect("decode json");
        let execution_id = created["execution_id"].as_str().expect("execution id");

        let app = router(state);
        let context_request = Request::builder()
            .method("GET")
            .uri(format!("/v1/executions/{execution_id}/context"))
            .body(Body::empty())
            .expect("build request");

        let context_response = app.oneshot(context_request).await.expect("send request");
        assert_eq!(context_response.status(), StatusCode::OK);

        let context_body = context_response
            .into_body()
            .collect()
            .await
            .expect("collect body")
            .to_bytes();
        let snapshot: Value = serde_json::from_slice(&context_body).expect("decode json");
        assert_eq!(snapshot["explicit_input"]["message"], "hello");
        assert_eq!(snapshot["explicit_references"][0]["id"], "doc-1");
        assert_eq!(snapshot["session_material"]["scope"], "session-open");
    }

    #[tokio::test]
    async fn delegate_execution_rejects_self_target() {
        let app = router(AppState::seeded());
        let delegate_request = Request::builder()
            .method("POST")
            .uri("/v1/executions/execution-running/delegate")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{
                    "target_agent_instance_id":"agent-inst-primary",
                    "input":{"message":"child"}
                }"#,
            ))
            .expect("build request");

        let delegate_response = app.oneshot(delegate_request).await.expect("send request");
        assert_eq!(delegate_response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn failed_execution_inspection_remains_normal_resource_inspection() {
        let app = router(AppState::seeded());
        let request = Request::builder()
            .method("GET")
            .uri("/v1/executions/execution-failed")
            .body(Body::empty())
            .expect("build request");

        let response = app.oneshot(request).await.expect("send request");
        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("collect body")
            .to_bytes();
        let value: Value = serde_json::from_slice(&body).expect("decode json");
        assert_eq!(value["state"], "failed");
    }

    #[tokio::test]
    async fn submit_work_progresses_to_completed_with_terminal_event_order() {
        let state = AppState::seeded();
        let app = router(state.clone());
        let request = Request::builder()
            .method("POST")
            .uri("/v1/agent-instances/agent-inst-primary/submit-work")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"input":{"message":"run"}}"#))
            .expect("build request");

        let response = app.oneshot(request).await.expect("send request");
        assert_eq!(response.status(), StatusCode::CREATED);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("collect body")
            .to_bytes();
        let created: Value = serde_json::from_slice(&body).expect("decode json");
        let execution_id = created["execution_id"].as_str().expect("execution id");

        sleep(Duration::from_millis(80)).await;

        let app = router(state);
        let request = Request::builder()
            .method("GET")
            .uri(format!("/v1/executions/{execution_id}"))
            .body(Body::empty())
            .expect("build request");

        let response = app.oneshot(request).await.expect("send request");
        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("collect body")
            .to_bytes();
        let snapshot: Value = serde_json::from_slice(&body).expect("decode json");
        assert_eq!(snapshot["state"], "completed");
        assert_eq!(snapshot["recent_events"][0]["event"], "execution.created");
        assert_eq!(
            snapshot["recent_events"][1]["event"],
            "execution.state_changed"
        );
        assert_eq!(snapshot["recent_events"][1]["lifecycle_state"], "running");
        assert_eq!(snapshot["recent_events"][2]["event"], "execution.completed");
        assert!(
            snapshot["recent_events"]
                .as_array()
                .expect("events array")
                .len()
                == 3
        );
    }

    #[tokio::test]
    async fn cancel_execution_transitions_seeded_pending_execution() {
        let state = AppState::seeded();
        let app = router(state.clone());
        let request = Request::builder()
            .method("POST")
            .uri("/v1/executions/execution-pending/cancel")
            .body(Body::empty())
            .expect("build request");

        let response = app.oneshot(request).await.expect("send request");
        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("collect body")
            .to_bytes();
        let canceled: Value = serde_json::from_slice(&body).expect("decode json");
        assert_eq!(canceled["state"], "canceled");

        let app = router(state);
        let request = Request::builder()
            .method("GET")
            .uri("/v1/executions/execution-pending")
            .body(Body::empty())
            .expect("build request");

        let response = app.oneshot(request).await.expect("send request");
        let body = response
            .into_body()
            .collect()
            .await
            .expect("collect body")
            .to_bytes();
        let snapshot: Value = serde_json::from_slice(&body).expect("decode json");
        assert_eq!(snapshot["state"], "canceled");
        assert_eq!(
            snapshot["recent_events"]
                .as_array()
                .and_then(|events| events.last())
                .and_then(|event| event.get("event"))
                .and_then(Value::as_str),
            Some("execution.canceled")
        );
    }

    #[tokio::test]
    async fn delegate_execution_success_preserves_parent_target_and_session() {
        let state = AppState::seeded();
        let app = router(state.clone());
        let request = Request::builder()
            .method("POST")
            .uri("/v1/executions/execution-running/delegate")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{
                    "target_agent_instance_id":"agent-inst-helper",
                    "input":{"message":"child","synthetic_outcome":"failed"},
                    "references":[{"kind":"doc","id":"child-ref"}]
                }"#,
            ))
            .expect("build request");

        let response = app.oneshot(request).await.expect("send request");
        assert_eq!(response.status(), StatusCode::CREATED);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("collect body")
            .to_bytes();
        let created: Value = serde_json::from_slice(&body).expect("decode json");
        let execution_id = created["execution_id"].as_str().expect("execution id");
        assert_eq!(created["parent_execution_id"], "execution-running");
        assert_eq!(created["agent_instance_id"], "agent-inst-helper");
        assert_eq!(created["session_id"], "session-open");
        assert_eq!(created["state"], "pending");

        let app = router(state.clone());
        let request = Request::builder()
            .method("GET")
            .uri(format!("/v1/executions/{execution_id}/context"))
            .body(Body::empty())
            .expect("build request");

        let response = app.oneshot(request).await.expect("send request");
        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("collect body")
            .to_bytes();
        let context: Value = serde_json::from_slice(&body).expect("decode json");
        assert_eq!(context["explicit_input"]["message"], "child");
        assert_eq!(context["explicit_references"][0]["id"], "child-ref");
        assert_eq!(context["session_material"]["scope"], "session-open");
        assert_eq!(
            context["inherited_lineage_material"]["parent_execution_id"],
            "execution-running"
        );

        sleep(Duration::from_millis(80)).await;

        let app = router(state);
        let request = Request::builder()
            .method("GET")
            .uri(format!("/v1/executions/{execution_id}"))
            .body(Body::empty())
            .expect("build request");

        let response = app.oneshot(request).await.expect("send request");
        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("collect body")
            .to_bytes();
        let snapshot: Value = serde_json::from_slice(&body).expect("decode json");
        assert_eq!(snapshot["state"], "failed");
        assert_eq!(snapshot["recent_events"][0]["event"], "execution.created");
        assert_eq!(snapshot["recent_events"][1]["event"], "execution.delegated");
        assert_eq!(
            snapshot["recent_events"][2]["event"],
            "execution.state_changed"
        );
        assert_eq!(snapshot["recent_events"][3]["event"], "execution.failed");
    }
}
