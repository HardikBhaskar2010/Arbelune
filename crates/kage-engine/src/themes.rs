//! Arbelune Webpage Theme Injection Subsystem
//!
//! Provides custom stylesheets and runtime injectors matching Arbelune's
//! refined Apple-calibrated palette:
//! - 70% Deep neutral canvas (#10090E)
//! - Natural typographic flow with ZERO arbitrary borders on icons, cards, and content
//! - Coral-pink brand accent (#F28BA8) reserved strictly for active tabs and focus
//! - High-contrast WCAG AA/AAA typography (#F7EDF1, #C8B8C0, #A99AA2)

pub const GOOGLE_LIQUID_GLASS_CSS: &str = r#"
/* ─── ARBELUNE Apple-Calibrated Theme for Google Search ───── */
:root, html {
  --arb-canvas: #10090E !important;
  --arb-surface: #1B1017 !important;
  --arb-surface-glass: rgba(27, 16, 23, 0.75) !important;
  --arb-accent: #F28BA8 !important;
  --arb-wine-primary: #450920 !important;
  --arb-wine-hover: #5c0f2d !important;

  /* Typography Hierarchy (Apple Optical Scale & WCAG AA/AAA) */
  --arb-text-primary: #F7EDF1 !important;   /* Titles, inputs & headers (14.5:1) */
  --arb-text-secondary: #C8B8C0 !important; /* Body snippets & descriptions (6.8:1) */
  --arb-text-muted: #A99AA2 !important;     /* Metadata & breadcrumbs (4.6:1) */

  color-scheme: dark !important;
  accent-color: var(--arb-accent) !important;

  --color-background: #10090E !important;
  --color-surface: #1B1017 !important;
  --color-primary: #F28BA8 !important;
  --color-on-surface: #F7EDF1 !important;
  --color-on-surface-variant: #C8B8C0 !important;
  --hl-bg: rgba(242, 139, 168, 0.18) !important;
  --hl-color: #F7EDF1 !important;
}

/* ── Base Canvas ────────────────────────────────────────────────── */
html, body, #fbar, .sfbg, .sbtc, .GyAeWb, .minidiv, #cnt, #botstuff, #footcnt, .mw, #searchform, #tsf {
  background: var(--arb-canvas) !important;
  color: var(--arb-text-primary) !important;
}

/* ── Layout & Viewport ──────────────────────────────────────────── */
#rcnt {
  background: var(--arb-canvas) !important;
  padding-top: 16px !important;
  margin-top: 0 !important;
}

#center_col, #main {
  background: transparent !important;
  padding-top: 0 !important;
}

#rso {
  display: flex !important;
  flex-direction: column !important;
  gap: 20px !important;
}

/* ── 1. Hero Search Input Capsule (The ONLY element with a main border) ── */
header, .sfbg, form[role="search"], .A8SBwf {
  background: transparent !important;
  border: none !important;
  box-shadow: none !important;
}

.RNNXgb {
  background: rgba(27, 16, 23, 0.85) !important;
  border: 1px solid rgba(242, 139, 168, 0.18) !important;
  border-radius: 24px !important;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.35), inset 0 1px 0 rgba(255, 255, 255, 0.05) !important;
  backdrop-filter: blur(16px) !important;
  transition: border-color 0.15s ease-out, box-shadow 0.15s ease-out !important;
}

.RNNXgb:hover {
  border-color: rgba(242, 139, 168, 0.35) !important;
}

.RNNXgb:focus-within {
  border-color: var(--arb-accent) !important;
  box-shadow: 0 0 0 3px rgba(242, 139, 168, 0.22), 0 6px 20px rgba(0, 0, 0, 0.45) !important;
}

textarea.gLFyf, input.gLFyf, .gLFyf {
  color: var(--arb-text-primary) !important;
  background: transparent !important;
  font-size: 16px !important;
}

/* Search bar internal elements (Clear, Mic, Lens): 100% borderless & transparent */
.RNNXgb *:not(.gLFyf):not(textarea):not(input) {
  border: none !important;
  box-shadow: none !important;
}

.RNNXgb div[role="button"],
.RNNXgb button,
.RNNXgb .XDyW0e,
.RNNXgb .nDcEnd,
.RNNXgb .ExCKkf,
.RNNXgb .BKRP6,
.RNNXgb [jscontroller],
.RNNXgb [jsname],
.RNNXgb svg,
.A8SBwf svg,
.Tg7LZd svg {
  background: transparent !important;
  border: none !important;
  box-shadow: none !important;
  outline: none !important;
  border-radius: 0 !important;
  fill: var(--arb-text-secondary) !important;
  color: var(--arb-text-secondary) !important;
}

