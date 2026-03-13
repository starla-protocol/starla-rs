use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Open,
    Closed,
    Deleted,
}

#[derive(Clone, Debug)]
pub struct SessionRecord {
    pub session_id: String,
    pub state: SessionState,
    pub session_material: Option<Value>,
}

impl SessionRecord {
    pub fn new(id: &str, state: SessionState, session_material: Option<Value>) -> Self {
        Self {
            session_id: id.to_string(),
            state,
            session_material,
        }
    }

    pub fn view(&self) -> SessionView {
        SessionView {
            session_id: self.session_id.clone(),
            state: self.state,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SessionView {
    pub session_id: String,
    pub state: SessionState,
}
