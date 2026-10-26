// Drawings on `<canvas>` elements, from the `draw` attribute of the `dom!` macro.
//
// A drawing is a list of operations of the 2D context, recorded in wasm. Its format is defined
// in `crates/vertigo/src/canvas.rs`, which also makes the fixture `canvas.test.ts` checks this
// file against.
//
// The bitmap of a canvas is kept as big as the canvas is on the screen, in device pixels, so it is
// sharp. When that size changes - the canvas grew, or went to a screen with denser pixels - a
// `ResizeObserver` draws the last drawing again, so a canvas whose drawing doesn't change any more
// doesn't show a stretched bitmap. The size on the screen comes from CSS. A canvas that CSS gives
// no size is as big as its bitmap, so it keeps the size it has and is drawn a pixel to a CSS pixel.
//
// A drawing that comes in a batch of DOM changes is drawn once the whole batch is in place, see
// `paintDrawn`.

import { decoder } from "../../text";

const Op = {
    FillStyle: 1,
    StrokeStyle: 2,
    GlobalAlpha: 3,
    LineWidth: 4,
    LineWidthPx: 5,
    LineCap: 6,
    FillRect: 7,
    FillCircle: 8,
    FillCirclePx: 9,
    StrokeCircle: 10,
    Polyline: 11,
    LineJoin: 12,
} as const;

const CAPS: Array<CanvasLineCap> = ['butt', 'round', 'square'];
const JOINS: Array<CanvasLineJoin> = ['miter', 'round', 'bevel'];

// The part of a 2D context a drawing uses, so a test can play one back on a recording.
export type Context2d = Pick<CanvasRenderingContext2D,
    'setTransform' | 'clearRect' | 'fillRect' | 'beginPath' | 'arc' | 'moveTo' | 'lineTo'
    | 'fill' | 'stroke' | 'fillStyle' | 'strokeStyle' | 'globalAlpha' | 'lineWidth' | 'lineCap'
    | 'lineJoin'>;

// Plays `bytes` back on `context`, whose bitmap is `width` x `height` device pixels, `ratio` of
// them to a CSS pixel.
export const play = (
    context: Context2d, bytes: Uint8Array, width: number, height: number, ratio: number,
) => {
    const data = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    let at = 0;

    const number = (): number => {
        const value = data.getFloat32(at, true);
        at += 4;
        return value;
    };

    const text = (): string => {
        const length = data.getUint16(at, true);
        const value = decoder.decode(bytes.subarray(at + 2, at + 2 + length));
        at += 2 + length;
        return value;
    };

    // Every drawing starts from a clear bitmap and the context's defaults.
    context.setTransform(1, 0, 0, 1, 0, 0);
    context.clearRect(0, 0, width, height);
    context.globalAlpha = 1;
    context.fillStyle = '#000';
    context.strokeStyle = '#000';
    context.lineWidth = 1;
    context.lineCap = 'butt';
    context.lineJoin = 'miter';

    // Device pixels to a unit of the drawing.
    let scale = ratio;

    if (data.getUint8(at++) === 1) {
        // A view box, fitted as SVG fits `xMidYMid meet`.
        const x = number();
        const y = number();
        const boxWidth = number();
        const boxHeight = number();

        scale = Math.min(width / boxWidth, height / boxHeight);
        context.setTransform(
            scale, 0, 0, scale,
            (width - boxWidth * scale) / 2 - x * scale,
            (height - boxHeight * scale) / 2 - y * scale,
        );
    } else {
        context.setTransform(ratio, 0, 0, ratio, 0, 0);
    }

    if (!(scale > 0) || !isFinite(scale)) {
        return;
    }

    // Units to a CSS pixel.
    const px = ratio / scale;

    const circle = (x: number, y: number, radius: number) => {
        context.beginPath();
        context.arc(x, y, Math.max(radius, 0), 0, 2 * Math.PI);
    };

    while (at < bytes.length) {
        const op = data.getUint8(at++);

        switch (op) {
            case Op.FillStyle: context.fillStyle = text(); break;
            case Op.StrokeStyle: context.strokeStyle = text(); break;
            case Op.GlobalAlpha: context.globalAlpha = number(); break;
            case Op.LineWidth: context.lineWidth = number(); break;
            case Op.LineWidthPx: context.lineWidth = number() * px; break;
            case Op.LineCap: context.lineCap = CAPS[data.getUint8(at++)] ?? 'butt'; break;
            case Op.LineJoin: context.lineJoin = JOINS[data.getUint8(at++)] ?? 'miter'; break;
            case Op.FillRect: context.fillRect(number(), number(), number(), number()); break;
            case Op.FillCircle: circle(number(), number(), number()); context.fill(); break;
            case Op.FillCirclePx: circle(number(), number(), number() * px); context.fill(); break;
            case Op.StrokeCircle: circle(number(), number(), number()); context.stroke(); break;
            case Op.Polyline: {
                const count = data.getUint32(at, true);
                at += 4;
                context.beginPath();
                for (let point = 0; point < count; point++) {
                    const x = number();
                    const y = number();
                    if (point === 0) {
                        context.moveTo(x, y);
                    } else {
                        context.lineTo(x, y);
                    }
                }
                if (count > 1) {
                    context.stroke();
                }
                break;
            }
            default:
                throw new Error(`canvas: unknown operation ${op} at byte ${at - 1}`);
        }
    }
};

// In device pixels, `ratio` of them to a CSS pixel.
type Size = { width: number, height: number, ratio: number };

