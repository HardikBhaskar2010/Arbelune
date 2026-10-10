# KAGE End-to-End (E2E) Product Acceptance Matrix

| Document Metadata | Details |
|---|---|
| **Document ID** | `KAGE-TEST-003` |
| **Status** | Active Operational Acceptance Matrix |
| **Version** | v0.4.1 (Milestone 2 Target) |
| **Architecture Phase** | Milestone 2: Full Product Tested in Motion |
| **Classification** | Integrated System Verification & Motion Testing |

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
│ [SEALED & COMPLETE]      │ [READY TO EXECUTE]          │ [PENDING RC VALIDATION]       │
│ Governance, ToolBus,     │ Live desktop app execution  │ Model B release bootstrap,    │
│ Agent runtime, STOP gate,│ through end-to-end user     │ CEF 152 Windows sandbox token │
│ Native CEF child HWND,   │ workflows with real LLM     │ integrity (Untrusted/Low),    │
│ Multi-DPI layout math,   │ provider & physical dual-   │ signed installer bundle       │
│ HUD IPC integration.     │ surface automation.         │ qualification.                │
└──────────────────────────┴─────────────────────────────┴───────────────────────────────┘
```

> [!IMPORTANT]
> **Ground-Truth Distinction:** Phase 1–11 automated CI test suites (`scripts/verify_contracts.py`) confirm that the Rust control plane, ToolBus, admission gates, and layout geometry are functionally sound. However, Phase 9/10 CI gates utilized a deterministic test harness rather than live external LLM API calls. **Milestone 2 exercises the full, un-mocked application in motion.**

---

## 2. Dual-Surface Desktop Testing Architecture

Because KAGE composites a native Win32 child HWND inside Tauri v2 WebView2 chrome, standard browser automation (such as Playwright running in standalone browser mode) cannot automate the native CEF surface directly.

KAGE establishes a **Tripartite Testing Stack**:

```text
┌────────────────────────────────────────────────────────────────────────┐
│                         KAGE TESTING STACK                             │
├───────────────────────────┬────────────────────────────────────────────┤
│ 1. Tauri Desktop Chrome   │ WebdriverIO + @wdio/tauri-service          │
│    (React 18 / WebView2)  │ Drives window, omnibox, tab strip, HUD     │
├───────────────────────────┼────────────────────────────────────────────┤
│ 2. Native Browsing Window │ Internal CDP Bridge + Production ToolBus   │
│    (Child HWND / Blink)   │ Drives PageNavigate, PageClick, PageFill   │
├───────────────────────────┼────────────────────────────────────────────┤
│ 3. Whole-System Assertions│ SQLite Ledger + DOM Hash + AX Tree Digests │
│    (Observable Outcome)   │ Audits audit_id, verifier_id, token limits │
└───────────────────────────┴────────────────────────────────────────────┘
```

* **Surface A (Presentation Layer):** Automated via `tauri-driver` / WebdriverIO. Validates UI states, button clicks, HUD outcome badges (`PASS`, `FAIL`, `UNKNOWN`), and trace lineage formatting.
* **Surface B (Chromium Webview):** Automated via internal CDP protocol gateway and Win32 hardware inputs (`WM_LBUTTONDOWN`, `WM_KEYDOWN`).
* **Surface C (Truth Boundary):** Validates that every UI interaction produces a cryptographically sealed record in `security_audit.db` with identical lineage IDs.

---

## 3. Product-Wide End-to-End Acceptance Matrix

This matrix covers every user-visible feature in KAGE, tracking its architecture tier, verification method, and motion test status:

| # | User-Visible Feature | Execution Surface | Architecture Contract | Real LLM Tested? | Desktop Motion Status | Acceptance Gate & Observable Verification |
|---|---|---|---|---|---|---|
| **01** | **Clean Startup & Engine Init** | Win32 Host + Tauri + CEF | `CEF-01`, `CEF-05` | N/A | **Ready (Model A)** | Parent HWND created; child CEF HWND attached (`GetParent == parent`); zero crash on launch. |
| **02** | **Omnibox URL Navigation** | Tauri UI → Host → CEF | `INV-02`, `CEF-06` | N/A | **Ready (Model A)** | Typing URL in omnibox loads page in child HWND; omnibox updates URL and security indicator. |
| **03** | **Tab Strips & Lifecycle** | Tauri UI → TabManager | `INV-10`, `CEF-05` | N/A | **Ready (Model A)** | `Ctrl+T` creates tab; clicking switches active child HWND; `Ctrl+W` releases HWND with zero leak. |
| **04** | **Multi-Profile Isolation** | ProfileManager + SQLite | `INV-10` | N/A | **Ready (Model A)** | Personal, Work, and Ephemeral Sandbox profiles maintain isolated cookie jars and cache stores. |
| **05** | **Window Resize & Multi-DPI** | Win32 Host → Layout Engine| `CEF-06A..C` | N/A | **Ready (Model A)** | Restoring, maximizing (4K), and resizing window maintains exact child HWND bounds with 0px overlap. |
| **06** | **Autonomous Agent Task** | AI Sidebar → Agent Runtime | `INV-01`, `INV-02` | **Pending Live API** | **In Motion Suite** | Natural language prompt triggers bounded planner; real LLM selects tools; ToolBus executes actions. |
| **07** | **DOM & AX Tree Observation**| Context Engine → CDP | `INV-03`, `INV-06` | **Pending Live API** | **In Motion Suite** | Context Engine compresses DOM to <4,000 tokens; credentials/passwords automatically redacted. |
| **08** | **Postcondition Verification**| Verifier Engine → DOM | `INV-08` | **Pending Live API** | **In Motion Suite** | Mutating action evaluates postcondition; emits verified `PASS`, `FAIL`, or `UNKNOWN` badge on HUD. |
| **09** | **Supervisory Takeover** | Agent HUD → AdmissionGate | `INV-09` | **Pending Live API** | **In Motion Suite** | User clicks "Takeover"; agent mutations lock; user enters form manually; agent resyncs fresh state. |
| **10** | **Atomic STOP Under Load** | Agent HUD → AdmissionGate | `INV-09` | **Pending Live API** | **In Motion Suite** | User clicks "STOP" during active task; gate rejects subsequent dispatches; in-flight settles cleanly. |
| **11** | **Uncertain Recovery Guard** | Planner → Recovery Matrix | `INV-08` | **Pending Live API** | **In Motion Suite** | Simulated network timeout triggers `PAUSE_FOR_GUIDANCE` rather than dangerous blind mutation retry. |
| **12** | **Micro Inspect Overlay** | React HUD + CDP Overlay | `INV-07` | N/A | **Ready (Model A)** | `Ctrl+Shift+C` hovers elements; draws bounding box; inspects computed styles and box model. |
| **13** | **Trace Lineage Card** | Agent HUD | `INV-05` | **Pending Live API** | **In Motion Suite** | Card renders `Task › Step › Req: req_xxx › Exec: exec_xxx › Audit: aud_xxx › Vfy: vfy_xxx`. |
| **14** | **Clean App Teardown** | Host → Teardown Protocol | `TEARDOWN` | N/A | **Ready (Model A)** | Closing window cleanly terminates all auxiliary CEF subprocesses with zero orphaned processes. |
| **15** | **Production Sandbox Enforcement** | Single Bootstrap `KAGE.exe` | `CEF-03b-C/D` | N/A | **Pending Model B RC**| Live token inspection confirms `Untrusted` / `Low` integrity on child renderer processes. |

---

## 4. The 8 End-to-End System Motion Tests

These 8 operational workflows test KAGE as a complete, running workstation:

### Test 1: Clean Startup & Surface Readiness (`E2E-MOTION-01`)
* **Goal:** Verify that KAGE launches from a cold state, initializes both rendering engines, and establishes live IPC without race conditions.
* **Execution:**
  1. Boot `kage-host.exe` via `tauri-driver`.
  2. Await `app_ready` IPC event from React presentation shell.
  3. Await `CefEngineState::BrowserCreationAllowed` event from Rust host.
  4. Assert Win32 parent window handle exists and has child window with class `Chrome_RenderWidgetHostHWND`.
  5. Assert idle RAM is $< 150\text{ MB}$.

### Test 2: Real Navigation & Viewport Alignment (`E2E-MOTION-02`)
* **Goal:** Verify the complete path from Omnibox input to native web rendering and URL synchronization.
* **Execution:**
  1. Input `https://test.kage.dev/checkout-lab` into `#omnibox-input` and press `Enter`.
  2. Verify ToolBus issues `page_navigate` with capability permission `ReadDOM`.
  3. Await CDP `Page.loadEventFired` event.
  4. Query native DOM via CDP: assert document title is `"KAGE Checkout Lab"`.
  5. Assert native child HWND coordinates match calculated layout aperture exactly (0px overlap with React chrome).

