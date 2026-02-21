use std::{future::Future, time::Duration};

use tokio_util::{sync::CancellationToken, task::TaskTracker};

#[derive(Clone)]
pub struct BackgroundTasks {
    tracker: TaskTracker,
    shutdown_token: CancellationToken,
}

impl BackgroundTasks {
    pub fn new() -> Self {
        Self {
            tracker: TaskTracker::new(),
            shutdown_token: CancellationToken::new(),
        }
    }

    pub fn spawn<F>(&self, task_name: &'static str, task: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        if self.shutdown_token.is_cancelled() {
            tracing::debug!(task = task_name, "skipping background task during shutdown");
            return;
        }

        let shutdown_token = self.shutdown_token.clone();
        self.tracker.spawn(async move {
            if shutdown_token.is_cancelled() {
                tracing::debug!(task = task_name, "background task canceled before start");
                return;
            }
            task.await;
        });
    }

    pub fn token(&self) -> CancellationToken {
        self.shutdown_token.clone()
    }

    pub fn begin_shutdown(&self) {
        self.shutdown_token.cancel();
        self.tracker.close();
    }

    pub fn pending_count(&self) -> usize {
        self.tracker.len()
    }

    pub async fn wait(&self, timeout: Duration) -> bool {
        tokio::time::timeout(timeout, self.tracker.wait())
            .await
            .is_ok()
    }
}
