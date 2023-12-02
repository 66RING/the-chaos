use axum::Extension;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::{signal, sync::broadcast};
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct Shutdown {
    pub sender: broadcast::Sender<()>,
}

#[derive(Debug, thiserror::Error)]
pub enum ShutdownError {
    #[error("shutdown handler already created")]
    AlreadyCreated,
}

static CREATED: AtomicBool = AtomicBool::new(false);

impl Shutdown {
    pub fn new() -> Result<Self, ShutdownError> {
        if (CREATED).swap(true, Ordering::SeqCst) {
            error!("shutdown handler called twice");
            return Err(ShutdownError::AlreadyCreated);
        }

        let (tx, _) = broadcast::channel(1);
        let signal_handler = Shutdown::signal_handler();

        // Spawn the signal handler to send the shutdown signal.
        let tx_for_handle = tx.clone();
        tokio::spawn(async move {
            info!("Registered shutdown signal handlers");
            signal_handler.await;
            tx_for_handle.send(()).ok();
        });

        Ok(Self { sender: tx })
    }

    pub fn extension(&self) -> Extension<Shutdown> {
        let shutdown = Shutdown {
            sender: self.sender.clone(),
        };
        Extension(shutdown)
    }

    pub fn start_shutdown(self) {
        self.sender.send(()).ok();
    }

    pub async fn wait(self) {
        let _ = self.sender.subscribe().recv().await;
    }

    /// Handler for the `ctrl+c` signal.
    pub async fn signal_handler() {
        // `ctrl+c` signal handler.
        let ctrl_c = signal::ctrl_c();

        // terminate signal handler.
        #[cfg(unix)]
        let mut terminate = signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler");

        // async move {
        tokio::select! {
            _ = ctrl_c => {
                info!("Received Ctrl+C signal");
            },
            _ = terminate.recv() => {
                info!("Received Ctrl+C signal");
            }
        }
        // }
    }
}
