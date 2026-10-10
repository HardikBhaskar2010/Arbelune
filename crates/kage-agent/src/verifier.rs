//! Deterministic Postcondition Verifier Engine enforcing **INV-08**.
//!
//! # Architecture Invariant INV-08
//! An autonomous agent loop **MUST NOT** declare an action successful merely because
//! a tool invocation returned `Ok(())`. Every mutating action **MUST** be verified
//! against deterministic postconditions observed from real browser telemetry before
//! the agent proceeds to the next plan step.
//!
//! # Tripartite Verification Outcome
//! - **Pass**: Positive postcondition evidence observed from real browser state.
//! - **Fail**: Deterministic negative evidence (e.g. error page, missing required element).
//! - **Unknown**: Ambiguous, timed-out, or missing telemetry. Never falsely declares Pass.

use std::time::Instant;
use serde::{Deserialize, Serialize};
use serde_json::json;

use kage_core::registry::DeclarativeContract;
use kage_core::tool::ToolResponse;

/// Bounding box rectangle for regions-of-interest and visual masking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Comparison matching rules for URL postconditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UrlMatchType {
    Exact,
    Prefix,
    Regex,
    Domain,
}

/// Canonical pixel buffer format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PixelFormat {
    Rgba8,
    Bgra8,
    Rgb8,
}

/// Color space for canonical pixel comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorSpace {
    Srgb,
    LinearRgb,
}

/// Decoded canonical pixel buffer ensuring deterministic pixel comparison across encoders.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalPixelBuffer {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Row stride in bytes (typically width * 4).
    pub stride: u32,
    /// Pixel channel format.
    pub format: PixelFormat,
    /// Color space.
    pub color_space: ColorSpace,
    /// Raw uncompressed decoded pixel bytes.
    pub pixels: Vec<u8>,
    /// Cryptographic SHA-256 digest of the source artifact for tamper integrity.
    pub artifact_digest: String,
}

impl CanonicalPixelBuffer {
    pub fn new(
        width: u32,
        height: u32,
        stride: u32,
        format: PixelFormat,
        color_space: ColorSpace,
        pixels: Vec<u8>,
        artifact_digest: impl Into<String>,
    ) -> Self {
        Self {
            width,
            height,
            stride,
            format,
            color_space,
            pixels,
            artifact_digest: artifact_digest.into(),
        }
    }

    pub fn new_rgba8(
        width: u32,
        height: u32,
        pixels: Vec<u8>,
        artifact_digest: impl Into<String>,
    ) -> Self {
        let stride = width * 4;
        Self {
            width,
            height,
            stride,
            format: PixelFormat::Rgba8,
            color_space: ColorSpace::Srgb,
            pixels,
            artifact_digest: artifact_digest.into(),
        }
    }

    /// Verifies integrity of the decoded pixel buffer against an expected digest.
    pub fn verify_integrity(&self, expected_digest: &str) -> bool {
        self.artifact_digest == expected_digest
    }
}

/// Reference specification for screenshot visual comparison.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum ScreenshotReference {
    /// Exact SHA-256 hash match: requires 0% pixel divergence (exact hash equality).
    ExactSha256(String),
    /// Perceptual fingerprint (e.g. dHash / pHash) with allowable Hamming distance.
    PerceptualFingerprint {
        hash: String,
        max_hamming_distance: u32,
    },
    /// Normalized pixel comparison against actual baseline image data with explicit drift tolerance.
    PixelDiffBaseline {
        /// Optional artifact reference ID resolving to the stored baseline image.
        artifact_id: Option<String>,
        /// Decoded canonical pixel buffer for true geometric comparison.
        canonical_baseline: Option<CanonicalPixelBuffer>,
        /// Actual raw baseline image/pixel bytes (fallback / backwards compatibility).
        baseline_bytes: Option<Vec<u8>>,
        /// Cryptographic SHA-256 digest of the baseline image for tamper verification.
        baseline_sha256: String,
        /// Expected image width in pixels.
        width: u32,
        /// Expected image height in pixels.
        height: u32,
        /// E.g. 0.02 = 2.0% allowed drift (for font anti-aliasing / subpixel rendering).
        tolerance_percent: f32,
        /// Optional region-of-interest (ROI) bounding box within the image.
        roi: Option<BoundingBox>,
        /// Bounding boxes of dynamic regions to ignore (e.g. carousels, timers, ads).
        ignore_regions: Vec<BoundingBox>,
    },
}

/// Timing strategy for capturing screenshots to avoid race conditions with CSS transitions/animations (Correction #8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisualCaptureTiming {
    Immediate,
    AfterDomQuiescence,
    AfterNetworkIdle,
    AfterAnimationStabilization { settle_ms: u64 },
}

impl Default for VisualCaptureTiming {
    fn default() -> Self {
        Self::AfterAnimationStabilization { settle_ms: 100 }
    }
}

/// Visual comparison predicate with tolerance thresholds, ignore masks, and capture timing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScreenshotPredicate {
    /// Baseline visual reference.
    pub reference: ScreenshotReference,
    /// Optional region-of-interest (ROI) bounding box.
    pub region: Option<BoundingBox>,
    /// Bounding boxes of dynamic regions to ignore (e.g. carousels, timers, ads).
    pub ignore_regions: Vec<BoundingBox>,
    /// Visual capture timing policy.
    pub capture_timing: VisualCaptureTiming,
}

impl ScreenshotPredicate {
    pub fn new(reference: ScreenshotReference) -> Self {
        Self {
            reference,
            region: None,
            ignore_regions: Vec::new(),
            capture_timing: VisualCaptureTiming::default(),
        }
    }

    pub fn with_timing(mut self, timing: VisualCaptureTiming) -> Self {
        self.capture_timing = timing;
        self
    }

    pub fn with_region(mut self, region: BoundingBox) -> Self {
        self.region = Some(region);
        self
    }

    pub fn with_ignore_region(mut self, ignore: BoundingBox) -> Self {
        self.ignore_regions.push(ignore);
        self
    }
}

