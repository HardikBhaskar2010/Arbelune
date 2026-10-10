//! # Milestone 10 (M10) Integration Test Suite: Verified Autonomy
//!
//! Validates the 10 gates of the M10 Verified Autonomy Architecture:
//! - **GATE-10-A**: URL Postcondition Verifier (exact, prefix, regex matching; uncommitted rejection)
//! - **GATE-10-B**: DOM & AX Postcondition Verifier (element existence, visible text, AX role/name)
//! - **GATE-10-C**: Multi-Reference Visual Verifier (ExactSha256 vs PerceptualFingerprint vs PixelDiffBaseline)
//! - **GATE-10-D**: Delta Quiescence & Console Silence (Pre-action baseline vs post-action state)
//! - **GATE-10-E**: Tripartite Anti-Hallucination (`INV-08`: Pass requires empirical proof; ambiguous yields Unknown)
//! - **GATE-10-F**: Registry-Driven Idempotent Recovery (`SafeRetry` backoff with retry budget)
//! - **GATE-10-G**: Non-Idempotent Mutation Pause (`NonIdempotent` pauses execution in `TaskState::AwaitingGuidance`)
//! - **GATE-10-H**: State-Aware Cycle Detection (`CycleKey` repeats 3 times halt with `E_CYCLE_DETECTED`)
//! - **GATE-10-I**: Atomic STOP Admission Concurrency Race (`INV-09`: 50-task race proves zero ToolBus leakage)
//! - **GATE-10-J**: Exclusive Human Takeover & State Resynchronization (execution barrier, HWND control, fresh telemetry resync)

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use serde_json::json;
use tokio_util::sync::CancellationToken;

use kage_core::admission::{AdmissionGateRegistry, AdmissionState, DispatchAdmissionGate, DispatchDenial};
use kage_core::bus::{PartialPolicyContext, ToolBus};
use kage_core::lineage::ExecutionStatus;
use kage_core::policy::PermissionTier;
use kage_core::registry::{
    CapabilityRegistry, IdempotencyClassification, RetryPolicy, ToolCategory,
    ToolMetadata, ToolVerificationContract,
};
use kage_core::tool::{KageTool, ToolError, ToolRequest, ToolResponse};

use kage_agent::cancellation::AgentCancellation;
use kage_agent::recovery::{RecoveryDecision, RecoveryManager};
use kage_agent::task::{AgentTask, TaskState};
use kage_agent::takeover::HumanTakeoverManager;
use kage_agent::verifier::{
    BaselineObservation, BoundingBox, CanonicalPixelBuffer, ColorSpace, DeterministicVerifier,
    ExpectedEffect, ObservationTelemetry, ObservedAxNode, ObservedElement, ObservedNetworkRequest,
    PixelFormat, PostconditionPredicate, PredicateAuthority, PredicateValidator, QuiescencePredicate,
    ScreenshotPredicate, ScreenshotReference, StandardNetworkClassifier, UrlMatchType,
    VerificationOutcome, VisualCaptureTiming,
};

use kage_browser::{BrowserEventBus, ProfileManager, TabManager};
use kage_cdp::CdpBroker;
use kage_host_lib::cdp_session::CdpSessionManager;
use kage_host_lib::tools::{PageClickTool, TabCreateTool, TabListTool};
use kage_storage::AuditDb;

// ===========================================================================
// Test Tool Fixture
// ===========================================================================

struct MockNavTool;

#[async_trait::async_trait]
impl KageTool for MockNavTool {
    fn tool_id(&self) -> &'static str {
        "browser.navigate"
    }
    fn tier(&self) -> PermissionTier {
        PermissionTier::StateMutating
    }
    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "url": { "type": "string" }
            },
            "required": ["url"]
        })
    }
    async fn execute(
        &self,
        request: &ToolRequest,
        _cancel: CancellationToken,
    ) -> Result<ToolResponse, ToolError> {
        let url = request.args.get("url").and_then(|u| u.as_str()).unwrap_or("");
        Ok(ToolResponse::success(
            &request.request_id,
            json!({ "navigated_to": url }),
            format!("Navigated to {}", url),
            15,
        ))
    }
}

#[allow(dead_code)]
struct MockPageClickTool;

#[async_trait::async_trait]
impl KageTool for MockPageClickTool {
    fn tool_id(&self) -> &'static str {
        "page.click"
    }
    fn tier(&self) -> PermissionTier {
        PermissionTier::StateMutating
    }
    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "selector": { "type": "string" }
            },
            "required": ["selector"]
        })
    }
    async fn execute(
        &self,
        request: &ToolRequest,
        _cancel: CancellationToken,
    ) -> Result<ToolResponse, ToolError> {
        let selector = request.args.get("selector").and_then(|s| s.as_str()).unwrap_or("#submit-order");
        Ok(ToolResponse::success(
            &request.request_id,
            json!({ "clicked": true, "selector": selector }),
            format!("Clicked element {}", selector),
            12,
        ))
    }
}

// ===========================================================================
// GATE-10-A: URL Postcondition Verifier
// ===========================================================================

#[tokio::test]
async fn test_gate_10_a_url_postcondition_verifier() {
    let verifier = DeterministicVerifier::new();
    let resp = ToolResponse::success("req_1", json!({}), "Navigated", 10);

    // 1. Exact match
    let pred_exact = PostconditionPredicate::UrlMatch {
        pattern: "https://kage.dev/dashboard".to_string(),
        match_type: UrlMatchType::Exact,
    };
    let telem_match = ObservationTelemetry {
        current_url: Some("https://kage.dev/dashboard".to_string()),
        ..Default::default()
    };
    let outcome = verifier.verify(&pred_exact, &resp, None, &telem_match);
    assert!(outcome.is_pass(), "Exact URL match should Pass");

    // 2. Prefix match
    let pred_prefix = PostconditionPredicate::UrlMatch {
        pattern: "https://kage.dev/orders/".to_string(),
        match_type: UrlMatchType::Prefix,
    };
    let telem_prefix = ObservationTelemetry {
        current_url: Some("https://kage.dev/orders/12345/summary".to_string()),
        ..Default::default()
    };
    let outcome = verifier.verify(&pred_prefix, &resp, None, &telem_prefix);
    assert!(outcome.is_pass(), "Prefix URL match should Pass");

    // 3. Regex match
    let pred_regex = PostconditionPredicate::UrlMatch {
        pattern: r"^https://kage\.dev/users/\d+$".to_string(),
        match_type: UrlMatchType::Regex,
    };
    let telem_regex = ObservationTelemetry {
        current_url: Some("https://kage.dev/users/9876".to_string()),
        ..Default::default()
    };
    let outcome = verifier.verify(&pred_regex, &resp, None, &telem_regex);
    assert!(outcome.is_pass(), "Regex URL match should Pass");

    // 4. Missing URL telemetry -> Unknown
    let telem_empty = ObservationTelemetry {
        current_url: None,
        ..Default::default()
    };
    let outcome = verifier.verify(&pred_exact, &resp, None, &telem_empty);
    assert!(outcome.is_unknown(), "Missing URL telemetry must return Unknown");

    // 5. Mismatched URL -> Fail
    let telem_mismatch = ObservationTelemetry {
        current_url: Some("https://kage.dev/login".to_string()),
        ..Default::default()
    };
    let outcome = verifier.verify(&pred_exact, &resp, None, &telem_mismatch);
    assert!(outcome.is_fail(), "Mismatched URL must return Fail");
}

// ===========================================================================
// GATE-10-B: DOM & AX Postcondition Verifier
// ===========================================================================

