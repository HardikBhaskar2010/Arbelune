# KAGE End-to-End (E2E) Product Acceptance Matrix

| Document Metadata | Details |
|---|---|
| **Document ID** | `KAGE-TEST-003` |
| **Status** | Active Operational Acceptance Matrix (Campaign In Progress) |
| **Version** | v0.4.3 (Milestone 2 Campaign Specification) |
| **Architecture Phase** | Milestone 2: Full Product Tested in Motion |
| **Classification** | Integrated Dual-Surface Verification & Motion Campaign |

---

## 1. The Three Milestone Horizons

To ensure complete engineering transparency, KAGE rigorously separates implementation claims from live desktop motion testing and release qualification:

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                              THE THREE MILESTONE HORIZONS                              │
├──────────────────────────┬─────────────────────────────┬───────────────────────────────┤
│ MILESTONE 1              │ MILESTONE 2 (CURRENT FOCUS) │ MILESTONE 3                   │
│ Core Architecture        │ Full Product in Motion      │ Fully Qualified Release       │
├──────────────────────────┼─────────────────────────────┼───────────────────────────────┤
│ [SEALED & COMPLETE]      │ [AWAITING MOTION RUNS]      │ [PENDING RC VALIDATION]       │
│ Governance, ToolBus,     │ Live desktop app execution  │ Model B release bootstrap,    │
│ Agent runtime, STOP gate,│ through end-to-end user     │ CEF 152 per-role token        │
│ Native CEF child HWND,   │ workflows with real LLM     │ integrity (Untrusted renderer,│
│ Multi-DPI layout math,   │ provider & physical dual-   │ Low GPU/Network), signed      │
│ HUD IPC integration.     │ surface automation.         │ installer qualification.      │
└──────────────────────────┴─────────────────────────────┴───────────────────────────────┘
```

> [!IMPORTANT]
> **Ground-Truth Distinction:** Phase 1–11 automated CI test suites (`scripts/verify_contracts.py`) confirm that the Rust control plane, ToolBus, admission gates, and layout geometry are functionally sound under unit and synthetic integration tests. However, Phase 9/10 CI gates utilized a deterministic test harness rather than live external LLM API calls. **Milestone 2 exercises the full, un-mocked application in motion. No feature is deemed "Passed" in Milestone 2 until physical execution produces verified traces.**

---

## 2. Dual-Surface Desktop Testing Architecture

Because KAGE composites a native Win32 child HWND inside Tauri v2 WebView2 presentation chrome, standard browser automation (such as Playwright running in standalone browser mode) cannot locate or manipulate the native CEF surface directly.

KAGE establishes a **Tripartite Testing Architecture** with strictly delineated input and assertion channels:

```text
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                          KAGE DUAL-SURFACE TESTING STACK                               │
├───────────────────────────┬───────────────────────────────┬────────────────────────────┤
│ TIER / SURFACE            │ INPUT CHANNELS                │ ASSERTION CHANNELS         │
├───────────────────────────┼───────────────────────────────┼────────────────────────────┤
│ 1. Presentation Layer     │ Desktop WebDriver             │ React DOM element states,  │
│    (Tauri UI / WebView2)  │ (tauri-driver / WebdriverIO)  │ HUD outcome badges,        │
│                           │ clicks tabs, omnibox, HUD     │ omnibox URL synchronization│
├───────────────────────────┼───────────────────────────────┼────────────────────────────┤
│ 2. Native Browsing Plane  │ • Agent: ToolBus / CDP        │ • CDP DOM / AX tree digest │
│    (Child HWND / Blink)   │   Input.dispatchMouseEvent    │ • CDP Page.loadEventFired  │
│                           │ • Human: OS System Input      │ • CDP Runtime.evaluate     │
│                           │   (SetForeground + SendInput) │ • Multi-DPI HWND bounds    │
├───────────────────────────┼───────────────────────────────┼────────────────────────────┤
│ 3. Authoritative Ledger   │ Dispatched ToolRequests via   │ Parameterized SQLite audit │
│    & Governance Boundary  │ ToolBus::dispatch             │ queries, SHA-256 hash chain│
│    (security_audit.db)    │ and AdmissionGate             │ lineage IDs, Verifier state│
└───────────────────────────┴───────────────────────────────┴────────────────────────────┘
```

### Channel & Input Mechanism Definitions

#### A. Input Channels
1. **Presentation / UI Input (`tauri-driver` / WebdriverIO):** Drives the outer OS window and React shell DOM elements (`#omnibox-input`, `#tab-strip`, `#btn-takeover`, `#btn-stop`).
2. **OS System Input Stream (`SendInput`):** Inserts synthesized keystrokes and mouse events into the Windows input stream via the system input manager.
   * *Critical Windows Requirement:* `SendInput` operates at the Windows input subsystem level and does **not** accept a target `HWND`. Windows can refuse foreground activation under its foreground-window lock rules (`LockSetForegroundWindow`). Therefore, the test driver must actively request foreground state (`SetForegroundWindow(host_hwnd)`), verify that foreground activation actually succeeded (`GetForegroundWindow()`), and ensure keyboard focus is on the native browser content before dispatching input.
   * *Role:* This is the **primary user-like input path** for testing human typing and clicking in web content during supervisory takeover.
