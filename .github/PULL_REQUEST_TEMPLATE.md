## 📋 Summary of Changes

Provide a clear and concise description of the changes introduced in this pull request.

- Fixes #(issue)
- Resolves:

---

## 🛡️ Architectural Contracts & Invariants Checklist

Please confirm that this pull request complies with KAGE's 10 Inviolable Architecture Contracts:

- [ ] **INV-01:** AI never directly accesses CEF or Chromium internals.
- [ ] **INV-02:** All mutating and capability actions route through the typed Rust `ToolBus`.
- [ ] **INV-03:** Web content is treated as untrusted data, never prompt authority.
- [ ] **INV-04:** Privileged mutations require capability policy adjudication.
- [ ] **INV-05:** Privileged mutations commit to `security_audit.db` before execution (Fail-closed).
- [ ] **INV-06:** Secrets never enter LLM prompt contexts (Sanitizer verified).
- [ ] **INV-07:** React UI chrome never connects directly to raw CDP WebSockets.
- [ ] **INV-08:** Every autonomous action requires empirical postcondition verification.
- [ ] **INV-09:** "STOP" closes new dispatch admission at an atomic linearization point.
- [ ] **INV-10:** Every browser tab maintains an explicit `(TabId, ProfileId, CefBrowserId)` mapping.

---

## 🧪 Testing & Verification

- [ ] `python scripts/verify_contracts.py` passed with 100% active gates.
- [ ] `cargo test --workspace` passed with zero failures.
- [ ] `npm --prefix src-ui run lint` passed (if UI modified).
- [ ] No raw `.unwrap()` or `.expect()` calls added in fallible paths.

---

## 📸 Visual Demonstrations (If UI Affected)

If modifying UI components, please attach screenshots or screen recordings showing the changes composited in the Liquid Glass theme.