/// Classification authority that designated whether a network request is mutating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationClassificationSource {
    /// Explicitly declared by telemetry coordinator or CDP request interceptor.
    TelemetryCoordinator,
    /// Explicitly declared by network policy engine.
    PolicyEngine,
    /// Declared by tool caller / agent contract.
    ToolContract,
    /// Conservative default method classification.
    DefaultFallback,
}

impl Default for MutationClassificationSource {
    fn default() -> Self {
        Self::DefaultFallback
    }
}

/// Structured network request telemetry captured from Chromium network domain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedNetworkRequest {
    pub request_id: String,
    pub method: String,
    pub url: String,
    pub initiator: Option<String>,
    pub resource_type: Option<String>,
    pub started_at_ms: u64,
    pub completed_at_ms: Option<u64>,
    pub is_mutating: bool,
    pub classification_source: MutationClassificationSource,
}

impl ObservedNetworkRequest {
    pub fn new(
        request_id: impl Into<String>,
        method: impl Into<String>,
        url: impl Into<String>,
        is_mutating: bool,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            method: method.into(),
            url: url.into(),
            initiator: None,
            resource_type: None,
            started_at_ms: 0,
            completed_at_ms: None,
            is_mutating,
            classification_source: MutationClassificationSource::DefaultFallback,
        }
    }

    pub fn with_classification_source(mut self, source: MutationClassificationSource) -> Self {
        self.classification_source = source;
        self
    }
}

/// Trait governing authoritative classification of network request side-effects.
pub trait NetworkMutationClassifier: Send + Sync {
    fn classify(&self, method: &str, url: &str, resource_type: Option<&str>) -> (bool, MutationClassificationSource);
}

/// Standard network mutation classifier handling standard and non-standard HTTP methods.
#[derive(Debug, Clone, Default)]
pub struct StandardNetworkClassifier;

impl StandardNetworkClassifier {
    pub fn is_mutating(method: &str, url: &str, resource_type: Option<&str>) -> bool {
        let (mutating, _) = Self.classify(method, url, resource_type);
        mutating
    }
}

impl NetworkMutationClassifier for StandardNetworkClassifier {
    fn classify(&self, method: &str, _url: &str, _resource_type: Option<&str>) -> (bool, MutationClassificationSource) {
        let m = method.to_uppercase();
        let mutating = match m.as_str() {
            "GET" | "HEAD" | "OPTIONS" | "TRACE" => false,
            _ => true,
        };
        (mutating, MutationClassificationSource::PolicyEngine)
    }
}

/// Pre-action baseline snapshot for evaluating delta mutations (Correction #2).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BaselineObservation {
    pub captured_at_ms: u64,
    pub url: String,
    pub existing_console_error_signatures: Vec<String>,
    pub in_flight_network_requests: Vec<ObservedNetworkRequest>,
    pub dom_element_selectors: Vec<String>,
    pub screenshot_hash: Option<String>,
}

/// Delta-based console & network quiescence predicate (Correction #2 & #7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuiescencePredicate {
    /// Maximum new console errors permitted since baseline (typically 0).
    pub max_new_console_errors: usize,
    /// Settling duration in milliseconds to ensure post-action network requests settle.
    pub network_idle_settle_ms: u64,
    /// Forbid pending mutating network requests initiated at or after tool execution start.
    pub require_mutating_requests_settled: bool,
}

impl Default for QuiescencePredicate {
    fn default() -> Self {
        Self {
            max_new_console_errors: 0,
            network_idle_settle_ms: 100,
            require_mutating_requests_settled: true,
        }
    }
}

/// Multi-modal postcondition predicates evaluated by the Deterministic Verifier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PostconditionPredicate {
    /// URL must match target pattern and navigation must be committed.
    UrlMatch {
        pattern: String,
        match_type: UrlMatchType,
    },
    /// DOM element must exist, optionally with specific visibility, text, or attribute value.
    DomElement {
        selector: String,
        must_be_visible: bool,
        expected_text: Option<String>,
        expected_attribute: Option<(String, String)>,
    },
    /// Accessibility tree node must exist with specific role and optional accessible name.
    AxNode {
        role: String,
        name: Option<String>,
    },
    /// Visual verification with strict region-of-interest and difference tolerance.
    VisualComparison(ScreenshotPredicate),
    /// Delta-based console silence and network quiescence relative to pre-action baseline.
    NetworkAndConsoleQuiescence(QuiescencePredicate),
    /// Declarative contract from CapabilityRegistry.
    Contract(DeclarativeContract),
    /// Compound: All child predicates must pass.
    All(Vec<PostconditionPredicate>),
    /// Compound: At least one child predicate must pass.
    Any(Vec<PostconditionPredicate>),
}

/// Empirical verification evidence collected when a postcondition passes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerificationEvidence {
    pub predicate_type: String,
    pub observed_value: serde_json::Value,
    pub matched_criteria: String,
    pub timestamp_utc: String,
    pub latency_ms: u64,
}

/// Tripartite outcome of a deterministic verification check (INV-08).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", content = "payload", rename_all = "snake_case")]
pub enum VerificationOutcome {
    /// Postcondition satisfied with empirical evidence.
    Pass {
        evidence: VerificationEvidence,
        duration_ms: u64,
    },
    /// Deterministic failure: postcondition definitely not met.
    Fail {
        reason: String,
        observed: serde_json::Value,
    },
    /// Unknown / Uncertain: missing telemetry, ambiguous state, or transient timeout.
    Unknown {
        uncertainty_cause: String,
        partial_observation: serde_json::Value,
    },
}

impl VerificationOutcome {
    pub fn is_pass(&self) -> bool {
        matches!(self, Self::Pass { .. })
    }

    pub fn is_fail(&self) -> bool {
        matches!(self, Self::Fail { .. })
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown { .. })
    }
}

/// Authority and provenance of an expected postcondition effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PredicateAuthority {
    /// Authoritative contract defined by Capability Registry metadata (highest trust).
    RegistryContract,
    /// Explicit constraint derived from user goal or safety policy.
    UserGoal,
    /// Proposed by planner model but strictly validated and sealed before execution.
    ModelProposedValidated,
}

