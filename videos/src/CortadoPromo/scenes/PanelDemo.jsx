import {
  useCurrentFrame,
  useVideoConfig,
  interpolate,
  spring,
  Easing,
  AbsoluteFill,
  Img,
  staticFile,
} from "remotion";
import { COLORS, FONT, FONT_MONO } from "../constants";

// --- Code editor mockup (abstract, blurred lines) ---

const CODE_LINES = [
  { indent: 0, width: 180, kw: true },
  { indent: 1, width: 280, kw: false },
  { indent: 1, width: 220, kw: false },
  { indent: 2, width: 340, kw: false },
  { indent: 2, width: 160, kw: true },
  { indent: 3, width: 300, kw: false },
  { indent: 3, width: 260, kw: false },
  { indent: 2, width: 80, kw: false },
  { indent: 1, width: 60, kw: false },
  { indent: 0, width: 0, kw: false }, // blank
  { indent: 0, width: 200, kw: true },
  { indent: 1, width: 320, kw: false },
  { indent: 1, width: 180, kw: false },
  { indent: 2, width: 280, kw: false },
  { indent: 2, width: 240, kw: false },
  { indent: 2, width: 200, kw: true },
  { indent: 3, width: 360, kw: false },
  { indent: 3, width: 140, kw: false },
  { indent: 2, width: 80, kw: false },
  { indent: 1, width: 60, kw: false },
  { indent: 0, width: 0, kw: false },
  { indent: 0, width: 160, kw: true },
  { indent: 1, width: 300, kw: false },
  { indent: 1, width: 240, kw: false },
];

const CodeLine = ({
  indent,
  width,
  kw,
  lineNum,
  cursorLine,
  cursorVisible,
}) => (
  <div
    style={{
      display: "flex",
      alignItems: "center",
      height: 26,
      paddingLeft: 60 + indent * 28,
      position: "relative",
    }}
  >
    {/* Line number */}
    <span
      style={{
        position: "absolute",
        left: 16,
        fontSize: 13,
        fontFamily: FONT_MONO,
        color: lineNum === cursorLine ? "#808080" : "#3a3a3a",
        width: 30,
        textAlign: "right",
      }}
    >
      {lineNum}
    </span>
    {/* Code block */}
    {width > 0 && (
      <div
        style={{
          width,
          height: 10,
          borderRadius: 4,
          backgroundColor: kw
            ? "rgba(157, 124, 216, 0.25)"
            : "rgba(238, 238, 238, 0.08)",
        }}
      />
    )}
    {/* Cursor */}
    {lineNum === cursorLine && (
      <div
        style={{
          width: 2,
          height: 18,
          backgroundColor: "#fab283",
          marginLeft: 4,
          opacity: cursorVisible ? 1 : 0,
        }}
      />
    )}
  </div>
);

// --- macOS notification banner ---

const NotificationBanner = ({ opacity, y, flash }) => (
  <div
    style={{
      position: "absolute",
      top: 20,
      right: 40,
      width: 420,
      backgroundColor: `rgba(50, 50, 55, ${0.95 + flash * 0.05})`,
      borderRadius: 16,
      padding: "14px 18px",
      display: "flex",
      alignItems: "center",
      gap: 14,
      boxShadow:
        flash > 0
          ? "0 8px 40px rgba(78, 205, 196, 0.25)"
          : "0 8px 40px rgba(0,0,0,0.5)",
      border:
        flash > 0
          ? "1px solid rgba(78, 205, 196, 0.4)"
          : "1px solid rgba(255,255,255,0.1)",
      opacity,
      transform: `translateY(${y}px) scale(${1 + flash * 0.02})`,
      zIndex: 20,
    }}
  >
    {/* App icon */}
    <Img
      src={staticFile("cortado-icon.png")}
      style={{
        width: 40,
        height: 40,
        borderRadius: 10,
        flexShrink: 0,
      }}
    />
    <div style={{ display: "flex", flexDirection: "column", gap: 3 }}>
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 8,
        }}
      >
        <span
          style={{
            fontSize: 14,
            fontWeight: 600,
            color: "#eeeeee",
            fontFamily: FONT,
          }}
        >
          cortado
        </span>
        <span
          style={{
            fontSize: 12,
            color: "#808080",
            fontFamily: FONT,
          }}
        >
          now
        </span>
      </div>
      <span
        style={{
          fontSize: 14,
          color: "#cccccc",
          fontFamily: FONT,
          fontWeight: 400,
        }}
      >
        OpenCode agent asked a question
      </span>
      <span
        style={{
          fontSize: 12,
          color: "#808080",
          fontFamily: FONT_MONO,
          fontWeight: 400,
        }}
      >
        cortado-backend &middot; feat/error-handling
      </span>
    </div>
  </div>
);