### Test 3: Tab Strips & Multi-Profile Isolation (`E2E-MOTION-03`)
* **Goal:** Verify that multiple tabs running in distinct profiles maintain strict cookie and storage isolation.
* **Execution:**
  1. Tab 1 (Personal Profile): Navigate to test site, set session cookie `auth=personal_token`.
  2. Click `+ New Tab`, select `Ephemeral Sandbox Profile`.
  3. Tab 2 (Sandbox): Navigate to same test site, inspect `document.cookie`.
  4. **Assertion:** `document.cookie` is empty (strict process/cookie partition).
  5. Close Tab 2: verify in-memory storage is wiped and child HWND destroyed.

### Test 4: Live Autonomous Task with Configured Model Provider (`E2E-MOTION-04`)
* **Goal:** Verify the autonomous agent loop using an actual configured AI provider (OpenRouter / Anthropic / OpenAI), without canned mock responses.
* **Execution:**
  1. Open AI Sidebar in KAGE; enter prompt: `"Navigate to checkout lab, select the standard shipping option, and enter postal code 94105."`
  2. Observe live LLM tool proposal stream.
  3. Verify ToolBus adjudicates each proposal against capabilities (`page_click`, `page_fill`).
  4. Assert each mutating action executes in CEF and passes empirical postcondition verification.
  5. Assert HUD displays outcome `PASS` with fresh `VerificationId`.

