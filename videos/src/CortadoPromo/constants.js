// Design tokens matching the cortado app's visual identity

export const COLORS = {
  bg: "#0d0f16",
  bgSurface: "#181b26",
  bgRaised: "#1e2230",
  bgInset: "#14161f",

  text: "#e6e8f0",
  textSecondary: "#9a9db2",
  textTertiary: "#6c6f84",

  accent: "#4ecdc4",
  accentDim: "rgba(78, 205, 196, 0.12)",

  statusRed: "#e05545",
  statusRedDim: "rgba(224, 85, 69, 0.15)",
  statusYellow: "#d4a838",
  statusYellowDim: "rgba(212, 168, 56, 0.15)",
  statusBlue: "#4a8de8",
  statusBlueDim: "rgba(74, 141, 232, 0.15)",
  statusGreen: "#5cb87a",
  statusGreenDim: "rgba(92, 184, 122, 0.15)",
  statusGray: "#8a8da0",
  statusGrayDim: "rgba(138, 141, 160, 0.15)",

  border: "rgba(255, 255, 255, 0.06)",
  borderLight: "rgba(255, 255, 255, 0.1)",

  menubar: "rgba(38, 40, 52, 0.95)",
  frosted: "rgba(22, 24, 36, 0.92)",
};

export const FONT = "'Space Grotesk', sans-serif";
export const FONT_MONO = "'Space Mono', monospace";

export const FPS = 30;
export const DURATION_FRAMES = 1065; // 35.5 seconds

// Scene timings (in frames)
export const SCENES = {
  hook: { from: 0, duration: 255 }, // 0-8.5s (phrase + notifications + logo)
  panel: { from: 255, duration: 240 }, // 8.5-16.5s (notification + OpenCode)
  menubar: { from: 495, duration: 240 }, // 16.5-24.5s (tray demo)
  closing: { from: 735, duration: 330 }, // 24.5-35.5s (2s extra linger + fadeout)
};
