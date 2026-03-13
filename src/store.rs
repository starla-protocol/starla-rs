use crate::domain::{
    AgentDefinitionRecord, AgentDefinitionState, AgentDefinitionView, AgentInstanceRecord,
    AgentInstanceState, AgentInstanceView, ContextSnapshot, DelegateExecutionCommand,
    DelegateExecutionView, EventRecord, ExecutionListItem, ExecutionRecord, ExecutionSnapshot,
    ExecutionState, ProtocolError, SessionRecord, SessionState, SessionView, SubmitWorkCommand,
    SubmitWorkView, SyntheticOutcome, synthetic_outcome,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    store: Arc<RwLock<Store>>,
    next_execution: Arc<AtomicU64>,
}

impl AppState {
    pub fn seeded() -> Self {
        Self {
            store: Arc::new(RwLock::new(Store::seeded())),
            next_execution: Arc::new(AtomicU64::new(1)),
        }
    }

    pub fn next_execution_id(&self) -> String {
        let value = self.next_execution.fetch_add(1, Ordering::SeqCst);
        format!("exec-{value}")
    }

    pub async fn list_agent_definitions(&self) -> Vec<AgentDefinitionView> {
        let store = self.store.read().await;
        store
            .agent_definitions
            .values()
            .map(AgentDefinitionRecord::view)
            .collect()
    }

    pub async fn get_agent_definition(
        &self,
        agent_definition_id: &str,
    ) -> Result<AgentDefinitionView, ProtocolError> {
        let store = self.store.read().await;
        let definition = store
            .agent_definitions
            .get(agent_definition_id)
            .ok_or(ProtocolError::NotFound)?;
        Ok(definition.view())
    }

    pub async fn disable_agent_definition(
        &self,
        agent_definition_id: &str,
    ) -> Result<AgentDefinitionView, ProtocolError> {
        let mut store = self.store.write().await;
        let definition = store
            .agent_definitions
            .get_mut(agent_definition_id)
            .ok_or(ProtocolError::NotFound)?;

        if definition.state != AgentDefinitionState::Enabled {
            return Err(ProtocolError::InvalidState);
        }

        definition.state = AgentDefinitionState::Disabled;
        Ok(definition.view())
    }

    pub async fn enable_agent_definition(
        &self,
        agent_definition_id: &str,
    ) -> Result<AgentDefinitionView, ProtocolError> {
        let mut store = self.store.write().await;
        let definition = store
            .agent_definitions
            .get_mut(agent_definition_id)
            .ok_or(ProtocolError::NotFound)?;

        if definition.state != AgentDefinitionState::Disabled {
            return Err(ProtocolError::InvalidState);
        }

        definition.state = AgentDefinitionState::Enabled;
        Ok(definition.view())
    }

    pub async fn list_agent_instances(&self) -> Vec<AgentInstanceView> {
        let store = self.store.read().await;
        store
            .agent_instances
            .values()
            .map(AgentInstanceRecord::view)
            .collect()
    }

    pub async fn get_agent_instance(
        &self,
        agent_instance_id: &str,
    ) -> Result<AgentInstanceView, ProtocolError> {
        let store = self.store.read().await;
        let instance = store
            .agent_instances
            .get(agent_instance_id)
            .ok_or(ProtocolError::NotFound)?;
        Ok(instance.view())
    }

    pub async fn pause_agent_instance(
        &self,
        agent_instance_id: &str,
    ) -> Result<AgentInstanceView, ProtocolError> {
        let mut store = self.store.write().await;
        let instance = store
            .agent_instances
            .get_mut(agent_instance_id)
            .ok_or(ProtocolError::NotFound)?;

        if instance.state != AgentInstanceState::Ready {
            return Err(ProtocolError::InvalidState);
        }

        instance.state = AgentInstanceState::Paused;
        Ok(instance.view())
    }

    pub async fn resume_agent_instance(
        &self,
        agent_instance_id: &str,
    ) -> Result<AgentInstanceView, ProtocolError> {
        let mut store = self.store.write().await;
        let instance = store
            .agent_instances
            .get_mut(agent_instance_id)
            .ok_or(ProtocolError::NotFound)?;

        if instance.state != AgentInstanceState::Paused {
            return Err(ProtocolError::InvalidState);
        }

        instance.state = AgentInstanceState::Ready;
        Ok(instance.view())
    }

