//! Noether Controller — Rust-native replacement for ROS 2 `noether_controller.py`.
//!
//! Monitors the HLLSet lattice for drift (flux = new keys - evictions)
//! and adjusts system parameters to maintain stability. Named after
//! Emmy Noether's theorem linking symmetry to conservation.

use crate::bus::MeshBus;
use crate::Message;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Flux monitor — tracks key creation rate and detects drift.
pub struct NoetherController {
    bus: Arc<dyn MeshBus>,
    state: Arc<Mutex<ControllerState>>,
}

struct ControllerState {
    flux: f64,
    threshold: f64,
    recent_keys: HashSet<String>,
    running: bool,
}

impl NoetherController {
    /// Create a new Noether controller.
    pub fn new(bus: Arc<dyn MeshBus>, threshold: f64) -> Self {
        Self {
            bus,
            state: Arc::new(Mutex::new(ControllerState {
                flux: 0.0,
                threshold,
                recent_keys: HashSet::new(),
                running: false,
            })),
        }
    }

    /// Start the controller — spawns a background task that monitors flux.
    pub async fn start(&self) {
        let mut state = self.state.lock().await;
        if state.running {
            return;
        }
        state.running = true;
        drop(state);

        let state = self.state.clone();
        let _bus = self.bus.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
            loop {
                interval.tick().await;
                let mut s = state.lock().await;
                if !s.running {
                    break;
                }

                debug!(
                    "Flux: {:.3} (threshold: {:.3}), keys tracked: {}",
                    s.flux,
                    s.threshold,
                    s.recent_keys.len()
                );

                if s.flux.abs() > s.threshold {
                    warn!(
                        "Flux drift detected: {:.3} > {:.3}. Adjusting parameters...",
                        s.flux, s.threshold
                    );
                    // Future: call a service to adjust system parameters
                }

                // Decay flux
                s.flux *= 0.9;
            }
        });

        info!("Noether controller started (threshold={})", {
            let s = self.state.lock().await;
            s.threshold
        });
    }

    /// Stop the controller.
    pub async fn stop(&self) {
        let mut state = self.state.lock().await;
        state.running = false;
        info!("Noether controller stopped");
    }

    /// Record a new key (positive flux).
    pub async fn record_key(&self, key: &str) {
        let mut state = self.state.lock().await;
        if state.recent_keys.insert(key.to_string()) {
            state.flux += 1.0;
        }
    }

    /// Get current flux value.
    pub async fn flux(&self) -> f64 {
        self.state.lock().await.flux
    }

    /// Get number of tracked keys.
    pub async fn tracked_keys(&self) -> usize {
        self.state.lock().await.recent_keys.len()
    }
}
