// Dual theme (UI overhaul): "dark" is the sharpened warm-umber look, "light" is warm paper - both defined as
// token overrides in styles.css keyed by <html data-theme="...">. The effective theme is the user's stored
// choice, else the OS preference; index.html applies the same rule inline before first paint so there is no
// flash, and this module keeps React, localStorage, and the theme-color meta in sync afterwards.

import { useEffect, useState } from "react";

export type Theme = "dark" | "light";

const KEY = "cyberos.theme";
// Mirrors --bg per theme, so the browser chrome (mobile status bar, PWA title bar) matches.
const META_COLOR: Record<Theme, string> = { dark: "#170b06", light: "#f3ead9" };

export function storedTheme(): Theme | null {
  try {
    const v = localStorage.getItem(KEY);
    return v === "dark" || v === "light" ? v : null;
  } catch {
    return null;
  }
}

export function systemTheme(): Theme {
  try {
    return window.matchMedia && window.matchMedia("(prefers-color-scheme: light)").matches
      ? "light"
      : "dark";
  } catch {
    return "dark";
  }
}

export function applyTheme(t: Theme) {
  document.documentElement.dataset.theme = t;
  document.querySelector('meta[name="theme-color"]')?.setAttribute("content", META_COLOR[t]);
}

/// The current theme + a toggle that persists the choice. Until the user toggles once, the theme follows the
/// OS preference live (matchMedia listener); after a toggle, the stored choice wins.
export function useTheme(): [Theme, () => void] {
  const [theme, setTheme] = useState<Theme>(() => storedTheme() ?? systemTheme());

  useEffect(() => {
    applyTheme(theme);
  }, [theme]);

  // Follow OS changes only while the user has not made an explicit choice.
  useEffect(() => {
    if (storedTheme()) return;
    const mq = window.matchMedia("(prefers-color-scheme: light)");
    const onChange = () => setTheme(storedTheme() ?? systemTheme());
    mq.addEventListener?.("change", onChange);
    return () => mq.removeEventListener?.("change", onChange);
  }, []);

  const toggle = () => {
    setTheme((cur) => {
      const next: Theme = cur === "dark" ? "light" : "dark";
      try {
        localStorage.setItem(KEY, next);
      } catch {
        /* private mode - the choice just does not persist */
      }
      return next;
    });
  };
  return [theme, toggle];
}
