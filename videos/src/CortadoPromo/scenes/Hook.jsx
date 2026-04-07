import React from "react";
import {
  useCurrentFrame,
  useVideoConfig,
  interpolate,
  spring,
  AbsoluteFill,
  Img,
  staticFile,
} from "remotion";
import { COLORS, FONT, FONT_MONO } from "../constants";

// --- Platform icons (simple, recognizable) ---

const SlackIcon = () => (
  <div
    style={{
      width: 42,
      height: 42,
      borderRadius: 10,
      backgroundColor: "#4A154B",
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
      flexShrink: 0,
    }}
  >
    <svg width="22" height="22" viewBox="0 0 24 24" fill="none">
      <path
        d="M5.5 9.5a2 2 0 1 1 0-4h2v2a2 2 0 0 1-2 2zm4 0a2 2 0 0 1-2-2V3.5a2 2 0 1 1 4 0v4a2 2 0 0 1-2 2z"
        fill="#E01E5A"
      />
      <path
        d="M14.5 5.5a2 2 0 1 1 4 0v2h-2a2 2 0 0 1-2-2zm0 4a2 2 0 0 1 2-2h4a2 2 0 1 1 0 4h-4a2 2 0 0 1-2-2z"
        fill="#36C5F0"
      />
      <path
        d="M18.5 14.5a2 2 0 1 1 0 4h-2v-2a2 2 0 0 1 2-2zm-4 0a2 2 0 0 1 2 2v4a2 2 0 1 1-4 0v-4a2 2 0 0 1 2-2z"
        fill="#2EB67D"
      />
      <path
        d="M9.5 18.5a2 2 0 1 1-4 0v-2h2a2 2 0 0 1 2 2zm0-4a2 2 0 0 1-2 2H3.5a2 2 0 1 1 0-4h4a2 2 0 0 1 2 2z"
        fill="#ECB22E"
      />
    </svg>
  </div>
);

const GitHubIcon = () => (
  <div
    style={{
      width: 42,
      height: 42,
      borderRadius: 10,
      backgroundColor: "#24292e",
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
      flexShrink: 0,
    }}
  >
    <svg width="24" height="24" viewBox="0 0 24 24" fill="white">
      <path d="M12 2C6.477 2 2 6.477 2 12c0 4.42 2.865 8.166 6.839 9.489.5.092.682-.217.682-.482 0-.237-.009-.866-.013-1.7-2.782.604-3.369-1.34-3.369-1.34-.454-1.156-1.11-1.463-1.11-1.463-.908-.62.069-.608.069-.608 1.003.07 1.531 1.03 1.531 1.03.892 1.529 2.341 1.087 2.91.831.092-.646.35-1.086.636-1.336-2.22-.253-4.555-1.11-4.555-4.943 0-1.091.39-1.984 1.029-2.683-.103-.253-.446-1.27.098-2.647 0 0 .84-.269 2.75 1.025A9.578 9.578 0 0112 6.836c.85.004 1.705.115 2.504.337 1.909-1.294 2.747-1.025 2.747-1.025.546 1.377.203 2.394.1 2.647.64.699 1.028 1.592 1.028 2.683 0 3.842-2.339 4.687-4.566 4.935.359.309.678.919.678 1.852 0 1.336-.012 2.415-.012 2.743 0 .267.18.578.688.48C19.138 20.161 22 16.416 22 12c0-5.523-4.477-10-10-10z" />
    </svg>
  </div>
);