    pub async fn terminate_agent_instance(
        &self,
        agent_instance_id: &str,
    ) -> Result<AgentInstanceView, ProtocolError> {
        let mut store = self.store.write().await;
        let instance = store
            .agent_instances
            .get_mut(agent_instance_id)
            .ok_or(ProtocolError::NotFound)?;

        if !matches!(
            instance.state,
            AgentInstanceState::Ready | AgentInstanceState::Paused
        ) {
            return Err(ProtocolError::InvalidState);
        }

        instance.state = AgentInstanceState::Terminated;
        Ok(instance.view())
    }

    pub async fn list_sessions(&self) -> Vec<SessionView> {
        let store = self.store.read().await;
        store.sessions.values().map(SessionRecord::view).collect()
    }

    pub async fn get_session(&self, session_id: &str) -> Result<SessionView, ProtocolError> {
        let store = self.store.read().await;
        let session = store
            .sessions
            .get(session_id)
            .ok_or(ProtocolError::NotFound)?;
        Ok(session.view())
    }

    pub async fn close_session(&self, session_id: &str) -> Result<SessionView, ProtocolError> {
        let mut store = self.store.write().await;
        let session = store
            .sessions
            .get_mut(session_id)
            .ok_or(ProtocolError::NotFound)?;

        if session.state != SessionState::Open {
            return Err(ProtocolError::InvalidState);
        }

        session.state = SessionState::Closed;
        Ok(session.view())
    }

    pub async fn submit_work(
        &self,
        agent_instance_id: &str,
        command: SubmitWorkCommand,
    ) -> Result<(String, SubmitWorkView), ProtocolError> {
        let execution_id = self.next_execution_id();

        {
            let mut store = self.store.write().await;
            let instance = store
                .agent_instances
                .get(agent_instance_id)
                .ok_or(ProtocolError::NotFound)?;

            if instance.state != AgentInstanceState::Ready {
                return Err(ProtocolError::InvalidState);
            }

            let session_material = match command.session_id.as_ref() {
                Some(session_id) => {
                    let session = store
                        .sessions
                        .get(session_id)
                        .ok_or(ProtocolError::NotFound)?;
                    if session.state != SessionState::Open {
                        return Err(ProtocolError::InvalidState);
                    }
                    session.session_material.clone()
                }
                None => None,
            };

            let synthetic_outcome = synthetic_outcome(&command.input);
            let execution = ExecutionRecord::seeded(
                &execution_id,
                agent_instance_id,
                ExecutionState::Pending,
                command.session_id.as_deref(),
                None,
                command.input,
                command.references,
                session_material,
                None,
                synthetic_outcome,
                vec![EventRecord::named("execution.created", None)],
            );

            store.executions.insert(execution_id.clone(), execution);
        }

        Ok((
            execution_id.clone(),
            SubmitWorkView {
                execution_id,
                state: ExecutionState::Pending,
                session_id: command.session_id,
            },
        ))
    }

    pub async fn list_executions(&self) -> Vec<ExecutionListItem> {
        let store = self.store.read().await;
        store
            .executions
            .values()
            .map(ExecutionRecord::list_item)
            .collect()
    }

    pub async fn get_execution(
        &self,
        execution_id: &str,
    ) -> Result<ExecutionSnapshot, ProtocolError> {
        let store = self.store.read().await;
        let execution = store
            .executions
            .get(execution_id)
            .ok_or(ProtocolError::NotFound)?;
        Ok(execution.snapshot())
    }

    pub async fn get_execution_context(
        &self,
        execution_id: &str,
    ) -> Result<ContextSnapshot, ProtocolError> {
        let store = self.store.read().await;
        let execution = store
            .executions
            .get(execution_id)
            .ok_or(ProtocolError::NotFound)?;
        Ok(execution.context.clone())
    }