3. **Targeted Window Message Injection (`PostMessage`/`SendMessage` with `WM_KEYDOWN`, `WM_CHAR`):** Posts messages directly into a target window's message queue.
   * *Critical Distinction:* Posting messages to an `HWND` bypasses the Windows input subsystem and keyboard state tables (`GetKeyboardState`, `GetAsyncKeyState`). It does not reliably reproduce keyboard focus state or complex text-input composition.
   * *Role:* Retained strictly as an explicitly scoped **diagnostic test** (`E2E-DIAG-WINMSG`) and never labeled as OS-level or physical hardware input.
4. **Agent Action Input (`ToolBus::dispatch` → Internal CDP Gateway):** Autonomous agent loop streaming typed tool proposals into Tauri IPC `tool_dispatch` → `ToolBus::dispatch` → Internal CDP session commands (`Input.dispatchMouseEvent`, `Input.dispatchKeyEvent`).
5. **IPC Navigation Transport (`navigate_to`):** Human omnibox entry invokes Tauri IPC `navigate_to(tab_id, url)` → `TabManager::navigate` → `CefRuntime::load_url_by_browser_id`.
   * *Architectural Rule:* Human omnibox navigation is direct host browser navigation governed by `CEF-06` and `CEF-05`. It does **not** route through the agent `ToolBus` (`INV-02`).

#### B. Assertion Channels & HWND Hierarchy Verification
1. **Presentation Assertions:** Validates React component trees, active tab indicators, HUD status badges (`PASS`, `FAIL`, `UNKNOWN`, `STOPPED`), and trace lineage formatting.
2. **Win32 Host Windowing Assertions (HWND Hierarchy):**
   * *KAGE's Embedded CEF Browser/Surface HWND (`CefBrowserHost::GetWindowHandle()`):* Assert that this native browser window is a direct child of the Tauri host window (`GetParent(browser_hwnd) == host_hwnd`) and matches the calculated layout aperture bounds (`cef_content_rect`).
   * *Render-Widget Descendant HWND (e.g., `Chrome_RenderWidgetHostHWND`):* Inspect and record the actual descendant window tree under the host and browser surface. Verify its actual location in the window hierarchy and confirm its bounds align with the active content viewport. Preserve this as an observed, build-specific assertion rather than an unverified universal architectural assumption.
3. **Chromium Engine Assertions:** Validates real web page state via internal CDP session (`DOM.getDocument`, `Accessibility.getFullAXTree`, `Runtime.evaluate`, `Page.loadEventFired`).
4. **Authoritative Truth Ledger Assertions:** Validates tamper-evident SHA-256 hash chains in `security_audit.db`, correlation of `task_id`, `step_id`, `request_id`, `execution_id`, `audit_id`, `verifier_id`, and `IdempotencyMatrix` recovery decisions (`PAUSE_FOR_GUIDANCE`).
   * *Applicability Rule:* When a test does not execute an agent action (e.g., cold startup, human omnibox navigation, tab strip switching), the Governance Ledger channel is explicitly marked `N/A (Human action / no ToolBus dispatch)`. No test forces a synthetic audit record where none architecturally exists.

---

## 3. Product-Wide End-to-End Acceptance Matrix

