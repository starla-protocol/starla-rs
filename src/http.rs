use crate::{
    domain::{DelegateExecutionCommand, ProtocolError, SubmitWorkCommand},
    runtime,
    store::AppState,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
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
            Self::IdempotencyConflict => (StatusCode::CONFLICT, "idempotency_conflict"),
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
        state: "early_implementation",
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
    headers: HeaderMap,
    Json(request): Json<SubmitWorkRequest>,
) -> Result<(StatusCode, Json<crate::domain::SubmitWorkView>), ProtocolError> {
    let idempotency_key = headers
        .get("Idempotency-Key")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);

    let outcome = state
        .submit_work(
            &agent_instance_id,
            SubmitWorkCommand {
                input: request.input,
                session_id: request.session_id,
                references: request.references,
                idempotency_key,
            },
        )
        .await?;

    if outcome.created {
        runtime::spawn_execution_progress(state, outcome.view.execution_id.clone());
    }

    let status = if outcome.created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((status, Json(outcome.view)))
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