    pub async fn cancel_execution(
        &self,
        execution_id: &str,
    ) -> Result<ExecutionListItem, ProtocolError> {
        let mut store = self.store.write().await;
        let execution = store
            .executions
            .get_mut(execution_id)
            .ok_or(ProtocolError::NotFound)?;

        if !matches!(
            execution.state,
            ExecutionState::Pending | ExecutionState::Running | ExecutionState::Blocked
        ) {
            return Err(ProtocolError::InvalidState);
        }

        execution.state = ExecutionState::Canceled;
        execution.push_event("execution.canceled", Some(ExecutionState::Canceled));
        Ok(execution.list_item())
    }

    pub async fn delegate_execution(
        &self,
        parent_execution_id: &str,
        command: DelegateExecutionCommand,
    ) -> Result<(String, DelegateExecutionView), ProtocolError> {
        let child_execution_id = self.next_execution_id();
        let child_outcome = synthetic_outcome(&command.input);

        {
            let mut store = self.store.write().await;
            let parent = store
                .executions
                .get(parent_execution_id)
                .ok_or(ProtocolError::NotFound)?
                .clone();

            if matches!(
                parent.state,
                ExecutionState::Completed | ExecutionState::Failed | ExecutionState::Canceled
            ) {
                return Err(ProtocolError::InvalidState);
            }

            let target = store
                .agent_instances
                .get(&command.target_agent_instance_id)
                .ok_or(ProtocolError::NotFound)?;

            if target.state != AgentInstanceState::Ready {
                return Err(ProtocolError::InvalidState);
            }

            if target.agent_instance_id == parent.agent_instance_id {
                return Err(ProtocolError::InvalidState);
            }

            let inherited_lineage_material = Some(json!({
                "parent_execution_id": parent.execution_id,
                "parent_explicit_input": parent.context.explicit_input,
            }));

            let child = ExecutionRecord::seeded(
                &child_execution_id,
                &command.target_agent_instance_id,
                ExecutionState::Pending,
                parent.session_id.as_deref(),
                Some(parent_execution_id),
                command.input,
                command.references,
                parent.context.session_material.clone(),
                inherited_lineage_material,
                child_outcome,
                vec![
                    EventRecord::named("execution.created", None),
                    EventRecord::named("execution.delegated", None),
                ],
            );

            store.executions.insert(child_execution_id.clone(), child);
        }

        let session_id = self.current_session_id(parent_execution_id).await;
        Ok((
            child_execution_id.clone(),
            DelegateExecutionView {
                execution_id: child_execution_id,
                parent_execution_id: parent_execution_id.to_string(),
                agent_instance_id: command.target_agent_instance_id,
                state: ExecutionState::Pending,
                session_id,
            },
        ))
    }

    pub async fn current_session_id(&self, execution_id: &str) -> Option<String> {
        let store = self.store.read().await;
        store
            .executions
            .get(execution_id)
            .and_then(|execution| execution.session_id.clone())
    }

    pub async fn mark_execution_running(&self, execution_id: &str) -> bool {
        let mut store = self.store.write().await;
        let Some(execution) = store.executions.get_mut(execution_id) else {
            return false;
        };

        if execution.state != ExecutionState::Pending {
            return false;
        }

        execution.state = ExecutionState::Running;
        execution.push_event("execution.state_changed", Some(ExecutionState::Running));
        true
    }

    pub async fn finish_execution(&self, execution_id: &str) -> bool {
        let mut store = self.store.write().await;
        let Some(execution) = store.executions.get_mut(execution_id) else {
            return false;
        };

        if execution.state != ExecutionState::Running {
            return false;
        }

        if execution.synthetic_outcome == SyntheticOutcome::Fail {
            execution.state = ExecutionState::Failed;
            execution.push_event("execution.failed", Some(ExecutionState::Failed));
        } else {
            execution.state = ExecutionState::Completed;
            execution.push_event("execution.completed", Some(ExecutionState::Completed));
        }

        true
    }
}

#[derive(Default)]
struct Store {
    agent_definitions: BTreeMap<String, AgentDefinitionRecord>,
    agent_instances: BTreeMap<String, AgentInstanceRecord>,
    sessions: BTreeMap<String, SessionRecord>,
    executions: BTreeMap<String, ExecutionRecord>,
}