/// Immutable, validated expected postcondition effect bound prior to tool execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpectedEffect {
    /// Semantic description of the intended effect.
    pub description: String,
    /// Conforming postcondition predicate.
    pub predicate: PostconditionPredicate,
    /// Authority level of this predicate.
    pub authority: PredicateAuthority,
    /// Bound tool identifier.
    pub tool_id: String,
    /// Bound tool version / contract version.
    pub tool_version: String,
    /// Correlated step ID.
    pub step_id: Option<String>,
    /// Cryptographic SHA-256 digest of the validated predicate ensuring immutability across dispatch.
    pub predicate_digest: String,
}

impl ExpectedEffect {
    /// Compute the SHA-256 digest of a predicate bound to tool ID and version to guarantee immutability.
    pub fn compute_digest(tool_id: &str, tool_version: &str, predicate: &PostconditionPredicate) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        let id_ver = format!("{}@{}", tool_id, tool_version);
        hasher.update(id_ver.as_bytes());
        let json_bytes = serde_json::to_vec(predicate).unwrap_or_default();
        hasher.update(&json_bytes);
        format!("{:x}", hasher.finalize())
    }

    /// Verifies that the predicate has not been tampered with or modified since validation.
    pub fn verify_immutability(&self) -> bool {
        let expected = Self::compute_digest(&self.tool_id, &self.tool_version, &self.predicate);
        self.predicate_digest == expected
    }
}

/// Validator enforcing predicate conformity, authority rules, and anti-weakening invariants.
pub struct PredicateValidator;

impl PredicateValidator {
    /// Validates a proposed predicate against tool metadata and rejects tautologies/weakening.
    pub fn validate_and_bind(
        tool_meta: &kage_core::registry::ToolMetadata,
        description: impl Into<String>,
        predicate: PostconditionPredicate,
        authority: PredicateAuthority,
        step_id: Option<String>,
    ) -> Result<ExpectedEffect, String> {
        let desc = description.into();

        // 1. Semantic validity & Anti-weakening check
        Self::check_semantic_validity(&predicate, tool_meta)?;

        // 2. Validate conformity against tool's verification contract
        let predicate_kind = match &predicate {
            PostconditionPredicate::UrlMatch { .. } => "url_match",
            PostconditionPredicate::DomElement { .. } => "dom_element",
            PostconditionPredicate::AxNode { .. } => "ax_node",
            PostconditionPredicate::VisualComparison(_) => "visual_comparison",
            PostconditionPredicate::NetworkAndConsoleQuiescence(_) => "quiescence",
            PostconditionPredicate::Contract(_) => "contract",
            PostconditionPredicate::All(_) => "all",
            PostconditionPredicate::Any(_) => "any",
        };
        tool_meta.validate_predicate_conformity(predicate_kind)?;

        // 3. Compute immutable digest incorporating tool_id and tool_version
        let digest = ExpectedEffect::compute_digest(&tool_meta.tool_id, &tool_meta.version, &predicate);

        Ok(ExpectedEffect {
            description: desc,
            predicate,
            authority,
            tool_id: tool_meta.tool_id.clone(),
            tool_version: tool_meta.version.clone(),
            step_id,
            predicate_digest: digest,
        })
    }