> [!CAUTION]
> **Operational Acceptance Campaign Rule:** Status labels in this matrix reflect **campaign execution status**, not code implementation status. The label `Awaiting Desktop Motion Run` indicates that the underlying architecture and CI unit/synthetic gates have passed, but the integrated workflow has **not yet been physically executed in the live desktop application**. A feature only transitions to `PASSED` when an end-to-end motion run records verified evidence traces.

### Deterministic Safety Acceptance vs. Live Model Acceptance
To prevent non-deterministic LLM variability (latency, hallucinated tool selections, prompt drift) from obscuring core safety guarantees, KAGE splits agent acceptance into two distinct gates:
* **Deterministic Safety Acceptance (Rows 09–11):** Exercises supervisory takeover, STOP linearization, and uncertain recovery against a **controlled planner/model fixture** to reliably verify the admission gate, token cancellation, and idempotency logic under known conditions.
* **Live-Model Acceptance (Rows 06–08, 13):** Exercises the complete autonomous workflow against an **external LLM provider** to evaluate prompt interpretation, tool proposal discovery, and dynamic page comprehension.

| # | User-Visible Feature | Interaction Surfaces & Input Channel | Architecture Contract | Model Dependency | Motion Campaign Status | Observable Assertion Channels & Pass Criteria |
|---|---|---|---|---|---|---|
| **01** | **Clean Startup & Engine Init** | Win32 Host + Tauri + CEF Native Surface<br>*(Launcher Input via WebDriver)* | `CEF-01`, `CEF-05` | Deterministic (No LLM) | **PASSED (Verified in Motion Run)** | • **Tauri:** `app_ready` IPC event emitted.<br>• **Host:** `CefEngineState::BrowserCreationAllowed`.<br>• **Win32:** CEF browser window parented to host (`GetParent == host`); descendant render-widget HWND location and viewport alignment recorded build-specifically.<br>• **Process:** Scoped process-tree working set $< 150\text{ MB}$ (product budget).<br>• **Governance:** *N/A (Host lifecycle).* |
| **02** | **Omnibox URL Navigation** | Tauri UI (`#omnibox-input`) → IPC `navigate_to` → Host → CEF<br>*(UI Input via WebDriver)* | `CEF-06`, `CEF-05` | Human Driven (No LLM) | **PASSED (Verified in Motion Run)** | • **Win32:** Browser surface HWND bounds match calculated aperture.<br>• **CEF:** CDP `Page.frameNavigated` & `Page.loadEventFired`; `document.title` matches.<br>• **Tauri:** Omnibox reflects URL; security badge shows TLS state.<br>• **Governance:** *N/A (Human navigation via Tauri IPC; no ToolBus dispatch).* |
| **03** | **Tab Strips & Lifecycle** | Tauri UI (`#tab-strip`) → IPC `create_tab` / `switch_tab` / `close_tab`<br>*(UI Input via WebDriver)* | `INV-10`, `CEF-05` | Human Driven (No LLM) | **PASSED (Verified in Motion Run)** | • **Win32:** Switching tabs toggles child HWND visibility.<br>• **CEF:** TabManager binds `BrowserSurfaceId` and browser ID.<br>• **Host:** Closing tab destroys child HWND with zero handle leaks.<br>• **Governance:** *N/A (Human tab management).* |
| **04** | **Multi-Profile Isolation** | Profile Picker → `ProfileManager` → CEF Request Contexts<br>*(UI Input via WebDriver)* | `INV-10` | Deterministic (No LLM) | **PASSED (Verified in Motion Run)** | • **CEF/CDP:** Cookie set in Personal profile tab is completely absent in Ephemeral Sandbox tab.<br>• **Storage:** In-memory sandbox cache wiped on tab close.<br>• **Governance:** *N/A (Session isolation).* |
| **05** | **Window Resize & Multi-DPI** | Win32 Host Window → `NativeSurfaceManager`<br>*(OS Window Resize Input)* | `CEF-06A..C` | Deterministic (No LLM) | **Awaiting Desktop Motion Run** | • **Win32:** `GetWindowRect(browser_hwnd)` equals calculated `cef_content_rect` with 0px overlap across 100%, 150%, and 200% DPI scaling.<br>• **Governance:** *N/A (Layout geometry).* |
| **06** | **Autonomous Agent Task** | AI Sidebar (`#agent-prompt-input`) → Runtime → ToolBus → CEF<br>*(Natural Language Prompt)* | `INV-01`, `INV-02`, `INV-04` | **Real Configured LLM Provider** | **Awaiting Live Model Run** | • **Runtime:** LLM tool proposals dispatched via `ToolBus`.<br>• **CEF:** Actions execute via CDP `Input.dispatchMouseEvent` / `KeyEvent`.<br>• **HUD:** Displays `PASS` badge.<br>• **Governance:** `security_audit.db` logs full SHA-256 lineage (`req_xxx`, `exec_xxx`, `aud_xxx`). |
| **07** | **DOM & AX Tree Observation** | Context Engine ↔ Internal CDP Gateway<br>*(Agent Observation Snapshot)* | `INV-03`, `INV-06` | **Real Configured LLM Provider** | **Awaiting Live Model Run** | • **Context:** Token budget $< 4,000$ tokens.<br>• **Security:** Zero passwords, authorization headers, or session cookies present in LLM prompt payload.<br>• **Governance:** Sanitizer audit verification. |
| **08** | **Postcondition Verification** | Verifier Engine ↔ Internal CDP Gateway<br>*(Agent Mutating Tool Execution)* | `INV-08` | **Real Configured LLM Provider** | **Awaiting Live Model Run** | • **CEF/CDP:** Verifier evaluates postcondition against live DOM transition.<br>• **HUD:** Outcome rendered as verified `PASS`, `FAIL`, or `UNKNOWN` badge.<br>• **Governance:** `VerificationResult` committed to audit ledger. |
| **09** | **Supervisory Takeover** | Agent HUD (`#btn-takeover`) + Native Child HWND<br>*(UI Click + OS `SendInput`)* | `INV-09`, `INV-04` | **Controlled Safety Fixture** | **Awaiting Desktop Motion Run** | • **Host:** AdmissionGate blocks agent tool dispatches.<br>• **CEF:** Verified foreground focus; OS `SendInput` modifies form fields in child HWND; CDP verifies field value.<br>• **Context:** Resuming forces fresh observation `obs_fresh_resync_xxx` before next step.<br>• **Governance:** Takeover state recorded in audit trail. |
| **10** | **Atomic STOP Under Load** | Agent HUD (`#btn-stop`) → `DispatchAdmissionGate`<br>*(UI Click during in-flight action)* | `INV-09` | **Controlled Safety Fixture** | **Awaiting Desktop Motion Run** | • **Admission Gate:** Establishes STOP linearization point; unconditionally rejects subsequent dispatches.<br>• **In-Flight Action:** Requests cooperative cancellation where supported, but allows an already-dispatched CEF action to settle if it cannot be preempted.<br>• **Postcondition:** Proves zero new tool dispatches admitted after STOP linearization point; records whether in-flight action completed, failed, or remains uncertain.<br>• **HUD:** Displays `STOPPED` without implying retrospective rollback.<br>• **Governance:** Records STOP event, linearization point, rejected dispatch attempts, and final outcome of in-flight action. |
| **11** | **Uncertain Recovery Guard** | Recovery Engine ↔ Verifier<br>*(Mutating Action + Synthetic ACK Drop)* | `INV-08` | **Controlled Safety Fixture** | **Awaiting Desktop Motion Run** | • **Verifier:** Missing transition ACK yields `UNKNOWN`.<br>• **Planner:** `IdempotencyMatrix` triggers `PAUSE_FOR_GUIDANCE` (blocks auto-retry).<br>• **HUD:** Human guidance prompt displayed.<br>• **Governance:** Audit ledger records `Indeterminate` state. |
| **12** | **Micro Inspect Overlay** | React HUD Overlay ↔ Internal CDP `Overlay`<br>*(UI Shortcut / Click Input)* | `INV-07` | Deterministic (No LLM) | **Awaiting Desktop Motion Run** | • **CEF:** CDP highlights hovered element bounding box.<br>• **Tauri UI:** Inspect card renders box model and computed CSS without direct WebSocket exposure.<br>• **Governance:** *N/A (Telemetry overlay).* |
| **13** | **Trace Lineage Card** | Agent HUD Trace Inspector<br>*(Task Step Execution)* | `INV-05` | **Real Configured LLM Provider** | **Awaiting Live Model Run** | • **HUD:** Card displays verifiable lineage breadcrumb matching SQLite: `Task › Step › Req: req_xxx › Exec: exec_xxx › Audit: aud_xxx › Vfy: vfy_xxx`.<br>• **Governance:** Hash-chain linkage verified. |
| **14** | **Clean App Teardown** | Win32 Host Window → Teardown Coordinator<br>*(OS Window Close Input)* | `TEARDOWN`, `CEF-01` | Deterministic (No LLM) | **Awaiting Desktop Motion Run** | • **Win32/CEF:** Two-stage teardown drains message loop and calls `cef::shutdown()`.<br>• **Process:** All host and helper PIDs exit within 3,000 ms with 0 orphaned processes.<br>• **Governance:** *N/A (Process termination).* |
| **15** | **Production Sandbox Enforcement** | Model B Release Bootstrap (`KAGE.exe`)<br>*(Production Binary Execution)* | `CEF-03b-C/D` | Packaging / Security (No LLM) | **Awaiting Milestone 3 RC Run** | • **Security:** Windows token inspection confirms exact per-role policies: Renderer (`Untrusted` integrity `0x0000`, `JOB_OBJECT_UILIMIT_ALL`, restricted token SIDs, 0 write privileges); Utility/GPU (`Low` integrity `0x1000` or AppContainer); Host (Limited Medium).<br>• **Governance:** Genesis hash verification. |

