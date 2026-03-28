import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface AppSettings {
  theme: string;
  text_size: string;
}

interface AppearancePayload {
  theme: string;
  text_size: string;
}

function applyAppearance(theme: string, textSize: string) {
  const root = document.documentElement;
  root.dataset.theme = theme;
  root.dataset.textSize = textSize;
}

/**
 * Reads appearance settings on mount and listens for live changes.
 * Sets `data-theme` and `data-text-size` attributes on `<html>`.
 */
export function useAppearance() {
  useEffect(() => {
    let cancelled = false;

    invoke<AppSettings>("get_settings").then((s) => {
      if (!cancelled) {
        applyAppearance(s.theme ?? "system", s.text_size ?? "m");
      }
    });

    const unlistenPromise = listen<AppearancePayload>(
      "appearance-changed",
      (event) => {
        applyAppearance(event.payload.theme, event.payload.text_size);
      },
    );

    return () => {
      cancelled = true;
      unlistenPromise.then((fn) => fn());
    };
  }, []);
}
