# ARBELUNE Rebrand & Migration Inventory

| Document Metadata | Details |
|---|---|
| **Document ID** | `ARBELUNE-REBRAND-001` |
| **Old Identity** | KAGE (影) |
| **New Identity** | **Arbelune** |
| **Repository Name** | `Arbelune` |
| **Technical Positioning** | *A governed browser runtime for AI agents, with controlled actions, verified outcomes, and human takeover.* |
| **Migration Branch** | `chore/rebrand-kage-to-arbelune` |
| **Baseline Git Commit** | `8c028fa` (`origin/main`) |
| **Baseline Status** | Clean working tree; Contract Verifier: 34/34 passing; Frontend Build: 100% passing (2.68s); Desktop Motion: 7/7 passing. |

---

## 1. Inventory Summary by Classification Category

Following a full-repository AST and text scan (excluding `.git`, `node_modules`, `target`, and toolchain caches):

```text
┌────────────────────────────────────────────────────────────────────────┐
│                   ARBELUNE REBRAND INVENTORY SUMMARY                   │
├──────────────────────────┬──────────────┬──────────────┬───────────────┤
│ CATEGORY                 │ MATCHES      │ FILES        │ RISK PROFILE  │
├──────────────────────────┼──────────────┼──────────────┼───────────────┤
│ 1. Rename                │ 1,581        │ 204          │ Low to Medium │
│    Public branding, docs,│              │              │ (Visual & doc │
│    UI labels, packages   │              │              │ consistency)  │
├──────────────────────────┼──────────────┼──────────────┼───────────────┤
│ 2. Inspect Before Change │ 213          │ 34           │ High          │
│    Executables, IPC, CEF │              │              │ (Runtime      │
│    subprocess, sandbox,  │              │              │ compatibility │
│    Tauri identifier/data │              │              │ & security)   │
├──────────────────────────┼──────────────┼──────────────┼───────────────┤
│ 3. Preserve              │ 112          │ 5            │ Zero          │
│    Original evidence     │              │              │ (Historical   │
│    traces & retrospectives│              │             │ continuity)   │
├──────────────────────────┼──────────────┼──────────────┼───────────────┤
│ TOTAL                    │ 1,906        │ 243          │ Controlled    │
└──────────────────────────┴──────────────┴──────────────┴───────────────┘
```

---

## 2. Category 1: Rename (Public Branding & Documentation)

These matches affect public-facing brand identity, UI titles, navigation labels, package metadata, and current documentation. They can be updated in controlled layers without breaking runtime IPC boundaries.

### 2.1 Public-Facing Branding & Top-Level Repository
* **Root Files:**
  * [`README.md`](README.md): Title, header badges, logo image link (`public/arbelune.png`), quickstart copy, specification links.
  * [`package.json`](package.json): `"name": "arbelune-workspace"`, description, repository URL.
  * [`Cargo.toml`](Cargo.toml): Workspace name, authors, repository URL.
  * [`CONTRIBUTING.md`](CONTRIBUTING.md): Welcome copy, repository clone URLs.
  * [`SECURITY.md`](SECURITY.md): Security policy references, disclosure links.
  * [`LICENSE-MIT`](LICENSE-MIT) & [`LICENSE-APACHE`](LICENSE-APACHE): Copyright notice update to `Arbelune Authors / Arbelune Project`.
  * [`.github/PULL_REQUEST_TEMPLATE.md`](.github/PULL_REQUEST_TEMPLATE.md) & [`.github/ISSUE_TEMPLATE/*`](.github/ISSUE_TEMPLATE/): Template titles and bug reproduction contexts.
  * [`info.md`](info.md) & [`assets/info.md`](assets/info.md): Project overview documents.

### 2.2 Active Product & Architecture Documentation (`docs/`)
* **Product Specifications:**
  * [`docs/01-product/PRD.md`](docs/01-product/PRD.md)
  * [`docs/01-product/Requirements.md`](docs/01-product/Requirements.md)
  * [`docs/01-product/Adoption_Strategy.md`](docs/01-product/Adoption_Strategy.md)
  * [`docs/01-product/Demo_Guide.md`](docs/01-product/Demo_Guide.md)
  * [`docs/Design.md`](docs/Design.md)