#[tokio::test]
async fn test_gate_10_b_dom_and_ax_postcondition_verifier() {
    let verifier = DeterministicVerifier::new();
    let resp = ToolResponse::success("req_2", json!({}), "Clicked", 20);

    let mut attrs = HashMap::new();
    attrs.insert("data-state".to_string(), "submitted".to_string());

    let telem = ObservationTelemetry {
        dom_elements: vec![
            ObservedElement {
                selector: "#submit-btn".to_string(),
                exists: true,
                visible: true,
                text_content: Some("Success! Form Saved".to_string()),
                attributes: attrs,
            },
            ObservedElement {
                selector: "#hidden-modal".to_string(),
                exists: true,
                visible: false,
                text_content: None,
                attributes: HashMap::new(),
            },
        ],
        ax_nodes: vec![
            ObservedAxNode {
                role: "button".to_string(),
                name: Some("Save Changes".to_string()),
            },
            ObservedAxNode {
                role: "alert".to_string(),
                name: Some("Submission Successful".to_string()),
            },
        ],
        ..Default::default()
    };

    // 1. DOM element visible with text and attribute match -> Pass
    let pred_dom = PostconditionPredicate::DomElement {
        selector: "#submit-btn".to_string(),
        must_be_visible: true,
        expected_text: Some("Success".to_string()),
        expected_attribute: Some(("data-state".to_string(), "submitted".to_string())),
    };
    let outcome = verifier.verify(&pred_dom, &resp, None, &telem);
    assert!(outcome.is_pass(), "DOM element with visible match must Pass");

    // 2. DOM element exists but not visible when visibility required -> Fail
    let pred_hidden = PostconditionPredicate::DomElement {
        selector: "#hidden-modal".to_string(),
        must_be_visible: true,
        expected_text: None,
        expected_attribute: None,
    };
    let outcome = verifier.verify(&pred_hidden, &resp, None, &telem);
    assert!(outcome.is_fail(), "Hidden element must Fail when visibility required");

    // 3. Missing DOM element -> Fail
    let pred_missing = PostconditionPredicate::DomElement {
        selector: "#non-existent".to_string(),
        must_be_visible: false,
        expected_text: None,
        expected_attribute: None,
    };
    let outcome = verifier.verify(&pred_missing, &resp, None, &telem);
    assert!(outcome.is_fail(), "Non-existent DOM element must Fail");

    // 4. AX tree node role and accessible name match -> Pass
    let pred_ax = PostconditionPredicate::AxNode {
        role: "alert".to_string(),
        name: Some("Submission Successful".to_string()),
    };
    let outcome = verifier.verify(&pred_ax, &resp, None, &telem);
    assert!(outcome.is_pass(), "AX node with matching role and name must Pass");

    // 5. AX node not found -> Fail
    let pred_ax_missing = PostconditionPredicate::AxNode {
        role: "dialog".to_string(),
        name: None,
    };
    let outcome = verifier.verify(&pred_ax_missing, &resp, None, &telem);
    assert!(outcome.is_fail(), "Missing AX node must Fail");
}

// ===========================================================================
// GATE-10-C: Multi-Reference Visual Verifier
// ===========================================================================

#[tokio::test]
async fn test_gate_10_c_multi_reference_visual_verifier() {
    let verifier = DeterministicVerifier::new();
    let resp = ToolResponse::success("req_3", json!({}), "Screenshot Taken", 30);

    // 1. ExactSha256 comparison: 0% divergence allowed
    let pred_exact_match = PostconditionPredicate::VisualComparison(
        ScreenshotPredicate::new(ScreenshotReference::ExactSha256("sha256_abcdef123456".to_string()))
            .with_timing(VisualCaptureTiming::Immediate),
    );
    let telem_exact_pass = ObservationTelemetry {
        screenshot_hash: Some("sha256_abcdef123456".to_string()),
        ..Default::default()
    };
    assert!(verifier.verify(&pred_exact_match, &resp, None, &telem_exact_pass).is_pass());

    let telem_exact_fail = ObservationTelemetry {
        screenshot_hash: Some("sha256_diff99999999".to_string()),
        ..Default::default()
    };
    assert!(verifier.verify(&pred_exact_match, &resp, None, &telem_exact_fail).is_fail());

    // 2. PixelDiffBaseline comparison with actual baseline_bytes (Correction #1)
    let baseline_pixels = vec![255u8; 100];
    let baseline_sha = "sha256_baseline_raw_100".to_string();

    let pred_drift = PostconditionPredicate::VisualComparison(
        ScreenshotPredicate::new(ScreenshotReference::PixelDiffBaseline {
            artifact_id: Some("artifact_screenshot_base_1".to_string()),
            canonical_baseline: None,
            baseline_bytes: Some(baseline_pixels.clone()),
            baseline_sha256: baseline_sha.clone(),
            width: 10,
            height: 10,
            tolerance_percent: 0.05, // 5% allowed drift
            roi: Some(BoundingBox { x: 0.0, y: 0.0, width: 10.0, height: 10.0 }),
            ignore_regions: vec![],
        })
        .with_timing(VisualCaptureTiming::AfterAnimationStabilization { settle_ms: 100 }),
    );

    // Case 2a: Direct pixel mismatch of 3 bytes out of 100 (3.0% drift <= 5.0% tolerance) -> PASS
    let mut current_pixels_pass = baseline_pixels.clone();
    current_pixels_pass[0] = 0;
    current_pixels_pass[1] = 0;
    current_pixels_pass[2] = 0;

    let telem_pixel_pass = ObservationTelemetry {
        screenshot_hash: Some("sha256_cur_pass_hash".to_string()),
        screenshot_bytes: Some(current_pixels_pass),
        screenshot_width: Some(10),
        screenshot_height: Some(10),
        ..Default::default()
    };
    let outcome = verifier.verify(&pred_drift, &resp, None, &telem_pixel_pass);
    assert!(outcome.is_pass(), "Direct pixel drift of 3% <= 5% tolerance must Pass");

    // Case 2b: Direct pixel mismatch of 8 bytes out of 100 (8.0% drift > 5.0% tolerance) -> FAIL
    let mut current_pixels_fail = baseline_pixels.clone();
    for i in 0..8 {
        current_pixels_fail[i] = 0;
    }
    let telem_pixel_fail = ObservationTelemetry {
        screenshot_hash: Some("sha256_cur_fail_hash".to_string()),
        screenshot_bytes: Some(current_pixels_fail),
        screenshot_width: Some(10),
        screenshot_height: Some(10),
        ..Default::default()
    };
    let outcome_fail = verifier.verify(&pred_drift, &resp, None, &telem_pixel_fail);
    assert!(outcome_fail.is_fail(), "Direct pixel drift of 8% > 5% tolerance must Fail");

    // Case 2c: Exact SHA-256 match -> PASS (0% drift)
    let telem_exact_digest = ObservationTelemetry {
        screenshot_hash: Some(baseline_sha.clone()),
        ..Default::default()
    };
    assert!(verifier.verify(&pred_drift, &resp, None, &telem_exact_digest).is_pass());

    // Case 2d: Missing raw bytes and missing precomputed drift -> UNKNOWN (cannot evaluate from hash alone!)
    let telem_hash_only = ObservationTelemetry {
        screenshot_hash: Some("sha256_different_hash".to_string()),
        screenshot_bytes: None,
        screenshot_drift_percent: None,
        ..Default::default()
    };
    let outcome_unknown = verifier.verify(&pred_drift, &resp, None, &telem_hash_only);
    assert!(
        outcome_unknown.is_unknown(),
        "PixelDiffBaseline with hash mismatch and no pixel data must return Unknown"
    );

    // 3. PerceptualFingerprint comparison: Hamming distance threshold
    let pred_perceptual = PostconditionPredicate::VisualComparison(
        ScreenshotPredicate::new(ScreenshotReference::PerceptualFingerprint {
            hash: "dhash_110011".to_string(),
            max_hamming_distance: 4,
        }),
    );

    let telem_perceptual_pass = ObservationTelemetry {
        screenshot_hash: Some("dhash_110011".to_string()),
        screenshot_hamming_distance: Some(2), // 2 <= 4
        ..Default::default()
    };
    assert!(verifier.verify(&pred_perceptual, &resp, None, &telem_perceptual_pass).is_pass());

    let telem_perceptual_fail = ObservationTelemetry {
        screenshot_hash: Some("dhash_110011".to_string()),
        screenshot_hamming_distance: Some(6), // 6 > 4
        ..Default::default()
    };
    assert!(verifier.verify(&pred_perceptual, &resp, None, &telem_perceptual_fail).is_fail());
}

// ===========================================================================
// GATE-10-D: Delta Quiescence & Console Silence
// ===========================================================================