---

## 4. The 8 End-to-End System Motion Tests

These 8 operational workflows test KAGE as a complete, running desktop application:

### Test 1: Clean Startup & Surface Readiness (`E2E-MOTION-01`)
* **Goal:** Verify that KAGE launches from a cold state, initializes both rendering engines, and establishes live IPC without race conditions.
* **Input Channels:**
  * **Process Launcher:** WebDriver / `tauri-driver` launches `kage-host.exe`.
* **Transport & Orchestration:**
  * Host creates Win32 parent window; initializes CEF runtime (`CefInitialize`); mounts Tauri presentation webview.
* **Assertion Channels & Pass Criteria:**
  * **Tauri UI:** Await `app_ready` IPC event from React presentation shell.
  * **Rust Host:** Await `CefEngineState::BrowserCreationAllowed` event from host engine.
  * **Win32 Windowing (HWND Hierarchy):**
    * *KAGE's Embedded CEF Browser/Surface HWND:* Query Win32 API (`CefBrowserHost::GetWindowHandle()`) to confirm browser window is a direct child of the host window (`GetParent(browser_hwnd) == host_hwnd`) and matches the calculated aperture bounds.
    * *Render-Widget HWND:* Recursively inspect child windows in the hierarchy under host and browser surface. Verify the actual location of the render-widget HWND (e.g. `Chrome_RenderWidgetHostHWND`) and its expected relationship to the content viewport (client rect matching aperture). Distinguish native browser surface handle from descendant widget handles, preserving observed hierarchy build-specifically without hardcoded assumptions.
  * **Process Telemetry (Scoped Process-Tree Memory Budget):**
    * *Scoping Protocol:* Identify the root `kage-host.exe` PID; recursively query `Win32_Process` (`ParentProcessId == current_set`) to collect the complete child process tree (including all `kage-cef-subprocess.exe` renderers, GPU, and utility helpers). Sum `WorkingSet64` strictly across this PID set. (Note: `Get-Process | Measure-Object -Property WorkingSet64 -Sum` without PID filtering is invalid as it sums all system processes).
    * *Sampling Protocol:* Launch cold to `about:blank`, settle for 5.0 seconds into idle state, sample 3 readings at 1,000 ms intervals, calculate the arithmetic mean.
    * *Product Budget:* Mean process-tree working set $< 150\text{ MB}$ (treated as a measurable product budget, not proof of "zero interference").
  * **Governance Ledger:** `N/A (Host initialization; no agent tool dispatches)`.