* **Architecture Specifications:**
  * [`docs/02-architecture/Architecture.md`](docs/02-architecture/Architecture.md)
  * [`docs/02-architecture/Architecture_Contracts.md`](docs/02-architecture/Architecture_Contracts.md)
  * [`docs/02-architecture/Browser_Shell.md`](docs/02-architecture/Browser_Shell.md)
  * [`docs/02-architecture/CEF_Integration.md`](docs/02-architecture/CEF_Integration.md)
  * [`docs/02-architecture/Tauri_Architecture.md`](docs/02-architecture/Tauri_Architecture.md)
  * [`docs/02-architecture/Rendering_Architecture.md`](docs/02-architecture/Rendering_Architecture.md)
* **Core & Agent Specifications:**
  * [`docs/03-core/Tool_System.md`](docs/03-core/Tool_System.md)
  * [`docs/03-core/Context_Engine.md`](docs/03-core/Context_Engine.md)
  * [`docs/03-core/Micro_Inspect.md`](docs/03-core/Micro_Inspect.md)
  * [`docs/03-core/Workspace_System.md`](docs/03-core/Workspace_System.md)
  * [`docs/04-ai/AI_Architecture.md`](docs/04-ai/AI_Architecture.md)
  * [`docs/05-devtools/DevTools_Architecture.md`](docs/05-devtools/DevTools_Architecture.md)
  * [`docs/08-security/Security_Model.md`](docs/08-security/Security_Model.md)
  * [`docs/09-ui/Design_System.md`](docs/09-ui/Design_System.md)
  * [`docs/10-build-release/Release_Strategy.md`](docs/10-build-release/Release_Strategy.md)

### 2.3 Frontend Presentation Shell (`src-ui/`)
* **Window & Document Titles:**
  * `document.title = "Arbelune - Developer Browser"` in React entry.
  * UI headers in [`src-ui/src/App.tsx`](src-ui/src/App.tsx), [`src-ui/src/components/Omnibox/Omnibox.tsx`](src-ui/src/components/Omnibox/Omnibox.tsx), [`src-ui/src/components/NewTab/NewTab.tsx`](src-ui/src/components/NewTab/NewTab.tsx).
* **Settings & About Dialog:**
  * About card text, product name, version label in [`src-ui/src/components/Settings/SettingsPage.tsx`](src-ui/src/components/Settings/SettingsPage.tsx).
* **CSS Brand Tokens & Themes:**
  * [`src-ui/src/tokens/tokens.css`](src-ui/src/tokens/tokens.css): Accent colors (neon pink `#FF2A85` / `#FF55A3` highlights complementing obsidian obsidian dark).
* **Component Renames:**
  * [`src-ui/src/components/ui/KageIcons.tsx`](src-ui/src/components/ui/KageIcons.tsx) &rarr; `ArbeluneIcons.tsx`.

---

## 3. Category 2: Inspect Before Changing (Runtime & Packaging Identity)

> [!WARNING]
> These identifiers directly control process execution, OS window hierarchy matching, inter-process communication, and filesystem storage paths. They must be updated in tight coordination.

### 3.1 Executable & Library Target Identifiers
| Component | Current Name | Target Migration Name | Impacted Files |
|---|---|---|---|
| **Host Binary** | `kage-host.exe` | `arbelune-host.exe` (or `arbelune.exe`) | `src-tauri/Cargo.toml`, `scripts/run_e2e_motion.ps1`, `scripts/package_release.ps1` |
| **CEF Subprocess** | `kage-cef-subprocess.exe` | `arbelune-cef-subprocess.exe` | `crates/kage-engine/src/subprocess.rs`, `crates/kage-cef-subprocess/Cargo.toml`, `scripts/package_release.ps1` |
| **Release Bootstrap** | `KAGE.exe` | `Arbelune.exe` | `scripts/package_release.ps1`, `src-tauri/tauri.conf.json`, `docs/10-build-release/Release_Strategy.md` |
| **Client DLL** | `kage_client.dll` | `arbelune_client.dll` | `crates/kage-client/Cargo.toml`, `crates/kage-engine/src/runtime.rs`, `scripts/package_release.ps1` |

### 3.2 Win32 Window Title & HWND Matching
* **Win32 Window Title Invariant:**
  * Current: `Kage - Developer Browser`
  * Target: `Arbelune - Developer Browser`
  * Impact: [`scripts/run_e2e_motion.ps1`](scripts/run_e2e_motion.ps1) and [`src-tauri/tauri.conf.json`](src-tauri/tauri.conf.json). Window lookup filters in test harnesses must match the new title.

