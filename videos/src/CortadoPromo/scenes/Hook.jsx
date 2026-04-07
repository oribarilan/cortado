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
      width: 52,
      height: 52,
      borderRadius: 12,
      backgroundColor: "#4A154B",
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
      flexShrink: 0,
    }}
  >
    <svg width="28" height="28" viewBox="0 0 2447.6 2452.5">
      <g clipRule="evenodd" fillRule="evenodd">
        <path
          d="m897.4 0c-135.3.1-244.8 109.9-244.7 245.2-.1 135.3 109.5 245.1 244.8 245.2h244.8v-245.1c.1-135.3-109.5-245.1-244.9-245.3.1 0 .1 0 0 0m0 654h-652.6c-135.3.1-244.9 109.9-244.8 245.2-.2 135.3 109.4 245.1 244.7 245.3h652.7c135.3-.1 244.9-109.9 244.8-245.2.1-135.4-109.5-245.2-244.8-245.3z"
          fill="#36c5f0"
        />
        <path
          d="m2447.6 899.2c.1-135.3-109.5-245.1-244.8-245.2-135.3.1-244.9 109.9-244.8 245.2v245.3h244.8c135.3-.1 244.9-109.9 244.8-245.3zm-652.7 0v-654c.1-135.2-109.4-245-244.7-245.2-135.3.1-244.9 109.9-244.8 245.2v654c-.2 135.3 109.4 245.1 244.7 245.3 135.3-.1 244.9-109.9 244.8-245.3z"
          fill="#2eb67d"
        />
        <path
          d="m1550.1 2452.5c135.3-.1 244.9-109.9 244.8-245.2.1-135.3-109.5-245.1-244.8-245.2h-244.8v245.2c-.1 135.2 109.5 245 244.8 245.2zm0-654.1h652.7c135.3-.1 244.9-109.9 244.8-245.2.2-135.3-109.4-245.1-244.7-245.3h-652.7c-135.3.1-244.9 109.9-244.8 245.2-.1 135.4 109.4 245.2 244.7 245.3z"
          fill="#ecb22e"
        />
        <path
          d="m0 1553.2c-.1 135.3 109.5 245.1 244.8 245.2 135.3-.1 244.9-109.9 244.8-245.2v-245.2h-244.8c-135.3.1-244.9 109.9-244.8 245.2zm652.7 0v654c-.2 135.3 109.4 245.1 244.7 245.3 135.3-.1 244.9-109.9 244.8-245.2v-653.9c.2-135.3-109.4-245.1-244.7-245.3-135.4 0-244.9 109.8-244.8 245.1 0 0 0 .1 0 0"
          fill="#e01e5a"
        />
      </g>
    </svg>
  </div>
);

const GitHubIcon = () => (
  <div
    style={{
      width: 52,
      height: 52,
      borderRadius: 12,
      backgroundColor: "#24292e",
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
      flexShrink: 0,
    }}
  >
    <svg width="30" height="30" viewBox="0 0 1024 1024" fill="white">
      <path
        fillRule="evenodd"
        clipRule="evenodd"
        d="M8 0C3.58 0 0 3.58 0 8C0 11.54 2.29 14.53 5.47 15.59C5.87 15.66 6.02 15.42 6.02 15.21C6.02 15.02 6.01 14.39 6.01 13.72C4 14.09 3.48 13.23 3.32 12.78C3.23 12.55 2.84 11.84 2.5 11.65C2.22 11.5 1.82 11.13 2.49 11.12C3.12 11.11 3.57 11.7 3.72 11.94C4.44 13.15 5.59 12.81 6.05 12.6C6.12 12.08 6.33 11.73 6.56 11.53C4.78 11.33 2.92 10.64 2.92 7.58C2.92 6.71 3.23 5.99 3.74 5.43C3.66 5.23 3.38 4.41 3.82 3.31C3.82 3.31 4.49 3.1 6.02 4.13C6.66 3.95 7.34 3.86 8.02 3.86C8.7 3.86 9.38 3.95 10.02 4.13C11.55 3.09 12.22 3.31 12.22 3.31C12.66 4.41 12.38 5.23 12.3 5.43C12.81 5.99 13.12 6.7 13.12 7.58C13.12 10.65 11.25 11.33 9.47 11.53C9.76 11.78 10.01 12.26 10.01 13.01C10.01 14.08 10 14.94 10 15.21C10 15.42 10.15 15.67 10.55 15.59C13.71 14.53 16 11.53 16 8C16 3.58 12.42 0 8 0Z"
        transform="scale(64)"
      />
    </svg>
  </div>
);