### Test 2: Real Navigation & Viewport Alignment (`E2E-MOTION-02`)
* **Goal:** Verify the complete path from human Omnibox input to native web rendering, viewport geometry, and URL synchronization.
* **Input Channels:**
  * **Tauri UI Input:** WebDriver inputs `https://test.kage.dev/checkout-lab` into `#omnibox-input` and sends `Enter`.
* **Transport & Orchestration:**
  * React invokes Tauri IPC `navigate_to(tab_id, url)`.
  * `TabManager::navigate` updates navigation history; calls `CefRuntime::load_url_by_browser_id`. *(Governed by `CEF-06`, `CEF-05`; not ToolBus).*
* **Assertion Channels & Pass Criteria:**
  * **Win32 Windowing:** `GetWindowRect(browser_hwnd)` coordinates match calculated layout aperture (`cef_content_rect`) with 0px overlap with React chrome.
  * **Chromium Content (CDP):** CDP session receives `Page.frameNavigated` and `Page.loadEventFired`; `Runtime.evaluate("document.title")` asserts `"KAGE Checkout Lab"`.
  * **Tauri UI:** Omnibox updates display URL, tab title reflects page title, and security lock icon updates to secure (`is_secure == true`).
  * **Governance Ledger:** `N/A (Human navigation via Tauri IPC; no ToolBus dispatch)`.