.RNNXgb svg:hover, .A8SBwf svg:hover, .Tg7LZd svg:hover {
  fill: var(--arb-accent) !important;
  color: var(--arb-accent) !important;
}

/* ── 2. Header Action Controls (Settings Gear, Share, Apps, Tools) ─── */
/* STRICT RULE: Zero borders, zero boxes. Clean borderless icon buttons */
div[role="button"][aria-label*="Settings"],
div[role="button"][aria-label*="Share"],
div[role="button"][aria-label*="Quick settings"],
button[aria-label*="Settings"],
button[aria-label*="Share"],
.gb_d, .gb_A, .t2fa2b, #hdtb-tls, .PNyW8c,
div[jsname="bKqLre"], g-raised-button {
  background: transparent !important;
  border: none !important;
  box-shadow: none !important;
  border-radius: 50% !important;
  padding: 6px !important;
  margin: 0 2px !important;
  transition: background 0.15s ease-out !important;
}

div[role="button"][aria-label*="Settings"]:hover,
div[role="button"][aria-label*="Share"]:hover,
div[role="button"][aria-label*="Quick settings"]:hover,
.gb_d:hover, .t2fa2b:hover, #hdtb-tls:hover {
  background: rgba(255, 255, 255, 0.08) !important;
  border: none !important;
}

div[role="button"][aria-label*="Settings"] svg,
div[role="button"][aria-label*="Share"] svg,
div[role="button"][aria-label*="Quick settings"] svg,
.gb_d svg, .t2fa2b svg {
  fill: var(--arb-text-secondary) !important;
  color: var(--arb-text-secondary) !important;
}

/* ── 3. Primary Action: Sign In Button (ONLY the actual button) ──── */
a[href*="accounts.google.com"], a.gb_1.gb_2, a.gb_4 {
  background: linear-gradient(135deg, var(--arb-wine-primary), var(--arb-wine-hover)) !important;
  border: 1px solid rgba(242, 139, 168, 0.25) !important;
  color: var(--arb-text-primary) !important;
  border-radius: 20px !important;
  font-weight: 600 !important;
  padding: 8px 18px !important;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.35), inset 0 1px 0 rgba(255, 255, 255, 0.08) !important;
  transition: transform 0.15s ease-out, border-color 0.15s ease-out, box-shadow 0.15s ease-out !important;
}
a[href*="accounts.google.com"]:hover, a.gb_1.gb_2:hover {
  border-color: var(--arb-accent) !important;
  color: #FFF !important;
  box-shadow: 0 4px 14px rgba(242, 139, 168, 0.3) !important;
  transform: translateY(-1px) !important;
}

/* ── 4. Navigation Tabs Strip (All, Images, Videos, More, Tools) ──── */
#hdtb, #hdtb-msb {
  background: var(--arb-canvas) !important;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06) !important;
}

#hdtb *,
#hdtb-msb *,
g-header-menu,
g-header-menu *,
g-popup,
g-popup *,
.hdtb-dd-b,
.LatpMc,
div[role="navigation"] [role="button"],
div[role="navigation"] a,
.GKSvGd, .T47uwc, .crJ18e {
  box-shadow: none !important;
}

#hdtb div[role="button"],
#hdtb-msb div[role="button"],
g-header-menu div[role="button"],
g-header-menu [aria-haspopup="true"],
.hdtb-dd-b,
.LatpMc:not([aria-selected="true"]),
div[role="navigation"] a {
  background: transparent !important;
  border: none !important;
  border-radius: 0 !important;
  color: var(--arb-text-secondary) !important;
  font-weight: 500 !important;
  transition: color 0.15s ease-out !important;
}
div[role="navigation"] a:hover, .LatpMc:hover, div[role="tab"]:hover, .hdtb-mitem a:hover {
  color: var(--arb-text-primary) !important;
}
.hdtb-mitem.hdtb-msel, div[role="tab"][aria-selected="true"], .LatpMc[aria-selected="true"], .LatpMc.Y6duEe {
  color: var(--arb-text-primary) !important;
  font-weight: 600 !important;
  border-bottom: 2px solid var(--arb-accent) !important;
  border-top: none !important;
  border-left: none !important;
  border-right: none !important;
}

