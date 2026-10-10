<p align="center">
  <img src="public/kage-logo.png" alt="KAGE Logo" width="130" />
</p>

<h1 align="center">KAGE (影)</h1>
<h3 align="center">The Governed Browser Runtime for AI Agents</h3>

<p align="center">
  <em>A governed browser runtime for AI agents, with controlled actions, verified outcomes, and human takeover.</em>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Host-Tauri%20v2%20(Rust)-orange?style=flat-square&logo=rust" alt="Tauri Rust" />
  <img src="https://img.shields.io/badge/Engine-Chromium%20CEF%20152-blue?style=flat-square&logo=googlechrome" alt="Chromium CEF" />
  <img src="https://img.shields.io/badge/Frontend-React%2018%20%2B%20TypeScript-61dafb?style=flat-square&logo=react" alt="React 18" />
  <img src="https://img.shields.io/badge/Contracts-100%25%20Verified-success?style=flat-square" alt="Contract Gates" />
  <img src="https://img.shields.io/badge/Release-v0.4.1--alpha.dev-blueviolet?style=flat-square" alt="Prerelease" />
  <img src="https://img.shields.io/badge/License-MIT%20%2F%20Apache--2.0-blueviolet?style=flat-square" alt="License" />
</p>

<p align="center">
  <a href="https://github.com/topics/ai-agent"><img src="https://img.shields.io/badge/%20-ai--agent-33202A?style=flat-square&logo=openai&logoColor=DA627D" alt="ai-agent" /></a>
  <a href="https://github.com/topics/browser-automation"><img src="https://img.shields.io/badge/%20-browser--automation-33202A?style=flat-square&logo=googlechrome&logoColor=DA627D" alt="browser-automation" /></a>
  <a href="https://github.com/topics/tauri"><img src="https://img.shields.io/badge/%20-tauri-33202A?style=flat-square&logo=tauri&logoColor=DA627D" alt="tauri" /></a>
  <a href="https://github.com/topics/rust"><img src="https://img.shields.io/badge/%20-rust-33202A?style=flat-square&logo=rust&logoColor=DA627D" alt="rust" /></a>
  <a href="https://github.com/topics/chromium"><img src="https://img.shields.io/badge/%20-chromium-33202A?style=flat-square&logo=chromium&logoColor=DA627D" alt="chromium" /></a>
  <a href="https://github.com/topics/cef"><img src="https://img.shields.io/badge/%20-cef-33202A?style=flat-square&logo=googlechrome&logoColor=DA627D" alt="cef" /></a>
  <a href="https://github.com/topics/devtools"><img src="https://img.shields.io/badge/%20-devtools-33202A?style=flat-square&logo=gnubash&logoColor=DA627D" alt="devtools" /></a>
  <a href="https://github.com/topics/autonomous-agents"><img src="https://img.shields.io/badge/%20-autonomous--agents-33202A?style=flat-square&logo=probot&logoColor=DA627D" alt="autonomous-agents" /></a>
</p>

---

<p align="center">
  <img src="public/preview.png" alt="KAGE Governed Desktop Browser Runtime Preview" width="100%" />
</p>

---

### Three Core Differentiators

* **Governed Actions:** Every browser mutation passes through a policy-adjudicated Rust **Tool Bus** (`INV-02`) with an append-only, SHA-256 hash-chained cryptographic audit ledger (`security_audit.db`). AI never directly controls CDP (`INV-01`, `INV-07`).
* **Verified Outcomes:** Every action has an observable result (`INV-08`). The **Deterministic Verifier** checks live DOM transitions before the agent proceeds—preventing hallucinated success.
* **Human Control:** Instant **Supervisory Takeover** and an atomic **STOP linearization point** (`INV-09`) that closes the admission gate to new dispatches without retrospective rollback illusions.

---