### Test 3: Tab Strips & Multi-Profile Isolation (`E2E-MOTION-03`)
* **Goal:** Verify that multiple tabs running in distinct profiles maintain strict cookie and storage isolation.
* **Input Channels:**
  * **Tauri UI Input:** WebDriver clicks `#tab-new` and selects profile; switches active tab via `#tab-{id}`.
* **Transport & Orchestration:**
  * React invokes Tauri IPC `create_tab` / `switch_tab` / `close_tab`.
  * `TabManager` binds `BrowserSurfaceId` and toggles CEF browser visibility via `set_browser_visible_by_id`.
* **Assertion Channels & Pass Criteria:**
  * **Chromium Content (CDP):** Tab 1 (Personal Profile) navigates to test site and sets cookie `auth=personal_token`. Tab 2 (Ephemeral Sandbox Profile) inspects `document.cookie` via its CDP session: asserts cookie is empty.
  * **Win32 Windowing:** Active tab's browser HWND is visible; inactive tab's browser HWND is hidden.
  * **Storage/Cleanup:** Closing Tab 2 destroys browser HWND and wipes in-memory sandbox cache.
  * **Governance Ledger:** `N/A (Tab strip management; no agent tool dispatch)`.

### Test 4: Live Autonomous Task with Configured Model Provider (`E2E-MOTION-04`)
* **Goal:** Verify the autonomous agent loop using an actual configured AI provider (OpenRouter / Anthropic / OpenAI), without canned mock responses.
* **Input Channels:**
  * **Tauri UI Input:** WebDriver enters natural language prompt into AI Sidebar: `"Navigate to checkout lab, select the standard shipping option, and enter postal code 94105."`
  * **Agent Action Input:** Live external LLM provider proposes typed tool calls (`page_click`, `page_fill`).
* **Transport & Orchestration:**
  * AI runtime dispatches requests via `ToolBus::dispatch`.
  * `ToolBus` adjudicates capabilities; commits pre-execution intent to `security_audit.db`; invokes internal CDP session (`Input.dispatchMouseEvent`, `Input.dispatchKeyEvent`).
* **Assertion Channels & Pass Criteria:**
  * **Chromium Content (CDP):** Form fields in child HWND update with requested postal code; standard shipping radio button is selected.
  * **Agent Verifier:** Evaluates postconditions against live DOM and emits outcome `PASS` with fresh `VerificationId`.
  * **Tauri UI:** HUD renders `PASS` badge and updates step progress.
  * **Truth Ledger:** `security_audit.db` contains complete SHA-256 hash-chained trace linking `request_id`, `execution_id`, and `audit_id`.

