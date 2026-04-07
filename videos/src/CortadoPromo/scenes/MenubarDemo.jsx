import {
  useCurrentFrame,
  useVideoConfig,
  interpolate,
  spring,
  Easing,
  AbsoluteFill,
} from "remotion";
import { COLORS, FONT } from "../constants";

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

const ActivityRow = ({
  dot,
  title,
  status,
  statusColor,
  delay,
  totalFrame,
  pulse,
}) => {
  const opacity = interpolate(totalFrame, [delay, delay + 12], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const x = interpolate(totalFrame, [delay, delay + 12], [16, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "9px 21px",
        opacity,
        transform: `translateX(${x}px)`,
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 13 }}>
        <StatusDot color={dot} size={9} pulse={pulse} />
        <span style={{ fontSize: 17.5, color: COLORS.text, fontWeight: 400 }}>
          {title}
        </span>
      </div>
      <span
        style={{
          fontSize: 15,
          color: statusColor,
          backgroundColor: `${statusColor}1a`,
          padding: "3px 13px",
          borderRadius: 13,
          fontWeight: 500,
        }}
      >
        {status}
      </span>
    </div>
  );
};

const FeedHeader = ({ title, count, delay, totalFrame }) => {
  const opacity = interpolate(totalFrame, [delay, delay + 10], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "13px 21px 4px",
        opacity,
      }}
    >
      <span
        style={{
          fontSize: 13.5,
          fontWeight: 600,
          color: COLORS.textTertiary,
          textTransform: "uppercase",
          letterSpacing: "0.08em",
        }}
      >
        {title}
      </span>
      <span style={{ fontSize: 13.5, color: COLORS.textTertiary }}>
        {count}
      </span>
    </div>
  );
};

// Cortado glass silhouette
const TrayIcon = () => (
  <svg width="21" height="21" viewBox="0 0 88 88">
    <defs>
      <mask id="tray-layer">
        <rect width="88" height="88" fill="white" />
        <line
          x1="23"
          y1="30"
          x2="65"
          y2="30"
          stroke="black"
          strokeWidth="4"
          strokeLinecap="round"
        />
      </mask>
    </defs>
    <path
      mask="url(#tray-layer)"
      d="M18 22 Q18 16,26 16 L62 16 Q70 16,70 22 L64 68 Q62 78,50 78 L38 78 Q26 78,24 68 Z"
      fill="rgba(255,255,255,0.9)"
    />
  </svg>
);

// Mouse cursor
const Cursor = ({ x, y, opacity }) => (
  <svg
    width="26"
    height="31"
    viewBox="0 0 20 24"
    style={{
      position: "absolute",
      left: x,
      top: y,
      opacity,
      filter: "drop-shadow(0 2.6px 5.2px rgba(0,0,0,0.5))",
      zIndex: 100,
      pointerEvents: "none",
    }}
  >
    <path
      d="M2 1 L2 18 L6.5 13.5 L11 21 L13.5 19.5 L9 12 L15 12 Z"
      fill="white"
      stroke="black"
      strokeWidth="1.2"
      strokeLinejoin="round"
    />
  </svg>
);

// --- Layout constants ---
// Viewport: 1920x1080. Menubar: 1200px wide, centered → left=360, right=1560.
// Tray icon is the left-most item in the right-side group, approx x=1340.
// Dropdown: 403px wide, centered under icon → left = 1340 - 201.5 = 1138.5.
const DROPDOWN_LEFT = 1138.5;
const DROPDOWN_TOP = 98;
const DROPDOWN_WIDTH = 403;
const DROPDOWN_HEIGHT = 442; // approx rendered height

// The point we zoom into (center of dropdown)
const FOCUS_X = DROPDOWN_LEFT + DROPDOWN_WIDTH / 2; // ~1340
const FOCUS_Y = DROPDOWN_TOP + DROPDOWN_HEIGHT / 2; // ~319

// Screen center
const SCREEN_CX = 960;
const SCREEN_CY = 540;

// --- Main scene ---

