import "./index.css";
import { Composition } from "remotion";
import { CortadoPromo } from "./CortadoPromo";
import { FPS, DURATION_FRAMES } from "./CortadoPromo/constants";

export const RemotionRoot = () => {
  return (
    <>
      <Composition
        id="CortadoPromo"
        component={CortadoPromo}
        durationInFrames={DURATION_FRAMES}
        fps={FPS}
        width={1920}
        height={1080}
      />
    </>
  );
};