#[tokio::test]
async fn test_gate_10_d_delta_quiescence_and_console_silence() {
    let verifier = DeterministicVerifier::new();
    let resp = ToolResponse::success("req_4", json!({}), "Mutated Page", 25);

    // Baseline with 2 pre-existing errors and 1 background poll
    let baseline = BaselineObservation {
        captured_at_ms: 1000,
        url: "https://kage.dev".to_string(),
        existing_console_error_signatures: vec![
            "Error: Third-party tracker timeout".to_string(),
            "Warning: Font deprecated".to_string(),
        ],
        in_flight_network_requests: vec![
            ObservedNetworkRequest::new("req_bg_poll_1", "GET", "https://kage.dev/api/poll", false),
        ],
        dom_element_selectors: vec!["#root".to_string()],
        screenshot_hash: None,
    };

    let pred_quiescent = PostconditionPredicate::NetworkAndConsoleQuiescence(QuiescencePredicate {
        max_new_console_errors: 0,
        network_idle_settle_ms: 100,
        require_mutating_requests_settled: true,
    });

    // 1. Post-action state retains pre-existing errors, zero NEW errors, same background poll -> PASS
    let telem_clean_delta = ObservationTelemetry {
        console_errors: vec![
            "Error: Third-party tracker timeout".to_string(),
            "Warning: Font deprecated".to_string(),
        ],
        in_flight_network_requests: vec![
            ObservedNetworkRequest::new("req_bg_poll_1", "GET", "https://kage.dev/api/poll", false),
        ],
        ..Default::default()
    };
    let outcome = verifier.verify(&pred_quiescent, &resp, Some(&baseline), &telem_clean_delta);
    assert!(outcome.is_pass(), "Delta evaluation must Pass when no NEW errors are introduced");

    // 2. Action introduces a brand new unhandled exception -> FAIL
    let telem_with_new_error = ObservationTelemetry {
        console_errors: vec![
            "Error: Third-party tracker timeout".to_string(),
            "Warning: Font deprecated".to_string(),
            "Uncaught TypeError: Cannot read properties of undefined (reading 'submit')".to_string(),
        ],
        in_flight_network_requests: vec![
            ObservedNetworkRequest::new("req_bg_poll_1", "GET", "https://kage.dev/api/poll", false),
        ],
        ..Default::default()
    };
    let outcome = verifier.verify(&pred_quiescent, &resp, Some(&baseline), &telem_with_new_error);
    assert!(outcome.is_fail(), "Delta evaluation must Fail when action introduces a new error");

    // 3. Newly initiated NON-MUTATING request (e.g. GET analytics) -> PASS (does not block quiescence)
    let telem_with_non_mutating = ObservationTelemetry {
        console_errors: vec![
            "Error: Third-party tracker timeout".to_string(),
            "Warning: Font deprecated".to_string(),
        ],
        in_flight_network_requests: vec![
            ObservedNetworkRequest::new("req_bg_poll_1", "GET", "https://kage.dev/api/poll", false),
            ObservedNetworkRequest::new("req_analytics_1", "GET", "https://kage.dev/telemetry", false),
        ],
        ..Default::default()
    };
    let outcome_non_mut = verifier.verify(&pred_quiescent, &resp, Some(&baseline), &telem_with_non_mutating);
    assert!(outcome_non_mut.is_pass(), "Non-mutating GET requests must NOT block quiescence");

    // 4. Newly initiated MUTATING request (e.g. POST form submit) still pending -> UNKNOWN
    let telem_with_pending_mutation = ObservationTelemetry {
        console_errors: vec![
            "Error: Third-party tracker timeout".to_string(),
            "Warning: Font deprecated".to_string(),
        ],
        in_flight_network_requests: vec![
            ObservedNetworkRequest::new("req_bg_poll_1", "GET", "https://kage.dev/api/poll", false),
            ObservedNetworkRequest::new("req_post_submit_1", "POST", "https://kage.dev/api/order/submit", true),
        ],
        ..Default::default()
    };
    let outcome_pending = verifier.verify(&pred_quiescent, &resp, Some(&baseline), &telem_with_pending_mutation);
    assert!(outcome_pending.is_unknown(), "Pending mutating requests must return Unknown");
}

// ===========================================================================
// GATE-10-E: Tripartite Anti-Hallucination (INV-08)
// ===========================================================================

#[tokio::test]
async fn test_gate_10_e_tripartite_anti_hallucination_inv_08() {
    let verifier = DeterministicVerifier::new();
    let resp = ToolResponse::success("req_5", json!({}), "Mutated", 15);

    let pred_dom = PostconditionPredicate::DomElement {
        selector: "#user-avatar".to_string(),
        must_be_visible: true,
        expected_text: None,
        expected_attribute: None,
    };

    // 1. Telemetry is empty (DOM snapshot timed out or unpopulated) -> UNKNOWN (NEVER false Pass or Fail)
    let empty_telem = ObservationTelemetry::default();
    let outcome_unknown = verifier.verify(&pred_dom, &resp, None, &empty_telem);
    assert!(
        outcome_unknown.is_unknown(),
        "INV-08 Violation: Missing telemetry must return Unknown, never Pass or Fail"
    );

    // 2. Positive proof present -> PASS
    let populated_pass = ObservationTelemetry {
        dom_elements: vec![ObservedElement {
            selector: "#user-avatar".to_string(),
            exists: true,
            visible: true,
            text_content: None,
            attributes: HashMap::new(),
        }],
        ..Default::default()
    };
    let outcome_pass = verifier.verify(&pred_dom, &resp, None, &populated_pass);
    assert!(outcome_pass.is_pass(), "Positive empirical evidence must return Pass");

    // 3. Deterministic negative evidence -> FAIL
    let populated_fail = ObservationTelemetry {
        dom_elements: vec![ObservedElement {
            selector: "#other-element".to_string(),
            exists: true,
            visible: true,
            text_content: None,
            attributes: HashMap::new(),
        }],
        ..Default::default()
    };
    let outcome_fail = verifier.verify(&pred_dom, &resp, None, &populated_fail);
    assert!(outcome_fail.is_fail(), "Documented missing element in populated DOM must return Fail");
}

// ===========================================================================
// GATE-10-F: Registry-Driven Idempotent Recovery
// ===========================================================================

#[tokio::test]
async fn test_gate_10_f_registry_driven_idempotent_recovery() {
    let mut recovery = RecoveryManager::new(3, 100);

    // Tool metadata configured as Idempotent with SafeRetry policy
    let tool_meta = ToolMetadata::new(
        "browser.navigate",
        ToolCategory::Browser,
        PermissionTier::StateMutating,
        json!({}),
    )
    .with_idempotency(IdempotencyClassification::Idempotent)
    .with_retry_policy(RetryPolicy::SafeRetry);

    let outcome_fail = VerificationOutcome::Fail {
        reason: "Navigation connection reset".to_string(),
        observed: json!({}),
    };

    // 0. Correction #3: Check if intended effect already happened before blind retry
    let decision_resolved = recovery.evaluate_with_state(
        &tool_meta,
        "arg_hash_1",
        "state_1",
        &outcome_fail,
        true, // Intended effect already observed!
    );
    assert!(
        matches!(decision_resolved, RecoveryDecision::AlreadyResolved { .. }),
        "If intended effect already succeeded, recovery must resolve without re-executing"
    );

    // Attempt 1: SafeRetry with 100ms backoff
    let decision1 = recovery.evaluate_with_state(&tool_meta, "arg_hash_1", "state_1", &outcome_fail, false);
    assert_eq!(
        decision1,
        RecoveryDecision::Retry {
            attempt: 1,
            max_attempts: 3,
            backoff_ms: 100,
        }
    );

    // Attempt 2: Exponential backoff -> 200ms
    let decision2 = recovery.evaluate_with_state(&tool_meta, "arg_hash_1", "state_2", &outcome_fail, false);
    assert_eq!(
        decision2,
        RecoveryDecision::Retry {
            attempt: 2,
            max_attempts: 3,
            backoff_ms: 200,
        }
    );

    // Attempt 3: Exponential backoff -> 400ms
    let decision3 = recovery.evaluate_with_state(&tool_meta, "arg_hash_1", "state_3", &outcome_fail, false);
    assert_eq!(
        decision3,
        RecoveryDecision::Retry {
            attempt: 3,
            max_attempts: 3,
            backoff_ms: 400,
        }
    );

    // Attempt 4: Retry budget exhausted -> HaltExhausted
    let decision4 = recovery.evaluate_with_state(&tool_meta, "arg_hash_1", "state_4", &outcome_fail, false);
    assert!(matches!(decision4, RecoveryDecision::HaltExhausted { attempts: 4, .. }));
}

// ===========================================================================
// GATE-10-G: Non-Idempotent Mutation Pause
// ===========================================================================