/* ── 5. Universal Card & Widget Border Annihilator ───────────────── */
/* Strip arbitrary card borders, box-shadows, and background boxes everywhere across SERP */
.Ww4FFb,
.g,
div.g,
.MjjYud,
.hlcw0c,
.kvH3mc,
div[data-sokoban-container],
.ULSxyf,
g-card,
g-inner-card,
.kp-blk:not(#rhs *),
.xpdopen,
.osrp-blk,
.cUnQKe,
.ifM9O,
.M8OgIe,
div[data-hveid].Ww4FFb,
video-voyager,
g-scrolling-carousel {
  background: transparent !important;
  border: none !important;
  box-shadow: none !important;
  border-radius: 0 !important;
  padding: 0 !important;
  margin: 0 !important;
}

/* Organic result item vertical rhythm */
.MjjYud:has(h3), div.g:has(h3) {
  margin-bottom: 24px !important;
  padding: 0 !important;
  background: transparent !important;
  border: none !important;
  box-shadow: none !important;
}

/* Typography Hierarchy (Apple Optical Scale) */
h3, .LC20lb, a h3, .DKV0Md {
  color: var(--arb-text-primary) !important;
  font-weight: 600 !important;
  font-size: 20px !important;
  letter-spacing: -0.015em !important;
  line-height: 1.35 !important;
  margin-bottom: 4px !important;
  transition: color 0.12s ease-out !important;
}
a:hover h3, a:hover .LC20lb, a:hover .DKV0Md {
  color: var(--arb-accent) !important;
  text-decoration: underline !important;
}

cite, .TbwUpd, .byrWgf, span.Zu6r8c, .H9W0cf, .tjvcx {
  color: var(--arb-text-muted) !important;
  font-size: 13px !important;
}
.VuuXrf {
  font-weight: 600 !important;
  color: var(--arb-text-primary) !important;
}

.VwiC3b, .yXK7lf, span.aCOpRe, div.s, .hgKElc, .wDYxhc, .MUxGbd {
  color: var(--arb-text-secondary) !important;
  line-height: 1.6 !important;
  font-size: 14px !important;
}
em, b, strong {
  color: var(--arb-text-primary) !important;
  font-weight: 600 !important;
}

/* ── 6. "People Also Ask": Clean Borderless List with Apple Dividers ── */
/* NO nested boxes, NO card outlines, NO capsules around questions */
div[jscontroller="eciyhc"],
div[jscontroller="eciyhc"] *,
.related-question-pair,
.related-question-pair *,
g-accordion-expander,
g-accordion-expander *,
div[data-attrid*="question"],
div[data-attrid*="question"] *,
.wQiwMc,
.wQiwMc *,
div[jsname="N760b"],
div[jsname="N760b"] *,
.match-mod-horizontal-padding,
.cbphWd {
  border: none !important;
  box-shadow: none !important;
  outline: none !important;
  border-radius: 0 !important;
  background: transparent !important;
}

/* Header title "People also ask" */
div[jscontroller="eciyhc"] h2,
div[jscontroller="eciyhc"] [role="heading"],
div[data-attrid*="question"] h2,
div[data-attrid*="question"] [role="heading"] {
  color: var(--arb-text-primary) !important;
  font-size: 18px !important;
  font-weight: 600 !important;
  margin-bottom: 12px !important;
}

/* Question list item: Only a clean bottom hairline divider */
.related-question-pair,
div[jscontroller="eciyhc"] .related-question-pair {
  border-bottom: 1px solid rgba(255, 255, 255, 0.08) !important;
  padding: 14px 0 !important;
  margin: 0 !important;
}

/* Question text typography */
div[jscontroller="eciyhc"] .cbphWd,
.related-question-pair .cbphWd,
div[jscontroller="eciyhc"] span,
.related-question-pair span {
  color: var(--arb-text-primary) !important;
  font-size: 15px !important;
  font-weight: 500 !important;
}

/* Accordion chevron icon: Clean, borderless SVG */
div[jscontroller="eciyhc"] svg,
.related-question-pair svg,
g-accordion-expander svg {
  fill: var(--arb-text-muted) !important;
  color: var(--arb-text-muted) !important;
  background: transparent !important;
  border: none !important;
  box-shadow: none !important;
  transition: fill 0.15s ease-out !important;
}

div[jscontroller="eciyhc"] div[role="button"]:hover svg,
.related-question-pair:hover svg {
  fill: var(--arb-accent) !important;
  color: var(--arb-accent) !important;
}

/* ── 7. AI Overview Panel (The ONLY Result Area with Glass Surface) ── */
[data-attrid="wa:/description"],
div[jscontroller="eG5V6e"],
div[data-sokoban-feature="ai_overview"] {
  background: var(--arb-surface-glass) !important;
  border: 1px solid rgba(255, 255, 255, 0.06) !important;
  border-radius: 20px !important;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.35), inset 0 1px 0 rgba(255, 255, 255, 0.05) !important;
  backdrop-filter: blur(20px) !important;
  padding: 22px 26px !important;
  color: var(--arb-text-primary) !important;
}

.HD8Pae svg, .tYFlId svg, span.hwt5vd svg {
  fill: var(--arb-accent) !important;
  filter: drop-shadow(0 0 6px rgba(242, 139, 168, 0.4)) !important;
}