const WhatsAppIcon = () => (
  <div
    style={{
      width: 42,
      height: 42,
      borderRadius: 10,
      backgroundColor: "#25D366",
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
      flexShrink: 0,
    }}
  >
    <svg width="24" height="24" viewBox="0 0 24 24" fill="white">
      <path d="M17.472 14.382c-.297-.149-1.758-.867-2.03-.967-.273-.099-.471-.148-.67.15-.197.297-.767.966-.94 1.164-.173.199-.347.223-.644.075-.297-.15-1.255-.463-2.39-1.475-.883-.788-1.48-1.761-1.653-2.059-.173-.297-.018-.458.13-.606.134-.133.298-.347.446-.52.149-.174.198-.298.298-.497.099-.198.05-.371-.025-.52-.075-.149-.669-1.612-.916-2.207-.242-.579-.487-.5-.669-.51-.173-.008-.371-.01-.57-.01-.198 0-.52.074-.792.372-.272.297-1.04 1.016-1.04 2.479 0 1.462 1.065 2.875 1.213 3.074.149.198 2.096 3.2 5.077 4.487.709.306 1.262.489 1.694.625.712.227 1.36.195 1.871.118.571-.085 1.758-.719 2.006-1.413.248-.694.248-1.289.173-1.413-.074-.124-.272-.198-.57-.347m-5.421 7.403h-.004a9.87 9.87 0 01-5.031-1.378l-.361-.214-3.741.982.998-3.648-.235-.374a9.86 9.86 0 01-1.51-5.26c.001-5.45 4.436-9.884 9.888-9.884 2.64 0 5.122 1.03 6.988 2.898a9.825 9.825 0 012.893 6.994c-.003 5.45-4.437 9.884-9.885 9.884m8.413-18.297A11.815 11.815 0 0012.05 0C5.495 0 .16 5.335.157 11.892c0 2.096.547 4.142 1.588 5.945L.057 24l6.305-1.654a11.882 11.882 0 005.683 1.448h.005c6.554 0 11.89-5.335 11.893-11.893a11.821 11.821 0 00-3.48-8.413Z" />
    </svg>
  </div>
);

const MailIcon = () => (
  <div
    style={{
      width: 42,
      height: 42,
      borderRadius: 10,
      backgroundColor: "#1A8CFF",
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
      flexShrink: 0,
    }}
  >
    <svg
      width="22"
      height="22"
      viewBox="0 0 24 24"
      fill="none"
      stroke="white"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <rect x="2" y="4" width="20" height="16" rx="2" />
      <polyline points="22,4 12,13 2,4" />
    </svg>
  </div>
);

const LinearIcon = () => (
  <div
    style={{
      width: 42,
      height: 42,
      borderRadius: 10,
      backgroundColor: "#5E6AD2",
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
      flexShrink: 0,
    }}
  >
    <svg width="22" height="22" viewBox="0 0 24 24" fill="white">
      <path d="M2.886 4.18A11.982 11.982 0 0 1 11.99 0C18.624 0 24 5.376 24 12.009c0 3.64-1.62 6.903-4.18 9.105L2.887 4.18ZM1.817 5.626l16.556 16.556c-.524.33-1.075.62-1.65.866L.951 7.277c.247-.575.537-1.126.866-1.65ZM.322 9.163l14.515 14.515c-.71.172-1.443.282-2.195.322L0 11.358a12 12 0 0 1 .322-2.195Zm-.17 4.862 9.823 9.824a12.02 12.02 0 0 1-9.824-9.824Z" />
    </svg>
  </div>
);

// --- Notification data ---

const NOTIFICATIONS = [
  {
    icon: <SlackIcon />,
    app: "Slack",
    sender: "Sarah",
    message: "Hey, the deploy to prod just failed. Can you check?",
    time: "now",
    x: -300,
    y: -280,
    rotate: -2.5,
    scatterX: -780,
    scatterY: -585,
    delay: 68,
  },
  {
    icon: <GitHubIcon />,
    app: "GitHub",
    sender: "notifications",
    message: "Review requested: fix: memory leak in connection pool #380",
    time: "2m ago",
    x: 220,
    y: -200,
    rotate: 1.8,
    scatterX: 715,
    scatterY: -520,
    delay: 77,
  },
  {
    icon: <SlackIcon />,
    app: "Slack",
    sender: "Mike",
    message: "Left some comments on your auth PR, take a look when you can",
    time: "5m ago",
    x: -180,
    y: -60,
    rotate: -1.2,
    scatterX: -650,
    scatterY: 390,
    delay: 86,
  },
  {
    icon: <WhatsAppIcon />,
    app: "WhatsApp",
    sender: "Alex",
    message: "Are you seeing the alerts from the monitoring dashboard?",
    time: "8m ago",
    x: 250,
    y: 30,
    rotate: 2.5,
    scatterX: 780,
    scatterY: 520,
    delay: 95,
  },
  {
    icon: <LinearIcon />,
    app: "Linear",
    sender: "COR-284",
    message: "Bug: Dashboard shows stale data after config reload",
    time: "12m ago",
    x: -220,
    y: 140,
    rotate: -1.8,
    scatterX: -715,
    scatterY: 585,
    delay: 104,
  },
];