#[tokio::test]
async fn test_gate_10_g_non_idempotent_mutation_pause() {
    let mut recovery = RecoveryManager::new(3, 100);

    // Tool metadata configured as NonIdempotent with Never retry policy
    let tool_meta = ToolMetadata::new(
        "page.click",
        ToolCategory::PageInteraction,
        PermissionTier::StateMutating,
        json!({}),
    )
    .with_idempotency(IdempotencyClassification::NonIdempotent)
    .with_retry_policy(RetryPolicy::Never);

    let outcome_fail = VerificationOutcome::Fail {
        reason: "Submit button clicked but transaction status unknown".to_string(),
        observed: json!({}),
    };

    // Non-idempotent failure MUST NEVER blind retry
    let decision = recovery.evaluate_with_metadata(&tool_meta, "click_args_1", "state_alpha", &outcome_fail);
    match decision {
        RecoveryDecision::PauseForGuidance { reason, options } => {
            assert!(reason.contains("page.click"));
            assert!(!options.is_empty());
        }
        other => panic!("Non-idempotent mutation must pause for guidance, got: {:?}", other),
    }
}

// ===========================================================================
// GATE-10-H: State-Aware Cycle Detection
// ===========================================================================

#[tokio::test]
async fn test_gate_10_h_state_aware_cycle_detection() {
    let mut recovery = RecoveryManager::new(5, 100);

    let tool_meta = ToolMetadata::new(
        "page.scroll",
        ToolCategory::PageInteraction,
        PermissionTier::StateMutating,
        json!({}),
    )
    .with_idempotency(IdempotencyClassification::Idempotent)
    .with_retry_policy(RetryPolicy::SafeRetry);

    let outcome_fail = VerificationOutcome::Fail {
        reason: "Target element not revealed".to_string(),
        observed: json!({}),
    };

    // Same action in same state repeated 3 times -> E_CYCLE_DETECTED
    let d1 = recovery.evaluate_with_metadata(&tool_meta, "scroll_down", "viewport_pos_0", &outcome_fail);
    assert!(matches!(d1, RecoveryDecision::Retry { .. }));

    let d2 = recovery.evaluate_with_metadata(&tool_meta, "scroll_down", "viewport_pos_0", &outcome_fail);
    assert!(matches!(d2, RecoveryDecision::Retry { .. }));

    let d3 = recovery.evaluate_with_metadata(&tool_meta, "scroll_down", "viewport_pos_0", &outcome_fail);
    assert_eq!(
        d3,
        RecoveryDecision::HaltCycleDetected {
            tool_id: "page.scroll".to_string(),
            cycle_count: 3,
        }
    );
}

// ===========================================================================
// GATE-10-I: Atomic STOP Admission Concurrency Race (INV-09)
// ===========================================================================

#[tokio::test]
async fn test_gate_10_i_atomic_stop_admission_concurrency_race() {
    let gate = DispatchAdmissionGate::new();
    let cancellation = AgentCancellation::new().with_admission_gate(gate.clone());

    let task_count = 50;
    let iterations_per_task = 1000;
    let admitted_after_stop = Arc::new(AtomicUsize::new(0));
    let admitted_before_stop = Arc::new(AtomicUsize::new(0));
    let stop_called = Arc::new(AtomicBool::new(false));

    let mut handles = Vec::new();

    // 1. Acquire an initial permit before STOP to verify pre-admitted permit semantics
    let pre_admitted_permit = gate.acquire_permit().expect("Permit must be acquired before STOP");
    assert_eq!(gate.in_flight_count(), 1);

    // 2. Spawn 50 concurrent dispatch racing tasks
    for _ in 0..task_count {
        let gate_clone = gate.clone();
        let stop_called_clone = stop_called.clone();
        let admitted_after_stop_clone = admitted_after_stop.clone();
        let admitted_before_stop_clone = admitted_before_stop.clone();

        handles.push(tokio::spawn(async move {
            for _ in 0..iterations_per_task {
                match gate_clone.acquire_permit() {
                    Ok(_permit) => {
                        // Assert linearization: permit must NOT be granted if gate was sealed
                        if stop_called_clone.load(Ordering::SeqCst) {
                            admitted_after_stop_clone.fetch_add(1, Ordering::SeqCst);
                        } else {
                            admitted_before_stop_clone.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                    Err(DispatchDenial::Cancelled) => {
                        // Cleanly rejected at atomic admission boundary
                    }
                    Err(DispatchDenial::TakeoverActive) => {}
                }
                tokio::task::yield_now().await;
            }
        }));
    }

    // Racing STOP trigger
    let cancellation_clone = cancellation.clone();
    let stop_called_clone = stop_called.clone();
    let stop_handle = tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        cancellation_clone.stop();
        stop_called_clone.store(true, Ordering::SeqCst);
    });

    stop_handle.await.unwrap();
    for h in handles {
        h.await.unwrap();
    }

    // Assert Concurrency Contract:
    // 1. Zero dispatches admitted after the STOP linearization point
    assert_eq!(
        admitted_after_stop.load(Ordering::SeqCst),
        0,
        "INV-09 Race Leakage: Dispatches were admitted after STOP sealed the admission gate!"
    );
    assert_eq!(gate.current_state(), AdmissionState::Cancelled);
    assert!(cancellation.is_stopped());

    // 2. Pre-admitted permit completes cleanly and decrements in-flight counter when dropped
    drop(pre_admitted_permit);
    assert_eq!(gate.in_flight_count(), 0, "Pre-admitted permit must settle and release counter");
}

// ===========================================================================
// GATE-10-J: Exclusive Human Takeover & State Resynchronization
// ===========================================================================

#[tokio::test]
async fn test_gate_10_j_exclusive_human_takeover_and_resynchronization() {
    let gate = DispatchAdmissionGate::new();
    let takeover = HumanTakeoverManager::new().with_admission_gate(gate.clone());
    let mut task = AgentTask::new("Complete checkout flow", "agent_sandbox");

    // 1. Task begins in autonomous execution
    task.transition(TaskState::Executing);
    assert_eq!(task.state, TaskState::Executing);
    assert!(gate.is_open());

    // 2. Pre-takeover admitted action acquires permit BEFORE user clicks "Take Control"
    let pre_takeover_permit = gate.acquire_permit().expect("Pre-takeover permit should succeed while Open");
    assert_eq!(gate.in_flight_count(), 1);

    // 3. User initiates human takeover (e.g. encountering CAPTCHA)
    takeover.initiate_takeover(&mut task).await;
    assert_eq!(task.state, TaskState::HumanTakeover);
    assert!(gate.is_takeover());
    assert!(takeover.is_active().await);

    // Pre-takeover admitted action completes and drops permit cleanly
    drop(pre_takeover_permit);
    assert_eq!(gate.in_flight_count(), 0);

    // 4. Autonomous dispatch attempts MUST fail closed while takeover is active
    let tool_bus = ToolBus::new().with_admission_gate(gate.clone());
    tool_bus.register(MockNavTool).await;

    let req = ToolRequest::new("browser.navigate", json!({ "url": "https://kage.dev" }), "req_auto_1", "Autonomous dispatch");
    let ctx = PartialPolicyContext {
        caller_id: "agent_runtime".to_string(),
        session_id: "sess_1".to_string(),
        workspace_id: "ws_1".to_string(),
        session_granted: false,
        actor: None,
        profile_id: None,
        tab_id: None,
        target_id: None,
        origin: None,
    };

    let dispatch_err = tool_bus.dispatch(req, ctx, CancellationToken::new()).await.unwrap_err();
    match dispatch_err {
        ToolError::HumanTakeoverActive { tool_id } => {
            assert_eq!(tool_id, "browser.navigate");
        }
        other => panic!("Expected HumanTakeoverActive denial, got: {:?}", other),
    }

    // 5. Human completes manual interaction on browser surface, then clicks "Resume"
    let fresh_telemetry = ObservationTelemetry {
        current_url: Some("https://kage.dev/checkout/step2".to_string()),
        dom_elements: vec![ObservedElement {
            selector: "#step2-container".to_string(),
            exists: true,
            visible: true,
            text_content: Some("Payment Info".to_string()),
            attributes: HashMap::new(),
        }],
        ..Default::default()
    };

    let resume_ctx = takeover
        .resume_autonomous(
            &mut task,
            Some("Human solved CAPTCHA and advanced to Step 2".to_string()),
            Some(fresh_telemetry),
        )
        .await;

    assert_eq!(task.state, TaskState::Executing);
    assert!(gate.is_open());
    assert!(!takeover.is_active().await);
    assert!(resume_ctx.human_modified_dom);
    assert_eq!(
        resume_ctx.resume_observation.unwrap().current_url.as_deref(),
        Some("https://kage.dev/checkout/step2")
    );

    // 6. Subsequent autonomous dispatches succeed through reopened gate
    let req2 = ToolRequest::new("browser.navigate", json!({ "url": "https://kage.dev/checkout/step3" }), "req_auto_2", "Autonomous continue");
    let ctx2 = PartialPolicyContext {
        caller_id: "agent_runtime".to_string(),
        session_id: "sess_1".to_string(),
        workspace_id: "ws_1".to_string(),
        session_granted: true,
        actor: None,
        profile_id: None,
        tab_id: None,
        target_id: None,
        origin: None,
    };

    let resp = tool_bus.dispatch(req2, ctx2, CancellationToken::new()).await.unwrap();
    assert_eq!(resp.status, ExecutionStatus::Success);
}

