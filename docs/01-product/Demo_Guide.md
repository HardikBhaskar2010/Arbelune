# The 90-Second Reproducible KAGE Showcase (Demo Guide)

| Document Metadata | Details |
|---|---|
| **Document ID** | `KAGE-DEMO-001` |
| **Stage Focus** | Stage 2: Show It — Compelling, Reproducible Demonstration |
| **Target Audience** | Developers, AI Agent Researchers, Security Engineers, Contributors |
| **Prerequisites** | Pre-built binary or cloned repository (`cargo` + `npm`) |
| **Total Duration** | ~90 to 120 seconds |

---

## 1. Why KAGE Looks and Feels Different

Most browser automation tools (Selenium, Puppeteer, Playwright, Browser Use) launch a detached, headless browser window or script an existing browser profile via an un-monitored WebSocket port (`ws://127.0.0.1:...`). 

**KAGE is different.** It is a **governed desktop browser runtime**:
* A **Liquid Glass React presentation shell** (Tauri v2) seamlessly wraps a **native Win32 child Chromium (CEF 152)** surface running at a locked 60–120 FPS.
* The AI never touches raw browser internals. Every click, fill, and navigation passes through a **typed Rust ToolBus** with millisecond capability gating.
* Every mutation is **deterministically verified against the live DOM**, and every action is committed to a **tamper-evident SHA-256 audit ledger**.

Here is how you can witness all 5 superpowers in 90 seconds.

---

## 2. Fast Launch Options

### Option A: Run Pre-Built Prerelease (Zero Compilation)
Download the latest development release from [GitHub Releases](https://github.com/HardikBhaskar2010/Kage/releases):
```powershell
# Extract release bundle and launch
.\Kage.exe
```

### Option B: Run from Local Source
```bash
# 1. Clone & install dependencies
git clone https://github.com/HardikBhaskar2010/Kage.git
cd Kage
npm install

# 2. Launch development runtime
npm run tauri dev
```

---

## 3. The 5-Step 90-Second Walkthrough

### Step 1: Witness the Dual-Surface Architecture (10 seconds)
1. Upon launch, observe the **KAGE Window**:
   * Top & sidebar: Smooth Liquid Glass interface rendered by Tauri v2.
   * Central viewport: A native Win32 child HWND (`Chrome_WidgetWin_1`) hosting multi-process Chromium 134 / CEF 152.
2. In the Omnibox, enter `https://example.com` and press `Enter`.
3. Notice:
   * Instant GPU-composited page rendering with zero frame stutter.
   * Omnibox displays the active URL and TLS security lock indicator.
   * **Under the hood:** Human navigation routes through direct host IPC (`CefRuntime::load_url_by_browser_id`), leaving the agent ToolBus completely clean (`INV-02`).

---

### Step 2: Governed Autonomous Action with Live Verification (25 seconds)
1. In the right-hand **AI Sidebar**, enter:
   ```text
   Navigate to checkout lab, select standard shipping, and verify order readiness.
   ```
2. Press `Start Task`.
3. Observe the HUD in the bottom-right corner:
   * **Step Progress:** `Step 1 of 3: Clicking standard shipping option`.
   * **Capability Check:** ToolBus adjudicates capability permit `page.click` against active policy.
   * **Execution:** CDP executes `Input.dispatchMouseEvent` inside the child HWND.
   * **Verification Badge:** The Verifier Engine queries the live DOM AX tree, confirms the radio button state changed, and emits a glowing green `PASS` badge with a fresh `VerificationId`.

---

### Step 3: Human Supervisory Takeover in Motion (20 seconds)
What happens if the agent encounters an unfamiliar captcha, MFA prompt, or incorrect form value?
1. While an agent task is active, click **[Take Control]** on the HUD.
2. Observe:
   * The HUD immediately switches to **`HUMAN TAKEOVER`**.
   * The host `DispatchAdmissionGate` locks out agent dispatches (`ToolError::HumanTakeoverActive`). Any background tool proposal from the LLM is instantly rejected.
3. Click directly inside the native Chromium webpage and type an address (e.g., `742 Evergreen Terrace`).
4. Click **[Resume Agent]** on the HUD.
5. Notice:
   * KAGE’s Context Engine captures a fresh observation snapshot (`obs_fresh_resync_xxx`).
   * The cryptographic digest of the new DOM is confirmed divergent from the pre-takeover baseline, wiping any stale LLM hallucination before the agent generates its next action.

---

### Step 4: Atomic STOP Under Load (15 seconds)
How does KAGE uphold safety when things go wrong?
1. Start a multi-step sequence.
2. Click **[STOP]** while mutations are running.
3. Observe:
   * The HUD immediately renders **`STOPPED`**.
   * **The Invariant (`INV-09`):** An immediate linearization point is established. Actions queued after the STOP button press are rejected (`DispatchDenial::Cancelled`).
   * The in-flight action settles cleanly without causing memory corruption, handle leaks, or the illusion of retrospective rollback.
   * **Zero subsequent mutations are admitted.**

---

### Step 5: Inspect the Tamper-Evident Audit Ledger (20 seconds)
Every single action taken in KAGE is recorded in an immutable SQLite database linked with SHA-256 hash chains.

Open a terminal in the KAGE data directory and inspect the audit log:
```bash
sqlite3 security_audit.db "SELECT timestamp, tool_id, status, hash_chain FROM audit_records ORDER BY id DESC LIMIT 5;"
```
Output:
```text
2026-10-10T13:43:02Z | page.click   | SUCCESS | sha256:42962c8312911ab5a04451d45f7f085af0050771189c61a955fe2f4e2c0364dd
2026-10-10T13:43:01Z | tab.create   | SUCCESS | sha256:ce6f932708c6ecc0093a529eaeea707f0c4923780f69405632f8d8f18421fd08
2026-10-10T13:42:58Z | genesis_init | SUCCESS | sha256:0000000000000000000000000000000000000000000000000000000000000000
```
Every record is cryptographically tied to the genesis block. If any byte is altered, KAGE’s audit verification detects tampering on boot.

---

## 4. Run the Automated 60-Second Acceptance Demo

Prefer an automated script that exercises the full application in motion?
Run the empirical test runner:
```powershell
powershell -ExecutionPolicy Bypass -File scripts\run_e2e_motion.ps1
```
This script launches the desktop application cold, verifies child HWNDs, measures scoped process tree memory, tests navigation, exercises supervisory takeover, executes atomic STOP, and confirms zero-orphan process teardown.

---

## 5. Next Steps
* Read the [Desktop Acceptance Report](file:///c:/Users/sneha/Videos/Kage/docs/06-testing/Desktop_Acceptance_Report.md) for full physical telemetry data.
* Consult the [Architecture Contracts](file:///c:/Users/sneha/Videos/Kage/docs/02-architecture/Architecture_Contracts.md) to understand the 10 inviolable security guarantees.
* Join the community and build with us in [CONTRIBUTING.md](file:///c:/Users/sneha/Videos/Kage/CONTRIBUTING.md).
