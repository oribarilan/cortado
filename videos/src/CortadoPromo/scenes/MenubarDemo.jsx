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
  flash = 0,
}) => {
  const opacity = interpolate(totalFrame, [delay, delay + 12], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const x = interpolate(totalFrame, [delay, delay + 12], [16, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const flashBg =
    flash > 0 ? `rgba(92, 184, 122, ${flash * 0.2})` : "transparent";

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "9px 21px",
        opacity,
        transform: `translateX(${x}px)`,
        backgroundColor: flashBg,
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

// The point we zoom into (the approved PR row)
// PR row is first row: dropdown top + header height + half row height ≈ 98 + 35 + 18 = 151
const PR_ROW_X = DROPDOWN_LEFT + DROPDOWN_WIDTH / 2; // center of dropdown
const PR_ROW_Y = DROPDOWN_TOP + 53; // center of first PR row

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
  const fadeOut = interpolate(frame, [210, 225], [1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Menubar fades in
  const menubarOpacity = interpolate(frame, [0, 15], [0, 1], {
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

  // --- Mouse cursor — two phases ---
  // Phase 1 (80-100): move to approved PR row in tray
  // Phase 2 (140-165): move to merge button in GitHub mockup
  const cursorOpacity = interpolate(frame, [80, 88, 200, 215], [0, 1, 1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  // Phase 1: toward PR row (in zoomable coords, so won't match after zoom)
  const phase1X = interpolate(frame, [80, 100], [960, PR_ROW_X - 10], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.quad),
  });
  const phase1Y = interpolate(frame, [80, 100], [450, PR_ROW_Y + 10], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.quad),
  });
  // Phase 2: move to merge button — faster, curved arc
  const cursorPhase = frame < 120 ? 1 : 2;
  const phase2X = interpolate(frame, [130, 148], [800, 660], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.inOut(Easing.quad),
  });
  const phase2Y = interpolate(frame, [130, 148], [350, 620], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.cubic),
  });

  // Click flash on PR row (frame 100)
  const rowClickFlash = interpolate(frame, [100, 102, 108], [0, 1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Dropdown + menubar fade out after click (frame 108-120)
  const trayFadeOut = interpolate(frame, [108, 120], [1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // --- Zoom into PR row (frame 80-100, before click at 100) ---
  const zoomProgress = interpolate(frame, [80, 100], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.quad),
  });
  const zoomScale = interpolate(zoomProgress, [0, 1], [1, 2.2]);
  const targetX = interpolate(zoomProgress, [0, 1], [PR_ROW_X, SCREEN_CX]);
  const targetY = interpolate(zoomProgress, [0, 1], [PR_ROW_Y, SCREEN_CY]);
  const zoomTx = targetX - PR_ROW_X * zoomScale;
  const zoomTy = targetY - PR_ROW_Y * zoomScale;

  // --- GitHub merge mockup (frame 125+) ---
  const ghProgress = spring({
    frame: frame - 125,
    fps,
    config: { damping: 14, mass: 0.7 },
  });
  const ghOpacity = interpolate(ghProgress, [0, 0.3], [0, 1], {
    extrapolateRight: "clamp",
  });
  const ghScale = interpolate(ghProgress, [0, 1], [0.93, 1]);
  const ghY = interpolate(ghProgress, [0, 1], [31, 0]);

  // Merge button click (frame 170)
  const mergeClick = interpolate(frame, [155, 157, 163], [0, 1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const merged = frame >= 160;

  // Zoom into merge button area (frame 135-155, before click at 155)
  const mergeZoom = interpolate(frame, [135, 155], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.quad),
  });
  const mergeZoomScale = interpolate(mergeZoom, [0, 1], [1, 1.8]);
  // Merge button is at bottom-left of centered card (~x=640, y=640)
  const MERGE_BTN_X = 640;
  const MERGE_BTN_Y = 640;
  const mzTargetX = interpolate(mergeZoom, [0, 1], [MERGE_BTN_X, SCREEN_CX]);
  const mzTargetY = interpolate(mergeZoom, [0, 1], [MERGE_BTN_Y, SCREEN_CY]);
  const mzTx = mzTargetX - MERGE_BTN_X * mergeZoomScale;
  const mzTy = mzTargetY - MERGE_BTN_Y * mergeZoomScale;

  return (
    <AbsoluteFill
      style={{
        fontFamily: FONT,
        opacity: fadeIn * fadeOut,
      }}
    >
      {/* Zoomable container */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          transformOrigin: "0 0",
          transform: `translate(${zoomTx}px, ${zoomTy}px) scale(${zoomScale})`,
          opacity: trayFadeOut,
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
            opacity: menubarOpacity,
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
            count="2"
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
            flash={rowClickFlash}
          />
          <ActivityRow
            dot={COLORS.statusGray}
            title="fix: memory leak"
            status="draft"
            statusColor={COLORS.statusGray}
            delay={10}
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
            title="GitHub Actions"
            count="2"
            delay={15}
            totalFrame={dropFrame}
          />
          <ActivityRow
            dot={COLORS.statusGray}
            title="deploy (main)"
            status="passing"
            statusColor={COLORS.statusGray}
            delay={20}
            totalFrame={dropFrame}
          />
          <ActivityRow
            dot={COLORS.statusBlue}
            title="test (feat/auth)"
            status="running"
            statusColor={COLORS.statusBlue}
            delay={25}
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
            delay={30}
            totalFrame={dropFrame}
          />
          <ActivityRow
            dot={COLORS.statusGray}
            title="api.example.com"
            status="healthy"
            statusColor={COLORS.statusGray}
            delay={35}
            totalFrame={dropFrame}
          />
          <ActivityRow
            dot={COLORS.statusRed}
            title="staging.example.com"
            status="degraded"
            statusColor={COLORS.statusRed}
            delay={40}
            totalFrame={dropFrame}
          />

          <div style={{ height: 10 }} />
        </div>

        {/* Mouse cursor — phase 1: inside zoom container */}
        <Cursor
          x={phase1X}
          y={phase1Y}
          opacity={cursorPhase === 1 ? cursorOpacity : 0}
        />
      </div>

      {/* Merge zoom container */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          transformOrigin: "0 0",
          transform: `translate(${mzTx}px, ${mzTy}px) scale(${mergeZoomScale})`,
        }}
      >
        {/* Mouse cursor — phase 2 */}
        <Cursor
          x={phase2X}
          y={phase2Y}
          opacity={cursorPhase === 2 ? cursorOpacity : 0}
        />

        {/* GitHub merge mockup */}
        <div
          style={{
            position: "absolute",
            top: 319,
            right: 1920 - 1138.5 + 30,
            transform: "translateY(-50%)",
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

        {/* GitHub merge mockup */}
        <div
          style={{
            position: "absolute",
            top: "50%",
            left: "50%",
            transform: `translate(-50%, -50%) scale(${ghScale}) translateY(${ghY}px)`,
            opacity: ghOpacity,
            width: 700,
            backgroundColor: "#0d1117",
            borderRadius: 16,
            border: "1px solid #30363d",
            boxShadow: "0 24px 80px rgba(0,0,0,0.55)",
            overflow: "hidden",
            fontFamily: FONT,
          }}
        >
          {/* PR header */}
          <div
            style={{
              padding: "24px 28px 16px",
              borderBottom: "1px solid #30363d",
            }}
          >
            <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
              <svg width="20" height="20" viewBox="0 0 16 16" fill="#3fb950">
                <path d="M1.5 3.25a2.25 2.25 0 113 2.122v5.256a2.251 2.251 0 11-1.5 0V5.372A2.25 2.25 0 011.5 3.25zm5.677-.177L9.573.677A.25.25 0 0110 .854V2.5h1A2.5 2.5 0 0113.5 5v5.628a2.251 2.251 0 11-1.5 0V5a1 1 0 00-1-1h-1v1.646a.25.25 0 01-.427.177L7.177 3.427a.25.25 0 010-.354z" />
              </svg>
              <span style={{ fontSize: 22, fontWeight: 600, color: "#e6edf3" }}>
                feat: add dark mode
              </span>
              <span style={{ fontSize: 16, color: "#7d8590" }}>#412</span>
            </div>
            <div style={{ fontSize: 14, color: "#7d8590", marginTop: 8 }}>
              <span style={{ color: "#3fb950" }}>Open</span> &middot; 2
              approvals &middot; All checks passed
            </div>
          </div>

          {/* Merge area */}
          <div
            style={{
              padding: "20px 28px 24px",
              display: "flex",
              flexDirection: "column",
              gap: 14,
            }}
          >
            {/* Status checks */}
            <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
              <svg width="16" height="16" viewBox="0 0 16 16" fill="#3fb950">
                <path d="M13.78 4.22a.75.75 0 010 1.06l-7.25 7.25a.75.75 0 01-1.06 0L2.22 9.28a.751.751 0 01.018-1.042.751.751 0 011.042-.018L6 10.94l6.72-6.72a.75.75 0 011.06 0z" />
              </svg>
              <span style={{ fontSize: 14, color: "#e6edf3" }}>
                All checks have passed
              </span>
            </div>
            <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
              <svg width="16" height="16" viewBox="0 0 16 16" fill="#3fb950">
                <path d="M13.78 4.22a.75.75 0 010 1.06l-7.25 7.25a.75.75 0 01-1.06 0L2.22 9.28a.751.751 0 01.018-1.042.751.751 0 011.042-.018L6 10.94l6.72-6.72a.75.75 0 011.06 0z" />
              </svg>
              <span style={{ fontSize: 14, color: "#e6edf3" }}>
                2 approving reviews
              </span>
            </div>

            {/* Merge button */}
            <div style={{ marginTop: 8 }}>
              <div
                style={{
                  display: "inline-flex",
                  alignItems: "center",
                  gap: 8,
                  padding: "10px 24px",
                  backgroundColor: merged
                    ? "#238636"
                    : mergeClick > 0
                      ? "#2ea043"
                      : "#238636",
                  borderRadius: 8,
                  fontSize: 16,
                  fontWeight: 600,
                  color: "white",
                  transform: `scale(${mergeClick > 0 ? 0.95 : 1})`,
                  boxShadow:
                    mergeClick > 0 ? "0 0 20px rgba(35, 134, 54, 0.5)" : "none",
                }}
              >
                {merged ? (
                  <>
                    <svg
                      width="16"
                      height="16"
                      viewBox="0 0 16 16"
                      fill="white"
                    >
                      <path d="M13.78 4.22a.75.75 0 010 1.06l-7.25 7.25a.75.75 0 01-1.06 0L2.22 9.28a.751.751 0 01.018-1.042.751.751 0 011.042-.018L6 10.94l6.72-6.72a.75.75 0 011.06 0z" />
                    </svg>
                    Merged
                  </>
                ) : (
                  "Merge pull request"
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
    </AbsoluteFill>
  );
};