### Test 5: Human Supervisory Takeover & Observation Resync (`E2E-MOTION-05`)
* **Goal:** Confirm that human takeover blocks agent mutations and forces a fresh page re-observation upon resumption.
* **Model Dependency:** **Controlled Safety Fixture** (ensures deterministic timing for takeover trigger).
* **Input Channels:**
  * **Tauri UI Input:** While autonomous task is in-flight, WebDriver clicks `#btn-takeover` in React HUD.
  * **OS System Input Stream (`SendInput`):**
    1. Activate the KAGE host window and establish focus on the intended native browser content (`SetForegroundWindow(host_hwnd)`).
    2. Verify foreground activation actually succeeded (`GetForegroundWindow()`); do not assume `SetForegroundWindow` always succeeds under Windows foreground lock rules.
    3. Submit the input with `SendInput` (synthesizing keystrokes into the system input stream without a target `HWND`) and verify the form field changed through CDP (`Runtime.evaluate`).
    4. Resume the agent and verify that a fresh observation reflects the change before the next action.
  * *Diagnostic Note (`E2E-DIAG-WINMSG`):* Direct window-message injection (`PostMessage` with `WM_KEYDOWN`/`WM_CHAR`) is maintained as a separate, scoped diagnostic test and is **not** conflated with user-like input.
  * **Tauri UI Input:** WebDriver clicks `#btn-resume` in React HUD.
* **Transport & Orchestration:**
  * `#btn-takeover` transitions `DispatchAdmissionGate` to `Takeover` state.
  * `#btn-resume` resets gate and triggers Context Engine page resynchronization.
* **Assertion Channels & Pass Criteria:**
  * **Host Governance:** Agent tool dispatches attempted during takeover are denied with `ToolError::AdmissionBlocked`.
  * **Chromium Content (CDP):** Child HWND DOM reflects text entered via `SendInput`.
  * **Agent Context:** Context Engine captures fresh observation `obs_fresh_resync_xxx` with updated DOM digest before the planner generates subsequent actions.
  * **Truth Ledger:** Takeover transition committed to audit log.

### Test 6: Atomic STOP Under Load (`E2E-MOTION-06`)
* **Goal:** Prove that STOP enforces atomic fail-closed rejection without making false promises of retrospective rollback (Strict `INV-09` Compliance).
* **Model Dependency:** **Controlled Safety Fixture** (dispatches a repeatable, predictable 5-step mutation chain).
* **Input Channels:**
  * **Agent Action Input:** Agent fixture dispatches 5 sequential mutating clicks.
  * **Tauri UI Input:** While action 2 is in-flight, WebDriver clicks `#btn-stop` in React HUD.
* **Transport & Orchestration:**
  * `DispatchAdmissionGate::stop()` cancels the `CancellationToken` and establishes the STOP linearization point.
* **Assertion Channels & Pass Criteria:**
  * **Admission Gate:** Establish the STOP linearization point and reject all subsequent dispatch attempts (actions 3, 4, 5) with `ToolError::Cancelled`.
  * **In-Flight Action:** Request cooperative cancellation where supported, but allow an already-dispatched CEF action (action 2) to settle if it cannot be preempted.
  * **Postcondition:** Prove that no new tool dispatch is admitted after the STOP linearization point. Record whether the in-flight action completed, failed, or remains uncertain. *(The acceptance condition is zero new dispatch admissions after the STOP linearization point, not zero mutation effects completing after STOP).*
  * **HUD:** Displays `STOPPED` without implying retrospective rollback.
  * **Governance:** Record the STOP event, its linearization point, rejected dispatch attempts, and the final outcome of the in-flight action with unbroken SHA-256 hash chains in `security_audit.db`.

### Test 7: Uncertain Outcome Recovery & Non-Idempotent Protection (`E2E-MOTION-07`)
* **Goal:** Ensure uncertain execution outcomes (`UNKNOWN`) do not trigger duplicated mutations for non-idempotent operations.
* **Model Dependency:** **Controlled Safety Fixture & Synthetic Fault Injection**.
* **Input Channels:**
  * **Agent Action Input:** Agent fixture executes non-idempotent tool `page_click("#submit-order-button")`.
  * **Fault Injection:** Synthetic CDP network disconnect simulates a lost transition acknowledgement.
* **Transport & Orchestration:**
  * Deterministic Verifier attempts postcondition verification; detects missing transition ACK; returns `UNKNOWN`.
  * Planner passes outcome to `IdempotencyMatrix`.