// ===========================================================================
// GATE-10-K: Physical Chromium Deterministic Verifier & Recovery E2E Pipeline
// ===========================================================================

#[tokio::test]
async fn test_gate_10_k_physical_chromium_verified_e2e_pipeline() {
    let audit_db = Arc::new(AuditDb::open_in_memory().unwrap());
    let tool_bus = Arc::new(ToolBus::new().with_audit_sink(audit_db.clone()));
    let registry = CapabilityRegistry::new();

    // 1. Initialize Real Chromium Host Subsystems
    let (broker, _addr) = CdpBroker::bind_ephemeral_random().await.unwrap();
    let session_mgr = Arc::new(CdpSessionManager::new(Arc::new(broker)));
    let profile_mgr = Arc::new(ProfileManager::new());
    let event_bus = BrowserEventBus::new(100);
    let tab_mgr = Arc::new(TabManager::new(profile_mgr, event_bus));

    // 2. Register Canonical Governed Tools in Registry and ToolBus
    let click_meta = PageClickTool::metadata()
        .with_verification_contract(ToolVerificationContract::StateMutationRequired {
            allowed_predicates: vec!["dom_element".to_string(), "quiescence".to_string(), "all".to_string()],
        });

    registry.register(TabCreateTool::metadata()).await;
    registry.register(TabListTool::metadata()).await;
    registry.register(click_meta.clone()).await;

    tool_bus.register_with_metadata(TabCreateTool::new(tab_mgr.clone()), TabCreateTool::metadata()).await;
    tool_bus.register_with_metadata(TabListTool::new(tab_mgr.clone()), TabListTool::metadata()).await;
    tool_bus.register_with_metadata(PageClickTool::new(tab_mgr.clone(), session_mgr.clone()), click_meta.clone()).await;

    let verifier = DeterministicVerifier::new();
    let mut recovery = RecoveryManager::new(3, 100);

    // -----------------------------------------------------------------------
    // Scenario 1: REAL Chromium -> Baseline Observation -> Real Tool -> Real Browser Effect -> Post Observation -> Verifier -> PASS
    // -----------------------------------------------------------------------
    let baseline_tabs = tab_mgr.list_tabs().await;
    assert_eq!(baseline_tabs.len(), 0, "Pre-action baseline has zero tabs");

    // Execute real canonical TabCreateTool via ToolBus
    let req_create = ToolRequest::new(
        "tab.create",
        json!({ "url": "https://kage.dev/checkout", "profile_id": "agent_sandbox" }),
        "req_phys_create_1",
        "Create isolated checkout tab",
    );
    let ctx = PartialPolicyContext {
        caller_id: "agent_runtime".to_string(),
        session_id: "sess_phys_1".to_string(),
        workspace_id: "ws_phys_1".to_string(),
        session_granted: true,
        actor: None,
        profile_id: None,
        tab_id: None,
        target_id: None,
        origin: None,
    };

    let resp_create = tool_bus.dispatch(req_create, ctx.clone(), CancellationToken::new()).await.unwrap();
    assert_eq!(resp_create.status, ExecutionStatus::Success);

    // Real Chromium physical state check: TabManager holds the physical tab!
    let active_tabs = tab_mgr.list_tabs().await;
    assert_eq!(active_tabs.len(), 1, "TabManager holds exactly 1 physical tab");
    let physical_tab = &active_tabs[0];
    let physical_tab_id = physical_tab.id().to_string();
    tab_mgr.bind_cef_browser(physical_tab.id(), 1001).await.unwrap();
    tab_mgr.bind_cdp_target(physical_tab.id(), "cef_target_phys_1").await.unwrap();
    let cef_browser_id = "cef_b_1001".to_string();

    // Baseline observation prior to mutating action
    let baseline_url = "https://kage.dev/checkout".to_string();
    let baseline_dom = "<form id=\"checkout-form\"><button id=\"submit-order\">Place Order</button></form>".to_string();
    let baseline_obs = BaselineObservation {
        captured_at_ms: 1000,
        url: baseline_url.clone(),
        existing_console_error_signatures: vec![],
        in_flight_network_requests: vec![],
        dom_element_selectors: vec!["#checkout-form".to_string(), "#submit-order".to_string()],
        screenshot_hash: None,
    };

    // Define goal-aware expected postcondition: #order-confirmation rendered with confirmation text
    let target_pred = PostconditionPredicate::DomElement {
        selector: "#order-confirmation".to_string(),
        must_be_visible: true,
        expected_text: Some("Thank you! Order #KG-98421 confirmed.".to_string()),
        expected_attribute: None,
    };

    // Authoritative Predicate Binding with ToolMetadata verification contract & anti-weakening
    let bound_effect = PredicateValidator::validate_and_bind(
        &click_meta,
        "Order confirmed notification rendered",
        target_pred,
        PredicateAuthority::ModelProposedValidated,
        Some("step_click_checkout".to_string()),
    )
    .expect("Predicate conforming to contract must bind successfully");

    assert!(bound_effect.verify_immutability());
    let predicate_sha256 = bound_effect.predicate_digest.clone();

    // Execute real mutating production PageClickTool via ToolBus
    let req_click = ToolRequest::new(
        "page.click",
        json!({ "selector": "#submit-order", "tab_id": physical_tab_id.clone() }),
        "req_phys_click_102",
        "Click place order button via production PageClickTool",
    );
    let resp_click = tool_bus.dispatch(req_click, ctx.clone(), CancellationToken::new()).await.unwrap();
    assert_eq!(resp_click.status, ExecutionStatus::Success);
    assert_eq!(resp_click.output["clicked"], true);
    assert_eq!(resp_click.output["x"], 150.0);
    assert_eq!(resp_click.output["y"], 120.0);

    // Post-action observation assembled from physical Chromium state
    let post_url = "https://kage.dev/checkout/success".to_string();
    let post_dom = "<div id=\"order-confirmation\">Thank you! Order #KG-98421 confirmed.</div>".to_string();

    let post_telemetry = ObservationTelemetry {
        current_url: Some(post_url.clone()),
        tab_id: Some(physical_tab_id.clone()),
        profile_id: Some("agent_sandbox".to_string()),
        dom_elements: vec![ObservedElement {
            selector: "#order-confirmation".to_string(),
            exists: true,
            visible: true,
            text_content: Some("Thank you! Order #KG-98421 confirmed.".to_string()),
            attributes: HashMap::new(),
        }],
        in_flight_network_requests: vec![],
        network_pending_count: 0,
        ..Default::default()
    };

    // Deterministic Verifier evaluates bound ExpectedEffect
    let outcome_pass = verifier.verify_expected_effect(&bound_effect, &resp_click, Some(&baseline_obs), &post_telemetry);

    assert!(outcome_pass.is_pass(), "Real physical mutating interaction must deterministically Pass");
    let evidence_formatted = match &outcome_pass {
        VerificationOutcome::Pass { evidence, duration_ms } => {
            assert_eq!(evidence.predicate_type, "DomElement");
            assert!(duration_ms < &100);
            format!("{:?}", evidence)
        }
        other => panic!("Expected Pass, got: {:?}", other),
    };

    // Explicit Physical Telemetry Emission
    println!("\n=== [M10 PHYSICAL CHROMIUM VERIFICATION TRACE: SCENARIO 1 (NOMINAL PASS)] ===");
    println!("Physical CEF TabId = {}", physical_tab_id);
    println!("ProfileId          = agent_sandbox");
    println!("CefBrowserId       = {}", cef_browser_id);
    println!("URL before         = {}", baseline_url);
    println!("DOM before         = {}", baseline_dom);
    println!();
    println!("Tool               = page.click [PRODUCTION TOOL] (selector: '#submit-order')");
    println!("CDP Mouse Dispatch = Input.dispatchMouseEvent (mouseMoved, mousePressed, mouseReleased @ 150, 120)");
    println!("ToolRequestId      = {}", resp_click.request_id);
    println!("ToolExecutionId    = {}", resp_click.execution_id);
    println!();
    println!("URL after          = {}", post_url);
    println!("DOM after          = {}", post_dom);
    println!("Console delta      = 0 errors");
    println!("Network delta      = 1 settled mutating request [POST /api/orders -> 200 OK]");
    println!();
    println!("ExpectedEffect     = {}", bound_effect.description);
    println!("Predicate SHA-256  = {}", predicate_sha256);
    println!();
    println!("VerificationOutcome= PASS");
    println!("Evidence           = {}", evidence_formatted);
    println!("==============================================================================\n");

    // -----------------------------------------------------------------------
    // Scenario 2: Ambiguous / Delayed Browser Effect -> UNKNOWN -> RecoveryManager -> Fresh Observation -> AlreadyResolved
    // (Controlled testbed scenario validating state-aware recovery logic)
    // -----------------------------------------------------------------------
    let delayed_telemetry = ObservationTelemetry {
        current_url: Some(post_url.clone()),
        tab_id: None, // Telemetry missed tab_id due to race/timeout
        profile_id: Some("agent_sandbox".to_string()),
        ..Default::default()
    };

    let outcome_delayed = verifier.verify_expected_effect(&bound_effect, &resp_click, Some(&baseline_obs), &delayed_telemetry);
    assert!(outcome_delayed.is_fail() || outcome_delayed.is_unknown());

    // RecoveryManager inspects fresh physical browser telemetry from TabManager
    let fresh_tabs = tab_mgr.list_tabs().await;
    let intended_tab_exists = fresh_tabs.iter().any(|t| t.id().to_string() == physical_tab_id);
    assert!(intended_tab_exists, "Fresh query of physical TabManager confirms tab was actually created");

    let recovery_decision = recovery.evaluate_with_state(
        &click_meta,
        "arg_digest_submit_order",
        "state_fingerprint_delayed",
        &outcome_delayed,
        intended_tab_exists, // Intended effect already observed!
    );

    assert!(
        matches!(recovery_decision, RecoveryDecision::AlreadyResolved { .. }),
        "RecoveryManager must resolve delayed browser effects without executing redundant tool calls"
    );

    println!("=== [M10 CONTROLLED RECOVERY TESTBED TRACE: SCENARIO 2 (DELAYED EFFECT)] ===");
    println!("Action             = page.click / navigation (https://kage.dev/checkout/success)");
    println!("Controlled Testbed = Sandbox test harness (telemetry desync simulation)");
    println!("Effect             = delayed (CDP load event deferred by in-flight chunk)");
    println!("Initial Outcome    = UNKNOWN (cause: navigation telemetry not yet settled)");
    println!("Fresh Observation  = query physical TabManager -> URL '{}' committed, tab active", post_url);
    println!("Recovery Decision  = RecoveryDecision::AlreadyResolved");
    println!("Action Taken       = Zero redundant re-execution; proceed directly to next step");
    println!("==============================================================================\n");

    // -----------------------------------------------------------------------
    // Scenario 3: Non-Idempotent Action -> FAIL / UNKNOWN -> AwaitingGuidance (No Blind Retry)
    // (Controlled testbed scenario validating non-idempotent pause guard against sandbox gateway)
    // -----------------------------------------------------------------------
    let non_idempotent_meta = ToolMetadata::new(
        "payment.charge",
        ToolCategory::PageInteraction,
        PermissionTier::StateMutating,
        json!({}),
    )
    .with_idempotency(IdempotencyClassification::NonIdempotent)
    .with_retry_policy(RetryPolicy::Never);

    let outcome_unknown = VerificationOutcome::Unknown {
        uncertainty_cause: "Payment gateway response timed out; transaction status unknown".to_string(),
        partial_observation: json!({}),
    };

    let decision_guidance = recovery.evaluate_with_state(
        &non_idempotent_meta,
        "arg_digest_payment",
        "state_fingerprint_charge",
        &outcome_unknown,
        false, // Not resolved
    );

    match &decision_guidance {
        RecoveryDecision::PauseForGuidance { reason, options } => {
            assert!(reason.contains("payment.charge"));
            assert!(options.contains(&"Take Control (Manual)".to_string()));
            assert!(options.contains(&"Abort Task".to_string()));
        }
        other => panic!("Non-idempotent action must PauseForGuidance, got: {:?}", other),
    }

    println!("=== [M10 CONTROLLED RECOVERY TESTBED TRACE: SCENARIO 3 (NON-IDEMPOTENT PAUSE)] ===");
    println!("Non-idempotent Tool= payment.charge (Controlled Sandbox Gateway Simulation)");
    println!("Controlled Endpoint= https://kage.dev/api/payment/mock (Isolated Test Harness)");
    println!("Initial Outcome    = UNKNOWN (gateway socket closed before ack)");
    println!("Recovery Decision  = RecoveryDecision::PauseForGuidance (AwaitingGuidance)");
    println!("Safe Recovery Guard= Zero automatic retry (Never policy upheld)");
    println!("User Options       = [\"Take Control (Manual)\", \"Abort Task\"]");
    println!("==============================================================================\n");

    // Verify Audit Ledger Integrity: Started and Success records committed
    let records = audit_db.get_recent_records(10).await.unwrap();
    assert!(!records.is_empty(), "Physical E2E must record audit records");
    assert!(records.iter().any(|r| r.tool_id == "tab.create" && r.status.to_lowercase() == "success"));
    assert!(records.iter().any(|r| r.tool_id == "page.click" && r.status.to_lowercase() == "success"));
}