export const MenubarDemo = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Scene fade-in/out
  const fadeIn = interpolate(frame, [0, 15], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const fadeOut = interpolate(frame, [225, 240], [1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Menubar slides in
  const menubarY = interpolate(frame, [0, 22], [-47, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Dropdown opens (from frame 30)
  const dropdownProgress = spring({
    frame: frame - 30,
    fps,
    config: { damping: 15, mass: 0.6 },
  });
  const dropdownOpacity = interpolate(dropdownProgress, [0, 0.3], [0, 1], {
    extrapolateRight: "clamp",
  });
  const dropdownScale = interpolate(dropdownProgress, [0, 1], [0.96, 1]);
  const dropdownLocalY = interpolate(dropdownProgress, [0, 1], [-10, 0]);

  // Stagger frame for dropdown items
  const dropFrame = Math.max(0, frame - 40);

  // --- Mouse cursor (frame 80-105) ---
  const cursorOpacity = interpolate(frame, [80, 88, 200, 215], [0, 1, 1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const cursorX = interpolate(frame, [80, 100], [960, FOCUS_X - 10], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.quad),
  });
  const cursorY = interpolate(frame, [80, 100], [450, FOCUS_Y + 30], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.quad),
  });

  // --- Zoom (frame 98-118, fast ~0.7s with ease-out) ---
  // Math: with transformOrigin at (0,0), scale(S) moves point (x,y) to (x*S, y*S).
  // To place FOCUS at a target screen position, we add translate AFTER scale:
  //   translate = (target - FOCUS * S)
  // At S=1, target=FOCUS → translate=(0,0). At S=max, target=SCREEN_CENTER.
  const zoomProgress = interpolate(frame, [98, 118], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.quad),
  });
  const zoomScale = interpolate(zoomProgress, [0, 1], [1, 2.2]);
  const targetX = interpolate(zoomProgress, [0, 1], [FOCUS_X, SCREEN_CX]);
  const targetY = interpolate(zoomProgress, [0, 1], [FOCUS_Y, SCREEN_CY]);
  const zoomTx = targetX - FOCUS_X * zoomScale;
  const zoomTy = targetY - FOCUS_Y * zoomScale;

  return (
    <AbsoluteFill
      style={{
        fontFamily: FONT,
        opacity: fadeIn * fadeOut,
      }}
    >
      {/* Zoomable container — origin at (0,0), translate computed to keep focus point centered */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          transformOrigin: "0 0",
          transform: `translate(${zoomTx}px, ${zoomTy}px) scale(${zoomScale})`,
        }}
      >
        {/* macOS Menubar */}
        <div
          style={{
            position: "absolute",
            top: 60,
            left: 360,
            width: 1200,
            height: 39,
            backgroundColor: COLORS.menubar,
            borderRadius: 10,
            display: "flex",
            alignItems: "center",
            justifyContent: "flex-end",
            paddingRight: 21,
            paddingLeft: 21,
            gap: 18,
            transform: `translateY(${menubarY}px)`,
            border: `1.3px solid ${COLORS.border}`,
          }}
        >
          {/* Left side */}
          <div
            style={{
              flex: 1,
              display: "flex",
              gap: 21,
              alignItems: "center",
            }}
          >
            <span style={{ fontSize: 19.5, color: COLORS.textSecondary }}>
              {"\uF8FF"}
            </span>
            <span
              style={{
                fontSize: 16.9,
                color: COLORS.textSecondary,
                fontWeight: 500,
              }}
            >
              Finder
            </span>
            <span style={{ fontSize: 16.9, color: COLORS.textTertiary }}>
              File
            </span>
            <span style={{ fontSize: 16.9, color: COLORS.textTertiary }}>
              Edit
            </span>
            <span style={{ fontSize: 16.9, color: COLORS.textTertiary }}>
              View
            </span>
          </div>

          {/* Right side — cortado first, then system icons */}
          <div style={{ display: "flex", alignItems: "center", gap: 15.6 }}>
            {/* Cortado tray icon + red dot */}
            <div
              style={{
                position: "relative",
                display: "flex",
                alignItems: "center",
                marginRight: 5,
              }}
            >
              <TrayIcon />
              <div style={{ position: "absolute", top: -4, right: -6.5 }}>
                <StatusDot color={COLORS.statusRed} size={8} />
              </div>
            </div>

            <span style={{ fontSize: 15.6, color: COLORS.textTertiary }}>
              Wi-Fi
            </span>
            <span style={{ fontSize: 15.6, color: COLORS.textTertiary }}>
              100%
            </span>
            <span
              style={{
                fontSize: 15.6,
                color: COLORS.textSecondary,
                fontWeight: 500,
              }}
            >
              9:41 AM
            </span>
          </div>
        </div>

        {/* Tray Dropdown — centered under tray icon */}
        <div
          style={{
            position: "absolute",
            top: DROPDOWN_TOP,
            left: DROPDOWN_LEFT,
            width: DROPDOWN_WIDTH,
            backgroundColor: COLORS.frosted,
            borderRadius: 15.6,
            border: `1.3px solid ${COLORS.borderLight}`,
            boxShadow: "0 26px 78px rgba(0,0,0,0.5)",
            overflow: "hidden",
            opacity: dropdownOpacity,
            transform: `scale(${dropdownScale}) translateY(${dropdownLocalY}px)`,
            transformOrigin: "top center",
          }}
        >
          <FeedHeader
            title="GitHub PRs"
            count="3"
            delay={0}
            totalFrame={dropFrame}
          />
          <ActivityRow
            dot={COLORS.statusGreen}
            title="feat: add dark mode"
            status="approved"
            statusColor={COLORS.statusGreen}
            delay={5}
            totalFrame={dropFrame}
          />
          <ActivityRow
            dot={COLORS.statusGray}
            title="fix: memory leak"
            status="draft"
            statusColor={COLORS.statusGray}
            delay={10}
            totalFrame={dropFrame}
          />
          <ActivityRow
            dot={COLORS.statusGray}
            title="docs: update readme"
            status="draft"
            statusColor={COLORS.statusGray}
            delay={15}
            totalFrame={dropFrame}
          />

          <div
            style={{
              height: 1.3,
              backgroundColor: COLORS.border,
              margin: "5px 16px",
            }}
          />

          <FeedHeader
            title="CI Runs"
            count="2"
            delay={20}
            totalFrame={dropFrame}
          />
          <ActivityRow
            dot={COLORS.statusGray}
            title="main -- deploy"
            status="passing"
            statusColor={COLORS.statusGray}
            delay={25}
            totalFrame={dropFrame}
          />
          <ActivityRow
            dot={COLORS.statusBlue}
            title="feat/auth -- test"
            status="running"
            statusColor={COLORS.statusBlue}
            delay={30}
            totalFrame={dropFrame}
            pulse
          />

          <div
            style={{
              height: 1.3,
              backgroundColor: COLORS.border,
              margin: "5px 16px",
            }}
          />

          <FeedHeader
            title="HTTP Health"
            count="2"
            delay={35}
            totalFrame={dropFrame}
          />
          <ActivityRow
            dot={COLORS.statusGray}
            title="api.example.com"
            status="healthy"
            statusColor={COLORS.statusGray}
            delay={40}
            totalFrame={dropFrame}
          />
          <ActivityRow
            dot={COLORS.statusRed}
            title="staging.example.com"
            status="degraded"
            statusColor={COLORS.statusRed}
            delay={45}
            totalFrame={dropFrame}
          />

          <div style={{ height: 10 }} />
        </div>

        {/* Mouse cursor (inside zoom container so it tracks with content) */}
        <Cursor x={cursorX} y={cursorY} opacity={cursorOpacity} />
      </div>

      {/* Scene subtitle — shown before zoom */}
      <div
        style={{
          position: "absolute",
          top: "50%",
          left: "50%",
          transform: "translate(-50%, -50%)",
          textAlign: "center",
          opacity: interpolate(frame, [0, 10, 90, 98], [0, 1, 1, 0], {
            extrapolateLeft: "clamp",
            extrapolateRight: "clamp",
          }),
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
          Everything at a glance.
        </div>
      </div>
    </AbsoluteFill>
  );
};
