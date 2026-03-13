use portpicker::pick_unused_port;
use reqwest::{Client, StatusCode};
use serde_json::{Value, json};
use std::{
    process::{Child, Command, Stdio},
    time::Duration,
};
use tokio::time::sleep;

struct RunningServer {
    child: Child,
    client: Client,
    base_url: String,
}

impl RunningServer {
    async fn start() -> Self {
        let port = pick_unused_port().expect("pick unused port");
        let base_url = format!("http://127.0.0.1:{port}");
        let child = Command::new(env!("CARGO_BIN_EXE_starla-rs"))
            .env("STARLA_BIND_ADDR", format!("127.0.0.1:{port}"))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn starla-rs");

        let server = Self {
            child,
            client: Client::new(),
            base_url,
        };

        server.wait_until_ready().await;
        server
    }

    async fn wait_until_ready(&self) {
        for _ in 0..50 {
            if let Ok(response) = self.client.get(self.url("/")).send().await {
                if response.status() == StatusCode::OK {
                    return;
                }
            }

            sleep(Duration::from_millis(50)).await;
        }

        panic!("starla-rs did not become ready");
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn get(&self, path: &str) -> (StatusCode, Value) {
        let response = self
            .client
            .get(self.url(path))
            .send()
            .await
            .expect("send GET request");
        let status = response.status();
        let body = response.json::<Value>().await.expect("decode GET json");
        (status, body)
    }

    async fn post(&self, path: &str, body: Option<Value>) -> (StatusCode, Value) {
        self.post_with_idempotency(path, body, None).await
    }

    async fn post_with_idempotency(
        &self,
        path: &str,
        body: Option<Value>,
        idempotency_key: Option<&str>,
    ) -> (StatusCode, Value) {
        let request = self.client.post(self.url(path));
        let request = match idempotency_key {
            Some(idempotency_key) => request.header("Idempotency-Key", idempotency_key),
            None => request,
        };
        let request = match body {
            Some(body) => request.json(&body),
            None => request,
        };
        let response = request.send().await.expect("send POST request");
        let status = response.status();
        let body = response.json::<Value>().await.expect("decode POST json");
        (status, body)
    }
}

impl Drop for RunningServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn assert_error_code(body: &Value, code: &str) {
    assert_eq!(body["error"]["code"], code);
}

#[tokio::test]
async fn core_http_claim_seed_passes_over_running_daemon() {
    let server = RunningServer::start().await;
    agent_definition_vectors(&server).await;
    drop(server);

    let server = RunningServer::start().await;
    agent_instance_vectors(&server).await;
    drop(server);

    let server = RunningServer::start().await;
    session_vectors(&server).await;
    drop(server);

    let server = RunningServer::start().await;
    submit_work_vectors(&server).await;
    drop(server);

    let server = RunningServer::start().await;
    execution_vectors(&server).await;
    drop(server);

    let server = RunningServer::start().await;
    context_vectors(&server).await;
    drop(server);

    let server = RunningServer::start().await;
    delegation_vectors(&server).await;
}

async fn agent_definition_vectors(server: &RunningServer) {
    let (status, body) = server.get("/v1/agent-definitions").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.as_array()
            .expect("definition list")
            .iter()
            .any(|item| item["agent_definition_id"] == "agent-def-enabled")
    );

    let (status, body) = server.get("/v1/agent-definitions/agent-def-enabled").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "enabled");

    let (status, body) = server
        .post("/v1/agent-definitions/agent-def-enabled/disable", None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "disabled");

    let (status, body) = server
        .post("/v1/agent-definitions/agent-def-enabled/enable", None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "enabled");
}

async fn agent_instance_vectors(server: &RunningServer) {
    let (status, body) = server.get("/v1/agent-instances").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.as_array()
            .expect("instance list")
            .iter()
            .any(|item| item["agent_instance_id"] == "agent-inst-primary")
    );

    let (status, body) = server.get("/v1/agent-instances/agent-inst-primary").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["agent_definition_id"], "agent-def-enabled");
    assert_eq!(body["state"], "ready");

    let (status, body) = server
        .post("/v1/agent-instances/agent-inst-primary/pause", None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "paused");

    let (status, body) = server
        .post("/v1/agent-instances/agent-inst-primary/resume", None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "ready");

    let (status, body) = server
        .post("/v1/agent-instances/agent-inst-primary/terminate", None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "terminated");
}

