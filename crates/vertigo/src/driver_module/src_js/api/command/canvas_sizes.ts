// The bitmaps of canvases with WebGL or WebGPU, kept as big as the canvases are on the screen, in
// device pixels, as `canvas.ts` keeps them, so the pictures are sharp, times the scale a canvas was
// observed with, fewer pixels for a slow GPU. A bitmap is resized before
// the calls that come next. When a canvas changes its size, wasm is told the size before anything
// is resized, so its calls drawing again resize the bitmap and draw in one go, and the canvas never
// shows the picture cleared, nor stretched.

export type Size = { width: number, height: number };

// `size`, in device pixels, times `scale`: at least a pixel each way.
const scaled = (size: Size, scale: number): Size => ({
    width: Math.max(1, Math.round(size.width * scale)),
    height: Math.max(1, Math.round(size.height * scale)),
});

// Tells wasm of a canvas, through the callback of its context, see `webgl.ts` and `webgpu.ts`.
export type Tell = (callback: number, event: Array<number | Uint8Array<ArrayBuffer>>) => void;

export class CanvasSizes {
    // The size in device pixels each canvas was seen at last, see `fit`.
    private readonly sizes: WeakMap<HTMLCanvasElement, Size> = new WeakMap();
    // Canvases as big as their bitmaps, which CSS gives no size, see `fit`.
    private readonly unscaled: WeakSet<HTMLCanvasElement> = new WeakSet();
    // Pixels of the bitmap to a device pixel, of the canvases not drawn on at 1.
    private readonly scales: WeakMap<HTMLCanvasElement, number> = new WeakMap();
    // The size each canvas was told at last.
    private readonly told: WeakMap<HTMLCanvasElement, Size> = new WeakMap();
    private observer: ResizeObserver | null = null;

    // `tell` tells a canvas's new size, if it has a context to tell it to, and answers whether
    // it had.
    public constructor(private readonly tell: (canvas: HTMLCanvasElement, size: Size) => boolean) {}

    public observe(canvas: HTMLCanvasElement, scale = 1) {
        if (scale !== 1) {
            this.scales.set(canvas, scale);
        }
        if (typeof ResizeObserver === 'undefined') {
            return;
        }

        if (this.observer === null) {
            this.observer = new ResizeObserver((entries) => {
                for (const entry of entries) {
                    if (entry.target instanceof HTMLCanvasElement) {
                        this.resized(entry.target, entry.contentRect);
                    }
                }
            });
        }

        try {
            this.observer.observe(canvas, { box: 'device-pixel-content-box' });
        } catch {
            this.observer.observe(canvas);
        }
    }

    public forget(canvas: HTMLCanvasElement) {
        this.observer?.unobserve(canvas);
        this.sizes.delete(canvas);
        this.unscaled.delete(canvas);
        this.scales.delete(canvas);
        this.told.delete(canvas);
    }

    // The canvas's new size, kept for the calls that come next to resize the bitmap to, and told:
    // calls drawing again come at once then. A canvas as big as its bitmap keeps its size.
    private resized(canvas: HTMLCanvasElement, box: { width: number, height: number }) {
        const ratio = window.devicePixelRatio || 1;
        const size = {
            width: Math.round(box.width * ratio),
            height: Math.round(box.height * ratio),
        };
        this.sizes.set(canvas, size);
        this.tellSize(canvas, size);
    }

    // A new scale of the canvas's bitmap, told as a new size, which the calls that come next draw
    // at.
    public rescale(canvas: HTMLCanvasElement, scale: number) {
        if (scale === 1) {
            this.scales.delete(canvas);
        } else {
            this.scales.set(canvas, scale);
        }
        const size = this.sizes.get(canvas);
        if (size) {
            this.tellSize(canvas, size);
        }
    }

    // Tells `size`, the size in device pixels the canvas was seen at, times its scale, unless it
    // was told.
    private tellSize(canvas: HTMLCanvasElement, size: Size) {
        const told = this.unscaled.has(canvas)
            ? { width: canvas.width, height: canvas.height }
            : scaled(size, this.scales.get(canvas) ?? 1);
        const last = this.told.get(canvas);
        if (told.width > 0 && told.height > 0
            && (last?.width !== told.width || last.height !== told.height)
            && this.tell(canvas, told)) {
            this.told.set(canvas, told);
        }
    }

    // Makes the bitmap as big as the canvas is on the screen, times its scale, before the calls that
    // draw on it, and answers whether it changed it. A canvas CSS gives no size is as big as its bitmap, which
    // would grow with each resize at a ratio above 1: it keeps the size it has, a pixel to a CSS
    // pixel.
    public fit(canvas: HTMLCanvasElement): boolean {
        if (this.unscaled.has(canvas)) {
            return false;
        }

        const ratio = window.devicePixelRatio || 1;
        const seen = this.sizes.get(canvas) ?? (canvas.isConnected ? {
            width: Math.round(canvas.clientWidth * ratio),
            height: Math.round(canvas.clientHeight * ratio),
        } : null);

        if (seen === null || seen.width <= 0 || seen.height <= 0) {
            return false;
        }
        const size = scaled(seen, this.scales.get(canvas) ?? 1);
        if (canvas.width === size.width && canvas.height === size.height) {
            return false;
        }

        const [width, height] = [canvas.width, canvas.height];
        const [cssWidth, cssHeight] = [canvas.clientWidth, canvas.clientHeight];
        canvas.width = size.width;
        canvas.height = size.height;

        if (ratio !== 1
            && canvas.clientWidth - cssWidth === size.width - width
            && canvas.clientHeight - cssHeight === size.height - height) {
            canvas.width = width;
            canvas.height = height;
            this.unscaled.add(canvas);
        }

        return true;
    }
}
