// WebGL and WebGPU, in a chunk of their own, loaded the first time a page draws with either of
// them, see `gpu_lazy.ts`. Together, as they share the code keeping the bitmaps of canvases
// sized, which a chunk can't import from another.

export { WebGls } from "./webgl";
export { WebGpus } from "./webgpu";