* **Assertion Channels & Pass Criteria:**
  * **Agent Recovery:** Because order submission is non-idempotent (`is_idempotent == false`), recovery engine halts with `PAUSE_FOR_GUIDANCE` rather than auto-retrying.
  * **Tauri UI:** HUD displays developer guidance modal with options: `[Inspect Page State]` and `[Confirm Mutation]`.
  * **Truth Ledger:** Audit log records `Indeterminate` verification status without auto-retry re-invocation.

### Test 8: Graceful Teardown & Zero-Orphan Cleanup (`E2E-MOTION-08`)
* **Goal:** Verify that closing the application terminates all native child processes without zombie helper leaks.
* **Input Channels:**
  * **Tauri UI / OS Input:** WebDriver issues window close (`appWindow.close()` / `WM_CLOSE`).
* **Transport & Orchestration:**
  * Two-stage teardown protocol executes: `NativeSurfaceManager` detaches child HWNDs; `CefDoMessageLoopWork` drains; host calls `cef::shutdown()`.
* **Assertion Channels & Pass Criteria:**
  * **Process Telemetry:** Windows process enumeration confirms all `kage-host.exe` and `kage-cef-subprocess.exe` PIDs exit cleanly within 3,000 ms.
  * **Zero Orphan:** Zero orphaned Chromium child processes remain in Windows Task Manager.
  * **Governance Ledger:** `N/A (Process termination)`.

---

## 5. Evidence Recording Standards

For every test executed during the Milestone 2 campaign, an immutable evidence record must be logged:

| Evidence Field | Required Record Specification |
|---|---|
| **Identity** | Test ID (`E2E-MOTION-xx`), Build Git SHA, Environment (Windows OS version, display DPI scale), and ISO-8601 Timestamp. |
| **Execution** | Actual physical execution status: `NOT RUN`, `IN PROGRESS`, `PASSED`, `FAILED`, or `BLOCKED`. *(Never implementation readiness).* |
| **UI Surface** | Observed HUD badges (`PASS`, `FAIL`, `UNKNOWN`, `STOPPED`), tab strip count/active ID, omnibox display URL, and TLS lock indicator. |
| **Native Browser** | Win32 HWND hierarchy (`GetParent == host`), observed browser and render-widget window classes, physical window rect coordinates, target page URL, and CDP DOM digest. |
| **Governance Ledger** | Hash-chained audit lineage (`task_id`, `step_id`, `request_id`, `execution_id`, `audit_id`, `verifier_id`) with SHA-256 chain validation. *(Explicitly recorded as `N/A: <Justification>` for non-agent tests).* |
| **Artifacts** | Execution log file URI, scoped process memory telemetry samples, CDP event logs, and failure stack traces where applicable. |

---

## 6. Acceptance Campaign Execution Tracker

> [!NOTE]
> Checking an item below is strictly reserved for when an automated test or interactive campaign run records an immutable execution trace demonstrating that all applicable assertion channels passed.

- [x] `E2E-MOTION-01`: Clean Startup & Surface Readiness *(Status: PASSED; verified in motion run 2026-10-10; evidence/e2e_motion_01.json)*
- [x] `E2E-MOTION-02`: Real Omnibox Navigation & Child HWND Placement *(Status: PASSED; verified in motion run 2026-10-10; evidence/e2e_motion_02.json)*
- [x] `E2E-MOTION-03`: Tab Strip Lifecycle & Multi-Profile Cookie Isolation *(Status: PASSED; verified in motion run 2026-10-10; evidence/e2e_motion_03.json)*
- [ ] `E2E-MOTION-04`: Real LLM-Backed Autonomous Task Execution *(Status: Awaiting Live Model Run)*
- [ ] `E2E-MOTION-05`: Human Takeover, Manual Edit & Observation Resync *(Status: Awaiting Desktop Motion Run; Controlled Fixture + SendInput)*
- [ ] `E2E-MOTION-06`: In-Flight STOP Linearization Point & Denial *(Status: Awaiting Desktop Motion Run; Controlled Fixture)*
- [ ] `E2E-MOTION-07`: Uncertain Action Recovery & Non-Idempotent Protection *(Status: Awaiting Desktop Motion Run; Controlled Fixture + Fault Injection)*
- [ ] `E2E-MOTION-08`: Clean Window Teardown & Zero-Orphan Process Exit *(Status: Awaiting Desktop Motion Run)*