> [!NOTE]
> **Development Status Notice:**  
> • **Milestone 1 (Core Architecture & 34 Contract Gates):** **100% Sealed & Passing** in automated CI (`python scripts/verify_contracts.py`).  
> • **Milestone 2 (Full Product Tested in Motion):** **Operational Acceptance Run Passed**. Verified across 7 physical workflows on live Windows desktop (`E2E-MOTION-01` through `08`: Clean Startup, Omnibox Navigation, Multi-Profile Isolation, Supervisory Takeover, Atomic STOP, Non-Idempotent Recovery, Clean Teardown). Full evidence recorded in [`evidence/`](evidence/) and documented in the [Desktop Acceptance Report](docs/06-testing/Desktop_Acceptance_Report.md).  
> • **Milestone 3 (Model B Release Qualification):** Production OS renderer sandbox restrictions (`Untrusted` integrity `0x0000`, `JOB_OBJECT_UILIMIT_ALL`) are **explicitly pending final release candidate qualification**.

---

## <img src="public/icons/terminal.svg" width="20" height="20" align="absmiddle" /> Quick Start & Releases

> [!TIP]
> **Try the 90-Second Walkthrough:** See the [90-Second Reproducible Demo Guide](docs/01-product/Demo_Guide.md) to witness governed actions, supervisory takeover, and atomic STOP in under two minutes.

