use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionState {
    Pending,
    Running,
    Blocked,
    Completed,
    Failed,
    Canceled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SyntheticOutcome {
    Complete,
    Fail,
}

#[derive(Clone, Debug)]
pub struct SubmitWorkCommand {
    pub input: Value,
    pub session_id: Option<String>,
    pub references: Vec<Value>,
    pub idempotency_key: Option<String>,
}

#[derive(Clone, Debug)]
pub struct DelegateExecutionCommand {
    pub target_agent_instance_id: String,
    pub input: Value,
    pub references: Vec<Value>,
}

#[derive(Clone, Debug, Serialize)]
pub struct EventRecord {
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<ExecutionState>,
}

impl EventRecord {
    pub fn named(event: &str, lifecycle_state: Option<ExecutionState>) -> Self {
        Self {
            event: event.to_string(),
            lifecycle_state,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ContextSnapshot {
    pub execution_id: String,
    pub agent_instance_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub explicit_input: Value,
    pub explicit_references: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_material: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherited_lineage_material: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_derived_material: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_derived_material: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation_supplied: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct ExecutionRecord {
    pub execution_id: String,
    pub agent_instance_id: String,
    pub state: ExecutionState,
    pub session_id: Option<String>,
    pub parent_execution_id: Option<String>,
    pub context: ContextSnapshot,
    pub recent_events: Vec<EventRecord>,
    pub synthetic_outcome: SyntheticOutcome,
}

impl ExecutionRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn seeded(
        execution_id: &str,
        agent_instance_id: &str,
        state: ExecutionState,
        session_id: Option<&str>,
        parent_execution_id: Option<&str>,
        explicit_input: Value,
        explicit_references: Vec<Value>,
        session_material: Option<Value>,
        inherited_lineage_material: Option<Value>,
        synthetic_outcome: SyntheticOutcome,
        recent_events: Vec<EventRecord>,
    ) -> Self {
        Self {
            execution_id: execution_id.to_string(),
            agent_instance_id: agent_instance_id.to_string(),
            state,
            session_id: session_id.map(str::to_string),
            parent_execution_id: parent_execution_id.map(str::to_string),
            context: ContextSnapshot {
                execution_id: execution_id.to_string(),
                agent_instance_id: agent_instance_id.to_string(),
                session_id: session_id.map(str::to_string),
                explicit_input,
                explicit_references,
                session_material,
                inherited_lineage_material,
                tool_derived_material: None,
                event_derived_material: None,
                implementation_supplied: None,
            },
            recent_events,
            synthetic_outcome,
        }
    }

    pub fn push_event(&mut self, event: &str, lifecycle_state: Option<ExecutionState>) {
        self.recent_events
            .push(EventRecord::named(event, lifecycle_state));
    }

    pub fn list_item(&self) -> ExecutionListItem {
        ExecutionListItem {
            execution_id: self.execution_id.clone(),
            agent_instance_id: self.agent_instance_id.clone(),
            state: self.state,
            parent_execution_id: self.parent_execution_id.clone(),
            session_id: self.session_id.clone(),
        }
    }

    pub fn snapshot(&self) -> ExecutionSnapshot {
        ExecutionSnapshot {
            execution_id: self.execution_id.clone(),
            state: self.state,
            agent_instance_id: self.agent_instance_id.clone(),
            parent_execution_id: self.parent_execution_id.clone(),
            session_id: self.session_id.clone(),
            context: self.context.clone(),
            recent_events: self.recent_events.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SubmitWorkView {
    pub execution_id: String,
    pub state: ExecutionState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct SubmitWorkOutcome {
    pub created: bool,
    pub view: SubmitWorkView,
}

#[derive(Clone, Debug, Serialize)]
pub struct DelegateExecutionView {
    pub execution_id: String,
    pub parent_execution_id: String,
    pub agent_instance_id: String,
    pub state: ExecutionState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExecutionListItem {
    pub execution_id: String,
    pub agent_instance_id: String,
    pub state: ExecutionState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_execution_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExecutionSnapshot {
    pub execution_id: String,
    pub state: ExecutionState,
    pub agent_instance_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_execution_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub context: ContextSnapshot,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub recent_events: Vec<EventRecord>,
}

pub fn synthetic_outcome(input: &Value) -> SyntheticOutcome {
    let Some(value) = input.get("synthetic_outcome").and_then(Value::as_str) else {
        return SyntheticOutcome::Complete;
    };

    if value == "failed" {
        SyntheticOutcome::Fail
    } else {
        SyntheticOutcome::Complete
    }
}