async fn session_vectors(server: &RunningServer) {
    let (status, body) = server.get("/v1/sessions").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.as_array()
            .expect("session list")
            .iter()
            .any(|item| item["session_id"] == "session-open")
    );

    let (status, body) = server.get("/v1/sessions/session-open").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "open");

    let (status, body) = server.post("/v1/sessions/session-open/close", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "closed");
}

async fn submit_work_vectors(server: &RunningServer) {
    let (status, body) = server
        .post(
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

    let (status, body) = server
        .post(
            "/v1/agent-instances/agent-inst-paused/submit-work",
            Some(json!({
                "input": {"message": "hi"}
            })),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error_code(&body, "invalid_state");

    let request_body = json!({
        "input": {"message": "idempotent"},
        "references": [{"kind": "doc", "id": "doc-1"}]
    });

    let (status, body) = server
        .post_with_idempotency(
            "/v1/agent-instances/agent-inst-primary/submit-work",
            Some(request_body.clone()),
            Some("submit-1"),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let execution_id = body["execution_id"].as_str().expect("execution id");

    let (status, body) = server
        .post_with_idempotency(
            "/v1/agent-instances/agent-inst-primary/submit-work",
            Some(request_body),
            Some("submit-1"),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["execution_id"], execution_id);

    let (status, body) = server
        .post_with_idempotency(
            "/v1/agent-instances/agent-inst-primary/submit-work",
            Some(json!({
                "input": {"message": "first"}
            })),
            Some("submit-2"),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert!(body["execution_id"].is_string());

    let (status, body) = server
        .post_with_idempotency(
            "/v1/agent-instances/agent-inst-primary/submit-work",
            Some(json!({
                "input": {"message": "second"}
            })),
            Some("submit-2"),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error_code(&body, "idempotency_conflict");
}

async fn execution_vectors(server: &RunningServer) {
    let (status, body) = server.get("/v1/executions").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.as_array()
            .expect("execution list")
            .iter()
            .any(|item| item["execution_id"] == "execution-running")
    );

    let (status, body) = server
        .post("/v1/executions/execution-pending/cancel", None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "canceled");

    let (status, body) = server
        .post("/v1/executions/execution-completed/cancel", None)
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error_code(&body, "invalid_state");

    let (status, body) = server.get("/v1/executions/missing").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error_code(&body, "not_found");

    let (status, body) = server.get("/v1/executions/execution-failed").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "failed");

    let (status, body) = server
        .post(
            "/v1/agent-instances/agent-inst-primary/submit-work",
            Some(json!({
                "input": {"message": "run"}
            })),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let execution_id = body["execution_id"].as_str().expect("execution id");

    sleep(Duration::from_millis(80)).await;

    let (status, body) = server.get(&format!("/v1/executions/{execution_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "completed");
    assert_eq!(body["recent_events"][0]["event"], "execution.created");
    assert_eq!(body["recent_events"][1]["event"], "execution.state_changed");
    assert_eq!(body["recent_events"][1]["lifecycle_state"], "running");
    assert_eq!(body["recent_events"][2]["event"], "execution.completed");

    let (status, body) = server
        .post(
            "/v1/agent-instances/agent-inst-primary/submit-work",
            Some(json!({
                "input": {"message": "run", "synthetic_outcome": "failed"}
            })),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let failed_execution_id = body["execution_id"].as_str().expect("failed execution id");

    sleep(Duration::from_millis(80)).await;

    let (status, body) = server
        .get(&format!("/v1/executions/{failed_execution_id}"))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "failed");
    assert_eq!(body["recent_events"][2]["event"], "execution.failed");
}

async fn context_vectors(server: &RunningServer) {
    let (status, body) = server
        .post(
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

    let (status, body) = server
        .get(&format!("/v1/executions/{execution_id}/context"))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["execution_id"], execution_id);
    assert_eq!(body["agent_instance_id"], "agent-inst-primary");
    assert_eq!(body["explicit_input"]["message"], "hello");
    assert_eq!(body["explicit_references"][0]["id"], "doc-1");
    assert_eq!(body["session_material"]["scope"], "session-open");
    assert!(body.get("inherited_lineage_material").is_none());
    assert!(body.get("implementation_supplied").is_none());

    let (status, body) = server
        .post(
            "/v1/agent-instances/agent-inst-primary/submit-work",
            Some(json!({
                "input": {"message": "standalone"}
            })),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let standalone_execution_id = body["execution_id"]
        .as_str()
        .expect("standalone execution id");

    let (status, body) = server
        .get(&format!("/v1/executions/{standalone_execution_id}/context"))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["explicit_references"], json!([]));
    assert!(body.get("session_material").is_none());
    assert!(body.get("inherited_lineage_material").is_none());

    let (status, body) = server.get("/v1/executions/execution-running").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["execution_id"], "execution-running");
    assert!(body["recent_events"].is_array());
    assert!(body["context"].get("recent_events").is_none());
}

async fn delegation_vectors(server: &RunningServer) {
    let (status, body) = server
        .post(
            "/v1/executions/missing-parent/delegate",
            Some(json!({
                "target_agent_instance_id": "agent-inst-helper",
                "input": {"message": "child"}
            })),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error_code(&body, "not_found");

    let (status, body) = server
        .post(
            "/v1/executions/execution-completed/delegate",
            Some(json!({
                "target_agent_instance_id": "agent-inst-helper",
                "input": {"message": "child"}
            })),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error_code(&body, "invalid_state");

    let (status, body) = server
        .post(
            "/v1/executions/execution-running/delegate",
            Some(json!({
                "target_agent_instance_id": "missing-target",
                "input": {"message": "child"}
            })),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error_code(&body, "not_found");

    let (status, body) = server
        .post(
            "/v1/executions/execution-running/delegate",
            Some(json!({
                "target_agent_instance_id": "agent-inst-paused",
                "input": {"message": "child"}
            })),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error_code(&body, "invalid_state");

    let (status, body) = server
        .post(
            "/v1/executions/execution-running/delegate",
            Some(json!({
                "target_agent_instance_id": "agent-inst-primary",
                "input": {"message": "child"}
            })),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error_code(&body, "invalid_state");

    let (status, body) = server
        .post(
            "/v1/executions/execution-running/delegate",
            Some(json!({
                "target_agent_instance_id": "agent-inst-helper",
                "input": {"message": "child", "synthetic_outcome": "failed"},
                "references": [{"kind": "doc", "id": "child-ref"}]
            })),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let execution_id = body["execution_id"]
        .as_str()
        .expect("delegated execution id");
    assert_eq!(body["parent_execution_id"], "execution-running");
    assert_eq!(body["agent_instance_id"], "agent-inst-helper");
    assert_eq!(body["session_id"], "session-open");

    let (status, body) = server
        .get(&format!("/v1/executions/{execution_id}/context"))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["explicit_input"]["message"], "child");
    assert_eq!(body["explicit_references"][0]["id"], "child-ref");
    assert_eq!(body["session_material"]["scope"], "session-open");
    assert_eq!(
        body["inherited_lineage_material"]["parent_execution_id"],
        "execution-running"
    );
    assert_eq!(
        body["inherited_lineage_material"]["parent_explicit_input"]["seed"],
        "running"
    );

    sleep(Duration::from_millis(80)).await;

    let (status, body) = server.get(&format!("/v1/executions/{execution_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "failed");
    assert_eq!(body["recent_events"][0]["event"], "execution.created");
    assert_eq!(body["recent_events"][1]["event"], "execution.delegated");
    assert_eq!(body["recent_events"][2]["event"], "execution.state_changed");
    assert_eq!(body["recent_events"][3]["event"], "execution.failed");
}
