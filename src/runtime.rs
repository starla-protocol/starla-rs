use crate::store::AppState;
use tokio::time::{Duration, sleep};

pub fn spawn_execution_progress(state: AppState, execution_id: String) {
    tokio::spawn(async move {
        sleep(Duration::from_millis(25)).await;
        if !state.mark_execution_running(&execution_id).await {
            return;
        }

        sleep(Duration::from_millis(25)).await;
        let _ = state.finish_execution(&execution_id).await;
    });
}
