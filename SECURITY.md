# Security Policy — KAGE (影)

Security is the foundational design pillar of **KAGE (影)**. Because KAGE operates an autonomous AI agent capable of driving live browser sessions and mutating web state, security is governed by strict, fail-closed architectural invariants rather than heuristic prompt guardrails.

---

## 1. Supported Versions

Security updates and critical fixes are applied to the active versions listed below:

| Version | Supported | Security Gate Status |
|---|---|---|
| **v0.4.x / Alpha** | :white_check_mark: Yes | Active Development / Core Implementation Verified |
| **< v0.4.0** | :x: No | Historical Milestones |

---

## 2. Inviolable Security Contracts

All code operating within KAGE must adhere to the **10 Inviolable Architecture Contracts**:

* **Web Content is Untrusted (`INV-03`):** Web content, console messages, and HTTP bodies are classified as untrusted data surfaces. They are wrapped in semantic frames and never granted prompt execution authority.
* **Two-Stage Execution Gate (`INV-04`, `INV-05`):** All mutations route through the typed Rust `ToolBus`. Privileged capabilities require capability policy adjudication, and execution fails closed if the append-only SHA-256 audit ledger cannot record the mutation.
* **Credential Broker (`INV-06`):** Authentication cookies, session tokens, passwords, and bearer keys are filtered and redacted before context assembly. Raw credentials never enter LLM prompt contexts.
* **Deterministic STOP Linearization (`INV-09`):** A human "STOP" command immediately rejects future mutation dispatches at the admission gate (`DispatchAdmissionGate`).

---

## 3. Reporting a Vulnerability

If you discover a security vulnerability in KAGE, please report it privately:

**DO NOT file a public GitHub issue for security vulnerabilities.**

### Contact
* **Primary Contact:** Hardik Bhaskar
* **Email:** [hardik.bhaskar2010@gmail.com](mailto:hardik.bhaskar2010@gmail.com)
* **Subject Line:** `[SECURITY] Vulnerability Report in KAGE: <Brief Description>`

### What to Include in Your Report
Please provide as much information as possible to help us reproduce and resolve the issue quickly:
1. Description of the vulnerability and its potential impact.
2. Step-by-step reproduction instructions or a minimal Proof of Concept (PoC).
3. The specific component affected (e.g., `kage-core`, `ToolBus`, `CdpBroker`, `Sanitizer`, CEF Sandbox).
4. Environment details (OS, version, hardware architecture).

---

## 4. Response & Disclosure Process

1. **Acknowledgment:** We will acknowledge receipt of your vulnerability report within **48 hours**.
2. **Assessment:** The security team will validate the issue and determine severity within **5 business days**.
3. **Patch & Mitigation:** We will prepare and test a patch across our automated CI contract gates.
4. **Coordinated Disclosure:** A public security advisory and CVE (if applicable) will be published alongside the release containing the fix.

We appreciate the efforts of the security research community in helping keep KAGE safe and resilient!
