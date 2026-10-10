//! Atomic admission gate for ToolBus dispatching and STOP boundaries.
//!
//! Enforces **INV-09**: Once STOP is initiated or Human Takeover is active,
//! zero requests can enter the execution queue. Eliminates check-then-act races
//! through an atomic state machine and RAII dispatch permits.
//!
//! # STOP Linearization Point & Concurrency Contract
//! 1. **STOP Linearization Point**:
//!    - Calling `seal_cancelled()` closes future admission immediately.
//!    - Executions already admitted before STOP may complete or be cancelled
//!      according to their tool's cancellation token semantics.
//!    - No new `ToolRequest` may be admitted after the STOP linearization point.
//!
//! 2. **Human Takeover Execution Barrier & Concurrency Contract**:
//!    - Entering takeover (`enter_takeover()`) closes future autonomous agent admission immediately.
//!    - Pre-takeover admitted actions are allowed to settle or cancel according to semantics.
//!    - Post-takeover new agent dispatches are rejected immediately with `ToolError::HumanTakeoverActive`.
//!    - Upon user resumption (`resume_open()`), the admission gate reopens, fresh baseline observation
//!      is captured, and autonomous execution resumes.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

/// Operational admission state of the ToolBus execution gateway.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionState {
    /// Gateway is open; autonomous and interactive dispatches admitted.
    Open,
    /// Gateway is sealed by user or system STOP; all subsequent dispatches denied.
    Cancelled,
    /// Gateway is locked for exclusive human intervention; autonomous dispatches blocked.
    HumanTakeover,
}

/// Reasons for dispatch denial at the admission gate boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchDenial {
    /// Denied due to user or runtime STOP signal.
    Cancelled,
    /// Denied due to active human takeover on the browser surface.
    TakeoverActive,
}

/// RAII permit proving atomic admission into the execution gateway.
///
/// Automatically decrements the active in-flight counter when dropped.
#[derive(Debug)]
pub struct DispatchPermit {
    counter: Arc<AtomicUsize>,
}

