import {
  AbsoluteFill,
  Audio,
  Sequence,
  interpolate,
  staticFile,
} from "remotion";
import { COLORS, SCENES, DURATION_FRAMES, FPS } from "./constants";
import { Hook } from "./scenes/Hook";
import { MenubarDemo } from "./scenes/MenubarDemo";
import { PanelDemo } from "./scenes/PanelDemo";
import { Closing } from "./scenes/Closing";

export const CortadoPromo = () => {
  return (
    <AbsoluteFill
      style={{
        backgroundColor: COLORS.bg,
        // Subtle radial glow for depth
        backgroundImage: `radial-gradient(ellipse at 50% 50%, rgba(78, 205, 196, 0.03) 0%, transparent 70%)`,
      }}
    >
      {/* Background music — starts after opening phrase, fade in over 1s, fade out over 2s */}
      <Sequence from={60} durationInFrames={DURATION_FRAMES - 60}>
        <Audio
          src={staticFile("music.mp3")}
          volume={(f) => {
            const fadeIn = interpolate(f, [0, FPS], [0, 0.25], {
              extrapolateRight: "clamp",
            });
            const fadeOut = interpolate(
              f,
              [DURATION_FRAMES - 60 - 2 * FPS, DURATION_FRAMES - 60],
              [0.25, 0],
              { extrapolateLeft: "clamp" },
            );
            return Math.min(fadeIn, fadeOut);
          }}
          loop
        />
      </Sequence>

      <Sequence from={SCENES.hook.from} durationInFrames={SCENES.hook.duration}>
        <Hook />
      </Sequence>
      <Sequence
        from={SCENES.panel.from}
        durationInFrames={SCENES.panel.duration}
      >
        {/* Notification sound when it slides in */}
        <Sequence from={45} durationInFrames={30}>
          <Audio src={staticFile("switch_002.ogg")} volume={3} />
        </Sequence>
        {/* Click sound when user clicks the notification */}
        <Sequence from={95} durationInFrames={30}>
          <Audio src="https://remotion.media/mouse-click.wav" volume={3} />
        </Sequence>
        {/* Down arrow keypress in OpenCode question */}
        <Sequence from={145} durationInFrames={30}>
          <Audio src={staticFile("switch_002.ogg")} volume={3} />
        </Sequence>
        {/* Enter keypress to confirm selection */}
        <Sequence from={155} durationInFrames={30}>
          <Audio src={staticFile("switch_002.ogg")} volume={3} />
        </Sequence>
        <PanelDemo />
      </Sequence>
      <Sequence
        from={SCENES.menubar.from}
        durationInFrames={SCENES.menubar.duration}
      >
        {/* Click sound on PR row */}
        <Sequence from={100} durationInFrames={30}>
          <Audio src="https://remotion.media/mouse-click.wav" volume={3} />
        </Sequence>
        {/* Click sound on merge button */}
        <Sequence from={155} durationInFrames={30}>
          <Audio src="https://remotion.media/mouse-click.wav" volume={3} />
        </Sequence>
        <MenubarDemo />
      </Sequence>
      <Sequence
        from={SCENES.closing.from}
        durationInFrames={SCENES.closing.duration}
      >
        <Closing />
      </Sequence>
    </AbsoluteFill>
  );
};
