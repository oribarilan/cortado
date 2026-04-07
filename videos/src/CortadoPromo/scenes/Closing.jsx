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
import { COLORS, FONT } from "../constants";

// --- Real feed type icons from the app ---

const IconGitHubPR = () => (
  <svg
    width="42"
    height="42"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <circle cx="18" cy="18" r="3" />
    <circle cx="6" cy="6" r="3" />
    <path d="M13 6h3a2 2 0 0 1 2 2v7" />
    <line x1="6" y1="9" x2="6" y2="21" />
  </svg>
);

const IconGitHubActions = () => (
  <svg
    width="42"
    height="42"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <circle cx="12" cy="12" r="10" />
    <polygon points="10 8 16 12 10 16 10 8" />
  </svg>
);

const IconCopilot = () => (
  <svg width="42" height="42" viewBox="0 0 24 24" fill="currentColor">
    <path d="M23.922 16.997C23.061 18.492 18.063 22.02 12 22.02 5.937 22.02.939 18.492.078 16.997A.641.641 0 0 1 0 16.741v-2.869a.883.883 0 0 1 .053-.22c.372-.935 1.347-2.292 2.605-2.656.167-.429.414-1.055.644-1.517a10.098 10.098 0 0 1-.052-1.086c0-1.331.282-2.499 1.132-3.368.397-.406.89-.717 1.474-.952C7.255 2.937 9.248 1.98 11.978 1.98c2.731 0 4.767.957 6.166 2.093.584.235 1.077.546 1.474.952.85.869 1.132 2.037 1.132 3.368 0 .368-.014.733-.052 1.086.23.462.477 1.088.644 1.517 1.258.364 2.233 1.721 2.605 2.656a.841.841 0 0 1 .053.22v2.869a.641.641 0 0 1-.078.256Zm-11.75-5.992h-.344a4.359 4.359 0 0 1-.355.508c-.77.947-1.918 1.492-3.508 1.492-1.725 0-2.989-.359-3.782-1.259a2.137 2.137 0 0 1-.085-.104L4 11.746v6.585c1.435.779 4.514 2.179 8 2.179 3.486 0 6.565-1.4 8-2.179v-6.585l-.098-.104s-.033.045-.085.104c-.793.9-2.057 1.259-3.782 1.259-1.59 0-2.738-.545-3.508-1.492a4.359 4.359 0 0 1-.355-.508Zm2.328 3.25c.549 0 1 .451 1 1v2c0 .549-.451 1-1 1-.549 0-1-.451-1-1v-2c0-.549.451-1 1-1Zm-5 0c.549 0 1 .451 1 1v2c0 .549-.451 1-1 1-.549 0-1-.451-1-1v-2c0-.549.451-1 1-1Zm3.313-6.185c.136 1.057.403 1.913.878 2.497.442.544 1.134.938 2.344.938 1.573 0 2.292-.337 2.657-.751.384-.435.558-1.15.558-2.361 0-1.14-.243-1.847-.705-2.319-.477-.488-1.319-.862-2.824-1.025-1.487-.161-2.192.138-2.533.529-.269.307-.437.808-.438 1.578v.021c0 .265.021.562.063.893Zm-1.626 0c.042-.331.063-.628.063-.894v-.02c-.001-.77-.169-1.271-.438-1.578-.341-.391-1.046-.69-2.533-.529-1.505.163-2.347.537-2.824 1.025-.462.472-.705 1.179-.705 2.319 0 1.211.175 1.926.558 2.361.365.414 1.084.751 2.657.751 1.21 0 1.902-.394 2.344-.938.475-.584.742-1.44.878-2.497Z" />
  </svg>
);

const IconOpenCode = () => (
  <svg
    width="42"
    height="42"
    viewBox="0 0 24 24"
    fill="currentColor"
    fillRule="evenodd"
  >
    <path d="M16 6H8v12h8V6zm4 16H4V2h16v20z" />
  </svg>
);

const IconHTTPHealth = () => (
  <svg
    width="42"
    height="42"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
  </svg>
);

const FEED_TYPES = [
  { icon: <IconGitHubPR />, name: "GitHub PRs", desc: "Pull requests" },
  { icon: <IconGitHubActions />, name: "GitHub Actions", desc: "CI workflows" },
  { icon: <IconCopilot />, name: "Copilot", desc: "Agent sessions" },
  { icon: <IconOpenCode />, name: "OpenCode", desc: "Agent sessions" },
  { icon: <IconHTTPHealth />, name: "HTTP Health", desc: "Endpoint status" },
];

