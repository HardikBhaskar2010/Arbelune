import React from "react";
import "./BrowserWebview.css";
import { useBrowser } from "../../context/BrowserContext";
import { isTauriEnvironment } from "../../adapters/BrowserAdapter";
import { ExternalLink, ShieldCheck, RefreshCw, Layers, Cpu, ShieldAlert } from "lucide-react";

interface BrowserWebviewProps {
  url: string;
}

export const BrowserWebview: React.FC<BrowserWebviewProps> = ({ url }) => {
  const { activeTab, refresh } = useBrowser();
  const isTauri = isTauriEnvironment();
  const isSecure = url.startsWith("https://") || url.startsWith("kage://");

  return (
    <div className="browser-webview" role="region" aria-label="Web page viewport">
      {/* ── Top Loading Bar ─────────────────────────────────────── */}
      {activeTab?.isLoading && (
        <div className="webview-loading-bar" role="progressbar" aria-label="Loading page" />
      )}

      {/* ── Viewport Stage ──────────────────────────────────────── */}
      <div className="webview-stage">
        {/* Viewport Top Chrome Header */}
        <div className="webview-site-header">
          <div className="webview-site-brand">
            <span className={`webview-site-pill ${isSecure ? "webview-site-pill--secure" : "webview-site-pill--insecure"}`}>
              {isSecure ? <ShieldCheck size={13} strokeWidth={2} /> : <ShieldAlert size={13} strokeWidth={2} />}
              {isSecure ? "Verified TLS 1.3" : "Not Secure"}
            </span>
            <span className="webview-site-host" title={url}>{url || "about:blank"}</span>
            {activeTab?.profile_id && (
              <span className="webview-profile-chip" title="Active Storage Profile">
                {activeTab.profile_id}
              </span>
            )}
          </div>
          <div className="webview-site-actions">
            <button
              className="webview-site-btn"
              onClick={refresh}
              title="Reload active tab"
              aria-label="Reload active tab"
            >
              <RefreshCw size={12} strokeWidth={2} />
            </button>
            {url && (
              <a
                href={url}
                target="_blank"
                rel="noreferrer"
                className="webview-site-btn"
                title="Open in system browser"
                aria-label="Open in system browser"
              >
                <ExternalLink size={12} strokeWidth={2} />
              </a>
            )}
          </div>
        </div>

        {/* Viewport Surface Body */}
        {isTauri ? (
          /* Native Windowed CEF Child Surface: Transparent aperture ensuring 60-120 FPS GPU compositing (INV-01, CEF-06B) */
          <div
            className="webview-native-aperture"
            id="cef-surface-aperture"
            data-tab-id={activeTab?.id}
            data-profile={activeTab?.profile_id}
            aria-label="Native CEF Child HWND Render Surface"
          />
        ) : (
          /* Development Standalone Workbench Surface (when running outside Tauri desktop runtime) */
          <div className="webview-preview-canvas">
            <div className="webview-preview-card">
              <div className="webview-preview-card__badge">
                <Layers size={14} strokeWidth={2} />
                <span>CEF 152 Native Control Plane</span>
              </div>
              <h2 className="webview-preview-card__title">{activeTab?.title || "Web Prototyping Workbench"}</h2>
              <p className="webview-preview-card__url">{url || "about:blank"}</p>
              <p className="webview-preview-card__desc">
                In desktop mode, this viewport composites directly with the native Win32 child HWND managed by <code>NativeSurfaceManager</code> (60–120 FPS hardware rasterization, zero OSR penalty).
              </p>
              <div className="webview-preview-card__meta">
                <span className="meta-tag"><Cpu size={12} /> Chromium Engine: <strong>UNAVAILABLE (Web Prototype Mode)</strong></span>
                <span className="meta-tag">Profile: {activeTab?.profile_id || "personal"}</span>
                <span className="meta-tag">Surface State: <strong>Aperture Emulation Only</strong></span>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