/* SGE Sentence Highlights */
mark, [data-mark], .cUnQKe mark, .k7k2Jb, .kO001e, .FozS8c, .bNgw9c, .wDYxhc mark,
[style*="background-color: rgb(26, 115, 232)"], [style*="background-color: rgb(194, 231, 255)"],
[style*="background-color: rgb(138, 180, 248)"] {
  background: rgba(242, 139, 168, 0.18) !important;
  color: var(--arb-text-primary) !important;
  border-radius: 4px !important;
  padding: 1px 4px !important;
  border-bottom: 1px solid var(--arb-accent) !important;
  border-top: none !important;
  border-left: none !important;
  border-right: none !important;
}

/* AI Overview Source Citations */
.XPtqee, .H9W0cf, .PUL4lc, .eA0Zae, .i10Hof, [data-attrid="wa:/source_list"] {
  background: rgba(20, 11, 17, 0.6) !important;
  border: none !important;
  border-radius: 16px !important;
}

.XPtqee a, .H9W0cf a, .PUL4lc a, .eA0Zae a {
  border-radius: 10px !important;
  border: none !important;
}
.XPtqee a:hover, .H9W0cf a:hover, .PUL4lc a:hover, .eA0Zae a:hover {
  background: rgba(255, 255, 255, 0.05) !important;
}

/* Citation pills */
.WvGvFc, .Fz20fc, a.fP1Qef, g-bubble, .cb7Rsd, span.Zu6r8c {
  background: rgba(255, 255, 255, 0.06) !important;
  border: none !important;
  border-radius: 12px !important;
  color: var(--arb-text-secondary) !important;
  font-size: 12px !important;
  padding: 2px 8px !important;
}

/* ── 8. Autocomplete Suggestions Dropdown ────────────────────────── */
.UUbT9, .aajZCb, ul[role="listbox"] {
  background: #180D15 !important;
  border: 1px solid rgba(242, 139, 168, 0.2) !important;
  border-radius: 18px !important;
  backdrop-filter: blur(20px) !important;
  box-shadow: 0 12px 36px rgba(0, 0, 0, 0.65), inset 0 1px 0 rgba(255, 255, 255, 0.05) !important;
}

.sbhl, li[role="presentation"]:hover {
  background: rgba(255, 255, 255, 0.06) !important;
  color: var(--arb-text-primary) !important;
}
.sbhl span, li[role="presentation"]:hover span {
  color: var(--arb-text-primary) !important;
}

/* ── 9. Right-Side Knowledge Graph (#rhs) ────────────────────────── */
#rhs > div {
  background: var(--arb-surface-glass) !important;
  border: 1px solid rgba(255, 255, 255, 0.06) !important;
  border-radius: 18px !important;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.35) !important;
  backdrop-filter: blur(18px) !important;
  padding: 16px 20px !important;
}

#rhs > div div, #rhs > div section, #rhs > div table {
  border-color: transparent !important;
  box-shadow: none !important;
}

/* ── 10. Related Searches Chips & Footer ─────────────────────────── */
.k8X0F, .CSkcDe, .s75Fsv, div[data-initq] a, a.JCzEY, .s75Fsv a, a.k8X0F {
  background: rgba(255, 255, 255, 0.05) !important;
  border: none !important;
  box-shadow: none !important;
  border-radius: 18px !important;
  color: var(--arb-text-primary) !important;
  padding: 8px 16px !important;
  transition: background 0.15s ease-out, color 0.15s ease-out !important;
}
.k8X0F:hover, .CSkcDe:hover, .s75Fsv:hover, div[data-initq] a:hover, a.JCzEY:hover, .s75Fsv a:hover {
  background: rgba(255, 255, 255, 0.1) !important;
  color: #FFF !important;
  border: none !important;
}

/* Footer & Pagination */
#botstuff, #footcnt, #fbar, table#nav, .SJ1Kgf {
  background: var(--arb-canvas) !important;
  color: var(--arb-text-muted) !important;
  border: none !important;
}
#pnnext, #pnprev, td.cur, a.fl {
  color: var(--arb-text-secondary) !important;
  border: none !important;
}
#pnnext:hover, #pnprev:hover, a.fl:hover {
  color: var(--arb-accent) !important;
}
td.cur {
  color: var(--arb-text-primary) !important;
}

/* ── 11. Eliminating Stray Spacer Boxes & Horizontal Lines ────────── */
.MjjYud:not(:has(h3)):not(:has([role="heading"])):not(:has(img)):not(:has(video)):not(:has(table)),
.Ww4FFb:not(:has(h3)):not(:has([role="heading"])):not(:has(img)),
.hlcw0c:not(:has(h3)),
hr,
.b2hzbe,
.Ww4FFb hr,
.VjDLad hr,
div[role="separator"] {
  display: none !important;
  height: 0 !important;
  border: none !important;
  padding: 0 !important;
  margin: 0 !important;
  background: transparent !important;
}