// --- Notification card (macOS-style banner) ---

const NotificationCard = ({
  icon,
  app,
  sender,
  message,
  time,
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

  // Scatter: blown outward from center (frames 138-156)
  const scatterProgress = interpolate(frame, [138, 156], [0, 1], {
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
        width: 460,
        backgroundColor: "rgba(50, 50, 55, 0.95)",
        borderRadius: 18,
        padding: "16px 20px",
        display: "flex",
        alignItems: "flex-start",
        gap: 14,
        boxShadow: "0 10px 50px rgba(0,0,0,0.5)",
        border: "1px solid rgba(255,255,255,0.08)",
      }}
    >
      {/* App icon */}
      {icon}

      {/* Content */}
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          gap: 3,
          flex: 1,
          overflow: "hidden",
        }}
      >
        {/* Header: app name + time */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
          }}
        >
          <span
            style={{
              fontSize: 14,
              fontWeight: 600,
              color: "#eeeeee",
              textTransform: "uppercase",
              letterSpacing: "0.02em",
            }}
          >
            {app}
          </span>
          <span style={{ fontSize: 12, color: "#808080" }}>{time}</span>
        </div>
        {/* Sender */}
        <span style={{ fontSize: 16, fontWeight: 600, color: "#e0e0e0" }}>
          {sender}
        </span>
        {/* Message */}
        <span
          style={{
            fontSize: 15,
            color: "#b0b0b0",
            lineHeight: 1.35,
            whiteSpace: "nowrap",
            overflow: "hidden",
            textOverflow: "ellipsis",
          }}
        >
          {message}
        </span>
      </div>
    </div>
  );
};

// --- Main scene ---

export const Hook = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Logo reveal — emerges as it punches notifications outward (frame 138)
  const logoProgress = spring({
    frame: frame - 138,
    fps,
    config: { damping: 10, mass: 0.5, stiffness: 200 },
  });
  const logoScale = interpolate(logoProgress, [0, 1], [0.3, 1]);
  const logoOpacity = interpolate(frame, [138, 145], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Convergence flash — shockwave as logo punches through
  const flashOpacity = interpolate(frame, [138, 142, 155], [0, 0.7, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const flashScale = interpolate(frame, [138, 158], [0.3, 3], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const nameOpacity = interpolate(frame, [152, 166], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const nameY = interpolate(frame, [152, 166], [23, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const taglineOpacity = interpolate(frame, [164, 178], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const taglineY = interpolate(frame, [164, 178], [16, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Scene fade-out (extended to let logo linger)
  const sceneOpacity = interpolate(frame, [243, 255], [1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Phrase — alone on screen (frames 0-60), before notifications and music
  // Line 1: "Your attention is priceless."
  const line1Opacity = interpolate(frame, [0, 10, 48, 60], [0, 1, 1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  // Line 2: "Stop wasting it between tabs." — appears after a pause
  const line2Opacity = interpolate(frame, [18, 28, 48, 60], [0, 1, 1, 0], {
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

      {/* Opening phrase */}
      <div
        style={{
          position: "absolute",
          top: "50%",
          left: "50%",
          transform: "translate(-50%, -50%)",
          zIndex: 10,
          textAlign: "center",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: 12,
        }}
      >
        <div
          style={{
            fontSize: 42,
            fontWeight: 500,
            color: COLORS.textSecondary,
            letterSpacing: "-0.01em",
            opacity: line1Opacity,
          }}
        >
          Your{" "}
          <span style={{ color: COLORS.accent }}>attention is priceless.</span>
        </div>
        <div
          style={{
            fontSize: 42,
            fontWeight: 500,
            color: COLORS.textSecondary,
            letterSpacing: "-0.01em",
            opacity: line2Opacity,
          }}
        >
          Stop wasting it between tabs.
        </div>
      </div>

      {/* Convergence flash */}
      <div
        style={{
          position: "absolute",
          width: 200,
          height: 200,
          borderRadius: "50%",
          background: `radial-gradient(circle, ${COLORS.accent} 0%, transparent 70%)`,
          opacity: flashOpacity,
          transform: `scale(${flashScale})`,
          pointerEvents: "none",
        }}
      />

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
          Attention, spent wisely.
        </div>
      </div>
    </AbsoluteFill>
  );
};