### Test 5: Human Supervisory Takeover & Observation Resync (`E2E-MOTION-05`)
* **Goal:** Confirm that human takeover blocks agent mutations and forces a fresh page re-observation upon resumption.
* **Execution:**
  1. Launch an autonomous form-filling task.
  2. While task is in-flight, click **"Take Control"** on the HUD.
  3. Assert `DispatchAdmissionGate` transitions to `Takeover` state; agent tool dispatches are denied.
  4. Manually edit shipping address input field using native keyboard input.
  5. Click **"Resume Agent"**.
  6. Assert Context Engine captures fresh observation `obs_fresh_resync_xxx` with updated DOM digest.
  7. Verify agent adapts to the human edit without overwriting manual input.

### Test 6: STOP Linearization Under Mutation Load (`E2E-MOTION-06`)
* **Goal:** Prove that STOP enforces atomic fail-closed rejection without making false promises of retrospective rollback.
* **Execution:**
  1. Dispatch multi-step mutation chain (e.g. 5 sequential clicks).
  2. User clicks **"STOP"** in the HUD status bar while action 2 is in-flight.
  3. Action 2 settles cleanly or cancels via its token.
  4. Actions 3, 4, and 5 are immediately rejected by `DispatchAdmissionGate` with `ToolError::Cancelled`.
  5. Assert HUD displays `STOPPED` and zero additional mutations reach the Chromium plane.

### Test 7: Non-Idempotent Recovery & Loss Simulation (`E2E-MOTION-07`)
* **Goal:** Ensure uncertain execution outcomes (`UNKNOWN`) do not trigger duplicated mutations for non-idempotent operations.
* **Execution:**
  1. Agent invokes `page_click("#submit-order-button")`.
  2. Inject synthetic network disconnection simulating a lost CDP acknowledgement.
  3. Verifier cannot confirm DOM transition; emits outcome `UNKNOWN`.
  4. **Assertion:** Recovery Engine halts with `PAUSE_FOR_GUIDANCE` rather than auto-retrying `RETRY`.
  5. HUD prompts the developer with explicit options: `[Inspect Page State]` or `[Confirm Mutation]`.

### Test 8: Graceful Teardown & Zero-Orphan Cleanup (`E2E-MOTION-08`)
* **Goal:** Verify that closing the application terminates all native child processes without zombie helper leaks.
* **Execution:**
  1. Enumerate all running `kage-host.exe` and `kage-cef-subprocess.exe` process IDs.
  2. Issue graceful window close event (`WM_CLOSE` / `appWindow.close()`).
  3. Verify two-stage teardown protocol executes (`CefDoMessageLoopWork` drain -> `cef::shutdown()`).
  4. Assert all process IDs terminate within 3,000 ms.
  5. Assert zero orphaned Chromium child processes remain in Windows Task Manager.

---

## 5. Acceptance Execution Checklist

- [ ] `E2E-MOTION-01`: Clean Startup & Engine Initialization
- [ ] `E2E-MOTION-02`: Real Omnibox Navigation & Child HWND Placement
- [ ] `E2E-MOTION-03`: Tab Strip Lifecycle & Multi-Profile Cookie Isolation
- [ ] `E2E-MOTION-04`: Real LLM-Backed Autonomous Task Execution
- [ ] `E2E-MOTION-05`: Human Takeover, Manual Edit & Observation Resync
- [ ] `E2E-MOTION-06`: In-Flight STOP Linearization Point & Denial
- [ ] `E2E-MOTION-07`: Uncertain Action Recovery & Non-Idempotent Protection
- [ ] `E2E-MOTION-08`: Clean Window Teardown & Zero-Orphan Process Exit