impl Store {
    fn seeded() -> Self {
        let mut store = Self::default();

        store.agent_definitions.insert(
            "agent-def-enabled".to_string(),
            AgentDefinitionRecord::new("agent-def-enabled", AgentDefinitionState::Enabled),
        );
        store.agent_definitions.insert(
            "agent-def-disabled".to_string(),
            AgentDefinitionRecord::new("agent-def-disabled", AgentDefinitionState::Disabled),
        );

        store.agent_instances.insert(
            "agent-inst-primary".to_string(),
            AgentInstanceRecord::new(
                "agent-inst-primary",
                "agent-def-enabled",
                AgentInstanceState::Ready,
            ),
        );
        store.agent_instances.insert(
            "agent-inst-helper".to_string(),
            AgentInstanceRecord::new(
                "agent-inst-helper",
                "agent-def-enabled",
                AgentInstanceState::Ready,
            ),
        );
        store.agent_instances.insert(
            "agent-inst-paused".to_string(),
            AgentInstanceRecord::new(
                "agent-inst-paused",
                "agent-def-enabled",
                AgentInstanceState::Paused,
            ),
        );
        store.agent_instances.insert(
            "agent-inst-terminated".to_string(),
            AgentInstanceRecord::new(
                "agent-inst-terminated",
                "agent-def-enabled",
                AgentInstanceState::Terminated,
            ),
        );

        store.sessions.insert(
            "session-open".to_string(),
            SessionRecord::new(
                "session-open",
                SessionState::Open,
                Some(json_value(vec![("scope", "session-open")])),
            ),
        );
        store.sessions.insert(
            "session-closed".to_string(),
            SessionRecord::new(
                "session-closed",
                SessionState::Closed,
                Some(json_value(vec![("scope", "session-closed")])),
            ),
        );

        store.executions.insert(
            "execution-failed".to_string(),
            ExecutionRecord::seeded(
                "execution-failed",
                "agent-inst-primary",
                ExecutionState::Failed,
                None,
                None,
                json_value(vec![("synthetic_outcome", "failed")]),
                Vec::new(),
                None,
                None,
                SyntheticOutcome::Fail,
                vec![
                    EventRecord::named("execution.created", None),
                    EventRecord::named("execution.state_changed", Some(ExecutionState::Running)),
                    EventRecord::named("execution.failed", Some(ExecutionState::Failed)),
                ],
            ),
        );

        store.executions.insert(
            "execution-completed".to_string(),
            ExecutionRecord::seeded(
                "execution-completed",
                "agent-inst-helper",
                ExecutionState::Completed,
                Some("session-open"),
                None,
                json_value(vec![("seed", "completed")]),
                Vec::new(),
                Some(json_value(vec![("scope", "session-open")])),
                None,
                SyntheticOutcome::Complete,
                vec![
                    EventRecord::named("execution.created", None),
                    EventRecord::named("execution.state_changed", Some(ExecutionState::Running)),
                    EventRecord::named("execution.completed", Some(ExecutionState::Completed)),
                ],
            ),
        );

        store.executions.insert(
            "execution-running".to_string(),
            ExecutionRecord::seeded(
                "execution-running",
                "agent-inst-primary",
                ExecutionState::Running,
                Some("session-open"),
                None,
                json_value(vec![("seed", "running")]),
                Vec::new(),
                Some(json_value(vec![("scope", "session-open")])),
                None,
                SyntheticOutcome::Complete,
                vec![
                    EventRecord::named("execution.created", None),
                    EventRecord::named("execution.state_changed", Some(ExecutionState::Running)),
                ],
            ),
        );

        store.executions.insert(
            "execution-pending".to_string(),
            ExecutionRecord::seeded(
                "execution-pending",
                "agent-inst-helper",
                ExecutionState::Pending,
                Some("session-open"),
                None,
                json_value(vec![("seed", "pending")]),
                Vec::new(),
                Some(json_value(vec![("scope", "session-open")])),
                None,
                SyntheticOutcome::Complete,
                vec![EventRecord::named("execution.created", None)],
            ),
        );

        store
    }
}

fn json_value(pairs: Vec<(&str, &str)>) -> Value {
    let object = pairs
        .into_iter()
        .map(|(key, value)| (key.to_string(), Value::String(value.to_string())))
        .collect();
    Value::Object(object)
}
