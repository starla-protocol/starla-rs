use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolDefinitionState {
    Enabled,
    Disabled,
    Deleted,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolInvokeOutcome {
    Completed,
    Failed,
}

#[derive(Clone, Debug)]
pub struct ToolDefinitionRecord {
    pub tool_id: String,
    pub state: ToolDefinitionState,
    pub synthetic_outcome: ToolInvokeOutcome,
}

impl ToolDefinitionRecord {
    pub fn new(id: &str, state: ToolDefinitionState, synthetic_outcome: ToolInvokeOutcome) -> Self {
        Self {
            tool_id: id.to_string(),
            state,
            synthetic_outcome,
        }
    }

    pub fn view(&self) -> ToolDefinitionView {
        ToolDefinitionView {
            tool_id: self.tool_id.clone(),
            state: self.state,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ToolDefinitionView {
    pub tool_id: String,
    pub state: ToolDefinitionState,
}

#[derive(Clone, Debug, Serialize)]
pub struct ToolResultView {
    pub tool_id: String,
    pub outcome: ToolInvokeOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ToolInvocationView {
    pub execution_id: String,
    pub state: crate::domain::ExecutionState,
    pub tool_result: ToolResultView,
}
