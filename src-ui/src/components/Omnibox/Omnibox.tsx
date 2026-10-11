import React, { useId, useState } from "react";
import "./Omnibox.css";
import {
  ArrowLeft,
  ArrowRight,
  RotateCw,
  Home,
  Lock,
  Sparkles,
  Star,
  Download,
  Blocks,
  Palette,
} from "lucide-react";
import { SmartTooltip } from "../ui";

interface OmniboxProps {
  url: string;
  onUrlChange: (url: string) => void;
  onNavigate: (url: string) => void;
  isSecure?: boolean;
  isStarred?: boolean;
  onFavourite?: () => void;
  onRefresh?: () => void;
  onBack?: () => void;
  onForward?: () => void;
  onToggleAi?: () => void;
  onToggleDownloads?: () => void;
  onToggleExtensions?: () => void;
  isThemeActive?: boolean;
  onToggleTheme?: () => void;
  canGoBack?: boolean;
  canGoForward?: boolean;
}

export const Omnibox: React.FC<OmniboxProps> = ({
  url,
  onUrlChange,
  onNavigate,
  isSecure = true,
  isStarred: propStarred,
  onFavourite,
  onRefresh,
  onBack,
  onForward,
  onToggleAi,
  onToggleDownloads,
  onToggleExtensions,
  isThemeActive = true,
  onToggleTheme,
  canGoBack = false,
  canGoForward = false,
}) => {
  const inputId = useId();
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [localStarred, setLocalStarred] = useState(false);

  const isStarred = propStarred ?? localStarred;

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      onNavigate(url);
    }
  };

  const handleRefreshClick = () => {
    setIsRefreshing(true);
    onRefresh?.();
    setTimeout(() => setIsRefreshing(false), 600);
  };

  const handleStarClick = () => {
    setLocalStarred((prev) => !prev);
    onFavourite?.();
  };

  return (
    <div className="omnibox" role="navigation" aria-label="Browser navigation bar">
      {/* ── Left Navigation Buttons ──────────────────────────────── */}
      <div className="omnibox__nav-btns">
        <SmartTooltip content="Back" shortcut="Alt+←" position="bottom">
          <button
            className="omnibox__nav-btn"
            onClick={onBack}
            disabled={!canGoBack}
            aria-label="Go back"
          >
            <ArrowLeft size={16} strokeWidth={2} className="omnibox__icon-hover-left" />
          </button>
        </SmartTooltip>
        <SmartTooltip content="Forward" shortcut="Alt+→" position="bottom">
          <button
            className="omnibox__nav-btn"
            onClick={onForward}
            disabled={!canGoForward}
            aria-label="Go forward"
          >
            <ArrowRight size={16} strokeWidth={2} className="omnibox__icon-hover-right" />
          </button>
        </SmartTooltip>
        <SmartTooltip content="Reload page" shortcut="Ctrl+R" position="bottom">
          <button
            className={`omnibox__nav-btn ${isRefreshing ? "omnibox__nav-btn--refreshing" : ""}`}
            onClick={handleRefreshClick}
            aria-label="Refresh page"
          >
            <RotateCw size={15} strokeWidth={2} className="omnibox__icon-hover-rotate" />
          </button>
        </SmartTooltip>
        <SmartTooltip content="Home / New Tab" shortcut="Ctrl+H" position="bottom">
          <button
            className="omnibox__nav-btn"
            onClick={() => onNavigate("")}
            aria-label="New Tab / Home"
          >
            <Home size={16} strokeWidth={2} />
          </button>
        </SmartTooltip>
      </div>

      {/* ── Center Omnibox Capsule ───────────────────────────────── */}
      <div className="omnibox__input-wrap">
        <span
          className={`omnibox__lock ${isSecure ? "omnibox__lock--secure" : "omnibox__lock--warn"}`}
          aria-label={isSecure ? "Secure connection" : "Not secure"}
        >
          <Lock size={13} strokeWidth={2.2} />
        </span>
        <input
          id={inputId}
          type="text"
          className="omnibox__input"
          placeholder="Search or enter a URL..."
          value={url}
          onChange={(e) => onUrlChange(e.target.value)}
          onKeyDown={handleKeyDown}
          aria-label="Address bar — enter a URL or search query"
          spellCheck={false}
          autoComplete="off"
        />
        <SmartTooltip content="Ask Arbelune AI" shortcut="Ctrl+Shift+A" position="bottom">
          <button
            className="omnibox__ai-btn"
            onClick={onToggleAi}
            aria-label="Ask Arbelune AI"
            type="button"
          >
            <Sparkles size={14} strokeWidth={2} className="omnibox__sparkle-icon" />
          </button>
        </SmartTooltip>
      </div>

      {/* ── Right Utility Actions + Profile Avatar ────────────────── */}
      <div className="omnibox__actions-right">
        {onToggleTheme && (
          <SmartTooltip
            content={isThemeActive ? "Arbelune Theme: Active" : "Arbelune Theme: Off"}
            position="bottom"
          >
            <button
              className={`omnibox__action-btn ${isThemeActive ? "omnibox__action-btn--active" : ""}`}
              onClick={onToggleTheme}
              aria-label={isThemeActive ? "Arbelune Theme: Active (Click to disable on this site)" : "Arbelune Theme: Disabled (Click to enable)"}
              type="button"
            >
              <Palette size={16} strokeWidth={2} />
            </button>
          </SmartTooltip>
        )}
        <SmartTooltip
          content={isStarred ? "Remove bookmark" : "Bookmark page"}
          shortcut="Ctrl+D"
          position="bottom"
        >
          <button
            className={`omnibox__action-btn ${isStarred ? "omnibox__action-btn--active" : ""}`}
            onClick={handleStarClick}
            aria-label="Bookmark page"
            type="button"
          >
            <Star size={16} strokeWidth={2} fill={isStarred ? "currentColor" : "none"} />
          </button>
        </SmartTooltip>
        <SmartTooltip content="Downloads" shortcut="Ctrl+J" position="bottom">
          <button
            className="omnibox__action-btn"
            onClick={onToggleDownloads}
            aria-label="Downloads"
            type="button"
          >
            <Download size={16} strokeWidth={2} />
          </button>
        </SmartTooltip>
        <SmartTooltip content="Extensions" position="bottom">
          <button
            className="omnibox__action-btn"
            onClick={onToggleExtensions}
            aria-label="Extensions"
            type="button"
          >
            <Blocks size={16} strokeWidth={2} />
          </button>
        </SmartTooltip>
        <SmartTooltip content="Active Profile" position="bottom">
          <div className="omnibox__profile-wrap">
            <img src="/arbelune-avatar.png" alt="Profile" className="omnibox__profile-avatar" />
            <span className="omnibox__profile-dot" />
          </div>
        </SmartTooltip>
      </div>
    </div>
  );
};
