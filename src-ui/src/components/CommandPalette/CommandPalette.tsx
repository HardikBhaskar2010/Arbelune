import React, { useState, useEffect, useRef, useId, useMemo } from "react";
import { createPortal } from "react-dom";
import "./CommandPalette.css";
import {
  Search,
  Plus,
  X,
  RotateCw,
  Home,
  ArrowLeft,
  ArrowRight,
  Star,
  Download,
  Sparkles,
  Bug,
  UserCheck,
  Palette,
  CornerDownLeft,
} from "lucide-react";
import {
  KageIconInspect,
  KageIconDOM,
  KageIconNetwork,
  KageIconPerformance,
  KageIconConsole,
  KageIconStorage,
  KageIconSecurity,
  KageIconExtensions,
  KageIconSettings,
  KageIconAI,
} from "../ui";
import { useBrowser } from "../../context/BrowserContext";
import type { ActiveTool } from "../../context/BrowserContext";

export interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
}

interface CommandItem {
  id: string;
  title: string;
  subtitle: string;
  category: "Navigation" | "Developer Tools" | "AI Agent" | "Workspace";
  shortcut?: string;
  icon: React.ReactNode;
  keywords?: string[];
  disabled?: boolean;
  action: () => void;
}

export const CommandPalette: React.FC<CommandPaletteProps> = ({ isOpen, onClose }) => {
  const {
    createTab,
    closeTab,
    activeTabId,
    activeTab,
    navigate,
    goBack,
    goForward,
    refresh,
    toggleBookmark,
    isBookmarked,
    setActiveTool,
    setDevToolsOpen,
    setInspectMode,
    aiOpen,
    setAiOpen,
    setDownloadsOpen,
    setProfileSwitcherOpen,
    togglePageTheme,
    themeEnabled,
  } = useBrowser();

  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);
  const previousActiveElementRef = useRef<HTMLElement | null>(null);
  const searchInputId = useId();

  // Registry of genuine Arbelune browser actions (Phase B - Priority 2)
  const commands: CommandItem[] = useMemo(() => {
    return [
      // ─── Navigation & Browser ───────────────────────────────────
      {
        id: "nav-new-tab",
        title: "New Tab",
        subtitle: "Open a fresh browsing tab",
        category: "Navigation",
        shortcut: "Ctrl+T",
        icon: <Plus size={16} strokeWidth={2} />,
        keywords: ["tab", "create", "blank", "open", "url"],
        action: () => createTab(),
      },
      {
        id: "nav-close-tab",
        title: "Close Active Tab",
        subtitle: "Close the currently focused tab",
        category: "Navigation",
        shortcut: "Ctrl+W",
        icon: <X size={16} strokeWidth={2} />,
        keywords: ["tab", "close", "destroy", "exit"],
        disabled: !activeTabId,
        action: () => {
          if (activeTabId) closeTab(activeTabId);
        },
      },
      {
        id: "nav-home",
        title: "Home / New Tab Page",
        subtitle: "Open Arbelune workstation center",
        category: "Navigation",
        shortcut: "Ctrl+H",
        icon: <Home size={16} strokeWidth={2} />,
        keywords: ["home", "workstation", "start", "landing"],
        action: () => navigate(""),
      },
      {
        id: "nav-reload",
        title: "Reload Active Page",
        subtitle: "Refresh current page content",
        category: "Navigation",
        shortcut: "Ctrl+R",
        icon: <RotateCw size={16} strokeWidth={2} />,
        keywords: ["refresh", "reload", "update"],
        action: () => refresh(),
      },
      {
        id: "nav-back",
        title: "Navigate Back",
        subtitle: "Return to previous page in history",
        category: "Navigation",
        shortcut: "Alt+←",
        icon: <ArrowLeft size={16} strokeWidth={2} />,
        keywords: ["back", "history", "previous"],
        disabled: !activeTab?.canGoBack,
        action: () => goBack(),
      },
      {
        id: "nav-forward",
        title: "Navigate Forward",
        subtitle: "Proceed to next page in history",
        category: "Navigation",
        shortcut: "Alt+→",
        icon: <ArrowRight size={16} strokeWidth={2} />,
        keywords: ["forward", "next", "history"],
        disabled: !activeTab?.canGoForward,
        action: () => goForward(),
      },
      {
        id: "nav-bookmark",
        title: isBookmarked(activeTab?.url || "") ? "Remove Bookmark" : "Bookmark Active Page",
        subtitle: "Save or remove current page from favorites",
        category: "Navigation",
        shortcut: "Ctrl+D",
        icon: <Star size={16} strokeWidth={2} />,
        keywords: ["star", "bookmark", "favorite", "save"],
        action: () => toggleBookmark(activeTab?.url || "", activeTab?.title),
      },
      {
        id: "nav-downloads",
        title: "Downloads Panel",
        subtitle: "View file downloads and transfer progress",
        category: "Navigation",
        shortcut: "Ctrl+J",
        icon: <Download size={16} strokeWidth={2} />,
        keywords: ["download", "files", "transfers"],
        action: () => setDownloadsOpen(true),
      },

      // ─── Developer Tools ────────────────────────────────────────
      {
        id: "dev-inspect",
        title: "Inspect Element",
        subtitle: "Toggle blueprint micro-inspect overlay",
        category: "Developer Tools",
        shortcut: "Ctrl+Shift+C",
        icon: <KageIconInspect size={16} strokeWidth={1.8} />,
        keywords: ["inspect", "element", "blueprint", "devtools", "hover", "pointer"],
        action: () => {
          setInspectMode(true);
          setActiveTool("inspect");
          setDevToolsOpen(false);
        },
      },
      {
        id: "dev-dom",
        title: "DOM Tree Explorer",
        subtitle: "Inspect CDP accessibility & DOM node tree",
        category: "Developer Tools",
        shortcut: "Ctrl+Alt+I",
        icon: <KageIconDOM size={16} strokeWidth={1.8} />,
        keywords: ["dom", "elements", "tree", "html", "nodes", "cdp"],
        action: () => {
          setActiveTool("dom");
          setDevToolsOpen(true);
        },
      },
      {
        id: "dev-network",
        title: "Network Activity Monitor",
        subtitle: "Inspect HTTP requests, headers and waterfall",
        category: "Developer Tools",
        icon: <KageIconNetwork size={16} strokeWidth={1.8} />,
        keywords: ["network", "http", "fetch", "xhr", "requests", "waterfall"],
        action: () => {
          setActiveTool("network");
          setDevToolsOpen(true);
        },
      },
      {
        id: "dev-perf",
        title: "Performance & FPS Profiler",
        subtitle: "Monitor compositor frames, memory and timing",
        category: "Developer Tools",
        icon: <KageIconPerformance size={16} strokeWidth={1.8} />,
        keywords: ["performance", "fps", "memory", "profiler", "speed", "metrics"],
        action: () => {
          setActiveTool("performance");
          setDevToolsOpen(true);
        },
      },
      {
        id: "dev-console",
        title: "Chromium Console Stream",
        subtitle: "View runtime logs, errors and evaluate JS",
        category: "Developer Tools",
        icon: <KageIconConsole size={16} strokeWidth={1.8} />,
        keywords: ["console", "log", "errors", "javascript", "eval"],
        action: () => {
          setActiveTool("console");
          setDevToolsOpen(true);
        },
      },
      {
        id: "dev-storage",
        title: "Storage & Cookie Vault",
        subtitle: "Inspect cookies, localStorage and session data",
        category: "Developer Tools",
        icon: <KageIconStorage size={16} strokeWidth={1.8} />,
        keywords: ["storage", "cookies", "localstorage", "session", "tokens"],
        action: () => {
          setActiveTool("storage");
          setDevToolsOpen(true);
        },
      },
      {
        id: "dev-security",
        title: "Security & Policy Ledger",
        subtitle: "Review capability governance and tamper audit",
        category: "Developer Tools",
        icon: <KageIconSecurity size={16} strokeWidth={1.8} />,
        keywords: ["security", "policy", "audit", "governance", "ledger"],
        action: () => {
          setActiveTool("security");
          setDevToolsOpen(true);
        },
      },

      // ─── AI Agent & Inspection ──────────────────────────────────
      {
        id: "ai-workspace",
        title: aiOpen ? "Close AI Workspace" : "Open Arbelune AI Workspace",
        subtitle: "Toggle autonomous agent sidebar & planner",
        category: "AI Agent",
        shortcut: "Ctrl+Shift+A",
        icon: <KageIconAI size={16} strokeWidth={1.8} />,
        keywords: ["ai", "agent", "assistant", "chat", "workspace", "sidebar"],
        action: () => setAiOpen(!aiOpen),
      },
      {
        id: "ai-analyze",
        title: "Analyze Active Page",
        subtitle: "Run comprehensive AI performance & SEO audit",
        category: "AI Agent",
        icon: <Sparkles size={16} strokeWidth={1.8} />,
        keywords: ["ai", "analyze", "audit", "seo", "accessibility", "report"],
        action: () => setAiOpen(true),
      },
      {
        id: "ai-find-issues",
        title: "Scan Page for Issues",
        subtitle: "Autonomous scan for layout and console defects",
        category: "AI Agent",
        icon: <Bug size={16} strokeWidth={1.8} />,
        keywords: ["ai", "scan", "issues", "bugs", "errors"],
        action: () => setAiOpen(true),
      },

      // ─── Workspace & Profiles ───────────────────────────────────
      {
        id: "workspace-profile",
        title: "Switch Identity Profile",
        subtitle: "Switch between Personal, Work, and Agent Sandbox",
        category: "Workspace",
        shortcut: "Ctrl+P",
        icon: <UserCheck size={16} strokeWidth={1.8} />,
        keywords: ["profile", "identity", "sandbox", "switch", "user"],
        action: () => setProfileSwitcherOpen(true),
      },
      {
        id: "workspace-theme",
        title: themeEnabled ? "Disable Liquid Glass Theme" : "Enable Liquid Glass Theme",
        subtitle: "Invert active page with dark obsidian glass theme",
        category: "Workspace",
        icon: <Palette size={16} strokeWidth={1.8} />,
        keywords: ["theme", "dark", "invert", "liquid", "glass"],
        action: () => togglePageTheme(),
      },
      {
        id: "workspace-extensions",
        title: "Extensions Manager",
        subtitle: "Manage installed WebExtensions & add-ons",
        category: "Workspace",
        icon: <KageIconExtensions size={16} strokeWidth={1.8} />,
        keywords: ["extensions", "addons", "plugins"],
        action: () => {
          setActiveTool("extensions" as ActiveTool);
          setDevToolsOpen(false);
        },
      },
      {
        id: "workspace-settings",
        title: "Browser Settings & Policy",
        subtitle: "Configure Arbelune preferences & agent safety",
        category: "Workspace",
        shortcut: "Ctrl+,",
        icon: <KageIconSettings size={16} strokeWidth={1.8} />,
        keywords: ["settings", "preferences", "config", "options"],
        action: () => {
          setActiveTool("settings" as ActiveTool);
          setDevToolsOpen(false);
        },
      },
    ];
  }, [
    createTab,
    closeTab,
    activeTabId,
    activeTab,
    navigate,
    refresh,
    goBack,
    goForward,
    toggleBookmark,
    isBookmarked,
    setDownloadsOpen,
    setInspectMode,
    setActiveTool,
    setDevToolsOpen,
    aiOpen,
    setAiOpen,
    setProfileSwitcherOpen,
    themeEnabled,
    togglePageTheme,
  ]);

  // Real-time search filter across title, subtitle, category, keywords
  const filteredCommands = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return commands;
    return commands.filter((cmd) => {
      if (cmd.title.toLowerCase().includes(q)) return true;
      if (cmd.subtitle.toLowerCase().includes(q)) return true;
      if (cmd.category.toLowerCase().includes(q)) return true;
      if (cmd.keywords?.some((k) => k.toLowerCase().includes(q))) return true;
      return false;
    });
  }, [commands, query]);

  // Focus management: store previous focus on open, restore on close
  useEffect(() => {
    if (isOpen) {
      previousActiveElementRef.current = document.activeElement as HTMLElement | null;
      setQuery("");
      setSelectedIndex(0);
      requestAnimationFrame(() => {
        inputRef.current?.focus();
      });
    } else {
      previousActiveElementRef.current?.focus();
    }
  }, [isOpen]);

  // Keep selected index within valid range
  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  // Scroll active item into view
  useEffect(() => {
    if (!listRef.current) return;
    const activeEl = listRef.current.querySelector<HTMLElement>(".command-palette__item--selected");
    if (activeEl) {
      activeEl.scrollIntoView({ block: "nearest" });
    }
  }, [selectedIndex]);

  // Keyboard navigation
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      onClose();
      return;
    }

    if (filteredCommands.length === 0) return;

    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((prev) => (prev + 1) % filteredCommands.length);
      return;
    }

    if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((prev) => (prev - 1 + filteredCommands.length) % filteredCommands.length);
      return;
    }

    if (e.key === "Enter") {
      e.preventDefault();
      const selected = filteredCommands[selectedIndex];
      if (selected && !selected.disabled) {
        selected.action();
        onClose();
      }
    }
  };

  if (!isOpen) return null;

  // Group filtered commands by category
  const categories: Array<CommandItem["category"]> = [
    "Navigation",
    "Developer Tools",
    "AI Agent",
    "Workspace",
  ];

  let itemCounter = 0;

  return createPortal(
    <div
      className="command-palette-backdrop"
      onClick={onClose}
      role="presentation"
      aria-hidden="true"
    >
      <div
        className="command-palette"
        role="dialog"
        aria-modal="true"
        aria-label="Command Search"
        onClick={(e) => e.stopPropagation()}
        onKeyDown={handleKeyDown}
      >
        {/* ── Search Input Header (Skiper Vercel #92 layout) ─────── */}
        <div className="command-palette__header">
          <Search size={18} strokeWidth={2} className="command-palette__search-icon" aria-hidden="true" />
          <input
            ref={inputRef}
            id={searchInputId}
            type="text"
            className="command-palette__input"
            placeholder="Type a command or search actions..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            autoComplete="off"
            spellCheck={false}
          />
          {query ? (
            <button
              type="button"
              className="command-palette__clear-btn"
              onClick={() => {
                setQuery("");
                inputRef.current?.focus();
              }}
              aria-label="Clear search"
            >
              <X size={14} strokeWidth={2} />
            </button>
          ) : (
            <kbd className="command-palette__esc-badge">ESC</kbd>
          )}
        </div>

        {/* ── Results List ────────────────────────────────────────── */}
        <div className="command-palette__body" ref={listRef} role="listbox">
          {filteredCommands.length === 0 ? (
            <div className="command-palette__empty" role="status">
              <span className="command-palette__empty-title">No matching commands</span>
              <span className="command-palette__empty-desc">
                No browser actions found for "{query}". Try searching for tabs, devtools, AI, or settings.
              </span>
            </div>
          ) : (
            categories.map((cat) => {
              const catItems = filteredCommands.filter((cmd) => cmd.category === cat);
              if (catItems.length === 0) return null;

              return (
                <div key={cat} className="command-palette__group" role="group" aria-label={cat}>
                  <div className="command-palette__group-header">{cat}</div>
                  {catItems.map((item) => {
                    const currentIndex = itemCounter++;
                    const isSelected = currentIndex === selectedIndex;

                    return (
                      <button
                        key={item.id}
                        type="button"
                        role="option"
                        aria-selected={isSelected}
                        disabled={item.disabled}
                        className={`command-palette__item ${
                          isSelected ? "command-palette__item--selected" : ""
                        } ${item.disabled ? "command-palette__item--disabled" : ""}`}
                        onClick={() => {
                          if (!item.disabled) {
                            item.action();
                            onClose();
                          }
                        }}
                        onMouseEnter={() => setSelectedIndex(currentIndex)}
                      >
                        <div className="command-palette__item-icon" aria-hidden="true">
                          {item.icon}
                        </div>
                        <div className="command-palette__item-content">
                          <span className="command-palette__item-title">{item.title}</span>
                          <span className="command-palette__item-subtitle">{item.subtitle}</span>
                        </div>
                        {item.shortcut && (
                          <kbd className="command-palette__item-shortcut">{item.shortcut}</kbd>
                        )}
                        {isSelected && (
                          <CornerDownLeft
                            size={13}
                            strokeWidth={2}
                            className="command-palette__item-enter-icon"
                            aria-hidden="true"
                          />
                        )}
                      </button>
                    );
                  })}
                </div>
              );
            })
          )}
        </div>

        {/* ── Footer Bar with Shortcuts & Context ─────────────────── */}
        <div className="command-palette__footer">
          <div className="command-palette__hints">
            <span className="command-palette__hint">
              <kbd className="command-palette__key">↑</kbd>
              <kbd className="command-palette__key">↓</kbd> Navigate
            </span>
            <span className="command-palette__hint">
              <kbd className="command-palette__key">↵</kbd> Select
            </span>
            <span className="command-palette__hint">
              <kbd className="command-palette__key">esc</kbd> Dismiss
            </span>
          </div>
          <div className="command-palette__brand">
            <span>Arbelune Control Plane</span>
          </div>
        </div>
      </div>
    </div>,
    document.body
  );
};
