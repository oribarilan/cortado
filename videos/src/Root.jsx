import "./index.css";
import { Composition } from "remotion";
import { CortadoPromo } from "./CortadoPromo";
import { FPS, DURATION_FRAMES } from "./CortadoPromo/constants";
import { SfxAudition } from "./SfxAudition";

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
      <Composition
        id="SfxAudition"
        component={SfxAudition}
        durationInFrames={360}
        fps={30}
        width={1920}
        height={1080}
      />
    </>
  );
};