export class Canvases {
    private readonly drawings: WeakMap<HTMLCanvasElement, Uint8Array> = new WeakMap();
    private readonly sizes: WeakMap<HTMLCanvasElement, Size> = new WeakMap();
    // Canvases as big as their bitmaps, drawn a pixel to a CSS pixel, see `paint`.
    private readonly unscaled: WeakSet<HTMLCanvasElement> = new WeakSet();
    // Canvases with a new drawing in the batch being applied, see `paintDrawn`.
    private readonly drawn: Set<HTMLCanvasElement> = new Set();
    private observer: ResizeObserver | null = null;

    // `bytes` is good only for the call it came in (see `DriverDom.update`), so it is copied, to
    // be drawn after the batch and again when the canvas changes its size.
    public draw(node: Element, bytes: Uint8Array) {
        if (!(node instanceof HTMLCanvasElement)) {
            throw new Error('draw: the element is not a canvas');
        }

        const observed = this.drawings.has(node);
        this.drawings.set(node, bytes.slice());

        if (!observed) {
            this.observe(node);
        }

        this.drawn.add(node);
    }

    // Draws what came in a batch, once the batch is in place. In the middle of it a canvas can be
    // without the size CSS gives it: the styles the app makes go into `<head>` at the end of its
    // first batch (`MapNodes.addStyles`), and with hydration the server's are gone by then. At a
    // pixel ratio above 1 such a canvas would also pass for one that CSS gives no size, and stay
    // at the size of its bitmap.
    public paintDrawn() {
        for (const canvas of this.drawn) {
            this.repaint(canvas);
        }

        this.drawn.clear();
    }

    public forget(node: Element | Comment | Text) {
        if (node instanceof HTMLCanvasElement && this.drawings.has(node)) {
            this.observer?.unobserve(node);
            this.drawings.delete(node);
            this.sizes.delete(node);
            this.unscaled.delete(node);
            this.drawn.delete(node);
        }
    }

    private observe(canvas: HTMLCanvasElement) {
        if (typeof ResizeObserver === 'undefined') {
            return;
        }

        if (this.observer === null) {
            this.observer = new ResizeObserver((entries) => {
                for (const entry of entries) {
                    const canvas = entry.target;
                    // Only canvases are observed.
                    if (!(canvas instanceof HTMLCanvasElement)) {
                        throw Error('#canvas');
                    }

                    // In the pixels `devicePixelRatio` says there are, as the drawing scales by
                    // it. A browser's own count of device pixels can disagree with it: in
                    // DevTools emulating another ratio it counts those of the real screen.
                    const ratio = window.devicePixelRatio || 1;
                    this.sizes.set(canvas, {
                        width: Math.round(entry.contentRect.width * ratio),
                        height: Math.round(entry.contentRect.height * ratio),
                        ratio,
                    });

                    this.repaint(canvas);
                }
            });
        }

        // Watching the device pixels, the observer tells also of a change of `devicePixelRatio`
        // alone, as when the window goes to another screen. A browser that doesn't know the box
        // tells only of a new size in CSS pixels.
        try {
            this.observer.observe(canvas, { box: 'device-pixel-content-box' });
        } catch {
            this.observer.observe(canvas);
        }
    }

    // Draws the last drawing of `canvas`. One that fails is logged, so that the canvases after it
    // still get theirs.
    private repaint(canvas: HTMLCanvasElement) {
        const drawing = this.drawings.get(canvas);

        if (drawing !== undefined) {
            try {
                this.paint(canvas, drawing);
            } catch (error) {
                console.error(error);
            }
        }
    }

    // The size the observer saw last, so that a drawing every frame doesn't force a layout.
    // Until it has seen one, the canvas is measured. `null` while the canvas is off the page or
    // has no size: the observer draws it once it has one.
    private size(canvas: HTMLCanvasElement): Size | null {
        if (this.unscaled.has(canvas)) {
            return { width: canvas.width, height: canvas.height, ratio: 1 };
        }

        const ratio = window.devicePixelRatio || 1;
        const size = this.sizes.get(canvas) ?? (canvas.isConnected ? {
            width: Math.round(canvas.clientWidth * ratio),
            height: Math.round(canvas.clientHeight * ratio),
            ratio,
        } : null);

        return size !== null && size.width > 0 && size.height > 0 ? size : null;
    }

    private paint(canvas: HTMLCanvasElement, drawing: Uint8Array) {
        let size = this.size(canvas);
        if (size === null) {
            return;
        }

        if (canvas.width !== size.width || canvas.height !== size.height) {
            const [width, height] = [canvas.width, canvas.height];
            const [cssWidth, cssHeight] = [canvas.clientWidth, canvas.clientHeight];
            // Setting either clears the bitmap even to the same number.
            if (canvas.width !== size.width) {
                canvas.width = size.width;
            }
            if (canvas.height !== size.height) {
                canvas.height = size.height;
            }

            // A canvas without a size of its own in CSS is as big as its bitmap, which would grow
            // with each drawing at a ratio above 1. It keeps the size it had, a pixel to a CSS
            // pixel.
            if (size.ratio !== 1
                && canvas.clientWidth - cssWidth === size.width - width
                && canvas.clientHeight - cssHeight === size.height - height) {
                canvas.width = width;
                canvas.height = height;
                this.unscaled.add(canvas);
                size = { width, height, ratio: 1 };
            }
        }

        const context = canvas.getContext('2d');

        if (context !== null) {
            play(context, drawing, size.width, size.height, size.ratio);
        }
    }
}