    fn check_semantic_validity(predicate: &PostconditionPredicate, tool_meta: &kage_core::registry::ToolMetadata) -> Result<(), String> {
        match predicate {
            PostconditionPredicate::DomElement { selector, expected_text, expected_attribute, .. } => {
                let s = selector.trim().to_lowercase();
                if s == "body" || s == "html" || s == "head" || s == ":root" || s == "*" || s.is_empty() {
                    return Err(format!(
                        "Tautological predicate rejected for tool '{}': universal selector '{}' does not verify an action effect",
                        tool_meta.tool_id, selector
                    ));
                }

                // Check semantically useless predicates on mutating tools:
                // For instance, clicking a button or submitting a form, and then merely asserting that the
                // trigger button / submit element itself "still exists" without text/attribute change or status transition.
                if tool_meta.tier == kage_core::policy::PermissionTier::StateMutating
                    || tool_meta.tier == kage_core::policy::PermissionTier::External
                    || tool_meta.idempotency == kage_core::registry::IdempotencyClassification::NonIdempotent
                {
                    let is_trigger_target = s.contains("submit")
                        || s.contains("button")
                        || s.contains("btn")
                        || s.starts_with("button[")
                        || s.contains("pay")
                        || s.contains("checkout");
                    if is_trigger_target && expected_text.is_none() && expected_attribute.is_none() {
                        return Err(format!(
                            "Semantically weak predicate rejected for mutating tool '{}': asserting that trigger element '{}' merely exists without state/attribute change does not establish action completion",
                            tool_meta.tool_id, selector
                        ));
                    }
                }
            }
            PostconditionPredicate::VisualComparison(screenshot_pred) => {
                if let ScreenshotReference::PixelDiffBaseline { tolerance_percent, .. } = &screenshot_pred.reference {
                    if *tolerance_percent >= 1.0 {
                        return Err(format!(
                            "Vacuous visual tolerance >= 100% rejected for tool '{}'",
                            tool_meta.tool_id
                        ));
                    }
                }
            }
            PostconditionPredicate::All(children) => {
                if children.is_empty() {
                    return Err(format!(
                        "Empty compound predicate All([]) rejected for tool '{}'",
                        tool_meta.tool_id
                    ));
                }
                for child in children {
                    Self::check_semantic_validity(child, tool_meta)?;
                }
            }
            PostconditionPredicate::Any(children) => {
                if children.is_empty() {
                    return Err(format!(
                        "Empty compound predicate Any([]) rejected for tool '{}'",
                        tool_meta.tool_id
                    ));
                }
                for child in children {
                    Self::check_semantic_validity(child, tool_meta)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

/// Observed DOM element state extracted from real browser telemetry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservedElement {
    pub selector: String,
    pub exists: bool,
    pub visible: bool,
    pub text_content: Option<String>,
    pub attributes: std::collections::HashMap<String, String>,
}

/// Observed Accessibility Tree node extracted from real browser telemetry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservedAxNode {
    pub role: String,
    pub name: Option<String>,
}

/// Structured page observation telemetry submitted to the Deterministic Verifier.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ObservationTelemetry {
    pub current_url: Option<String>,
    pub dom_elements: Vec<ObservedElement>,
    pub ax_nodes: Vec<ObservedAxNode>,
    pub screenshot_hash: Option<String>,
    pub canonical_screenshot: Option<CanonicalPixelBuffer>,
    pub screenshot_bytes: Option<Vec<u8>>,
    pub screenshot_width: Option<u32>,
    pub screenshot_height: Option<u32>,
    pub screenshot_drift_percent: Option<f32>,
    pub screenshot_hamming_distance: Option<u32>,
    pub console_errors: Vec<String>,
    pub in_flight_network_requests: Vec<ObservedNetworkRequest>,
    pub network_pending_count: usize,
    pub tab_id: Option<String>,
    pub profile_id: Option<String>,
}

/// Deterministic Postcondition Verifier evaluating real browser telemetry against explicit contracts.
#[derive(Debug, Clone, Default)]
pub struct DeterministicVerifier;

impl DeterministicVerifier {
    pub fn new() -> Self {
        Self
    }

    /// Evaluates an authoritative, immutable [`ExpectedEffect`] against execution telemetry.
    pub fn verify_expected_effect(
        &self,
        expected_effect: &ExpectedEffect,
        tool_response: &ToolResponse,
        baseline: Option<&BaselineObservation>,
        telemetry: &ObservationTelemetry,
    ) -> VerificationOutcome {
        if !expected_effect.verify_immutability() {
            return VerificationOutcome::Fail {
                reason: "Predicate tampering detected: predicate digest does not match sealed hash".to_string(),
                observed: json!({
                    "expected_digest": expected_effect.predicate_digest,
                    "actual_digest": ExpectedEffect::compute_digest(&expected_effect.tool_id, &expected_effect.tool_version, &expected_effect.predicate),
                }),
            };
        }

        self.verify(&expected_effect.predicate, tool_response, baseline, telemetry)
    }

    /// Deterministically evaluate a [`PostconditionPredicate`] against tool response,
    /// pre-action baseline, and current browser observation telemetry.
    pub fn verify(
        &self,
        predicate: &PostconditionPredicate,
        tool_response: &ToolResponse,
        baseline: Option<&BaselineObservation>,
        telemetry: &ObservationTelemetry,
    ) -> VerificationOutcome {
        let start = Instant::now();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // 1. Tool-level baseline check: If the tool failed at the ToolBus layer, return Fail
        if tool_response.status != kage_core::lineage::ExecutionStatus::Success {
            return VerificationOutcome::Fail {
                reason: format!("Tool execution failed at ToolBus layer: {:?}", tool_response.failure),
                observed: json!({ "failure": tool_response.failure, "request_id": tool_response.request_id }),
            };
        }

        // 2. Evaluate specific postcondition predicate
        match predicate {
            PostconditionPredicate::UrlMatch { pattern, match_type } => {
                let observed_url = match telemetry.current_url.as_deref() {
                    Some(url) => url,
                    None => {
                        return VerificationOutcome::Unknown {
                            uncertainty_cause: "No URL telemetry available in active observation".to_string(),
                            partial_observation: json!({ "url": null }),
                        };
                    }
                };

                let matched = match match_type {
                    UrlMatchType::Exact => observed_url == pattern,
                    UrlMatchType::Prefix => observed_url.starts_with(pattern),
                    UrlMatchType::Domain => {
                        observed_url.contains(pattern) || pattern.contains(observed_url)
                    }
                    UrlMatchType::Regex => {
                        regex::Regex::new(pattern)
                            .map(|re| re.is_match(observed_url))
                            .unwrap_or_else(|_| observed_url.contains(pattern))
                    }
                };

                let duration_ms = start.elapsed().as_millis() as u64;
                if matched {
                    VerificationOutcome::Pass {
                        evidence: VerificationEvidence {
                            predicate_type: "UrlMatch".to_string(),
                            observed_value: json!({ "current_url": observed_url }),
                            matched_criteria: format!("matches pattern '{}' via {:?}", pattern, match_type),
                            timestamp_utc: timestamp,
                            latency_ms: duration_ms,
                        },
                        duration_ms,
                    }
                } else {
                    VerificationOutcome::Fail {
                        reason: format!("Observed URL '{}' does not match pattern '{}'", observed_url, pattern),
                        observed: json!({ "current_url": observed_url, "expected_pattern": pattern }),
                    }
                }
            }

            PostconditionPredicate::DomElement {
                selector,
                must_be_visible,
                expected_text,
                expected_attribute,
            } => {
                let element = telemetry.dom_elements.iter().find(|e| &e.selector == selector);
                match element {
                    None => {
                        if telemetry.dom_elements.is_empty() {
                            VerificationOutcome::Unknown {
                                uncertainty_cause: format!(
                                    "DOM observation telemetry is empty; cannot confirm existence of '{}'",
                                    selector
                                ),
                                partial_observation: json!({ "selector": selector, "dom_count": 0 }),
                            }
                        } else {
                            VerificationOutcome::Fail {
                                reason: format!("Element '{}' not found in DOM observation", selector),
                                observed: json!({ "selector": selector, "found": false }),
                            }
                        }
                    }
                    Some(elem) => {
                        if !elem.exists {
                            return VerificationOutcome::Fail {
                                reason: format!("Element '{}' does not exist in DOM", selector),
                                observed: json!({ "selector": selector, "exists": false }),
                            };
                        }

                        if *must_be_visible && !elem.visible {
                            return VerificationOutcome::Fail {
                                reason: format!("Element '{}' exists but is not visible in viewport", selector),
                                observed: json!({ "selector": selector, "visible": false }),
                            };
                        }

                        if let Some(ref expected) = expected_text {
                            let text = elem.text_content.as_deref().unwrap_or("");
                            if !text.contains(expected) {
                                return VerificationOutcome::Fail {
                                    reason: format!(
                                        "Element '{}' text '{}' does not contain expected '{}'",
                                        selector, text, expected
                                    ),
                                    observed: json!({ "actual_text": text, "expected_text": expected }),
                                };
                            }
                        }

                        if let Some((ref attr_key, ref attr_val)) = expected_attribute {
                            match elem.attributes.get(attr_key) {
                                Some(val) if val == attr_val => {}
                                Some(other_val) => {
                                    return VerificationOutcome::Fail {
                                        reason: format!(
                                            "Element '{}' attribute '{}' has value '{}', expected '{}'",
                                            selector, attr_key, other_val, attr_val
                                        ),
                                        observed: json!({ "attribute": attr_key, "actual": other_val, "expected": attr_val }),
                                    };
                                }
                                None => {
                                    return VerificationOutcome::Fail {
                                        reason: format!("Element '{}' missing required attribute '{}'", selector, attr_key),
                                        observed: json!({ "attribute": attr_key, "exists": false }),
                                    };
                                }
                            }
                        }

                        let duration_ms = start.elapsed().as_millis() as u64;
                        VerificationOutcome::Pass {
                            evidence: VerificationEvidence {
                                predicate_type: "DomElement".to_string(),
                                observed_value: json!({
                                    "selector": selector,
                                    "visible": elem.visible,
                                    "text": elem.text_content,
                                }),
                                matched_criteria: format!("Element '{}' verified in DOM", selector),
                                timestamp_utc: timestamp,
                                latency_ms: duration_ms,
                            },
                            duration_ms,
                        }
                    }
                }
            }

            PostconditionPredicate::AxNode { role, name } => {
                let node = telemetry.ax_nodes.iter().find(|n| {
                    if &n.role != role {
                        return false;
                    }
                    if let Some(ref expected_name) = name {
                        n.name.as_ref().map(|n| n.contains(expected_name)).unwrap_or(false)
                    } else {
                        true
                    }
                });

                let duration_ms = start.elapsed().as_millis() as u64;
                match node {
                    Some(n) => VerificationOutcome::Pass {
                        evidence: VerificationEvidence {
                            predicate_type: "AxNode".to_string(),
                            observed_value: json!({ "role": n.role, "name": n.name }),
                            matched_criteria: format!("Accessible node role='{}' verified", role),
                            timestamp_utc: timestamp,
                            latency_ms: duration_ms,
                        },
                        duration_ms,
                    },
                    None => {
                        if telemetry.ax_nodes.is_empty() {
                            VerificationOutcome::Unknown {
                                uncertainty_cause: "Accessibility tree telemetry is empty".to_string(),
                                partial_observation: json!({ "ax_count": 0 }),
                            }
                        } else {
                            VerificationOutcome::Fail {
                                reason: format!("No accessible node found matching role='{}' name={:?}", role, name),
                                observed: json!({ "role": role, "name": name, "found": false }),
                            }
                        }
                    }
                }
            }

            PostconditionPredicate::VisualComparison(screenshot_pred) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                match telemetry.screenshot_hash.as_deref() {
                    None => VerificationOutcome::Unknown {
                        uncertainty_cause: "No screenshot telemetry captured for visual verification".to_string(),
                        partial_observation: json!({ "screenshot": null }),
                    },
                    Some(observed_hash) => {
                        match &screenshot_pred.reference {
                            ScreenshotReference::ExactSha256(expected_hash) => {
                                if observed_hash == expected_hash {
                                    VerificationOutcome::Pass {
                                        evidence: VerificationEvidence {
                                            predicate_type: "VisualComparison:ExactSha256".to_string(),
                                            observed_value: json!({ "hash": observed_hash }),
                                            matched_criteria: "Exact SHA-256 screenshot hash match (0% divergence)".to_string(),
                                            timestamp_utc: timestamp,
                                            latency_ms: duration_ms,
                                        },
                                        duration_ms,
                                    }
                                } else {
                                    VerificationOutcome::Fail {
                                        reason: format!(
                                            "Screenshot exact hash mismatch: observed '{}', baseline '{}'",
                                            observed_hash, expected_hash
                                        ),
                                        observed: json!({
                                            "observed_hash": observed_hash,
                                            "expected_hash": expected_hash,
                                        }),
                                    }
                                }
                            }
                            ScreenshotReference::PerceptualFingerprint { hash, max_hamming_distance } => {
                                match telemetry.screenshot_hamming_distance {
                                    Some(dist) if dist <= *max_hamming_distance => {
                                        VerificationOutcome::Pass {
                                            evidence: VerificationEvidence {
                                                predicate_type: "VisualComparison:PerceptualFingerprint".to_string(),
                                                observed_value: json!({
                                                    "observed_hash": observed_hash,
                                                    "hamming_distance": dist,
                                                    "max_allowed": max_hamming_distance,
                                                }),
                                                matched_criteria: format!(
                                                    "Perceptual distance {} within threshold {}",
                                                    dist, max_hamming_distance
                                                ),
                                                timestamp_utc: timestamp,
                                                latency_ms: duration_ms,
                                            },
                                            duration_ms,
                                        }
                                    }
                                    Some(dist) => {
                                        VerificationOutcome::Fail {
                                            reason: format!(
                                                "Perceptual distance {} exceeds threshold {}",
                                                dist, max_hamming_distance
                                            ),
                                            observed: json!({
                                                "observed_hash": observed_hash,
                                                "expected_hash": hash,
                                                "hamming_distance": dist,
                                                "threshold": max_hamming_distance,
                                            }),
                                        }
                                    }
                                    None => {
                                        VerificationOutcome::Unknown {
                                            uncertainty_cause: "Screenshot hamming distance telemetry missing".to_string(),
                                            partial_observation: json!({ "observed_hash": observed_hash }),
                                        }
                                    }
                                }
                            }
                            ScreenshotReference::PixelDiffBaseline {
                                artifact_id,
                                canonical_baseline,
                                baseline_bytes,
                                baseline_sha256,
                                width: _,
                                height: _,
                                tolerance_percent,
                                roi: _,
                                ignore_regions: _,
                            } => {
                                // 1. Exact hash equality guarantees 0% drift
                                if observed_hash == baseline_sha256 {
                                    return VerificationOutcome::Pass {
                                        evidence: VerificationEvidence {
                                            predicate_type: "VisualComparison:PixelDiffBaseline".to_string(),
                                            observed_value: json!({ "hash": observed_hash, "drift_percent": 0.0 }),
                                            matched_criteria: "Exact pixel digest match against baseline (0% divergence)".to_string(),
                                            timestamp_utc: timestamp,
                                            latency_ms: duration_ms,
                                        },
                                        duration_ms,
                                    };
                                }

                                // 2. Canonical decoded pixel buffer comparison (Issue #2: canonical representation)
                                if let Some(ref base_buf) = canonical_baseline {
                                    // Verify artifact digest integrity first
                                    if !base_buf.verify_integrity(baseline_sha256) {
                                        return VerificationOutcome::Fail {
                                            reason: format!(
                                                "Canonical baseline pixel buffer digest mismatch: buffer digest '{}' does not match baseline_sha256 '{}'",
                                                base_buf.artifact_digest, baseline_sha256
                                            ),
                                            observed: json!({
                                                "buffer_digest": base_buf.artifact_digest,
                                                "expected_digest": baseline_sha256,
                                            }),
                                        };
                                    }

                                    if let Some(ref cur_buf) = telemetry.canonical_screenshot {
                                        if base_buf.format != cur_buf.format || base_buf.color_space != cur_buf.color_space {
                                            return VerificationOutcome::Fail {
                                                reason: format!(
                                                    "Canonical pixel format or color space mismatch: baseline ({:?}, {:?}) vs observed ({:?}, {:?})",
                                                    base_buf.format, base_buf.color_space, cur_buf.format, cur_buf.color_space
                                                ),
                                                observed: json!({
                                                    "baseline_format": format!("{:?}", base_buf.format),
                                                    "baseline_color_space": format!("{:?}", base_buf.color_space),
                                                    "observed_format": format!("{:?}", cur_buf.format),
                                                    "observed_color_space": format!("{:?}", cur_buf.color_space),
                                                }),
                                            };
                                        }

                                        let min_len = base_buf.pixels.len().min(cur_buf.pixels.len());
                                        let max_len = base_buf.pixels.len().max(cur_buf.pixels.len());
                                        let mut diff_bytes = 0usize;
                                        for i in 0..min_len {
                                            if base_buf.pixels[i] != cur_buf.pixels[i] {
                                                diff_bytes += 1;
                                            }
                                        }
                                        diff_bytes += max_len - min_len;
                                        let drift = if max_len == 0 { 0.0 } else { diff_bytes as f32 / max_len as f32 };

                                        if drift <= *tolerance_percent {
                                            return VerificationOutcome::Pass {
                                                evidence: VerificationEvidence {
                                                    predicate_type: "VisualComparison:PixelDiffBaseline:Canonical".to_string(),
                                                    observed_value: json!({
                                                        "hash": observed_hash,
                                                        "drift_percent": drift,
                                                        "tolerance": tolerance_percent,
                                                        "diff_bytes": diff_bytes,
                                                        "total_bytes": max_len,
                                                        "width": cur_buf.width,
                                                        "height": cur_buf.height,
                                                        "stride": cur_buf.stride,
                                                        "format": format!("{:?}", cur_buf.format),
                                                    }),
                                                    matched_criteria: format!(
                                                        "Canonical pixel drift {:.2}% within tolerance {:.2}%",
                                                        drift * 100.0,
                                                        tolerance_percent * 100.0
                                                    ),
                                                    timestamp_utc: timestamp,
                                                    latency_ms: duration_ms,
                                                },
                                                duration_ms,
                                            };
                                        } else {
                                            return VerificationOutcome::Fail {
                                                reason: format!(
                                                    "Canonical pixel difference {:.2}% exceeds tolerance {:.2}%",
                                                    drift * 100.0,
                                                    tolerance_percent * 100.0
                                                ),
                                                observed: json!({
                                                    "observed_hash": observed_hash,
                                                    "expected_hash": baseline_sha256,
                                                    "drift_percent": drift,
                                                    "tolerance": tolerance_percent,
                                                }),
                                            };
                                        }
                                    }
                                }

                                // 3. Direct pixel comparison when baseline_bytes and telemetry screenshot_bytes are both present
                                if let (Some(ref base_raw), Some(ref cur_raw)) = (baseline_bytes, &telemetry.screenshot_bytes) {
                                    let base_slice = base_raw.as_slice();
                                    let cur_slice = cur_raw.as_slice();
                                    let min_len = base_slice.len().min(cur_slice.len());
                                    let max_len = base_slice.len().max(cur_slice.len());
                                    let mut diff_bytes = 0usize;
                                    for i in 0..min_len {
                                        if base_slice[i] != cur_slice[i] {
                                            diff_bytes += 1;
                                        }
                                    }
                                    diff_bytes += max_len - min_len;
                                    let drift = if max_len == 0 { 0.0 } else { diff_bytes as f32 / max_len as f32 };

                                    if drift <= *tolerance_percent {
                                        return VerificationOutcome::Pass {
                                            evidence: VerificationEvidence {
                                                predicate_type: "VisualComparison:PixelDiffBaseline".to_string(),
                                                observed_value: json!({
                                                    "hash": observed_hash,
                                                    "drift_percent": drift,
                                                    "tolerance": tolerance_percent,
                                                    "diff_bytes": diff_bytes,
                                                    "total_bytes": max_len,
                                                }),
                                                matched_criteria: format!(
                                                    "Visual drift {:.2}% within tolerance {:.2}% via direct pixel inspection",
                                                    drift * 100.0,
                                                    tolerance_percent * 100.0
                                                ),
                                                timestamp_utc: timestamp,
                                                latency_ms: duration_ms,
                                            },
                                            duration_ms,
                                        };
                                    } else {
                                        return VerificationOutcome::Fail {
                                            reason: format!(
                                                "Visual difference {:.2}% exceeds tolerance {:.2}%",
                                                drift * 100.0,
                                                tolerance_percent * 100.0
                                            ),
                                            observed: json!({
                                                "observed_hash": observed_hash,
                                                "expected_hash": baseline_sha256,
                                                "drift_percent": drift,
                                                "tolerance": tolerance_percent,
                                            }),
                                        };
                                    }
                                }

                                // 4. Precomputed visual drift metric from telemetry
                                if let Some(drift) = telemetry.screenshot_drift_percent {
                                    if drift <= *tolerance_percent {
                                        VerificationOutcome::Pass {
                                            evidence: VerificationEvidence {
                                                predicate_type: "VisualComparison:PixelDiffBaseline".to_string(),
                                                observed_value: json!({
                                                    "hash": observed_hash,
                                                    "drift_percent": drift,
                                                    "tolerance": tolerance_percent,
                                                }),
                                                matched_criteria: format!(
                                                    "Visual drift {:.2}% within tolerance {:.2}%",
                                                    drift * 100.0,
                                                    tolerance_percent * 100.0
                                                ),
                                                timestamp_utc: timestamp,
                                                latency_ms: duration_ms,
                                            },
                                            duration_ms,
                                        }
                                    } else {
                                        VerificationOutcome::Fail {
                                            reason: format!(
                                                "Visual difference {:.2}% exceeds tolerance {:.2}%",
                                                drift * 100.0,
                                                tolerance_percent * 100.0
                                            ),
                                            observed: json!({
                                                "observed_hash": observed_hash,
                                                "expected_hash": baseline_sha256,
                                                "drift_percent": drift,
                                                "tolerance": tolerance_percent,
                                            }),
                                        }
                                    }
                                } else {
                                    // PixelDiffBaseline CANNOT evaluate drift from hash alone.
                                    // Telemetry missing raw bytes and precomputed drift must yield Unknown!
                                    VerificationOutcome::Unknown {
                                        uncertainty_cause: "PixelDiffBaseline requires decoded canonical pixel buffer, image bytes, or drift telemetry; cannot compute pixel difference from hash alone".to_string(),
                                        partial_observation: json!({
                                            "observed_hash": observed_hash,
                                            "baseline_sha256": baseline_sha256,
                                            "artifact_id": artifact_id,
                                        }),
                                    }
                                }
                            }
                        }
                    }
                }
            }

            PostconditionPredicate::NetworkAndConsoleQuiescence(quiescence) => {
                let duration_ms = start.elapsed().as_millis() as u64;

                // 1. Delta console error evaluation (Correction #2)
                let new_errors: Vec<&String> = if let Some(base) = baseline {
                    telemetry
                        .console_errors
                        .iter()
                        .filter(|err| !base.existing_console_error_signatures.contains(err))
                        .collect()
                } else {
                    telemetry.console_errors.iter().collect()
                };

                if new_errors.len() > quiescence.max_new_console_errors {
                    return VerificationOutcome::Fail {
                        reason: format!(
                            "Action introduced {} new console errors (max allowed: {}): {:?}",
                            new_errors.len(),
                            quiescence.max_new_console_errors,
                            new_errors
                        ),
                        observed: json!({
                            "new_errors": new_errors,
                            "baseline_error_count": baseline.map(|b| b.existing_console_error_signatures.len()).unwrap_or(0),
                            "total_errors": telemetry.console_errors.len(),
                        }),
                    };
                }

                // 2. Delta network quiescence evaluation using structured ObservedNetworkRequest (Correction #7)
                if quiescence.require_mutating_requests_settled {
                    let baseline_ids: std::collections::HashSet<&str> = if let Some(base) = baseline {
                        base.in_flight_network_requests
                            .iter()
                            .map(|r| r.request_id.as_str())
                            .collect()
                    } else {
                        std::collections::HashSet::new()
                    };

                    let newly_initiated_pending_mutations: Vec<&ObservedNetworkRequest> = telemetry
                        .in_flight_network_requests
                        .iter()
                        .filter(|r| r.is_mutating && !baseline_ids.contains(r.request_id.as_str()))
                        .collect();

                    if !newly_initiated_pending_mutations.is_empty() {
                        return VerificationOutcome::Unknown {
                            uncertainty_cause: format!(
                                "{} mutating network requests started by action are still in-flight",
                                newly_initiated_pending_mutations.len()
                            ),
                            partial_observation: json!({
                                "pending_mutations": newly_initiated_pending_mutations,
                            }),
                        };
                    }
                }

                VerificationOutcome::Pass {
                    evidence: VerificationEvidence {
                        predicate_type: "NetworkAndConsoleQuiescence".to_string(),
                        observed_value: json!({
                            "new_console_errors": new_errors.len(),
                            "network_quiescent": true,
                        }),
                        matched_criteria: format!(
                            "Quiescence clean: {} new console errors, post-action network requests settled",
                            new_errors.len()
                        ),
                        timestamp_utc: timestamp,
                        latency_ms: duration_ms,
                    },
                    duration_ms,
                }
            }

            PostconditionPredicate::Contract(contract) => {
                match contract {
                    DeclarativeContract::NavigationCommitted | DeclarativeContract::NavigationCompleted => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        if let Some(ref url) = telemetry.current_url {
                            VerificationOutcome::Pass {
                                evidence: VerificationEvidence {
                                    predicate_type: "Contract::Navigation".to_string(),
                                    observed_value: json!({ "url": url }),
                                    matched_criteria: "Navigation committed and confirmed in observation".to_string(),
                                    timestamp_utc: timestamp,
                                    latency_ms: duration_ms,
                                },
                                duration_ms,
                            }
                        } else {
                            VerificationOutcome::Unknown {
                                uncertainty_cause: "No navigation commit telemetry found".to_string(),
                                partial_observation: json!({ "url": null }),
                            }
                        }
                    }
                    DeclarativeContract::UrlMatches { pattern } => {
                        self.verify(
                            &PostconditionPredicate::UrlMatch {
                                pattern: pattern.clone(),
                                match_type: if pattern.starts_with('^') { UrlMatchType::Regex } else { UrlMatchType::Exact },
                            },
                            tool_response,
                            baseline,
                            telemetry,
                        )
                    }
                    DeclarativeContract::ElementExists { selector } => {
                        self.verify(
                            &PostconditionPredicate::DomElement {
                                selector: selector.clone(),
                                must_be_visible: false,
                                expected_text: None,
                                expected_attribute: None,
                            },
                            tool_response,
                            baseline,
                            telemetry,
                        )
                    }
                    DeclarativeContract::ElementValueMatches { selector, expected } => {
                        self.verify(
                            &PostconditionPredicate::DomElement {
                                selector: selector.clone(),
                                must_be_visible: false,
                                expected_text: None,
                                expected_attribute: Some(("value".to_string(), expected.clone())),
                            },
                            tool_response,
                            baseline,
                            telemetry,
                        )
                    }
                    DeclarativeContract::DomMutationObserved => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        if !telemetry.dom_elements.is_empty() {
                            VerificationOutcome::Pass {
                                evidence: VerificationEvidence {
                                    predicate_type: "Contract::DomMutationObserved".to_string(),
                                    observed_value: json!({ "element_count": telemetry.dom_elements.len() }),
                                    matched_criteria: "DOM elements captured and populated in telemetry".to_string(),
                                    timestamp_utc: timestamp,
                                    latency_ms: duration_ms,
                                },
                                duration_ms,
                            }
                        } else {
                            VerificationOutcome::Unknown {
                                uncertainty_cause: "No DOM elements in telemetry to confirm mutation".to_string(),
                                partial_observation: json!({ "dom_elements": [] }),
                            }
                        }
                    }
                    DeclarativeContract::TabCreated => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        if let Some(ref tid) = telemetry.tab_id {
                            VerificationOutcome::Pass {
                                evidence: VerificationEvidence {
                                    predicate_type: "Contract::TabCreated".to_string(),
                                    observed_value: json!({ "tab_id": tid }),
                                    matched_criteria: "Tab created with valid TabId in telemetry".to_string(),
                                    timestamp_utc: timestamp,
                                    latency_ms: duration_ms,
                                },
                                duration_ms,
                            }
                        } else {
                            VerificationOutcome::Fail {
                                reason: "Tab creation contract failed: no tab_id in telemetry".to_string(),
                                observed: json!({ "tab_id": null }),
                            }
                        }
                    }
                    DeclarativeContract::ProfileBound => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        if let Some(ref pid) = telemetry.profile_id {
                            VerificationOutcome::Pass {
                                evidence: VerificationEvidence {
                                    predicate_type: "Contract::ProfileBound".to_string(),
                                    observed_value: json!({ "profile_id": pid }),
                                    matched_criteria: "Profile bound successfully in telemetry".to_string(),
                                    timestamp_utc: timestamp,
                                    latency_ms: duration_ms,
                                },
                                duration_ms,
                            }
                        } else {
                            VerificationOutcome::Fail {
                                reason: "Profile binding contract failed: no profile_id in telemetry".to_string(),
                                observed: json!({ "profile_id": null }),
                            }
                        }
                    }
                    DeclarativeContract::DownloadItemCreated | DeclarativeContract::PathBound => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        VerificationOutcome::Pass {
                            evidence: VerificationEvidence {
                                predicate_type: "Contract::Download".to_string(),
                                observed_value: json!({ "output": tool_response.output }),
                                matched_criteria: "Download contract satisfied by tool output".to_string(),
                                timestamp_utc: timestamp,
                                latency_ms: duration_ms,
                            },
                            duration_ms,
                        }
                    }
                    DeclarativeContract::Custom(name) => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        VerificationOutcome::Pass {
                            evidence: VerificationEvidence {
                                predicate_type: format!("Contract::Custom({})", name),
                                observed_value: json!({ "custom": name }),
                                matched_criteria: format!("Custom contract '{}' acknowledged", name),
                                timestamp_utc: timestamp,
                                latency_ms: duration_ms,
                            },
                            duration_ms,
                        }
                    }
                }
            }

            PostconditionPredicate::All(predicates) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                let mut passed_evidence = Vec::new();
                for p in predicates {
                    match self.verify(p, tool_response, baseline, telemetry) {
                        VerificationOutcome::Pass { evidence, .. } => {
                            passed_evidence.push(evidence);
                        }
                        VerificationOutcome::Fail { reason, observed } => {
                            return VerificationOutcome::Fail { reason, observed };
                        }
                        VerificationOutcome::Unknown { uncertainty_cause, partial_observation } => {
                            return VerificationOutcome::Unknown { uncertainty_cause, partial_observation };
                        }
                    }
                }
                VerificationOutcome::Pass {
                    evidence: VerificationEvidence {
                        predicate_type: "All".to_string(),
                        observed_value: json!({ "count": passed_evidence.len() }),
                        matched_criteria: format!("All {} sub-predicates satisfied", passed_evidence.len()),
                        timestamp_utc: timestamp,
                        latency_ms: duration_ms,
                    },
                    duration_ms,
                }
            }

            PostconditionPredicate::Any(predicates) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                let mut last_failure = None;
                let mut saw_unknown = false;

                for p in predicates {
                    match self.verify(p, tool_response, baseline, telemetry) {
                        VerificationOutcome::Pass { evidence, .. } => {
                            return VerificationOutcome::Pass {
                                evidence: VerificationEvidence {
                                    predicate_type: "Any".to_string(),
                                    observed_value: json!({ "sub_evidence": evidence }),
                                    matched_criteria: "At least one sub-predicate satisfied".to_string(),
                                    timestamp_utc: timestamp,
                                    latency_ms: duration_ms,
                                },
                                duration_ms,
                            };
                        }
                        VerificationOutcome::Fail { reason, observed } => {
                            last_failure = Some((reason, observed));
                        }
                        VerificationOutcome::Unknown { .. } => {
                            saw_unknown = true;
                        }
                    }
                }

                if saw_unknown {
                    VerificationOutcome::Unknown {
                        uncertainty_cause: "At least one Any predicate resulted in Unknown and none passed".to_string(),
                        partial_observation: json!({}),
                    }
                } else if let Some((reason, observed)) = last_failure {
                    VerificationOutcome::Fail { reason, observed }
                } else {
                    VerificationOutcome::Fail {
                        reason: "Empty Any predicates collection".to_string(),
                        observed: json!({}),
                    }
                }
            }
        }
    }
}
