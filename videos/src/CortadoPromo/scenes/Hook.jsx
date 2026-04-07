import {
  useCurrentFrame,
  useVideoConfig,
  interpolate,
  spring,
  AbsoluteFill,
  Img,
  staticFile,
} from "remotion";
import { COLORS, FONT } from "../constants";

// --- Notification data ---

const NOTIFICATIONS = [
  {
    color: COLORS.statusGreen,
    line1: "Alex opened PR #412",
    line2: "auth refactor",
    x: -312,
    y: -195,
    rotate: -2.5,
    scatterX: -780,
    scatterY: -585,
    delay: 8,
  },
  {
    color: COLORS.statusRed,
    line1: "CI failed: deploy-prod",
    line2: "main branch",
    x: 208,
    y: -117,
    rotate: 1.8,
    scatterX: 715,
    scatterY: -520,
    delay: 17,
  },
  {
    color: COLORS.statusYellow,
    line1: "2 new comments on your PR",
    line2: "fix: memory leak #380",
    x: -156,
    y: 26,
    rotate: -1.2,
    scatterX: -650,
    scatterY: 390,
    delay: 26,
  },
  {
    color: COLORS.statusRed,
    line1: "OpenCode is asking a question",
    line2: "cortado-backend",
    x: 260,
    y: 117,
    rotate: 2.5,
    scatterX: 780,
    scatterY: 520,
    delay: 35,
  },
  {
    color: COLORS.statusYellow,
    line1: "Review requested on PR #389",
    line2: "docs update",
    x: -234,
    y: 221,
    rotate: -1.8,
    scatterX: -715,
    scatterY: 585,
    delay: 44,
  },
];

// --- Notification card ---

const NotificationCard = ({
  color,
  line1,
  line2,
  x,
  y,
  rotate,
  scatterX,
  scatterY,
  delay,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Appear: spring in from below
  const appearProgress = spring({
    frame: frame - delay,
    fps,
    config: { damping: 13, mass: 0.6 },
  });
  const cardScale = interpolate(appearProgress, [0, 1], [0.6, 1]);
  const cardOpacity = interpolate(appearProgress, [0, 0.4], [0, 1], {
    extrapolateRight: "clamp",
  });
  const dropY = interpolate(appearProgress, [0, 1], [52, 0]);

  // Scatter: fly outward and shrink (frames 78-96)
  const scatterProgress = interpolate(frame, [78, 96], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const sx = interpolate(scatterProgress, [0, 1], [0, scatterX]);
  const sy = interpolate(scatterProgress, [0, 1], [0, scatterY]);
  const scatterOpacity = interpolate(scatterProgress, [0, 0.6, 1], [1, 0.4, 0]);
  const scatterScale = interpolate(scatterProgress, [0, 1], [1, 0.2]);

  return (
    <div
      style={{
        position: "absolute",
        transform: `translate(${x + sx}px, ${y + dropY + sy}px) rotate(${rotate}deg) scale(${cardScale * scatterScale})`,
        opacity: cardOpacity * scatterOpacity,
        width: 500,
        padding: "20px 26px",
        backgroundColor: COLORS.bgRaised,
        borderRadius: 16,
        borderLeft: `5px solid ${color}`,
        boxShadow: "0 14px 54px rgba(0,0,0,0.4)",
      }}
    >
      <div
        style={{
          fontSize: 24,
          fontWeight: 500,
          color: COLORS.text,
          lineHeight: 1.3,
        }}
      >
        {line1}
      </div>
      <div
        style={{
          fontSize: 20,
          color: COLORS.textTertiary,
          marginTop: 5,
        }}
      >
        {line2}
      </div>
    </div>
  );
};

// --- Main scene ---

export const Hook = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Logo reveal (frames 96-150)
  const logoProgress = spring({
    frame: frame - 98,
    fps,
    config: { damping: 12, mass: 0.8 },
  });
  const logoScale = interpolate(logoProgress, [0, 1], [0.5, 1]);
  const logoOpacity = interpolate(frame, [98, 112], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const nameOpacity = interpolate(frame, [110, 124], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const nameY = interpolate(frame, [110, 124], [23, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const taglineOpacity = interpolate(frame, [122, 136], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const taglineY = interpolate(frame, [122, 136], [16, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Scene fade-out (extended to let logo linger)
  const sceneOpacity = interpolate(frame, [183, 195], [1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <AbsoluteFill
      style={{
        justifyContent: "center",
        alignItems: "center",
        fontFamily: FONT,
        opacity: sceneOpacity,
      }}
    >
      {/* Notification chaos */}
      {NOTIFICATIONS.map((notif, i) => (
        <NotificationCard key={i} {...notif} />
      ))}

      {/* Logo reveal */}
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          position: "absolute",
        }}
      >
        <Img
          src={staticFile("cortado-icon.png")}
          style={{
            width: 182,
            height: 182,
            borderRadius: 36,
            opacity: logoOpacity,
            transform: `scale(${logoScale})`,
          }}
        />
        <div
          style={{
            fontSize: 94,
            fontWeight: 600,
            color: COLORS.text,
            letterSpacing: "-0.03em",
            marginTop: 36,
            opacity: nameOpacity,
            transform: `translateY(${nameY}px)`,
          }}
        >
          cortado
        </div>
        <div
          style={{
            fontSize: 34,
            fontWeight: 400,
            color: COLORS.textSecondary,
            letterSpacing: "0.01em",
            marginTop: 10,
            opacity: taglineOpacity,
            transform: `translateY(${taglineY}px)`,
          }}
        >
          A feed for the busy builder
        </div>
      </div>
    </AbsoluteFill>
  );
};
