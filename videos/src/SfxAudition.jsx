import {
  AbsoluteFill,
  Audio,
  Sequence,
  staticFile,
  useCurrentFrame,
} from "remotion";

const SOUNDS = [
  { file: "click_001.ogg", label: "click_001 — soft tap" },
  { file: "click_003.ogg", label: "click_003 — crisp snap" },
  { file: "click_005.ogg", label: "click_005 — muted tick" },
  { file: "switch_002.ogg", label: "switch_002 — toggle" },
  { file: "select_003.ogg", label: "select_003 — gentle select" },
  { file: "toggle_001.ogg", label: "toggle_001 — soft toggle" },
  { file: "remotion-switch", label: "remotion switch.wav (current kbd)" },
  { file: "remotion-mouse", label: "remotion mouse-click.wav (current row)" },
];

const GAP = 45; // 1.5s per sound

export const SfxAudition = () => {
  const frame = useCurrentFrame();
  const activeIdx = Math.floor(frame / GAP);

  return (
    <AbsoluteFill
      style={{
        backgroundColor: "#0d0f16",
        justifyContent: "center",
        alignItems: "center",
        fontFamily: "'Space Grotesk', sans-serif",
      }}
    >
      <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
        {SOUNDS.map((s, i) => (
          <div
            key={s.file}
            style={{
              fontSize: 28,
              color: activeIdx === i ? "#4ecdc4" : "#6c6f84",
              fontWeight: activeIdx === i ? 600 : 400,
              transition: "color 0.1s",
            }}
          >
            {activeIdx === i ? "▸ " : "  "}
            {i + 1}. {s.label}
          </div>
        ))}
      </div>

      {SOUNDS.map((s, i) => (
        <Sequence key={s.file} from={i * GAP} durationInFrames={GAP}>
          <Audio
            src={
              s.file === "remotion-switch"
                ? "https://remotion.media/switch.wav"
                : s.file === "remotion-mouse"
                  ? "https://remotion.media/mouse-click.wav"
                  : staticFile(s.file)
            }
            volume={0.7}
          />
        </Sequence>
      ))}
    </AbsoluteFill>
  );
};
