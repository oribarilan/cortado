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
      {/* Background music — fade in over 1s, fade out over 2s */}
      <Audio
        src={staticFile("music.mp3")}
        volume={(f) => {
          const fadeIn = interpolate(f, [0, FPS], [0, 0.25], {
            extrapolateRight: "clamp",
          });
          const fadeOut = interpolate(
            f,
            [DURATION_FRAMES - 2 * FPS, DURATION_FRAMES],
            [0.25, 0],
            { extrapolateLeft: "clamp" },
          );
          return Math.min(fadeIn, fadeOut);
        }}
        loop
      />

      <Sequence from={SCENES.hook.from} durationInFrames={SCENES.hook.duration}>
        <Hook />
      </Sequence>
      <Sequence
        from={SCENES.menubar.from}
        durationInFrames={SCENES.menubar.duration}
      >
        <MenubarDemo />
      </Sequence>
      <Sequence
        from={SCENES.panel.from}
        durationInFrames={SCENES.panel.duration}
      >
        {/* Keyboard click when shortcut keys appear */}
        <Sequence from={10} durationInFrames={30}>
          <Audio src="https://remotion.media/switch.wav" volume={0.4} />
        </Sequence>
        {/* Mouse click when OpenCode row gets selected */}
        <Sequence from={75} durationInFrames={30}>
          <Audio src="https://remotion.media/mouse-click.wav" volume={0.5} />
        </Sequence>
        <PanelDemo />
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