### 3.3 Security & Sandbox Environment Overrides
* **Variable:** `KAGE_DISABLE_CEF_SANDBOX`
  * Action: Update to `ARBELUNE_DISABLE_CEF_SANDBOX` across:
    * [`scripts/verify_contracts.py`](scripts/verify_contracts.py) (CI security gate).
    * [`crates/kage-engine/src/runtime.rs`](crates/kage-engine/src/runtime.rs) (sandbox initialization logic).
    * Both old and new names should be recognized during transition with deprecation logging, ensuring zero security regressions.

### 3.4 Tauri Application Identifier & Filesystem Storage Continuity
* **Current Identifier:** `"identifier": "com.kage.browser"` in [`src-tauri/tauri.conf.json`](src-tauri/tauri.conf.json).
* **Target Identifier:** `"identifier": "com.arbelune.browser"`.
* **Data Continuity Protocol:**
  * Storage path on Windows: `%LOCALAPPDATA%\com.arbelune.browser` (and roaming AppData).
  * Existing SQLite ledger: `security_audit.db`.
  * Existing profiles: `profiles/` directory.
  * **Continuity Rule:** On startup, if `%LOCALAPPDATA%\com.arbelune.browser` does not exist but `%LOCALAPPDATA%\com.kage.browser` exists, the host runtime will copy or migrate existing profiles and audit records safely with zero data loss, logging the migration to the audit ledger.

---

## 4. Category 3: Preserve (Historical Evidence & Retrospectives)

To preserve immutable engineering integrity, the following historical records will be preserved:
1. **Git Commit History:** Historical commit SHAs (e.g. `8c028fa`, `b8d3024`) and original commit messages will not be rewritten.
2. **Acceptance Evidence Traces (`evidence/`):** Files such as [`evidence/e2e_motion_01.json`](evidence/e2e_motion_01.json) through `08.json` document historical physical executions completed on October 10, 2026. They are marked with historical lineage notes rather than overwritten.
3. **Archived Retrospective Reports:** Historical milestone closeout reports in `docs/reports/` (e.g., `M9_Agent_Runtime_Seal.md`, `M10_Verified_Autonomy_Seal.md`, `M11_Hardened_Shell_Integration.md`) will include a header notation: `*Historical Record: Project formerly known as KAGE (影)*`.

---

## 5. Execution Phasing & Verification Gate

```
┌────────────────────────────────────────────────────────────────────────┐
│                      REBRAND EXECUTION ROADMAP                         │
├─────────────────────┬──────────────────────────────────────────────────┤
│ Phase 1             │ Establish baseline & inventory (Current Step).   │
├─────────────────────┼──────────────────────────────────────────────────┤
│ Phase 2             │ Prepare & integrate ARBELUNE brand assets:       │
│                     │ - Transparent wordmark (BROWSER FOR BUILDERS)    │
│                     │ - Standalone geometric 'A' mark (.png, .ico)     │
│                     │ - Social preview banner & README header          │
├─────────────────────┼──────────────────────────────────────────────────┤
│ Phase 3A            │ Public branding rename (docs, README, UI chrome).│
├─────────────────────┼──────────────────────────────────────────────────┤
│ Phase 3B            │ Runtime identity & packaging rename:             │
│                     │ - Binaries: arbelune-host, arbelune-subprocess   │
│                     │ - Sandbox env: ARBELUNE_DISABLE_CEF_SANDBOX      │
├─────────────────────┼──────────────────────────────────────────────────┤
│ Phase 3C            │ Tauri identifier & storage migration logic.      │
├─────────────────────┼──────────────────────────────────────────────────┤
│ Phase 4             │ Replace icons & app bundle assets.               │
├─────────────────────┼──────────────────────────────────────────────────┤
│ Phase 5             │ Validation: CI gates, tests, build & motion run. │
├─────────────────────┼──────────────────────────────────────────────────┤
│ Phase 6             │ GitHub repository rename & remote URL sync.      │
├─────────────────────┼──────────────────────────────────────────────────┤
│ Phase 7             │ Final cleanup & public relaunch release.         │
└─────────────────────┴──────────────────────────────────────────────────┘
```