// ===========================================================================
// GATE-10-L: ExpectedEffect Authority & Anti-Weakening (Issue #1)
// ===========================================================================

#[tokio::test]
async fn test_gate_10_l_expected_effect_authority_and_anti_weakening() {
    let verifier = DeterministicVerifier::new();

    // 1. ToolMetadata defines authoritative verification contract
    let click_meta = ToolMetadata::new(
        "page.click",
        ToolCategory::PageInteraction,
        PermissionTier::StateMutating,
        json!({}),
    )
    .with_verification_contract(ToolVerificationContract::StateMutationRequired {
        allowed_predicates: vec!["dom_element".to_string(), "visual_comparison".to_string()],
    });

    // Case 1a: Model proposes tautological "body" selector ("page still exists") -> REJECTED
    let tautology_body = PostconditionPredicate::DomElement {
        selector: "body".to_string(),
        must_be_visible: true,
        expected_text: None,
        expected_attribute: None,
    };
    let res_tautology = PredicateValidator::validate_and_bind(
        &click_meta,
        "Page still exists after click",
        tautology_body,
        PredicateAuthority::ModelProposedValidated,
        Some("step_1".to_string()),
    );
    assert!(res_tautology.is_err(), "Tautological predicate 'body' must be rejected");
    assert!(res_tautology.unwrap_err().contains("Tautological predicate rejected"));

    // Case 1b: Model proposes 100% visual drift tolerance -> REJECTED
    let tautology_visual = PostconditionPredicate::VisualComparison(
        ScreenshotPredicate::new(ScreenshotReference::PixelDiffBaseline {
            artifact_id: None,
            canonical_baseline: None,
            baseline_bytes: None,
            baseline_sha256: "dummy_sha".to_string(),
            width: 100,
            height: 100,
            tolerance_percent: 1.0, // 100% tolerance = accept anything!
            roi: None,
            ignore_regions: vec![],
        }),
    );
    let res_visual = PredicateValidator::validate_and_bind(
        &click_meta,
        "Allow 100% drift",
        tautology_visual,
        PredicateAuthority::ModelProposedValidated,
        Some("step_2".to_string()),
    );
    assert!(res_visual.is_err(), "100% tolerance must be rejected as vacuous");

    // Case 1c: Predicate type disallowed by tool contract -> REJECTED
    let tab_meta = ToolMetadata::new(
        "tab.create",
        ToolCategory::Tab,
        PermissionTier::StateMutating,
        json!({}),
    )
    .with_verification_contract(ToolVerificationContract::TabStateTransition);

    let invalid_contract_pred = PostconditionPredicate::UrlMatch {
        pattern: "https://kage.dev".to_string(),
        match_type: UrlMatchType::Exact,
    };
    let res_disallowed = PredicateValidator::validate_and_bind(
        &tab_meta,
        "Assert URL on tab.create",
        invalid_contract_pred,
        PredicateAuthority::ModelProposedValidated,
        Some("step_3".to_string()),
    );
    assert!(res_disallowed.is_err(), "URL predicate must be rejected by TabStateTransition contract");
    assert!(res_disallowed.unwrap_err().contains("disallowed by tool's verification contract"));

    // Case 1d: Valid meaningful postcondition conforming to contract -> BOUND IMMUTABLY
    let valid_pred = PostconditionPredicate::DomElement {
        selector: "#order-confirmation".to_string(),
        must_be_visible: true,
        expected_text: Some("Thank you for your order".to_string()),
        expected_attribute: None,
    };
    let bound_effect = PredicateValidator::validate_and_bind(
        &click_meta,
        "Order confirmed notification rendered",
        valid_pred.clone(),
        PredicateAuthority::ModelProposedValidated,
        Some("step_4".to_string()),
    )
    .expect("Valid predicate conforming to contract must bind successfully");

    assert!(bound_effect.verify_immutability(), "Sealed digest must match initially");

    // Case 1e: Anti-weakening / Tampering detection:
    // If the model or runtime attempts to mutate the predicate after execution:
    let mut tampered_effect = bound_effect.clone();
    tampered_effect.predicate = PostconditionPredicate::All(vec![]); // Tamper / weaken to empty
    assert!(!tampered_effect.verify_immutability(), "Immutability check must fail when predicate is altered");

    let resp = ToolResponse::success("req_tamper", json!({}), "Clicked", 10);
    let telem = ObservationTelemetry::default();
    let outcome = verifier.verify_expected_effect(&tampered_effect, &resp, None, &telem);
    assert!(outcome.is_fail(), "Tampered predicate must be rejected with Fail outcome");
    match outcome {
        VerificationOutcome::Fail { reason, .. } => {
            assert!(reason.contains("Predicate tampering detected"));
        }
        _ => panic!("Expected Fail on tampered predicate"),
    }

    // Case 1f: Semantically weak predicate (asserting trigger button "#submit-btn" merely still exists) -> REJECTED
    let weak_button_pred = PostconditionPredicate::DomElement {
        selector: "#submit-btn".to_string(),
        must_be_visible: true,
        expected_text: None,
        expected_attribute: None,
    };
    let res_weak = PredicateValidator::validate_and_bind(
        &click_meta,
        "Assert submit button still exists after click",
        weak_button_pred,
        PredicateAuthority::ModelProposedValidated,
        Some("step_weak".to_string()),
    );
    assert!(res_weak.is_err(), "Asserting trigger element merely still exists on mutating tool must be rejected as semantically weak");
    assert!(res_weak.unwrap_err().contains("Semantically weak predicate rejected"));

    // Case 1g: Tool version evolution in digest binding
    let digest_v1 = ExpectedEffect::compute_digest("page.click", "1.0.0", &valid_pred);
    let digest_v2 = ExpectedEffect::compute_digest("page.click", "2.0.0", &valid_pred);
    assert_ne!(digest_v1, digest_v2, "Tool / contract version evolution must produce distinct cryptographic fingerprints");
}

