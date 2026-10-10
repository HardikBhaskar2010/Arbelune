# Contributing to KAGE (影)

Thank you for your interest in contributing to **KAGE (影)** — the developer-first autonomous browser and instrumentation workstation!

KAGE is an ambitious open-source project uniting native Chromium rendering, high-velocity DevTools, and permission-governed autonomous agent intelligence. We welcome contributions ranging from bug fixes and documentation to new developer tools and engine improvements.

---

## 1. Architectural Philosophy & The 10 Inviolable Contracts

Before submitting any code, please review our foundational architecture in [`docs/README.md`](docs/README.md) and [`AGENTS.md`](AGENTS.md). All contributions must strictly uphold the **10 Inviolable Architectural Contracts**:

1. **INV-01:** AI never directly accesses CEF or Chromium internals.
2. **INV-02:** All mutating and capability actions must route through the typed Rust `ToolBus`.
3. **INV-03:** Web content is untrusted data, never prompt authority.
4. **INV-04:** Privileged mutations require capability policy adjudication.
5. **INV-05:** Privileged mutations require successful audit commitment (`security_audit.db`).
6. **INV-06:** Secrets never enter LLM prompt contexts (Credential Broker & Sanitizer).
7. **INV-07:** React UI chrome never directly controls CDP WebSockets (`ws://127.0.0.1:...`).
8. **INV-08:** Every autonomous action requires empirical postcondition verification.
9. **INV-09:** "STOP" immediately closes new dispatch admission at an atomic linearization point.
10. **INV-10:** Every browser tab maintains an explicit `(TabId, ProfileId, CefBrowserId)` mapping.

---

## 2. Development Prerequisites

* **Rust:** 1.78+ (`rustup default stable`)
* **Node.js:** 18+ (with `npm` or `pnpm`)
* **Python:** 3.10+ (for CI contract verification gates)
* **Operating System:** Windows 10/11 (x86_64), macOS 13+, or Linux (Ubuntu 22.04+)
* **C++ Build Tools:** MSVC v143 (Windows), Xcode CLI (macOS), or `build-essential` (Linux)

---

## 3. Getting Started

### 1. Clone the Repository
```bash
git clone https://github.com/HardikBhaskar2010/Kage.git
cd Kage
```

### 2. Install UI Dependencies
```bash
npm install
```

### 3. Verify Contracts & Architecture Gates
Before making any changes, run the automated architecture contract gate to ensure your environment passes all checks:
```bash
python scripts/verify_contracts.py
```

### 4. Run the Development Browser
```bash
npm run tauri dev
```

---

## 4. Execution Topologies (Dev vs. Release)

* **Model A (Development & Debug):** Uses a separate subprocess executable (`kage-cef-subprocess.exe`) with `no_sandbox = true` for high-velocity local debugging, WinDbg symbols, and hot-reloading.
* **Model B (Production Release):** Uses a single unified `KAGE.exe` bootstrap executable linking `cef_sandbox.lib` forwarding to `kage_client.dll`. The browser and child processes share the same executable with Chromium sandbox token restrictions enforced.

---

## 5. Coding Standards

### Rust Core (`crates/` and `src-tauri/`)
* Use `thiserror` for domain-specific errors and `anyhow` for top-level entry points.
* **Never use raw `.unwrap()` or `.expect()`** on fallible external inputs, IPC payloads, or CEF callbacks.
* All SQLite queries must use parameterized statements (`params![]`) via `rusqlite`.
* Format with `cargo fmt` and check with `cargo clippy --workspace -- -D warnings`.

### Frontend Chrome (`src-ui/`)
* TypeScript strict mode enabled (`strict: true`, `noImplicitAny: true`).
* Use tokens from `src-ui/src/tokens/tokens.css` (Liquid Glass theme: peach-to-burgundy gradient `#F9DBBD` → `#450920`).
* Never ship `transition: all`. Animate only `transform` and `opacity` on GPU compositing layers.
* Check with `npm --prefix src-ui run lint`.

---

## 6. Pull Request Workflow

1. **Fork & Branch:** Create a feature branch from `main` (`git checkout -b feat/my-feature`).
2. **Commit Convention:** Follow Conventional Commits:
   * `feat: add network throttling preset tool`
   * `fix: prevent race condition in child HWND resize`
   * `docs: clarify CEF-02 development topology`
3. **Run CI Gates:** Ensure `python scripts/verify_contracts.py` and `cargo test --workspace` pass with zero failures.
4. **Submit PR:** Fill out the [Pull Request Template](.github/PULL_REQUEST_TEMPLATE.md) and link relevant issues.

Thank you for helping build the future of autonomous web browsing!