### Option A: Pre-Built Development Prerelease
For developers evaluating the desktop runtime without compiling native CEF from source:
* Grab the latest prerelease build from [GitHub Releases](https://github.com/HardikBhaskar2010/Kage/releases/tag/v0.4.1-alpha.dev) *(Development profile build; Model B release sandbox qualification pending)*.

### Option B: Build From Source (60 Seconds)

#### 1. Prerequisites
* **Rust:** 1.78+ (`rustup default stable`)
* **Node.js:** 18+ (`npm` or `pnpm`)
* **Python:** 3.10+ (for CI contract verification)
* **OS:** Windows 10/11 (x86_64), macOS 13+, or Linux (Ubuntu 22.04+)

#### 2. Clone & Install
```bash
git clone https://github.com/HardikBhaskar2010/Kage.git
cd Kage
npm install
```

#### 3. Verify Architectural Contracts
Validate that all 34 internal safety contracts, STOP linearization, and packaging invariants pass:
```bash
python scripts/verify_contracts.py
```

#### 4. Launch the Desktop Browser Workstation
```bash
npm run tauri dev
```

---

## <img src="public/icons/browser.svg" width="20" height="20" align="absmiddle" /> Why KAGE?

Today's browser-automation frameworks (Playwright, Puppeteer, raw CDP WebSockets) were built for deterministic integration testing, **not autonomous AI reasoning**. 

When modern LLMs operate browsers without governance, they run blind, leak secrets, hallucinate action success, and cannot be interrupted safely.

| Challenge | Raw Browser Automation | KAGE Governed Runtime |
|---|---|---|
| **Action Authority** | Unconstrained WebSocket commands directly to CDP | Every action passes through a policy-adjudicated **Rust Tool Bus** (`INV-02`) |
| **Outcome Verification** | Tool invocation is assumed to equal success | **Deterministic Verifier** checks live DOM transitions before proceeding (`INV-08`) |
| **Human Safety & STOP** | Killing the process or brittle script aborts | **Atomic STOP linearization point** freezes dispatches; human takeover is instant (`INV-09`) |
| **Data Protection** | Entire HTML blobs or passwords dumped into LLM context | **Context Engine** token-budgets DOM box models and redacts secrets (`INV-03`, `INV-06`) |
| **Execution Surface** | Headless browsers with bot detection flags | **Native Chromium (CEF 152)** child HWND with isolated multi-profile sessions |
| **Audit & Forensic Trail** | Ephemeral console output | Tamper-evident **SHA-256 hash-chained SQLite ledger** (`security_audit.db`) |

---

## <img src="public/icons/monitor.svg" width="20" height="20" align="absmiddle" /> Workstation Tour

### 1. Autonomous Agent HUD & Audit Lineage
When KAGE's agent operates web pages, every action displays its complete, cryptographically verified lineage in the supervisory HUD:

```text
task_linearization › step_1 › Req: req_981a › Exec: exec_bc23 › Audit: aud_f401 › Vfy: vfy_e812
```

* **Outcome Badges:** `PASS` (Green), `FAIL` (Red), `UNKNOWN` (Amber).
* **Recovery Decisions:** `RETRY`, `ALREADY_RESOLVED`, `PAUSE_FOR_GUIDANCE` (blocks runaway loops on non-idempotent operations).
* **Human Escape Hatch:** Click **Stop** or **Takeover** at any millisecond to immediately halt autonomous dispatches and assume direct manual mouse/keyboard control.

### 2. Hardware-Accelerated Browsing Workstation
Native Win32 child HWND composition inside Tauri v2 WebView2 chrome. 60–120 FPS GPU compositing with mathematically proven zero-overlap geometry across multi-DPI screens.

<p align="center">
  <img src="public/Mockup_Browsing Screen.png" alt="KAGE Live Browsing Screen" width="100%" />
</p>

### 3. The Omnibox Command Hub
A distraction-free, anime-futuristic command hub. Instant workspace switching, pinned developer profiles, live system telemetry, and keyboard-first command dispatch.

<p align="center">
  <img src="public/Mockup_New Tab Screen.png" alt="KAGE New Tab Hub" width="100%" />
</p>

### 4. The Liquid Glass Design System
High visual clarity with translucent acrylic depth, obsidian backings, and warm peach-to-burgundy gradients (`#F9DBBD` → `#450920`). Physics-based micro-interactions, spring gestures, and strict 60 FPS transitions.

<p align="center">
  <img src="public/Design System.png" alt="KAGE Liquid Glass Design System" width="100%" />
</p>

---

## <img src="public/icons/layers.svg" width="20" height="20" align="absmiddle" /> The Five-Plane Architecture

KAGE divides browser control into five isolated planes to guarantee that web content and AI models never gain unauthorized OS or browser authority:

```
┌────────────────────────────────────────────────────────────────────────┐
│ 1. CHROMIUM PLANE (CEF Multi-Process)                                 │
│    Blink Layout • V8 JS Engine • GPU Compositor • Network / TLS Stack  │
│    Isolated Renderer Processes • Win32 / Cocoa Child HWND Surface      │
├────────────────────────────────────────────────────────────────────────┤
│ 2. BROWSER CONTROL PLANE (Rust Core)                                   │
│    BrowserController • TabManager • WindowManager • ProfileManager     │
│    NavigationManager • PermissionManager • DownloadManager • Session    │
├────────────────────────────────────────────────────────────────────────┤
│ 3. DEVELOPER PLANE (CDP Gateway & Telemetry)                           │
│    Multiplexed CDP Sessions • DOM / BoxModel • CSS Computed Styles     │
│    Network Interception (Fetch) • Console Stream • Tracing / Memory    │
├────────────────────────────────────────────────────────────────────────┤
│ 4. GOVERNANCE PLANE (Security & Audit)                                 │
│    ToolBus Dispatcher • Scoped Capability Engine • Secret Sanitizer     │
│    Immutable SHA-256 Hash-Chained Audit Ledger (security_audit.db)     │
├────────────────────────────────────────────────────────────────────────┤
│ 5. AGENT PLANE (Autonomous Reasoning & Verification)                  │
│    Intent Parser • Bounded Planner • BrowserObservation Model          │
│    Deterministic Verifier • Human Takeover Escape Hatch                │
└────────────────────────────────────────────────────────────────────────┘
```

---

## <img src="public/icons/shield.svg" width="20" height="20" align="absmiddle" /> Inviolable Architecture Contracts

All capabilities in KAGE are governed by strict architectural invariants verified in automated CI gates:

| Contract ID | Invariant Principle | Automated CI Verification Gate |
|---|---|---|
| **INV-01** | AI never directly accesses CEF | Static AST inspection + Phase 8/9/10 tests |
| **INV-02** | All mutations pass through ToolBus | Governed pipeline (`eval_js`, `page_click`, etc.) |
| **INV-03** | Web content is data, never authority | Prompt boundary isolation & untrusted framing |
| **INV-04** | Privileged actions require policy approval | Scoped Capability Engine & Tier 1-4 policy check |
| **INV-05** | Mutations require successful audit commit | Two-stage fail-closed ledger commit |
| **INV-06** | Secrets never cross agent boundary | Credential Broker + regex/entropy secret sanitizer |
| **INV-07** | React UI never controls CDP WebSockets | Internal Rust broker boundary (`Tauri IPC` only) |
| **INV-08** | Every action has an observable result | Tripartite Verifier Engine (`PASS`, `FAIL`, `UNKNOWN`) |
| **INV-09** | STOP rejects subsequent dispatches | Atomic `DispatchAdmissionGate` linearization point |
| **INV-10** | Explicit ProfileId + TargetId mapping | Multi-profile cookie and cache isolation |
| **INV-11A**| Renderer crash fails closed cleanly | Engine crash handler & tab boundary isolation |
| **INV-12** | Browser & surface identity explicit | Deterministic TabId / SurfaceId resolution |
| **NO-MOCKS**| Zero fake browser mock paths | Secondary regex anti-regression scanner |

---

## <img src="public/icons/book.svg" width="20" height="20" align="absmiddle" /> Documentation & Specifications

KAGE is fully documented across 25 foundational specifications:

| Area | Specifications |
|---|---|
| **Product & Vision** | [PRD](docs/01-product/PRD.md) · [Requirements](docs/01-product/Requirements.md) · [Adoption Strategy](docs/01-product/Adoption_Strategy.md) · [90-Sec Demo Guide](docs/01-product/Demo_Guide.md) · [Technical Design](docs/Design.md) |
| **Architecture** | [Master Architecture](docs/02-architecture/Architecture.md) · [CEF Integration](docs/02-architecture/CEF_Integration.md) · [Tauri Architecture](docs/02-architecture/Tauri_Architecture.md) · [Architecture Contracts](docs/02-architecture/Architecture_Contracts.md) |
| **Core Subsystems** | [Tool Bus](docs/03-core/Tool_System.md) · [Context Engine](docs/03-core/Context_Engine.md) · [Micro Inspect](docs/03-core/Micro_Inspect.md) · [Workspaces](docs/03-core/Workspace_System.md) |
| **Intelligence** | [AI Architecture](docs/04-ai/AI_Architecture.md) · [DevTools Architecture](docs/05-devtools/DevTools_Architecture.md) |
| **Testing & Quality** | [Testing Lab](docs/06-testing/Testing_Lab.md) · [Testing Strategy](docs/06-testing/Testing_Strategy.md) · [E2E Acceptance Matrix](docs/06-testing/E2E_Acceptance_Matrix.md) · [Desktop Acceptance Report](docs/06-testing/Desktop_Acceptance_Report.md) |
| **Security & Data** | [Security Model](docs/08-security/Security_Model.md) · [Data Model](docs/07-platform/Data_Model.md) · [Plugin API](docs/07-platform/Plugin_API.md) |
| **UI & Experience** | [Liquid Glass Design System](docs/09-ui/Design_System.md) |
| **Milestone Reports** | [Phase 11 Report (M11 Hardened Shell)](docs/reports/M11_Hardened_Shell_Integration.md) · [Ground Truth Audit](docs/KAGE_Ground_Truth_Audit.md) |

---

## <img src="public/icons/activity.svg" width="20" height="20" align="absmiddle" /> Community & Contributing

We welcome contributions from engineers building browser automation, AI agent runtimes, and Chromium-based tools:

* **Contribute Code:** Read [CONTRIBUTING.md](CONTRIBUTING.md) for local environment setup, branching conventions, and PR guidelines.
* **Security & Vulnerabilities:** Review [SECURITY.md](SECURITY.md) for our responsible disclosure policy.
* **Code of Conduct:** Please review our [Code of Conduct](CODE_OF_CONDUCT.md).

---

## <img src="public/icons/file-text.svg" width="20" height="20" align="absmiddle" /> License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.

All rights reserved © 2026 KAGE Project.