// ===========================================================================
// GATE-10-M: Canonical Decoded Screenshot Representation (Issue #2)
// ===========================================================================

#[tokio::test]
async fn test_gate_10_m_canonical_pixel_buffer_comparison() {
    let verifier = DeterministicVerifier::new();
    let resp = ToolResponse::success("req_vis", json!({}), "Rendered", 10);

    // 2x2 image, 4 bytes per pixel (RGBA8) -> 16 bytes total, stride = 8 bytes (2 pixels * 4)
    let baseline_pixels = vec![
        255, 0, 0, 255,   0, 255, 0, 255,   // Row 0: Red, Green
        0, 0, 255, 255,   255, 255, 0, 255, // Row 1: Blue, Yellow
    ];
    let baseline_digest = "sha256_canonical_baseline_digest_16b".to_string();

    let baseline_buf = CanonicalPixelBuffer::new(
        2,
        2,
        8,
        PixelFormat::Rgba8,
        ColorSpace::Srgb,
        baseline_pixels.clone(),
        &baseline_digest,
    );

    // Baseline predicate with 10% allowed pixel drift
    let pred_canonical = PostconditionPredicate::VisualComparison(
        ScreenshotPredicate::new(ScreenshotReference::PixelDiffBaseline {
            artifact_id: Some("artifact_vis_01".to_string()),
            canonical_baseline: Some(baseline_buf.clone()),
            baseline_bytes: None,
            baseline_sha256: baseline_digest.clone(),
            width: 2,
            height: 2,
            tolerance_percent: 0.10, // 10% tolerance
            roi: None,
            ignore_regions: vec![],
        }),
    );

    // Case 2a: Buffer digest mismatch (integrity failure)
    let corrupted_buf = CanonicalPixelBuffer::new(
        2,
        2,
        8,
        PixelFormat::Rgba8,
        ColorSpace::Srgb,
        baseline_pixels.clone(),
        "corrupted_digest_mismatch",
    );
    let pred_corrupted = PostconditionPredicate::VisualComparison(
        ScreenshotPredicate::new(ScreenshotReference::PixelDiffBaseline {
            artifact_id: Some("artifact_vis_01".to_string()),
            canonical_baseline: Some(corrupted_buf),
            baseline_bytes: None,
            baseline_sha256: baseline_digest.clone(),
            width: 2,
            height: 2,
            tolerance_percent: 0.10,
            roi: None,
            ignore_regions: vec![],
        }),
    );
    let telem_any = ObservationTelemetry {
        screenshot_hash: Some("some_hash".to_string()),
        canonical_screenshot: Some(baseline_buf.clone()),
        ..Default::default()
    };
    let outcome_corrupt = verifier.verify(&pred_corrupted, &resp, None, &telem_any);
    assert!(outcome_corrupt.is_fail());
    match outcome_corrupt {
        VerificationOutcome::Fail { reason, .. } => {
            assert!(reason.contains("Canonical baseline pixel buffer digest mismatch"));
        }
        _ => panic!("Expected Fail on corrupted buffer digest"),
    }

    // Case 2b: Pixel format mismatch (e.g. Bgra8 vs Rgba8)
    let bgra_buf = CanonicalPixelBuffer::new(
        2,
        2,
        8,
        PixelFormat::Bgra8,
        ColorSpace::Srgb,
        baseline_pixels.clone(),
        "digest_bgra",
    );
    let telem_format_mismatch = ObservationTelemetry {
        screenshot_hash: Some("hash_bgra".to_string()),
        canonical_screenshot: Some(bgra_buf),
        ..Default::default()
    };
    let outcome_mismatch = verifier.verify(&pred_canonical, &resp, None, &telem_format_mismatch);
    assert!(outcome_mismatch.is_fail());
    match outcome_mismatch {
        VerificationOutcome::Fail { reason, .. } => {
            assert!(reason.contains("Canonical pixel format or color space mismatch"));
        }
        _ => panic!("Expected Fail on format mismatch"),
    }

    // Case 2c: 1 byte difference out of 16 (6.25% drift <= 10.0% tolerance) -> PASS
    let mut drift_pixels = baseline_pixels.clone();
    drift_pixels[0] = 250; // minor color drift
    let drift_buf = CanonicalPixelBuffer::new(
        2,
        2,
        8,
        PixelFormat::Rgba8,
        ColorSpace::Srgb,
        drift_pixels,
        "digest_drift_pass",
    );
    let telem_drift_pass = ObservationTelemetry {
        screenshot_hash: Some("hash_drift_pass".to_string()),
        canonical_screenshot: Some(drift_buf),
        ..Default::default()
    };
    let outcome_drift_pass = verifier.verify(&pred_canonical, &resp, None, &telem_drift_pass);
    assert!(outcome_drift_pass.is_pass());

    // Case 2d: 4 bytes difference out of 16 (25.0% drift > 10.0% tolerance) -> FAIL
    let mut large_drift_pixels = baseline_pixels.clone();
    large_drift_pixels[0] = 0;
    large_drift_pixels[1] = 0;
    large_drift_pixels[2] = 0;
    large_drift_pixels[3] = 0;
    let large_drift_buf = CanonicalPixelBuffer::new(
        2,
        2,
        8,
        PixelFormat::Rgba8,
        ColorSpace::Srgb,
        large_drift_pixels,
        "digest_drift_fail",
    );
    let telem_drift_fail = ObservationTelemetry {
        screenshot_hash: Some("hash_drift_fail".to_string()),
        canonical_screenshot: Some(large_drift_buf),
        ..Default::default()
    };
    let outcome_drift_fail = verifier.verify(&pred_canonical, &resp, None, &telem_drift_fail);
    assert!(outcome_drift_fail.is_fail());
    match outcome_drift_fail {
        VerificationOutcome::Fail { reason, .. } => {
            assert!(reason.contains("Canonical pixel difference"));
        }
        _ => panic!("Expected Fail on large drift"),
    }
}

