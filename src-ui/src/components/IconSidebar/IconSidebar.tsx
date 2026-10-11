import React, { useState } from "react";
import "./IconSidebar.css";
import { SmartTooltip } from "../ui";
import { ChevronsLeft, ChevronsRight } from "lucide-react";

export type SidebarItem = {
  id: string;
  icon: React.ReactNode;
  label: string;
  shortcut?: string;
  isActive?: boolean;
  isRunning?: boolean;
  badge?: string;
};

export type SidebarGroup = {
  id: string;
  label?: string;
  items: SidebarItem[];
};

interface IconSidebarProps {
  groups?: SidebarGroup[];
  items?: SidebarItem[];
  bottomItems?: SidebarItem[];
  onSelect: (id: string) => void;
  activeId?: string;
  collapsed?: boolean;
  onToggleCollapse?: () => void;
}

export const IconSidebar: React.FC<IconSidebarProps> = ({
  groups,
  items,
  bottomItems = [],
  onSelect,
  activeId,
  collapsed: propCollapsed,
  onToggleCollapse,
}) => {
  const [internalCollapsed, setInternalCollapsed] = useState(false);
  const isCollapsed = propCollapsed !== undefined ? propCollapsed : internalCollapsed;
  const toggleCollapse = onToggleCollapse || (() => setInternalCollapsed((prev) => !prev));

  // If groups are provided, use them; otherwise, wrap legacy items into a default group
  const effectiveGroups: SidebarGroup[] = groups || (items ? [{ id: "main", items }] : []);

  const renderItem = (item: SidebarItem) => {
    const isItemActive = activeId === item.id || item.isActive;
    const button = (
      <button
        className={`sidebar-item sidebar-item--${item.id} ${isItemActive ? "sidebar-item--active" : ""}`}
        onClick={() => onSelect(item.id)}
        aria-label={item.label}
        aria-current={isItemActive ? "page" : undefined}
        id={`sidebar-${item.id}`}
      >
        <span className="sidebar-item__icon" aria-hidden="true">
          {item.icon}
          {item.isRunning && isCollapsed && (
            <span className="sidebar-item__status-dot sidebar-item__status-dot--corner" aria-hidden="true" />
          )}
        </span>
        <span className="sidebar-item__label">{item.label}</span>
        {item.isRunning && !isCollapsed && (
          <span className="sidebar-item__status-dot" aria-label="Active status" />
        )}
      </button>
    );

    if (isCollapsed) {
      return (
        <SmartTooltip key={item.id} content={item.label} shortcut={item.shortcut} position="right">
          {button}
        </SmartTooltip>
      );
    }

    return <React.Fragment key={item.id}>{button}</React.Fragment>;
  };

  return (
    <nav
      className={`icon-sidebar ${isCollapsed ? "icon-sidebar--collapsed" : "icon-sidebar--expanded"}`}
      aria-label="Browser navigation and developer tools"
      role="navigation"
    >
      <div className="icon-sidebar__top">
        {/* Brand & Workspace Header (Visible when expanded) */}
        {!isCollapsed && (
          <div className="icon-sidebar__brand-header">
            <span className="icon-sidebar__brand-title">arbelune</span>
            <span className="icon-sidebar__brand-subtitle">BROWSER WORKSPACE</span>
          </div>
        )}

        {/* Grouped Navigation Sections */}
        <div className="icon-sidebar__groups">
          {effectiveGroups.map((group, groupIndex) => (
            <div key={group.id} className="sidebar-group">
              {groupIndex > 0 && (
                <div
                  className={`sidebar-group__separator ${
                    isCollapsed ? "sidebar-group__separator--collapsed" : ""
                  }`}
                  aria-hidden="true"
                />
              )}
              {group.label && !isCollapsed && (
                <div className="sidebar-group__label">{group.label}</div>
              )}
              <div className="sidebar-group__items">
                {group.items.map(renderItem)}
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Footer Area: Settings + Utility Controls */}
      <div className="icon-sidebar__bottom">
        {bottomItems.length > 0 && (
          <div className="icon-sidebar__bottom-items">
            {bottomItems.map(renderItem)}
          </div>
        )}

        <div className="icon-sidebar__footer-utility">
          <SmartTooltip
            content={isCollapsed ? "Expand sidebar" : "Collapse sidebar"}
            shortcut="Ctrl+["
            position="right"
          >
            <button
              className="icon-sidebar__collapse-btn"
              onClick={toggleCollapse}
              aria-label={isCollapsed ? "Expand sidebar" : "Collapse sidebar"}
            >
              {isCollapsed ? <ChevronsRight size={16} /> : <ChevronsLeft size={16} />}
            </button>
          </SmartTooltip>

          {!isCollapsed && (
            <SmartTooltip
              content="Arbelune Browser v0.4.1-alpha · CEF 152 · Rust Core"
              position="top"
            >
              <div
                className="icon-sidebar__version-badge"
                role="status"
                aria-label="Arbelune v0.4.1"
                tabIndex={0}
              >
                <span className="icon-sidebar__version-dot" aria-hidden="true" />
                <span className="icon-sidebar__version-text">v0.4.1</span>
              </div>
            </SmartTooltip>
          )}
        </div>
      </div>
    </nav>
  );
};
