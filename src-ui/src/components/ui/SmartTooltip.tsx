import React, { useState, useRef, useEffect, useCallback, useId } from "react";
import { createPortal } from "react-dom";
import "./SmartTooltip.css";

export interface SmartTooltipProps {
  content: React.ReactNode;
  children: React.ReactElement;
  position?: "top" | "bottom" | "left" | "right";
  delayMs?: number;
  shortcut?: string;
  className?: string;
}

// Global state tracking whether any tooltip is currently open, to enable instant transition for subsequent hovers (Emil Kowalski principle)
let globalTooltipActiveTime = 0;
const TOOLTIP_WARM_WINDOW_MS = 400;

export const SmartTooltip: React.FC<SmartTooltipProps> = ({
  content,
  children,
  position = "top",
  delayMs = 220,
  shortcut,
  className = "",
}) => {
  const [isVisible, setIsVisible] = useState(false);
  const [isInstant, setIsInstant] = useState(false);
  const [coords, setCoords] = useState<{ top: number; left: number }>({ top: 0, left: 0 });
  
  const wrapRef = useRef<HTMLDivElement>(null);
  const timerRef = useRef<number | null>(null);
  const tooltipId = useId();

  const updateCoordinates = useCallback(() => {
    if (!wrapRef.current) return;
    const rect = wrapRef.current.getBoundingClientRect();
    const padding = 8;
    let top = 0;
    let left = 0;

    switch (position) {
      case "right":
        left = rect.right + padding;
        top = rect.top + rect.height / 2;
        // Clamp top to viewport bounds
        top = Math.max(16, Math.min(top, window.innerHeight - 16));
        break;
      case "left":
        left = rect.left - padding;
        top = rect.top + rect.height / 2;
        top = Math.max(16, Math.min(top, window.innerHeight - 16));
        break;
      case "bottom":
        left = rect.left + rect.width / 2;
        top = rect.bottom + padding;
        left = Math.max(24, Math.min(left, window.innerWidth - 24));
        break;
      case "top":
      default:
        left = rect.left + rect.width / 2;
        top = rect.top - padding;
        left = Math.max(24, Math.min(left, window.innerWidth - 24));
        break;
    }

    setCoords({ top, left });
  }, [position]);

  const showTooltip = useCallback(() => {
    const now = Date.now();
    const isWarm = now - globalTooltipActiveTime < TOOLTIP_WARM_WINDOW_MS;

    updateCoordinates();

    if (isWarm) {
      // Skip delay and transition animation on subsequent adjacent hovers
      setIsInstant(true);
      setIsVisible(true);
      globalTooltipActiveTime = now;
    } else {
      setIsInstant(false);
      timerRef.current = window.setTimeout(() => {
        updateCoordinates();
        setIsVisible(true);
        globalTooltipActiveTime = Date.now();
      }, delayMs);
    }
  }, [delayMs, updateCoordinates]);

  const hideTooltip = useCallback(() => {
    if (timerRef.current !== null) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }
    if (isVisible) {
      globalTooltipActiveTime = Date.now();
    }
    setIsVisible(false);
  }, [isVisible]);

  useEffect(() => {
    return () => {
      if (timerRef.current !== null) {
        clearTimeout(timerRef.current);
      }
    };
  }, []);

  // Update coords on window resize while visible
  useEffect(() => {
    if (!isVisible) return;
    const handleResize = () => updateCoordinates();
    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, [isVisible, updateCoordinates]);

  // Determine origin for Emil Kowalski's origin-aware animations
  const originMap: Record<string, string> = {
    top: "bottom center",
    bottom: "top center",
    left: "center right",
    right: "center left",
  };

  const clonedChild = React.isValidElement(children)
    ? React.cloneElement(children as React.ReactElement<{ "aria-describedby"?: string }>, {
        "aria-describedby": isVisible ? tooltipId : undefined,
      })
    : children;

  return (
    <div
      ref={wrapRef}
      className="smart-tooltip-wrap"
      onMouseEnter={showTooltip}
      onMouseLeave={hideTooltip}
      onFocus={showTooltip}
      onBlur={hideTooltip}
    >
      {clonedChild}
      {isVisible &&
        createPortal(
          <div
            id={tooltipId}
            role="tooltip"
            className={`smart-tooltip smart-tooltip--${position} ${
              isInstant ? "smart-tooltip--instant" : ""
            } ${className}`}
            style={
              {
                left: `${coords.left}px`,
                top: `${coords.top}px`,
                "--tooltip-origin": originMap[position],
              } as React.CSSProperties
            }
          >
            <span className="smart-tooltip__label">{content}</span>
            {shortcut && <kbd className="smart-tooltip__shortcut">{shortcut}</kbd>}
          </div>,
          document.body
        )}
    </div>
  );
};
