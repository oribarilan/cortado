import {
  useCurrentFrame,
  useVideoConfig,
  interpolate,
  spring,
  AbsoluteFill,
} from "remotion";
import { COLORS, FONT, FONT_MONO } from "../constants";

// --- Sub-components ---

const StatusDot = ({ color, pulse, size = 9 }) => {
  const frame = useCurrentFrame();
  const pulseScale = pulse ? 1 + 0.25 * Math.sin(frame * 0.15) : 1;

  return (
    <div
      style={{
        width: size,
        height: size,
        borderRadius: "50%",
        backgroundColor: color,
        transform: `scale(${pulseScale})`,
        boxShadow: pulse ? `0 0 ${size * 1.5}px ${color}` : "none",
        flexShrink: 0,
      }}
    />
  );
};

const PanelRow = ({
  dot,
  title,
  status,
  statusColor,
  selected,
  pulse,
  feedHint,
  flash = 0,
}) => {
  const flashBg =
    selected && flash > 0
      ? `rgba(78, 205, 196, ${0.12 + flash * 0.18})`
      : selected
        ? COLORS.accentDim
        : "transparent";
  const flashBorder =
    selected && flash > 0
      ? `3px solid rgba(78, 205, 196, ${0.6 + flash * 0.4})`
      : selected
        ? `3px solid ${COLORS.accent}`
        : "3px solid transparent";
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "10px 21px",
        backgroundColor: flashBg,
        borderLeft: flashBorder,
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 13 }}>
        <StatusDot color={dot} size={9} pulse={pulse} />
        <span
          style={{
            fontSize: 17.5,
            color: selected ? COLORS.text : COLORS.textSecondary,
            fontWeight: selected ? 500 : 400,
          }}
        >
          {title}
        </span>
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: 13 }}>
        <span
          style={{
            fontSize: 15,
            color: statusColor,
            backgroundColor: `${statusColor}1a`,
            padding: "3px 13px",
            borderRadius: 13,
          }}
        >
          {status}
        </span>
        {feedHint && (
          <span
            style={{
              fontSize: 11,
              color: COLORS.textTertiary,
              textTransform: "uppercase",
              letterSpacing: "0.04em",
            }}
          >
            {feedHint}
          </span>
        )}
      </div>
    </div>
  );
};

const DetailField = ({ label, value, color }) => (
  <div
    style={{
      display: "flex",
      justifyContent: "space-between",
      alignItems: "center",
    }}
  >
    <span style={{ fontSize: 17, color: COLORS.textTertiary }}>{label}</span>
    <span
      style={{
        fontSize: 17,
        color: color || COLORS.textSecondary,
        fontWeight: 500,
      }}
    >
      {value}
    </span>
  </div>
);

// Terminal tab component
const TerminalTab = ({ name, active }) => (
  <div
    style={{
      padding: "8px 21px",
      fontSize: 16,
      fontFamily: FONT_MONO,
      color: active ? COLORS.text : COLORS.textTertiary,
      backgroundColor: active ? COLORS.bgInset : "transparent",
      borderBottom: active
        ? `3px solid ${COLORS.accent}`
        : "3px solid transparent",
      fontWeight: active ? 500 : 400,
    }}
  >
    {name}
  </div>
);