impl Drop for DispatchPermit {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Atomic admission gate governing entry into the ToolBus dispatch boundary.
#[derive(Debug, Clone)]
pub struct DispatchAdmissionGate {
    task_id: Option<String>,
    state: Arc<RwLock<AdmissionState>>,
    in_flight: Arc<AtomicUsize>,
}

impl Default for DispatchAdmissionGate {
    fn default() -> Self {
        Self::new()
    }
}

impl DispatchAdmissionGate {
    /// Create a new, open admission gate without specific task scope.
    pub fn new() -> Self {
        Self {
            task_id: None,
            state: Arc::new(RwLock::new(AdmissionState::Open)),
            in_flight: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Create an admission gate explicitly scoped to a discrete agent task ID.
    ///
    /// # Task-Scoped STOP Isolation
    /// When STOP is triggered on a task-scoped gate, only that specific task's
    /// admission is sealed. Independent concurrent agent tasks retain their own
    /// active, open gates.
    pub fn for_task(task_id: impl Into<String>) -> Self {
        Self {
            task_id: Some(task_id.into()),
            state: Arc::new(RwLock::new(AdmissionState::Open)),
            in_flight: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Optional task scope ID associated with this gate.
    pub fn task_id(&self) -> Option<&str> {
        self.task_id.as_deref()
    }

    /// Atomically check state and acquire an in-flight dispatch permit.
    ///
    /// Once sealed (`Cancelled` or `HumanTakeover`), all calls immediately fail with
    /// `DispatchDenial`. Any permit acquired prior to sealing represents an already-admitted
    /// dispatch that may complete or cancel according to its tool's cancellation token.
    pub fn acquire_permit(&self) -> Result<DispatchPermit, DispatchDenial> {
        let guard = self.state.read().unwrap();
        match *guard {
            AdmissionState::Open => {
                self.in_flight.fetch_add(1, Ordering::SeqCst);
                Ok(DispatchPermit {
                    counter: self.in_flight.clone(),
                })
            }
            AdmissionState::Cancelled => Err(DispatchDenial::Cancelled),
            AdmissionState::HumanTakeover => Err(DispatchDenial::TakeoverActive),
        }
    }

    /// Atomically seal the admission boundary. Zero dispatches are admitted after this returns.
    pub fn seal_cancelled(&self) {
        let mut guard = self.state.write().unwrap();
        *guard = AdmissionState::Cancelled;
    }

    /// Atomically lock the boundary for exclusive human interaction.
    pub fn enter_takeover(&self) {
        let mut guard = self.state.write().unwrap();
        *guard = AdmissionState::HumanTakeover;
    }

    /// Atomically resume Open mode.
    pub fn resume_open(&self) {
        let mut guard = self.state.write().unwrap();
        *guard = AdmissionState::Open;
    }

    /// Get current admission state.
    pub fn current_state(&self) -> AdmissionState {
        *self.state.read().unwrap()
    }

    /// Check if admission gate is open.
    pub fn is_open(&self) -> bool {
        *self.state.read().unwrap() == AdmissionState::Open
    }

    /// Check if admission gate is cancelled (STOP triggered).
    pub fn is_cancelled(&self) -> bool {
        *self.state.read().unwrap() == AdmissionState::Cancelled
    }

    /// Check if admission gate is in human takeover mode.
    pub fn is_takeover(&self) -> bool {
        *self.state.read().unwrap() == AdmissionState::HumanTakeover
    }

    /// Count of currently active in-flight tool dispatches.
    pub fn in_flight_count(&self) -> usize {
        self.in_flight.load(Ordering::SeqCst)
    }
}

/// Registry managing task-scoped admission gates to guarantee cross-session isolation.
///
/// Ensures Agent Task A calling STOP cancels only Task A's admission gate without
/// affecting Agent Task B or global developer interactive sessions.
#[derive(Debug, Clone, Default)]
pub struct AdmissionGateRegistry {
    gates: Arc<RwLock<std::collections::HashMap<String, DispatchAdmissionGate>>>,
    global_gate: DispatchAdmissionGate,
}

impl AdmissionGateRegistry {
    /// Create a new admission gate registry.
    pub fn new() -> Self {
        Self {
            gates: Arc::new(RwLock::new(std::collections::HashMap::new())),
            global_gate: DispatchAdmissionGate::new(),
        }
    }

    /// Access the global / unscoped admission gate.
    pub fn global_gate(&self) -> &DispatchAdmissionGate {
        &self.global_gate
    }

    /// Get or create an admission gate scoped to a specific task.
    pub fn get_or_create(&self, task_id: &str) -> DispatchAdmissionGate {
        let mut guard = self.gates.write().unwrap();
        guard
            .entry(task_id.to_string())
            .or_insert_with(|| DispatchAdmissionGate::for_task(task_id))
            .clone()
    }

    /// Retrieve an appropriate admission gate given an optional task scope.
    pub fn get_gate(&self, task_id: Option<&str>) -> DispatchAdmissionGate {
        match task_id {
            Some(tid) => self.get_or_create(tid),
            None => self.global_gate.clone(),
        }
    }

    /// Explicitly bind a task-scoped admission gate.
    pub fn set_task_gate(&self, task_id: &str, gate: DispatchAdmissionGate) {
        let mut guard = self.gates.write().unwrap();
        guard.insert(task_id.to_string(), gate);
    }

    /// Set the global / fallback admission gate.
    pub fn set_global_gate(&mut self, gate: DispatchAdmissionGate) {
        self.global_gate = gate;
    }

    /// Seal admission for a specific task. Zero impact on other tasks.
    pub fn seal_task(&self, task_id: &str) {
        let guard = self.gates.read().unwrap();
        if let Some(gate) = guard.get(task_id) {
            gate.seal_cancelled();
        }
    }

    /// Remove a terminated task's gate.
    pub fn remove_task(&self, task_id: &str) -> Option<DispatchAdmissionGate> {
        let mut guard = self.gates.write().unwrap();
        guard.remove(task_id)
    }
}