// --- Sub-components ---

const FeedCard = ({ icon, name, desc, delay, frame, fps }) => {
  const progress = spring({
    frame: frame - delay,
    fps,
    config: { damping: 14, mass: 0.5 },
  });
  const scale = interpolate(progress, [0, 1], [0.8, 1]);
  const opacity = interpolate(progress, [0, 0.5], [0, 1], {
    extrapolateRight: "clamp",
  });
  const y = interpolate(progress, [0, 1], [20, 0]);

  return (
    <div
      style={{
        width: 260,
        padding: "36px 23px",
        backgroundColor: COLORS.bgRaised,
        borderRadius: 16,
        border: `1.3px solid ${COLORS.border}`,
        textAlign: "center",
        opacity,
        transform: `scale(${scale}) translateY(${y}px)`,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: 8,
      }}
    >
      <div
        style={{
          color: COLORS.accent,
          lineHeight: 1,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        {icon}
      </div>
      <div
        style={{
          fontSize: 21,
          fontWeight: 600,
          color: COLORS.text,
          marginTop: 5,
        }}
      >
        {name}
      </div>
      <div
        style={{
          fontSize: 16,
          color: COLORS.textTertiary,
        }}
      >
        {desc}
      </div>
    </div>
  );
};

// --- Main scene ---

export const Closing = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Scene fade-in and fade-out
  const fadeIn = interpolate(frame, [0, 12], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const fadeOut = interpolate(frame, [300, 330], [1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Feed types section (frames 0-130)
  const feedsLabelOpacity = interpolate(frame, [0, 18], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // "...and more" text
  const moreOpacity = interpolate(frame, [70, 88], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Transition: feeds fade out, logo comes in
  const feedsFade = interpolate(frame, [115, 140], [1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Logo section (frames 135-270)
  const logoProgress = spring({
    frame: frame - 140,
    fps,
    config: { damping: 14, mass: 0.8 },
  });
  const logoOpacity = interpolate(frame, [140, 160], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const logoScale = interpolate(logoProgress, [0, 1], [0.8, 1]);

  const taglineOpacity = interpolate(frame, [165, 185], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const taglineY = interpolate(frame, [165, 185], [10, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const urlOpacity = interpolate(frame, [195, 215], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const urlY = interpolate(frame, [195, 215], [10, 0], {
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
      {/* Feed types grid */}
      <div
        style={{
          position: "absolute",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: 26,
          opacity: feedsFade,
        }}
      >
        <div
          style={{
            fontSize: 36,
            color: COLORS.textSecondary,
            fontWeight: 400,
            opacity: feedsLabelOpacity,
            marginBottom: 10,
          }}
        >
          Configure once. Stay informed.
        </div>
        <div
          style={{
            display: "flex",
            gap: 23,
            justifyContent: "center",
          }}
        >
          {FEED_TYPES.map((feed, i) => (
            <FeedCard
              key={feed.name}
              {...feed}
              delay={i * 8 + 12}
              frame={frame}
              fps={fps}
            />
          ))}
        </div>
        <div
          style={{
            fontSize: 36,
            color: COLORS.textSecondary,
            opacity: moreOpacity,
            fontWeight: 400,
          }}
        >
          ...and more
        </div>
      </div>

      {/* Final logo + CTA */}
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: 0,
          opacity: logoOpacity,
          transform: `scale(${logoScale})`,
        }}
      >
        <Img
          src={staticFile("cortado-icon.png")}
          style={{
            width: 130,
            height: 130,
            borderRadius: 26,
          }}
        />
        <div
          style={{
            fontSize: 73,
            fontWeight: 600,
            color: COLORS.text,
            letterSpacing: "-0.03em",
            marginTop: 26,
          }}
        >
          cortado
        </div>
        <div
          style={{
            fontSize: 29,
            color: COLORS.textSecondary,
            fontWeight: 400,
            marginTop: 8,
            opacity: taglineOpacity,
            transform: `translateY(${taglineY}px)`,
          }}
        >
          Focus on building.
        </div>
        <div
          style={{
            fontSize: 21,
            color: COLORS.accent,
            fontWeight: 500,
            marginTop: 36,
            padding: "13px 31px",
            backgroundColor: COLORS.accentDim,
            borderRadius: 13,
            opacity: urlOpacity,
            transform: `translateY(${urlY}px)`,
          }}
        >
          github.com/oribarilan/cortado
        </div>
      </div>
    </AbsoluteFill>
  );
};