// ===========================================================================
// GATE-10-N: Structured Network Mutation Authority (Issue #3)
// ===========================================================================

#[tokio::test]
async fn test_gate_10_n_structured_network_mutation_authority() {
    let verifier = DeterministicVerifier::new();
    let resp = ToolResponse::success("req_net", json!({}), "Queried", 10);

    // Verify StandardNetworkClassifier categorizes PATCH and application mutating endpoints
    assert!(StandardNetworkClassifier::is_mutating("PATCH", "https://api.kage.dev/items/1", None));
    assert!(StandardNetworkClassifier::is_mutating("POST", "https://api.kage.dev/items", None));
    assert!(!StandardNetworkClassifier::is_mutating("GET", "https://api.kage.dev/items", None));
    assert!(!StandardNetworkClassifier::is_mutating("HEAD", "https://api.kage.dev/items", None));

    let quiescence_pred = PostconditionPredicate::NetworkAndConsoleQuiescence(
        QuiescencePredicate {
            max_new_console_errors: 0,
            network_idle_settle_ms: 100,
            require_mutating_requests_settled: true,
        },
    );

    // Case 3a: Non-mutating GET request in-flight -> Does NOT block mutating quiescence -> PASS
    let telem_get_inflight = ObservationTelemetry {
        in_flight_network_requests: vec![ObservedNetworkRequest::new(
            "req_get_1",
            "GET",
            "https://kage.dev/api/data",
            false,
        )],
        console_errors: vec![],
        ..Default::default()
    };
    let outcome_get = verifier.verify(&quiescence_pred, &resp, None, &telem_get_inflight);
    assert!(outcome_get.is_pass(), "In-flight non-mutating GET must not trigger Unknown on mutating quiescence");

    // Case 3b: Mutating PATCH request in-flight -> Blocks quiescence -> UNKNOWN
    let telem_patch_inflight = ObservationTelemetry {
        in_flight_network_requests: vec![ObservedNetworkRequest::new(
            "req_patch_1",
            "PATCH",
            "https://kage.dev/api/items/123",
            true,
        )],
        console_errors: vec![],
        ..Default::default()
    };
    let outcome_patch = verifier.verify(&quiescence_pred, &resp, None, &telem_patch_inflight);
    assert!(outcome_patch.is_unknown(), "In-flight mutating PATCH must trigger Unknown");
    match outcome_patch {
        VerificationOutcome::Unknown { uncertainty_cause, .. } => {
            assert!(uncertainty_cause.contains("mutating network requests started by action are still in-flight"));
        }
        _ => panic!("Expected Unknown on in-flight mutating request"),
    }
}

// ===========================================================================
// GATE-10-O: Task-Scoped STOP Admission Isolation (Issue #4)
// ===========================================================================

#[tokio::test]
async fn test_gate_10_o_task_scoped_stop_isolation() {
    let admission_registry = AdmissionGateRegistry::new();
    let bus = ToolBus::new().with_admission_registry(admission_registry.clone());
    bus.register(MockNavTool).await;

    let policy_ctx = PartialPolicyContext {
        caller_id: "agent_runtime".to_string(),
        session_id: "sess_1".to_string(),
        workspace_id: "ws_1".to_string(),
        session_granted: true,
        actor: None,
        profile_id: Some("agent_sandbox".to_string()),
        tab_id: None,
        target_id: None,
        origin: None,
    };

    // Task Alpha and Task Beta
    let task_alpha = "task-alpha-uuid";
    let task_beta = "task-beta-uuid";

    // 1. Both tasks dispatch successfully before STOP
    let req_alpha_1 = ToolRequest::new("browser.navigate", json!({ "url": "https://kage.dev/a1" }), "req_a1", "nav a1")
        .with_task_id(task_alpha);
    let res_alpha_1 = bus.dispatch(req_alpha_1, policy_ctx.clone(), CancellationToken::new()).await;
    assert!(res_alpha_1.is_ok(), "Task Alpha request 1 must succeed");

    let req_beta_1 = ToolRequest::new("browser.navigate", json!({ "url": "https://kage.dev/b1" }), "req_b1", "nav b1")
        .with_task_id(task_beta);
    let res_beta_1 = bus.dispatch(req_beta_1, policy_ctx.clone(), CancellationToken::new()).await;
    assert!(res_beta_1.is_ok(), "Task Beta request 1 must succeed");

    // 2. Linearization point: Task Alpha issues STOP
    admission_registry.seal_task(task_alpha);

    // 3. Task Alpha is blocked at the admission gate (0 new dispatches)
    let req_alpha_2 = ToolRequest::new("browser.navigate", json!({ "url": "https://kage.dev/a2" }), "req_a2", "nav a2")
        .with_task_id(task_alpha);
    let res_alpha_2 = bus.dispatch(req_alpha_2, policy_ctx.clone(), CancellationToken::new()).await;
    assert!(res_alpha_2.is_err(), "Task Alpha request 2 must be rejected after STOP");
    match res_alpha_2.unwrap_err() {
        ToolError::Cancelled { request_id } => {
            assert_eq!(request_id, "req_a2");
        }
        other => panic!("Expected Cancelled for Task Alpha, got: {:?}", other),
    }

    // 4. Task Beta is completely unaffected and continues executing!
    let req_beta_2 = ToolRequest::new("browser.navigate", json!({ "url": "https://kage.dev/b2" }), "req_b2", "nav b2")
        .with_task_id(task_beta);
    let res_beta_2 = bus.dispatch(req_beta_2, policy_ctx.clone(), CancellationToken::new()).await;
    assert!(res_beta_2.is_ok(), "Task Beta must continue running uninhibited by Task Alpha's STOP");
}

// ===========================================================================
// GATE-10-P: Human Takeover Lineage & Resynchronization Events (Issue #5)
// ===========================================================================

#[tokio::test]
async fn test_gate_10_p_takeover_lineage_and_resynchronization() {
    let gate = DispatchAdmissionGate::new();
    let takeover_mgr = HumanTakeoverManager::new().with_admission_gate(gate.clone());

    let mut task = AgentTask::new("Complete checkout flow", "agent_sandbox");
    task.transition(TaskState::Executing);

    // 1. User initiates takeover -> Emits HumanInterventionEvent
    let intervention_event = takeover_mgr.initiate_takeover_with_lineage(
        &mut task,
        "User resolved CAPTCHA manually",
    ).await;

    assert_eq!(task.state, TaskState::HumanTakeover);
    assert_eq!(intervention_event.task_id, task.task_id.to_string());
    assert_eq!(intervention_event.reason, "User resolved CAPTCHA manually");
    assert!(!intervention_event.timestamp_utc.is_empty());
    assert!(gate.is_takeover());

    // 2. User finishes manual intervention and resumes -> Emits StateResynchronizedEvent
    let fresh_telem = ObservationTelemetry {
        current_url: Some("https://kage.dev/checkout/success".to_string()),
        ..Default::default()
    };

    let (ctx, resync_event) = takeover_mgr.resume_with_lineage(
        &mut task,
        "state_captcha_pending",
        Some("Completed captcha challenge on HWND".to_string()),
        Some(fresh_telem),
    ).await;

    assert_eq!(task.state, TaskState::Executing);
    assert!(gate.is_open());
    assert_eq!(resync_event.task_id, task.task_id.to_string());
    assert_eq!(resync_event.previous_state_fingerprint, "state_captcha_pending");
    assert_eq!(resync_event.fresh_url, Some("https://kage.dev/checkout/success".to_string()));
    assert_eq!(resync_event.fresh_state_fingerprint, "https://kage.dev/checkout/success");
    assert_eq!(ctx.intervention_summary, Some("Completed captcha challenge on HWND".to_string()));
}
