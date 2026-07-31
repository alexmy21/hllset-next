//! Noether Controller — integer-rank-aligned flux monitor.
//!
//! Monitors the HLLSet lattice for drift (flux = new keys - evictions)
//! and adjusts system parameters to maintain stability. Named after
//! Emmy Noether's theorem linking symmetry to conservation.
//!
//! All arithmetic is integer-only, aligned with the five-level rank
//! algebra (§3.2 of STANDARD.md). Flux is measured as signed i64 keys
//! per tick with integer halving decay — no floating point.

use crate::bus::MeshBus;
use crate::Message;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Flux monitor — tracks key creation/eviction rate and detects drift.
pub struct NoetherController {
    bus: Arc<dyn MeshBus>,
    state: Arc<Mutex<ControllerState>>,
}

struct ControllerState {
    flux: i64,
    threshold: i64,
    recent_keys: HashSet<String>,
    running: bool,
}

impl NoetherController {
    /// Create a new Noether controller with integer threshold.
    pub fn new(bus: Arc<dyn MeshBus>, threshold: i64) -> Self {
        Self {
            bus,
            state: Arc::new(Mutex::new(ControllerState {
                flux: 0,
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
                    "Flux: {} (threshold: {}), keys tracked: {}",
                    s.flux,
                    s.threshold,
                    s.recent_keys.len()
                );

                if s.flux.abs() > s.threshold {
                    warn!(
                        "Flux drift detected: {} > {}. Adjusting parameters...",
                        s.flux, s.threshold
                    );
                    // Future: call a service to adjust system parameters
                }

                // Integer halving decay — equivalent to exponential decay
                // but in pure integer space, aligned with rank algebra
                s.flux /= 2;
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
            state.flux += 1;
        }
    }

    /// Record an evicted key (negative flux).
    pub async fn record_eviction(&self, key: &str) {
        let mut state = self.state.lock().await;
        state.recent_keys.remove(key);
        state.flux -= 1;
    }

    /// Get current flux value (integer).
    pub async fn flux(&self) -> i64 {
        self.state.lock().await.flux
    }

    /// Get number of tracked keys.
    pub async fn tracked_keys(&self) -> usize {
        self.state.lock().await.recent_keys.len()
    }
}