// --- OpenCode TUI terminal ---

const OpenCodeTerminal = ({ opacity, scale, y, cursorVisible }) => (
  <div
    style={{
      width: 1365,
      height: 806,
      backgroundColor: "#0a0a0a",
      borderRadius: 18,
      border: "1px solid rgba(255,255,255,0.1)",
      boxShadow: "0 31px 104px rgba(0,0,0,0.55)",
      overflow: "hidden",
      opacity,
      transform: `scale(${scale}) translateY(${y}px)`,
      display: "flex",
      flexDirection: "column",
      position: "absolute",
      fontFamily: FONT_MONO,
    }}
  >
    {/* Header bar */}
    <div
      style={{
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
        padding: "10px 21px",
        backgroundColor: "#141414",
        borderBottom: "1px solid #484848",
      }}
    >
      <span style={{ fontSize: 15, fontWeight: 700, color: "#eeeeee" }}>
        # Error handling strategy for cortado-backend
      </span>
      <span style={{ fontSize: 15, color: "#808080" }}>
        42,817 &middot; 24% &middot; ($0.31)
      </span>
    </div>

    {/* Message area */}
    <div
      style={{
        flex: 1,
        display: "flex",
        flexDirection: "column",
        padding: "21px 26px",
        gap: 16,
        overflow: "hidden",
      }}
    >
      {/* User message */}
      <div
        style={{
          backgroundColor: "#141414",
          padding: "13px 18px",
          borderLeft: "3px solid #fab283",
        }}
      >
        <span style={{ fontSize: 16, color: "#eeeeee" }}>
          Which approach should I use for error handling in the API handlers?
        </span>
      </div>

      {/* Assistant message */}
      <div style={{ display: "flex", flexDirection: "column" }}>
        <div style={{ fontSize: 15, color: "#808080", lineHeight: 1.7 }}>
          <div>
            <span style={{ color: "#fab283" }}>*</span> Grep
            &quot;error.*handling|anyhow|thiserror&quot;
          </div>
          <div>
            <span style={{ color: "#fab283" }}>*</span> Read src/api/handlers.rs{" "}
            <span style={{ color: "#808080" }}>(247 lines)</span>
          </div>
        </div>

        <div style={{ height: 10 }} />

        <div style={{ fontSize: 16, color: "#eeeeee", lineHeight: 1.6 }}>
          I found two potential approaches:
        </div>

        <div style={{ height: 8 }} />

        <div
          style={{
            fontSize: 15,
            color: "#eeeeee",
            paddingLeft: 21,
            display: "flex",
            flexDirection: "column",
            gap: 4,
            lineHeight: 1.6,
          }}
        >
          <div>
            1. Use <span style={{ color: "#7fd88f" }}>anyhow::Result</span> with
            context
          </div>
          <div>
            2. Define custom error types with{" "}
            <span style={{ color: "#7fd88f" }}>thiserror</span>
          </div>
        </div>

        <div style={{ height: 10 }} />

        <div style={{ fontSize: 14, color: "#808080" }}>
          <span style={{ color: "#f5a742" }}>~</span> Asking question...
        </div>
      </div>
    </div>

    {/* Input area */}
    <div
      style={{
        borderTop: "1px solid #484848",
        backgroundColor: "#141414",
        padding: "13px 21px",
      }}
    >
      <div>
        <span
          style={{
            fontSize: 16,
            color: "#fab283",
            opacity: cursorVisible ? 1 : 0,
          }}
        >
          {"\u2588"}
        </span>
      </div>
      <div
        style={{
          fontSize: 13,
          marginTop: 4,
          display: "flex",
          alignItems: "center",
          gap: 6,
        }}
      >
        <span style={{ color: "#5c9cf5" }}>{"\u25C9"}</span>
        <span style={{ color: "#eeeeee" }}>Build</span>
        <span style={{ color: "#808080" }}>&middot; claude-opus-4-5</span>
      </div>
    </div>

    {/* Footer */}
    <div
      style={{
        backgroundColor: "#141414",
        padding: "6px 21px",
        borderTop: "1px solid #484848",
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
      }}
    >
      <span style={{ fontSize: 12, color: "#808080" }}>esc interrupt</span>
      <div style={{ display: "flex", gap: 16, fontSize: 12 }}>
        <span>
          <span style={{ color: "#eeeeee" }}>ctrl+t</span>{" "}
          <span style={{ color: "#808080" }}>variants</span>
        </span>
        <span>
          <span style={{ color: "#eeeeee" }}>tab</span>{" "}
          <span style={{ color: "#808080" }}>agents</span>
        </span>
        <span>
          <span style={{ color: "#eeeeee" }}>ctrl+p</span>{" "}
          <span style={{ color: "#808080" }}>commands</span>
        </span>
      </div>
    </div>
  </div>
);

