import React from "react";
import "./BrowserWebview.css";
import { useBrowser } from "../../context/BrowserContext";
import { isTauriEnvironment } from "../../adapters/BrowserAdapter";
import { Layers, Cpu } from "lucide-react";

interface BrowserWebviewProps {
  url: string;
}

export const BrowserWebview: React.FC<BrowserWebviewProps> = ({ url }) => {
  const { activeTab } = useBrowser();
  const isTauri = isTauriEnvironment();

  return (
    <div className="browser-webview" role="region" aria-label="Web page viewport">
      {/* ── Top Loading Bar ─────────────────────────────────────── */}
      {activeTab?.isLoading && (
        <div className="webview-loading-bar" role="progressbar" aria-label="Loading page" />
      )}

      {/* ── Viewport Stage ──────────────────────────────────────── */}
      <div className="webview-stage">
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
