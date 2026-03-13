use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentDefinitionState {
    Enabled,
    Disabled,
    Deleted,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentInstanceState {
    Ready,
    Paused,
    Terminated,
}

#[derive(Clone, Debug)]
pub struct AgentDefinitionRecord {
    pub agent_definition_id: String,
    pub state: AgentDefinitionState,
}

impl AgentDefinitionRecord {
    pub fn new(id: &str, state: AgentDefinitionState) -> Self {
        Self {
            agent_definition_id: id.to_string(),
            state,
        }
    }

    pub fn view(&self) -> AgentDefinitionView {
        AgentDefinitionView {
            agent_definition_id: self.agent_definition_id.clone(),
            state: self.state,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AgentInstanceRecord {
    pub agent_instance_id: String,
    pub agent_definition_id: String,
    pub state: AgentInstanceState,
}

impl AgentInstanceRecord {
    pub fn new(id: &str, definition_id: &str, state: AgentInstanceState) -> Self {
        Self {
            agent_instance_id: id.to_string(),
            agent_definition_id: definition_id.to_string(),
            state,
        }
    }

    pub fn view(&self) -> AgentInstanceView {
        AgentInstanceView {
            agent_instance_id: self.agent_instance_id.clone(),
            agent_definition_id: self.agent_definition_id.clone(),
            state: self.state,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct AgentDefinitionView {
    pub agent_definition_id: String,
    pub state: AgentDefinitionState,
}

#[derive(Clone, Debug, Serialize)]
pub struct AgentInstanceView {
    pub agent_instance_id: String,
    pub agent_definition_id: String,
    pub state: AgentInstanceState,
}