// Terminal line
const TermLine = ({ children, color, indent = 0, mono = true }) => (
  <div
    style={{
      fontSize: 17,
      fontFamily: mono ? FONT_MONO : FONT,
      color: color || COLORS.textSecondary,
      paddingLeft: indent * 21,
      lineHeight: 1.6,
    }}
  >
    {children}
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

  // Keyboard shortcut (frames 0-40)
  const kbdOpacity = interpolate(frame, [0, 15, 30, 42], [0, 1, 1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const kbdScale = interpolate(frame, [0, 15], [0.92, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  // Key press visual — keys depress at frame 10 (synced with click SFX)
  const keyPress = interpolate(frame, [10, 12, 16], [0, 1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Panel appears (frames 30-70)
  const panelProgress = spring({
    frame: frame - 32,
    fps,
    config: { damping: 14, mass: 0.7 },
  });
  const panelOpacity = interpolate(panelProgress, [0, 0.3], [0, 1], {
    extrapolateRight: "clamp",
  });
  const panelScale = interpolate(panelProgress, [0, 1], [0.93, 1]);
  const panelY = interpolate(panelProgress, [0, 1], [31, 0]);

  // Selection highlights OpenCode row (frame 75)
  const selectionOn = frame >= 75;
  // Row click flash — brief bright highlight at moment of click
  const rowFlash = interpolate(frame, [75, 77, 83], [0, 1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Panel slides away (frame 120)
  const panelExit = interpolate(frame, [118, 135], [1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const panelExitX = interpolate(frame, [118, 135], [0, -78], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Terminal slides in (frame 130)
  const termProgress = spring({
    frame: frame - 132,
    fps,
    config: { damping: 14, mass: 0.7 },
  });
  const termOpacity = interpolate(termProgress, [0, 0.3], [0, 1], {
    extrapolateRight: "clamp",
  });
  const termScale = interpolate(termProgress, [0, 1], [0.93, 1]);
  const termY = interpolate(termProgress, [0, 1], [31, 0]);

  // Cursor blink in terminal
  const cursorVisible = Math.floor(frame * 0.06) % 2 === 0;

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
      {/* Keyboard shortcut overlay */}
      <div
        style={{
          position: "absolute",
          opacity: kbdOpacity,
          transform: `scale(${kbdScale})`,
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: 21,
          zIndex: 10,
        }}
      >
        <div
          style={{
            fontSize: 42,
            color: COLORS.textTertiary,
            fontWeight: 400,
          }}
        >
          ...or a full view
        </div>
        <div style={{ display: "flex", gap: 13, alignItems: "center" }}>
          {["⌘", "⇧", "Space"].map((key) => {
            const pressY = interpolate(keyPress, [0, 1], [0, 3]);
            const pressScale = interpolate(keyPress, [0, 1], [1, 0.93]);
            const pressBg = interpolate(keyPress, [0, 1], [0, 0.15]);
            return (
              <div
                key={key}
                style={{
                  padding: "13px 26px",
                  backgroundColor: `rgba(255,255,255,${0.04 + pressBg})`,
                  border: `1px solid ${keyPress > 0.5 ? COLORS.accent : COLORS.borderLight}`,
                  borderRadius: 13,
                  fontSize: 39,
                  color: COLORS.text,
                  fontWeight: 500,
                  transform: `translateY(${pressY}px) scale(${pressScale})`,
                  boxShadow:
                    keyPress > 0.5 ? `0 0 16px ${COLORS.accentDim}` : "none",
                }}
              >
                {key}
              </div>
            );
          })}
        </div>
      </div>

      {/* Panel */}
      <div
        style={{
          width: 1365,
          height: 806,
          backgroundColor: COLORS.frosted,
          borderRadius: 18,
          border: `1px solid ${COLORS.borderLight}`,
          boxShadow: "0 31px 104px rgba(0,0,0,0.55)",
          overflow: "hidden",
          opacity: panelOpacity * panelExit,
          transform: `scale(${panelScale}) translateY(${panelY}px) translateX(${panelExitX}px)`,
          display: "flex",
          position: "absolute",
        }}
      >
        {/* List pane */}
        <div
          style={{
            width: "58%",
            borderRight: `1px solid ${COLORS.border}`,
            display: "flex",
            flexDirection: "column",
            overflow: "hidden",
          }}
        >
          {/* Attention */}
          <div style={{ borderBottom: `1px solid ${COLORS.border}` }}>
            <div
              style={{
                padding: "13px 21px 4px",
                display: "flex",
                alignItems: "center",
                gap: 8,
              }}
            >
              <div
                style={{
                  width: 3,
                  height: 18,
                  backgroundColor: COLORS.statusGreen,
                  borderRadius: 3,
                }}
              />
              <span
                style={{
                  fontSize: 13.5,
                  fontWeight: 600,
                  color: COLORS.textTertiary,
                  textTransform: "uppercase",
                  letterSpacing: "0.08em",
                }}
              >
                Attention
              </span>
              <span
                style={{
                  fontSize: 13,
                  color: COLORS.textTertiary,
                  marginLeft: 3,
                }}
              >
                1
              </span>
            </div>
            <PanelRow
              dot={COLORS.statusGreen}
              title="cortado-backend"
              status="question asked"
              statusColor={COLORS.statusGreen}
              feedHint="opencode"
              selected={selectionOn}
              flash={rowFlash}
            />
          </div>

          {/* Coding Agents */}
          <div style={{ flex: 1, padding: "8px 0" }}>
            <div
              style={{
                padding: "8px 21px 4px",
                fontSize: 13.5,
                fontWeight: 600,
                color: COLORS.textTertiary,
                textTransform: "uppercase",
                letterSpacing: "0.08em",
              }}
            >
              Copilot
            </div>
            <PanelRow
              dot={COLORS.statusBlue}
              title="cortado-frontend"
              status="working"
              statusColor={COLORS.statusBlue}
              pulse
            />
            <PanelRow
              dot={COLORS.statusGray}
              title="docs-update"
              status="idle"
              statusColor={COLORS.statusGray}
            />

            <div
              style={{
                padding: "10px 21px 4px",
                fontSize: 13.5,
                fontWeight: 600,
                color: COLORS.textTertiary,
                textTransform: "uppercase",
                letterSpacing: "0.08em",
              }}
            >
              OpenCode
            </div>
            <PanelRow
              dot={COLORS.statusGray}
              title="api-service"
              status="idle"
              statusColor={COLORS.statusGray}
            />

            <div
              style={{
                padding: "10px 21px 4px",
                fontSize: 13.5,
                fontWeight: 600,
                color: COLORS.textTertiary,
                textTransform: "uppercase",
                letterSpacing: "0.08em",
              }}
            >
              GitHub PRs
            </div>
            <PanelRow
              dot={COLORS.statusGreen}
              title="feat: add dark mode"
              status="approved"
              statusColor={COLORS.statusGreen}
            />
            <PanelRow
              dot={COLORS.statusYellow}
              title="fix: memory leak"
              status="awaiting"
              statusColor={COLORS.statusYellow}
            />
          </div>
        </div>

        {/* Detail pane */}
        <div
          style={{
            width: "42%",
            backgroundColor: COLORS.bgInset,
            padding: "29px 26px",
            display: "flex",
            flexDirection: "column",
            gap: 18,
          }}
        >
          <div
            style={{
              fontSize: 22,
              fontWeight: 600,
              color: COLORS.text,
              lineHeight: 1.4,
            }}
          >
            cortado-backend
          </div>
          <div
            style={{
              fontSize: 17,
              color: COLORS.textTertiary,
            }}
          >
            OpenCode session
          </div>

          <div
            style={{
              display: "flex",
              flexDirection: "column",
              gap: 13,
              marginTop: 5,
            }}
          >
            <DetailField
              label="status"
              value="question asked"
              color={COLORS.statusGreen}
            />
            <DetailField label="repo" value="cortado-backend" />
            <DetailField label="branch" value="feat/error-handling" />
          </div>

          <div
            style={{
              fontSize: 17,
              color: COLORS.accent,
              marginTop: 16,
            }}
          >
            Focus Terminal {"->"}
          </div>
        </div>
      </div>

      {/* Terminal mockup */}
      <div
        style={{
          width: 1365,
          height: 806,
          backgroundColor: "#0a0a0a",
          borderRadius: 18,
          border: `1px solid ${COLORS.borderLight}`,
          boxShadow: "0 31px 104px rgba(0,0,0,0.55)",
          overflow: "hidden",
          opacity: termOpacity,
          transform: `scale(${termScale}) translateY(${termY}px)`,
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
          <span
            style={{
              fontSize: 15,
              fontWeight: 700,
              color: "#eeeeee",
            }}
          >
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
          {/* User message bubble */}
          <div
            style={{
              backgroundColor: "#141414",
              padding: "13px 18px",
              borderLeft: "3px solid #fab283",
            }}
          >
            <span style={{ fontSize: 16, color: "#eeeeee" }}>
              Which approach should I use for error handling in the API
              handlers?
            </span>
          </div>

          {/* Assistant message */}
          <div
            style={{
              display: "flex",
              flexDirection: "column",
            }}
          >
            {/* Tool calls */}
            <div style={{ fontSize: 15, color: "#808080", lineHeight: 1.7 }}>
              <div>
                <span style={{ color: "#fab283" }}>*</span> Grep
                &quot;error.*handling|anyhow|thiserror&quot;
              </div>
              <div>
                <span style={{ color: "#fab283" }}>*</span> Read
                src/api/handlers.rs{" "}
                <span style={{ color: "#808080" }}>(247 lines)</span>
              </div>
            </div>

            <div style={{ height: 10 }} />

            {/* Response text */}
            <div style={{ fontSize: 16, color: "#eeeeee", lineHeight: 1.6 }}>
              I found two potential approaches:
            </div>

            <div style={{ height: 8 }} />

            {/* Numbered options */}
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
                1. Use <span style={{ color: "#7fd88f" }}>anyhow::Result</span>{" "}
                with context
              </div>
              <div>
                2. Define custom error types with{" "}
                <span style={{ color: "#7fd88f" }}>thiserror</span>
              </div>
            </div>

            <div style={{ height: 10 }} />

            {/* Status line */}
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

        {/* Footer/status bar */}
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

      {/* Scene subtitle */}
      <div
        style={{
          position: "absolute",
          bottom: 104,
          width: "100%",
          textAlign: "center",
          opacity: subtitleOpacity,
        }}
      >
        <div
          style={{
            fontSize: 26,
            color: COLORS.textTertiary,
            fontWeight: 400,
          }}
        >
          Your agent needs you. One click.
        </div>
      </div>
    </AbsoluteFill>
  );
};