// --- Main scene ---

export const PanelDemo = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Scene fade-in/out
  const fadeIn = interpolate(frame, [0, 12], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const fadeOut = interpolate(frame, [225, 240], [1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // --- Code editor phase (frames 0-100) ---
  const editorOpacity = interpolate(frame, [0, 12, 100, 115], [0, 1, 1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  // Cursor blink for editor
  const editorCursorVisible = Math.floor(frame * 0.08) % 2 === 0;
  // Cursor line slowly moves down to simulate typing
  const editorCursorLine = Math.floor(
    interpolate(frame, [0, 90], [8, 14], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
    }),
  );

  // --- Notification (frames 45-100) ---
  const notifSlideIn = spring({
    frame: frame - 45,
    fps,
    config: { damping: 14, mass: 0.6 },
  });
  const notifY = interpolate(notifSlideIn, [0, 1], [-80, 0]);
  const notifOpacity = interpolate(notifSlideIn, [0, 0.3], [0, 1], {
    extrapolateRight: "clamp",
  });
  // Notification disappears with editor
  const notifFade = interpolate(frame, [100, 115], [1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  // Click flash on notification (frame 95, synced with mouse-click SFX)
  const notifFlash = interpolate(frame, [95, 97, 103], [0, 1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // --- Zoom into notification (frames 60-80) ---
  // Notification center: right=40, width=420 → centerX ≈ 1920-40-210 = 1670
  // top=20, height≈68 → centerY ≈ 54
  const NOTIF_CX = 1670;
  const NOTIF_CY = 54;
  const SCREEN_CX = 960;
  const SCREEN_CY = 540;
  const zoomProgress = interpolate(frame, [60, 80], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.quad),
  });
  const zoomScale = interpolate(zoomProgress, [0, 1], [1, 2.2]);
  const zoomTargetX = interpolate(zoomProgress, [0, 1], [NOTIF_CX, SCREEN_CX]);
  const zoomTargetY = interpolate(zoomProgress, [0, 1], [NOTIF_CY, SCREEN_CY]);
  const zoomTx = zoomTargetX - NOTIF_CX * zoomScale;
  const zoomTy = zoomTargetY - NOTIF_CY * zoomScale;

  // --- Terminal phase (frames 115-240) ---
  const termProgress = spring({
    frame: frame - 118,
    fps,
    config: { damping: 14, mass: 0.7 },
  });
  const termOpacity = interpolate(termProgress, [0, 0.3], [0, 1], {
    extrapolateRight: "clamp",
  });
  const termScale = interpolate(termProgress, [0, 1], [0.93, 1]);
  const termY = interpolate(termProgress, [0, 1], [31, 0]);
  const termCursorVisible = Math.floor(frame * 0.06) % 2 === 0;

  // Subtitle
  const subtitleOpacity = interpolate(frame, [140, 160], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <AbsoluteFill
      style={{
        fontFamily: FONT,
        justifyContent: "center",
        alignItems: "center",
        opacity: fadeIn * fadeOut,
      }}
    >
      {/* Zoomable container — editor + notification zoom into the notification */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          transformOrigin: "0 0",
          transform: `translate(${zoomTx}px, ${zoomTy}px) scale(${zoomScale})`,
          display: "flex",
          justifyContent: "center",
          alignItems: "center",
        }}
      >
        {/* Code editor mockup */}
        <div
          style={{
            width: 1365,
            height: 806,
            backgroundColor: "#0d0f16",
            borderRadius: 18,
            border: "1px solid rgba(255,255,255,0.06)",
            boxShadow: "0 31px 104px rgba(0,0,0,0.55)",
            overflow: "hidden",
            opacity: editorOpacity,
            position: "absolute",
            display: "flex",
            flexDirection: "column",
          }}
        >
          {/* Editor title bar */}
          <div
            style={{
              display: "flex",
              alignItems: "center",
              padding: "10px 18px",
              backgroundColor: "#181b26",
              borderBottom: "1px solid rgba(255,255,255,0.06)",
              gap: 10,
            }}
          >
            {/* Traffic lights */}
            <div style={{ display: "flex", gap: 7 }}>
              <div
                style={{
                  width: 12,
                  height: 12,
                  borderRadius: "50%",
                  backgroundColor: "#e05545",
                }}
              />
              <div
                style={{
                  width: 12,
                  height: 12,
                  borderRadius: "50%",
                  backgroundColor: "#d4a838",
                }}
              />
              <div
                style={{
                  width: 12,
                  height: 12,
                  borderRadius: "50%",
                  backgroundColor: "#5cb87a",
                }}
              />
            </div>
            <span
              style={{
                fontSize: 13,
                color: "#808080",
                fontFamily: FONT_MONO,
                marginLeft: 12,
              }}
            >
              handlers.rs -- cortado-backend
            </span>
          </div>

          {/* Code lines */}
          <div
            style={{
              flex: 1,
              padding: "12px 0",
              display: "flex",
              flexDirection: "column",
            }}
          >
            {CODE_LINES.map((line, i) => (
              <CodeLine
                key={i}
                {...line}
                lineNum={i + 1}
                cursorLine={editorCursorLine}
                cursorVisible={editorCursorVisible}
              />
            ))}
          </div>
        </div>

        {/* macOS notification */}
        <NotificationBanner
          opacity={notifOpacity * notifFade}
          y={notifY}
          flash={notifFlash}
        />
      </div>

      {/* OpenCode terminal */}
      <OpenCodeTerminal
        opacity={termOpacity}
        scale={termScale}
        y={termY}
        cursorVisible={termCursorVisible}
      />

      {/* Subtitle — mid-screen, prominent */}
      <div
        style={{
          position: "absolute",
          top: "50%",
          left: "50%",
          transform: "translate(-50%, -50%)",
          textAlign: "center",
          opacity: subtitleOpacity,
          zIndex: 30,
        }}
      >
        <div
          style={{
            fontSize: 42,
            color: "#4ecdc4",
            fontWeight: 500,
            letterSpacing: "-0.01em",
            backgroundColor: "rgba(78, 205, 196, 0.1)",
            padding: "16px 36px",
            borderRadius: 14,
            border: "1px solid rgba(78, 205, 196, 0.2)",
          }}
        >
          Let agents reach out when it matters.
        </div>
      </div>
    </AbsoluteFill>
  );
};