/* Media: Natural borders & sizing */
img, video, canvas {
  border-radius: 12px !important;
  border: none !important;
}

g-scrolling-carousel, g-inner-card, div[data-attrid="images universal"] {
  background: transparent !important;
  border: none !important;
}

/* Subtle Apple Scrollbars */
::-webkit-scrollbar {
  width: 7px !important;
  height: 7px !important;
}
::-webkit-scrollbar-track {
  background: transparent !important;
}
::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.18) !important;
  border-radius: 9999px !important;
}
::-webkit-scrollbar-thumb:hover {
  background: rgba(242, 139, 168, 0.45) !important;
}
"#;

pub const GOOGLE_THEME_INJECTOR_SCRIPT: &str = r#"
(function() {
  function applyArbeluneTheme() {
    let style = document.getElementById('arbelune-liquid-glass-theme');
    if (!style) {
      style = document.createElement('style');
      style.id = 'arbelune-liquid-glass-theme';
      (document.head || document.documentElement).appendChild(style);
    }
    style.textContent = `
/* ─── ARBELUNE Apple-Calibrated Theme for Google Search ───── */
:root, html {
  --arb-canvas: #10090E !important;
  --arb-surface: #1B1017 !important;
  --arb-surface-glass: rgba(27, 16, 23, 0.75) !important;
  --arb-accent: #F28BA8 !important;
  --arb-wine-primary: #450920 !important;
  --arb-wine-hover: #5c0f2d !important;

  --arb-text-primary: #F7EDF1 !important;
  --arb-text-secondary: #C8B8C0 !important;
  --arb-text-muted: #A99AA2 !important;

  color-scheme: dark !important;
  accent-color: var(--arb-accent) !important;

  --color-background: #10090E !important;
  --color-surface: #1B1017 !important;
  --color-primary: #F28BA8 !important;
  --color-on-surface: #F7EDF1 !important;
  --color-on-surface-variant: #C8B8C0 !important;
  --hl-bg: rgba(242, 139, 168, 0.18) !important;
  --hl-color: #F7EDF1 !important;
}

html, body, #fbar, .sfbg, .sbtc, .GyAeWb, .minidiv, #cnt, #botstuff, #footcnt, .mw, #searchform, #tsf {
  background: var(--arb-canvas) !important;
  color: var(--arb-text-primary) !important;
}

#rcnt {
  background: var(--arb-canvas) !important;
  padding-top: 16px !important;
  margin-top: 0 !important;
}

#center_col, #main {
  background: transparent !important;
  padding-top: 0 !important;
}

#rso {
  display: flex !important;
  flex-direction: column !important;
  gap: 20px !important;
}

header, .sfbg, form[role="search"], .A8SBwf {
  background: transparent !important;
  border: none !important;
  box-shadow: none !important;
}

.RNNXgb {
  background: rgba(27, 16, 23, 0.85) !important;
  border: 1px solid rgba(242, 139, 168, 0.18) !important;
  border-radius: 24px !important;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.35), inset 0 1px 0 rgba(255, 255, 255, 0.05) !important;
  backdrop-filter: blur(16px) !important;
  transition: border-color 0.15s ease-out, box-shadow 0.15s ease-out !important;
}

.RNNXgb:focus-within {
  border-color: var(--arb-accent) !important;
  box-shadow: 0 0 0 3px rgba(242, 139, 168, 0.22), 0 6px 20px rgba(0, 0, 0, 0.45) !important;
}

textarea.gLFyf, input.gLFyf, .gLFyf {
  color: var(--arb-text-primary) !important;
  background: transparent !important;
  font-size: 16px !important;
}

.RNNXgb *:not(.gLFyf):not(textarea):not(input) {
  border: none !important;
  box-shadow: none !important;
}

.RNNXgb div[role="button"],
.RNNXgb button,
.RNNXgb .XDyW0e,
.RNNXgb .nDcEnd,
.RNNXgb .ExCKkf,
.RNNXgb .BKRP6,
.RNNXgb [jscontroller],
.RNNXgb [jsname],
.RNNXgb svg,
.A8SBwf svg,
.Tg7LZd svg {
  background: transparent !important;
  border: none !important;
  box-shadow: none !important;
  outline: none !important;
  border-radius: 0 !important;
  fill: var(--arb-text-secondary) !important;
  color: var(--arb-text-secondary) !important;
}

.RNNXgb svg:hover, .A8SBwf svg:hover, .Tg7LZd svg:hover {
  fill: var(--arb-accent) !important;
  color: var(--arb-accent) !important;
}

div[role="button"][aria-label*="Settings"],
div[role="button"][aria-label*="Share"],
div[role="button"][aria-label*="Quick settings"],
button[aria-label*="Settings"],
button[aria-label*="Share"],
.gb_d, .gb_A, .t2fa2b, #hdtb-tls, .PNyW8c,
div[jsname="bKqLre"], g-raised-button {
  background: transparent !important;
  border: none !important;
  box-shadow: none !important;
  border-radius: 50% !important;
  padding: 6px !important;
  margin: 0 2px !important;
}

div[role="button"][aria-label*="Settings"]:hover,
div[role="button"][aria-label*="Share"]:hover,
div[role="button"][aria-label*="Quick settings"]:hover,
.gb_d:hover, .t2fa2b:hover, #hdtb-tls:hover {
  background: rgba(255, 255, 255, 0.08) !important;
  border: none !important;
}

div[role="button"][aria-label*="Settings"] svg,
div[role="button"][aria-label*="Share"] svg,
div[role="button"][aria-label*="Quick settings"] svg,
.gb_d svg, .t2fa2b svg {
  fill: var(--arb-text-secondary) !important;
  color: var(--arb-text-secondary) !important;
}

a[href*="accounts.google.com"], a.gb_1.gb_2, a.gb_4 {
  background: linear-gradient(135deg, var(--arb-wine-primary), var(--arb-wine-hover)) !important;
  border: 1px solid rgba(242, 139, 168, 0.25) !important;
  color: var(--arb-text-primary) !important;
  border-radius: 20px !important;
  font-weight: 600 !important;
  padding: 8px 18px !important;
}

#hdtb, #hdtb-msb {
  background: var(--arb-canvas) !important;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06) !important;
}

#hdtb *,
#hdtb-msb *,
g-header-menu,
g-header-menu *,
g-popup,
g-popup *,
.hdtb-dd-b,
.LatpMc,
div[role="navigation"] [role="button"],
div[role="navigation"] a,
.GKSvGd, .T47uwc, .crJ18e {
  box-shadow: none !important;
}

#hdtb div[role="button"],
#hdtb-msb div[role="button"],
g-header-menu div[role="button"],
g-header-menu [aria-haspopup="true"],
.hdtb-dd-b,
.LatpMc:not([aria-selected="true"]),
div[role="navigation"] a {
  background: transparent !important;
  border: none !important;
  border-radius: 0 !important;
  color: var(--arb-text-secondary) !important;
  font-weight: 500 !important;
}
div[role="navigation"] a:hover, .LatpMc:hover, div[role="tab"]:hover, .hdtb-mitem a:hover {
  color: var(--arb-text-primary) !important;
}
.hdtb-mitem.hdtb-msel, div[role="tab"][aria-selected="true"], .LatpMc[aria-selected="true"], .LatpMc.Y6duEe {
  color: var(--arb-text-primary) !important;
  font-weight: 600 !important;
  border-bottom: 2px solid var(--arb-accent) !important;
  border-top: none !important;
  border-left: none !important;
  border-right: none !important;
}

/* Universal Card & Widget Border Annihilator */
.Ww4FFb,
.g,
div.g,
.MjjYud,
.hlcw0c,
.kvH3mc,
div[data-sokoban-container],
.ULSxyf,
g-card,
g-inner-card,
.kp-blk:not(#rhs *),
.xpdopen,
.osrp-blk,
.cUnQKe,
.ifM9O,
.M8OgIe,
div[data-hveid].Ww4FFb,
video-voyager,
g-scrolling-carousel {
  background: transparent !important;
  border: none !important;
  box-shadow: none !important;
  border-radius: 0 !important;
  padding: 0 !important;
  margin: 0 !important;
}

.MjjYud:has(h3), div.g:has(h3) {
  margin-bottom: 24px !important;
  padding: 0 !important;
  background: transparent !important;
  border: none !important;
  box-shadow: none !important;
}

h3, .LC20lb, a h3, .DKV0Md {
  color: var(--arb-text-primary) !important;
  font-weight: 600 !important;
  font-size: 20px !important;
  letter-spacing: -0.015em !important;
  line-height: 1.35 !important;
}
a:hover h3, a:hover .LC20lb, a:hover .DKV0Md {
  color: var(--arb-accent) !important;
  text-decoration: underline !important;
}

cite, .TbwUpd, .byrWgf, span.Zu6r8c, .H9W0cf, .tjvcx {
  color: var(--arb-text-muted) !important;
  font-size: 13px !important;
}
.VuuXrf {
  font-weight: 600 !important;
  color: var(--arb-text-primary) !important;
}

.VwiC3b, .yXK7lf, span.aCOpRe, div.s, .hgKElc, .wDYxhc, .MUxGbd {
  color: var(--arb-text-secondary) !important;
  line-height: 1.6 !important;
  font-size: 14px !important;
}
em, b, strong {
  color: var(--arb-text-primary) !important;
  font-weight: 600 !important;
}

/* People Also Ask: 100% borderless list */
div[jscontroller="eciyhc"],
div[jscontroller="eciyhc"] *,
.related-question-pair,
.related-question-pair *,
g-accordion-expander,
g-accordion-expander *,
div[data-attrid*="question"],
div[data-attrid*="question"] *,
.wQiwMc,
.wQiwMc *,
div[jsname="N760b"],
div[jsname="N760b"] *,
.match-mod-horizontal-padding,
.cbphWd {
  border: none !important;
  box-shadow: none !important;
  outline: none !important;
  border-radius: 0 !important;
  background: transparent !important;
}

div[jscontroller="eciyhc"] h2,
div[jscontroller="eciyhc"] [role="heading"],
div[data-attrid*="question"] h2,
div[data-attrid*="question"] [role="heading"] {
  color: var(--arb-text-primary) !important;
  font-size: 18px !important;
  font-weight: 600 !important;
  margin-bottom: 12px !important;
}

.related-question-pair,
div[jscontroller="eciyhc"] .related-question-pair {
  border-bottom: 1px solid rgba(255, 255, 255, 0.08) !important;
  padding: 14px 0 !important;
  margin: 0 !important;
}

div[jscontroller="eciyhc"] .cbphWd,
.related-question-pair .cbphWd,
div[jscontroller="eciyhc"] span,
.related-question-pair span {
  color: var(--arb-text-primary) !important;
  font-size: 15px !important;
  font-weight: 500 !important;
}

div[jscontroller="eciyhc"] svg,
.related-question-pair svg,
g-accordion-expander svg {
  fill: var(--arb-text-muted) !important;
  color: var(--arb-text-muted) !important;
  background: transparent !important;
  border: none !important;
  box-shadow: none !important;
}

div[jscontroller="eciyhc"] div[role="button"]:hover svg,
.related-question-pair:hover svg {
  fill: var(--arb-accent) !important;
  color: var(--arb-accent) !important;
}

[data-attrid="wa:/description"],
div[jscontroller="eG5V6e"],
div[data-sokoban-feature="ai_overview"] {
  background: var(--arb-surface-glass) !important;
  border: 1px solid rgba(255, 255, 255, 0.06) !important;
  border-radius: 20px !important;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.35) !important;
  backdrop-filter: blur(20px) !important;
  padding: 22px 26px !important;
  color: var(--arb-text-primary) !important;
}

.HD8Pae svg, .tYFlId svg, span.hwt5vd svg {
  fill: var(--arb-accent) !important;
  filter: drop-shadow(0 0 6px rgba(242, 139, 168, 0.4)) !important;
}

mark, [data-mark], .cUnQKe mark, .k7k2Jb, .kO001e, .FozS8c, .bNgw9c, .wDYxhc mark,
[style*="background-color: rgb(26, 115, 232)"], [style*="background-color: rgb(194, 231, 255)"],
[style*="background-color: rgb(138, 180, 248)"] {
  background: rgba(242, 139, 168, 0.18) !important;
  color: var(--arb-text-primary) !important;
  border-radius: 4px !important;
  padding: 1px 4px !important;
  border-bottom: 1px solid var(--arb-accent) !important;
  border-top: none !important;
  border-left: none !important;
  border-right: none !important;
}

.XPtqee, .H9W0cf, .PUL4lc, .eA0Zae, .i10Hof, [data-attrid="wa:/source_list"] {
  background: rgba(20, 11, 17, 0.6) !important;
  border: none !important;
  border-radius: 16px !important;
}

.WvGvFc, .Fz20fc, a.fP1Qef, g-bubble, .cb7Rsd, span.Zu6r8c {
  background: rgba(255, 255, 255, 0.06) !important;
  border: none !important;
  border-radius: 12px !important;
  color: var(--arb-text-secondary) !important;
  font-size: 12px !important;
  padding: 2px 8px !important;
}

.UUbT9, .aajZCb, ul[role="listbox"] {
  background: #180D15 !important;
  border: 1px solid rgba(242, 139, 168, 0.2) !important;
  border-radius: 18px !important;
  backdrop-filter: blur(20px) !important;
  box-shadow: 0 12px 36px rgba(0, 0, 0, 0.65) !important;
}

.sbhl, li[role="presentation"]:hover {
  background: rgba(255, 255, 255, 0.06) !important;
  color: var(--arb-text-primary) !important;
}

#rhs > div {
  background: var(--arb-surface-glass) !important;
  border: 1px solid rgba(255, 255, 255, 0.06) !important;
  border-radius: 18px !important;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.35) !important;
}

.k8X0F, .CSkcDe, .s75Fsv, div[data-initq] a, a.JCzEY, .s75Fsv a, a.k8X0F {
  background: rgba(255, 255, 255, 0.05) !important;
  border: none !important;
  box-shadow: none !important;
  border-radius: 18px !important;
  color: var(--arb-text-primary) !important;
  padding: 8px 16px !important;
}
.k8X0F:hover, .CSkcDe:hover, .s75Fsv:hover, div[data-initq] a:hover, a.JCzEY:hover, .s75Fsv a:hover {
  background: rgba(255, 255, 255, 0.1) !important;
  color: #FFF !important;
}

.MjjYud:not(:has(h3)):not(:has([role="heading"])):not(:has(img)):not(:has(video)):not(:has(table)),
.Ww4FFb:not(:has(h3)):not(:has([role="heading"])):not(:has(img)),
.hlcw0c:not(:has(h3)),
hr,
.b2hzbe,
.Ww4FFb hr,
.VjDLad hr,
div[role="separator"] {
  display: none !important;
  height: 0 !important;
  border: none !important;
  padding: 0 !important;
  margin: 0 !important;
  background: transparent !important;
}

img, video, canvas {
  border-radius: 12px !important;
  border: none !important;
}

g-scrolling-carousel, g-inner-card, div[data-attrid="images universal"] {
  background: transparent !important;
  border: none !important;
}

#botstuff, #footcnt, #fbar, table#nav, .SJ1Kgf {
  background: var(--arb-canvas) !important;
  color: var(--arb-text-muted) !important;
}
#pnnext, #pnprev, td.cur, a.fl {
  color: var(--arb-text-secondary) !important;
}
#pnnext:hover, #pnprev:hover, a.fl:hover {
  color: var(--arb-accent) !important;
}
td.cur {
  color: var(--arb-text-primary) !important;
}

::-webkit-scrollbar {
  width: 7px !important;
  height: 7px !important;
}
::-webkit-scrollbar-track {
  background: transparent !important;
}
::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.18) !important;
  border-radius: 9999px !important;
}
    `;
    document.documentElement.setAttribute('data-arbelune-theme', 'liquid-glass');
  }

  function purgeAllStrayBorders() {
    const containers = document.querySelectorAll(
      '.kp-blk:not(#rhs *), .xpdopen, .Ww4FFb, .wQiwMc, .match-mod-horizontal-padding, ' +
      'div[jsname="N760b"], div[jscontroller="eciyhc"], .related-question-pair, ' +
      'g-accordion-expander, g-card, g-inner-card, .MjjYud, div.g, .cUnQKe, .ifM9O, ' +
      'div[jscontroller="eciyhc"] div[role="button"], .related-question-pair div[role="button"]'
    );
    for (let i = 0; i < containers.length; i++) {
      const el = containers[i];
      if (el.hasAttribute('data-sokoban-feature') || el.closest('[data-sokoban-feature="ai_overview"]') || el.closest('[data-attrid="wa:/description"]')) {
        continue;
      }
      el.style.setProperty('border', 'none', 'important');
      el.style.setProperty('box-shadow', 'none', 'important');
      el.style.setProperty('background', 'transparent', 'important');
      el.style.setProperty('border-radius', '0px', 'important');
    }
  }

  function sanitizeDynamicElements() {
    const blueHighlights = document.querySelectorAll('[style*="rgb(26, 115, 232)"], [style*="rgb(194, 231, 255)"], [style*="rgb(138, 180, 248)"]');
    for (let i = 0; i < blueHighlights.length; i++) {
      blueHighlights[i].style.setProperty('background-color', 'rgba(242, 139, 168, 0.18)', 'important');
      blueHighlights[i].style.setProperty('color', '#F7EDF1', 'important');
      blueHighlights[i].style.setProperty('border-radius', '4px', 'important');
      blueHighlights[i].style.setProperty('border-bottom', '1px solid #F28BA8', 'important');
    }
    purgeAllStrayBorders();
  }

  applyArbeluneTheme();
  sanitizeDynamicElements();

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
      applyArbeluneTheme();
      sanitizeDynamicElements();
    });
  }

  try {
    const observer = new MutationObserver(() => {
      if (!document.getElementById('arbelune-liquid-glass-theme')) {
        applyArbeluneTheme();
      }
      sanitizeDynamicElements();
    });
    observer.observe(document.documentElement, { childList: true, subtree: true, attributes: true, attributeFilter: ['style'] });
  } catch (e) {
    // Observer fallback
  }
})();
"#;

pub const GOOGLE_THEME_REMOVE_SCRIPT: &str = r#"
(function() {
  const style = document.getElementById('arbelune-liquid-glass-theme');
  if (style) {
    style.remove();
  }
  document.documentElement.removeAttribute('data-arbelune-theme');
})();
"#;