const WhatsAppIcon = () => (
  <div
    style={{
      width: 52,
      height: 52,
      borderRadius: 12,
      backgroundColor: "#25D366",
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
      flexShrink: 0,
    }}
  >
    <svg width="30" height="30" viewBox="0 0 360 362" fill="white">
      <path
        fillRule="evenodd"
        clipRule="evenodd"
        d="M307.546 52.566C273.709 18.684 228.706.017 180.756 0 81.951 0 1.538 80.404 1.504 179.235c-.017 31.594 8.242 62.432 23.928 89.609L0 361.736l95.024-24.925c26.179 14.285 55.659 21.805 85.655 21.814h.077c98.788 0 179.21-80.413 179.244-179.244.017-47.898-18.608-92.926-52.454-126.807v-.008Zm-126.79 275.788h-.06c-26.73-.008-52.952-7.194-75.831-20.765l-5.44-3.231-56.391 14.791 15.05-54.981-3.542-5.638c-14.912-23.721-22.793-51.139-22.776-79.286.035-82.14 66.867-148.973 149.051-148.973 39.793.017 77.198 15.53 105.328 43.695 28.131 28.157 43.61 65.596 43.593 105.398-.035 82.149-66.867 148.982-148.982 148.982v.008Zm81.719-111.577c-4.478-2.243-26.497-13.073-30.606-14.568-4.108-1.496-7.09-2.243-10.073 2.243-2.982 4.487-11.568 14.577-14.181 17.559-2.613 2.991-5.226 3.361-9.704 1.117-4.477-2.243-18.908-6.97-36.02-22.226-13.313-11.878-22.304-26.54-24.916-31.027-2.613-4.486-.275-6.91 1.959-9.136 2.011-2.011 4.478-5.234 6.721-7.847 2.244-2.613 2.983-4.486 4.478-7.469 1.496-2.991.748-5.603-.369-7.847-1.118-2.243-10.073-24.289-13.812-33.253-3.636-8.732-7.331-7.546-10.073-7.692-2.613-.13-5.595-.155-8.586-.155-2.991 0-7.839 1.118-11.947 5.604-4.108 4.486-15.677 15.324-15.677 37.361s16.047 43.344 18.29 46.335c2.243 2.991 31.585 48.225 76.51 67.632 10.684 4.615 19.029 7.374 25.535 9.437 10.727 3.412 20.49 2.931 28.208 1.779 8.604-1.289 26.498-10.838 30.228-21.298 3.73-10.46 3.73-19.433 2.613-21.298-1.117-1.865-4.108-2.991-8.586-5.234l.008-.017Z"
      />
    </svg>
  </div>
);

const MailIcon = () => (
  <div
    style={{
      width: 52,
      height: 52,
      borderRadius: 12,
      backgroundColor: "#ffffff",
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
      flexShrink: 0,
    }}
  >
    <svg width="30" height="24" viewBox="0 49.4 512 399.42">
      <g fill="none" fillRule="evenodd">
        <g fillRule="nonzero">
          <path
            fill="#4285f4"
            d="M34.91 448.818h81.454V251L0 163.727V413.91c0 19.287 15.622 34.91 34.91 34.91z"
          />
          <path
            fill="#34a853"
            d="M395.636 448.818h81.455c19.287 0 34.909-15.622 34.909-34.909V163.727L395.636 251z"
          />
          <path
            fill="#fbbc04"
            d="M395.636 99.727V251L512 163.727v-46.545c0-43.142-49.25-67.782-83.782-41.891z"
          />
        </g>
        <path
          fill="#ea4335"
          d="M116.364 251V99.727L256 204.455 395.636 99.727V251L256 355.727z"
        />
        <path
          fill="#c5221f"
          fillRule="nonzero"
          d="M0 117.182v46.545L116.364 251V99.727L83.782 75.291C49.25 49.4 0 74.04 0 117.18z"
        />
      </g>
    </svg>
  </div>
);

const LinearIcon = () => (
  <div
    style={{
      width: 52,
      height: 52,
      borderRadius: 12,
      backgroundColor: "#5E6AD2",
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
      flexShrink: 0,
    }}
  >
    <svg width="28" height="28" viewBox="0 0 100 100" fill="white">
      <path d="M1.225 61.523c-.222-.949.908-1.546 1.597-.857l36.512 36.512c.69.69.092 1.82-.857 1.597-18.425-4.323-32.93-18.827-37.252-37.252ZM.002 46.889a.99.99 0 0 0 .29.76L52.35 99.71c.201.2.478.307.76.29 2.37-.149 4.695-.46 6.963-.927.765-.157 1.03-1.096.478-1.648L2.576 39.448c-.552-.551-1.491-.286-1.648.479a50.067 50.067 0 0 0-.926 6.962ZM4.21 29.705a.988.988 0 0 0 .208 1.1l64.776 64.776c.289.29.726.375 1.1.208a49.908 49.908 0 0 0 5.185-2.684.981.981 0 0 0 .183-1.54L8.436 24.336a.981.981 0 0 0-1.541.183 49.896 49.896 0 0 0-2.684 5.185Zm8.448-11.631a.986.986 0 0 1-.045-1.354C21.78 6.46 35.111 0 49.952 0 77.592 0 100 22.407 100 50.048c0 14.84-6.46 28.172-16.72 37.338a.986.986 0 0 1-1.354-.045L12.659 18.074Z" />
    </svg>
  </div>
);

// --- Notification data ---

const NOTIFICATIONS = [
  {
    icon: <SlackIcon />,
    app: "Slack",
    sender: "Mike",
    message: "Left some comments on your auth PR, take a look when you can",
    time: "5m ago",
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
    icon: <MailIcon />,
    app: "Gmail",
    sender: "Sarah",
    message: "Hey team, the deploy to prod just failed. Can you check?",
    time: "now",
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
        width: 575,
        backgroundColor: "rgba(50, 50, 55, 0.95)",
        borderRadius: 22,
        padding: "20px 25px",
        display: "flex",
        alignItems: "flex-start",
        gap: 18,
        boxShadow: "0 12px 62px rgba(0,0,0,0.5)",
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
          gap: 4,
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
              fontSize: 18,
              fontWeight: 600,
              color: "#eeeeee",
              textTransform: "uppercase",
              letterSpacing: "0.02em",
            }}
          >
            {app}
          </span>
          <span style={{ fontSize: 15, color: "#808080" }}>{time}</span>
        </div>
        {/* Sender */}
        <span style={{ fontSize: 20, fontWeight: 600, color: "#e0e0e0" }}>
          {sender}
        </span>
        {/* Message */}
        <span
          style={{
            fontSize: 19,
            color: "#b0b0b0",
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
