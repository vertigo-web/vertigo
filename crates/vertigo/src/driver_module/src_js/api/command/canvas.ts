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
// `paintDrawn`. Whatever it does to the state of the context - styles, transforms, clips - is
// undone when it ends, so the next drawing starts from the defaults.
//
// Images are loaded once per URL, the first time a drawing asks for one or when the app asks to
// load it beforehand. A canvas whose drawing asked for an image still loading is drawn again when
// the image comes, see `Images`, and so is one that wrote text when a font loads.

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
    FillPolygon: 13,
    StrokePolygon: 14,
    DrawImage: 15,
    Pattern: 16,
    LinearGradient: 17,
    Append: 18,
    AppendEnd: 19,
    Save: 20,
    Restore: 21,
    Translate: 22,
    Rotate: 23,
    Scale: 24,
    Transform: 25,
    ResetTransform: 26,
    FillPath: 27,
    StrokePath: 28,
    Clip: 29,
    StrokeRect: 30,
    ClearRect: 31,
    Font: 32,
    FontPx: 33,
    TextAlign: 34,
    TextBaseline: 35,
    Text: 36,
    RadialGradient: 37,
    ConicGradient: 38,
    LineDash: 39,
    MiterLimit: 40,
    Composite: 41,
    Shadow: 42,
    Filter: 43,
    ImageSmoothing: 44,
    DrawPixels: 45,
} as const;

// Commands of a path, see `Path2D` in canvas.rs.
const PathCommand = {
    MoveTo: 0,
    LineTo: 1,
    QuadTo: 2,
    CubicTo: 3,
    ArcTo: 4,
    Ellipse: 5,
    Close: 6,
} as const;

const CAPS: Array<CanvasLineCap> = ['butt', 'round', 'square'];
const JOINS: Array<CanvasLineJoin> = ['miter', 'round', 'bevel'];
const QUALITIES: Array<ImageSmoothingQuality> = ['low', 'medium', 'high'];

// The part of a 2D context a drawing uses, so a test can play one back on a recording.
export type Context2d = Pick<CanvasRenderingContext2D,
    'setTransform' | 'getTransform' | 'translate' | 'rotate' | 'scale' | 'transform' | 'save' | 'restore'
    | 'clearRect' | 'fillRect' | 'strokeRect' | 'beginPath' | 'arc' | 'moveTo' | 'lineTo' | 'closePath'
    | 'fill' | 'stroke' | 'clip' | 'fillStyle' | 'strokeStyle' | 'globalAlpha' | 'lineWidth' | 'lineCap'
    | 'lineJoin' | 'miterLimit' | 'setLineDash' | 'lineDashOffset' | 'drawImage' | 'createPattern'
    | 'createLinearGradient' | 'createRadialGradient' | 'createConicGradient' | 'font' | 'textAlign'
    | 'textBaseline' | 'fillText' | 'strokeText' | 'globalCompositeOperation' | 'shadowColor'
    | 'shadowBlur' | 'shadowOffsetX' | 'shadowOffsetY' | 'filter' | 'imageSmoothingEnabled'
    | 'imageSmoothingQuality'>;

// The image at a URL, if it is loaded and can be drawn.
export type ImageAt = (url: string) => CanvasImageSource | null;

// What `pixels`, rows `width` wide of four bytes each, are drawn from.
export type Bitmap = (pixels: Uint8Array, width: number, height: number) => CanvasImageSource | null;

const bitmap: Bitmap = (pixels, width, height) => {
    const made = document.createElement('canvas');
    made.width = width;
    made.height = height;
    made.getContext('2d')?.putImageData(new ImageData(new Uint8ClampedArray(pixels), width, height), 0, 0);
    return made;
};

// The sizes in pixels of a CSS value, f. ex. a font, `factor` times as big.
const resized = (css: string, factor: number): string =>
    css.replace(/(\d*\.?\d+(?:e[+-]?\d+)?)px/gi, (_match, size: string) => `${Number(size) * factor}px`);

// Plays `bytes` back on `context`, whose bitmap is `width` x `height` device pixels, `ratio` of
// them to a CSS pixel. An image that `image` doesn't have is left out, and pixels are drawn from
// what `pixels` makes of them. Whether it wrote text, which a font that loads later changes.
export const play = (
    context: Context2d, bytes: Uint8Array, width: number, height: number, ratio: number,
    image: ImageAt = () => null, pixels: Bitmap = bitmap,
): boolean => {
    const data = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    let at = 0;

    const number = (): number => {
        const value = data.getFloat32(at, true);
        at += 4;
        return value;
    };

    const count = (): number => {
        const value = data.getUint32(at, true);
        at += 4;
        return value;
    };

    const text = (): string => {
        const length = data.getUint16(at, true);
        const value = decoder.decode(bytes.subarray(at + 2, at + 2 + length));
        at += 2 + length;
        return value;
    };

    const long = (): string => {
        const length = count();
        const value = decoder.decode(bytes.subarray(at, at + length));
        at += length;
        return value;
    };

    // The context's defaults, which every drawing starts from, and a drawing appended too: every
    // style a drawing can set.
    const defaults = () => {
        context.globalAlpha = 1;
        context.fillStyle = '#000';
        context.strokeStyle = '#000';
        context.lineWidth = 1;
        context.lineCap = 'butt';
        context.lineJoin = 'miter';
        context.miterLimit = 10;
        context.setLineDash([]);
        context.lineDashOffset = 0;
        context.font = '10px sans-serif';
        context.textAlign = 'start';
        context.textBaseline = 'alphabetic';
        context.globalCompositeOperation = 'source-over';
        context.shadowColor = 'rgba(0, 0, 0, 0)';
        context.shadowBlur = 0;
        context.shadowOffsetX = 0;
        context.shadowOffsetY = 0;
        context.filter = 'none';
        context.imageSmoothingEnabled = true;
        context.imageSmoothingQuality = 'low';
    };

    // The state before the drawing, which the end gives back however it ends: a clip, unlike the
    // styles, has no default to set again.
    context.save();
    // Saves of the drawing not restored yet, which the end restores.
    let saved = 0;
    let wrote = false;

    try {
        context.setTransform(1, 0, 0, 1, 0, 0);
        context.clearRect(0, 0, width, height);
        defaults();

        // Device pixels to a unit of the drawing.
        let scale = ratio;
        let base: [number, number, number, number, number, number] = [ratio, 0, 0, ratio, 0, 0];

        if (data.getUint8(at++) === 1) {
            // A view box, fitted as SVG fits `xMidYMid meet`. One without a width and a height
            // above zero, or with a number that is NaN or infinite, draws nothing.
            const x = number();
            const y = number();
            const boxWidth = number();
            const boxHeight = number();

            scale = [x, y, boxWidth, boxHeight].every(isFinite) && boxWidth > 0 && boxHeight > 0
                ? Math.min(width / boxWidth, height / boxHeight)
                : NaN;
            base = [
                scale, 0, 0, scale,
                (width - boxWidth * scale) / 2 - x * scale,
                (height - boxHeight * scale) / 2 - y * scale,
            ];
        }

        context.setTransform(...base);

        if (!(scale > 0) || !isFinite(scale)) {
            return false;
        }

        // Device pixels to a unit under the transform at the time, `null` once it has changed.
        let device: number | null = scale;
        const deviceScale = (): number => {
            if (device === null) {
                const { a, b, c, d } = context.getTransform();
                device = Math.sqrt(Math.abs(a * d - b * c));
            }
            return device;
        };

        // Units to a CSS pixel.
        const px = () => ratio / deviceScale();

        // Traces the next `points` points as one path, closed if asked to.
        const path = (points: number, closed: boolean) => {
            context.beginPath();
            for (let point = 0; point < points; point++) {
                const x = number();
                const y = number();
                if (point === 0) {
                    context.moveTo(x, y);
                } else {
                    context.lineTo(x, y);
                }
            }
            if (closed) {
                context.closePath();
            }
        };

        // A path of `Path2D` in canvas.rs: SVG data, then commands that go on from it. A radius
        // below zero, which the context would throw on, counts as zero.
        const shape = (): Path2D => {
            const made = new Path2D(long());
            for (let left = count(); left > 0; left--) {
                const command = data.getUint8(at++);
                switch (command) {
                    case PathCommand.MoveTo: made.moveTo(number(), number()); break;
                    case PathCommand.LineTo: made.lineTo(number(), number()); break;
                    case PathCommand.QuadTo: made.quadraticCurveTo(number(), number(), number(), number()); break;
                    case PathCommand.CubicTo:
                        made.bezierCurveTo(number(), number(), number(), number(), number(), number());
                        break;
                    case PathCommand.ArcTo:
                        made.arcTo(number(), number(), number(), number(), Math.max(number(), 0));
                        break;
                    case PathCommand.Ellipse:
                        made.ellipse(number(), number(), Math.max(number(), 0), Math.max(number(), 0),
                            number(), number(), number(), data.getUint8(at++) === 1);
                        break;
                    case PathCommand.Close: made.closePath(); break;
                    default:
                        throw new Error(`canvas: unknown path command ${command} at byte ${at - 1}`);
                }
            }
            return made;
        };

        const rule = (): CanvasFillRule => data.getUint8(at++) === 1 ? 'evenodd' : 'nonzero';

        // A property that takes a keyword, set to the text that comes: the context leaves it as
        // it was for a text that is no keyword of it.
        const keyword = (name: string) => {
            Reflect.set(context, name, text());
        };

        // Patterns by URL and scale, made once per drawing played: a map draws thousands of
        // shapes with a few. Each keeps the transform it is made with: a fill or a stroke style
        // holds the pattern itself, and would follow it to another scale, also back from a
        // drawing appended.
        const patterns: Map<string, CanvasPattern | null> = new Map();
        const pattern = (url: string, scale: number): CanvasPattern | null => {
            const key = `${scale} ${url}`;
            if (!patterns.has(key)) {
                const source = image(url);
                const made = source === null ? null : context.createPattern(source, 'repeat');
                // On top of the drawing's transform: a pixel of the image is `scale` units.
                made?.setTransform({ a: scale, b: 0, c: 0, d: scale, e: 0, f: 0 });
                patterns.set(key, made);
            }
            return patterns.get(key) ?? null;
        };

        // Sets the style for fills (0) or for lines and outlines (1), see `target` in canvas.rs.
        const style = (target: number, value: string | CanvasGradient | CanvasPattern) => {
            if (target === 0) {
                context.fillStyle = value;
            } else {
                context.strokeStyle = value;
            }
        };

        // A gradient for `target` that `create` makes, with the stops that come. One the context
        // refuses - a radius below zero, a number that is NaN or infinite, or a conic gradient in
        // an old browser - leaves the style as it was. A gradient whose ends are one (`point`)
        // paints nothing on a canvas: SVG fills with its last stop instead, so a triangle of one
        // shade in a mesh is no hole.
        const gradient = (target: number, create: () => CanvasGradient, point: boolean) => {
            let made: CanvasGradient | null = null;
            try {
                made = create();
            } catch {
                // refused
            }
            // The stop at the end of the gradient: of the highest offset, added last.
            let last: { offset: number, color: string } | null = null;
            const stops = data.getUint16(at, true);
            at += 2;
            for (let stop = 0; stop < stops; stop++) {
                // The context refuses an offset outside 0 to 1, which counts as the nearer end,
                // as in SVG. One that is NaN, or a text that is no color, leaves the stop out, as
                // a bad fill style is.
                const offset = Math.min(Math.max(number(), 0), 1);
                const color = text();
                if (made !== null && !Number.isNaN(offset)) {
                    try {
                        made.addColorStop(offset, color);
                        if (last === null || offset >= last.offset) {
                            last = { offset, color };
                        }
                    } catch {
                        // left out
                    }
                }
            }
            if (made !== null) {
                style(target, point && last !== null ? last.color : made);
            }
        };

        // Text at the place that comes, as big as its font says however small the units are: a
        // browser leaves out a font below a pixel even when the transform makes it bigger. So the
        // transform's scale is taken out, and the font's sizes go to device pixels instead.
        const write = (target: number) => {
            const value = text();
            const x = number();
            const y = number();
            const factor = deviceScale();
            wrote = true;
            if ([x, y, factor].every(isFinite) && factor > 0) {
                context.save();
                context.translate(x, y);
                context.scale(1 / factor, 1 / factor);
                context.font = resized(context.font, factor);
                if (target === 0) {
                    context.fillText(value, 0, 0);
                } else {
                    context.lineWidth *= factor;
                    context.strokeText(value, 0, 0);
                }
                context.restore();
            }
        };

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
                case Op.LineWidthPx: context.lineWidth = number() * px(); break;
                case Op.LineCap: context.lineCap = CAPS[data.getUint8(at++)] ?? 'butt'; break;
                case Op.LineJoin: context.lineJoin = JOINS[data.getUint8(at++)] ?? 'miter'; break;
                case Op.MiterLimit: context.miterLimit = number(); break;
                case Op.LineDash: {
                    const unit = data.getUint8(at++) === 1 ? px() : 1;
                    const length = data.getUint16(at, true);
                    at += 2;
                    context.setLineDash(Array.from({ length }, () => number() * unit));
                    context.lineDashOffset = number() * unit;
                    break;
                }
                case Op.FillRect: context.fillRect(number(), number(), number(), number()); break;
                case Op.StrokeRect: context.strokeRect(number(), number(), number(), number()); break;
                case Op.ClearRect: context.clearRect(number(), number(), number(), number()); break;
                case Op.FillCircle: circle(number(), number(), number()); context.fill(); break;
                case Op.FillCirclePx: circle(number(), number(), number() * px()); context.fill(); break;
                case Op.StrokeCircle: circle(number(), number(), number()); context.stroke(); break;
                case Op.Polyline: {
                    const points = count();
                    path(points, false);
                    if (points > 1) {
                        context.stroke();
                    }
                    break;
                }
                case Op.FillPolygon: {
                    const points = count();
                    path(points, true);
                    if (points > 2) {
                        context.fill();
                    }
                    break;
                }
                case Op.StrokePolygon: {
                    const points = count();
                    path(points, true);
                    if (points > 2) {
                        context.stroke();
                    }
                    break;
                }
                case Op.FillPath: {
                    const fillRule = rule();
                    context.fill(shape(), fillRule);
                    break;
                }
                case Op.StrokePath: context.stroke(shape()); break;
                case Op.Clip: {
                    const clipRule = rule();
                    context.clip(shape(), clipRule);
                    break;
                }
                case Op.DrawImage: {
                    const source = image(text());
                    const [sx, sy, sw, sh, dx, dy, dw, dh] = [
                        number(), number(), number(), number(), number(), number(), number(), number(),
                    ];
                    if (source !== null) {
                        context.drawImage(source, sx, sy, sw, sh, dx, dy, dw, dh);
                    }
                    break;
                }
                case Op.DrawPixels: {
                    const columns = count();
                    const rows = count();
                    const [x, y, w, h] = [number(), number(), number(), number()];
                    const length = columns * rows * 4;
                    const source = pixels(bytes.subarray(at, at + length), columns, rows);
                    at += length;
                    if (source !== null) {
                        context.drawImage(source, x, y, w, h);
                    }
                    break;
                }
                case Op.Pattern: {
                    const target = data.getUint8(at++);
                    const url = text();
                    const scale = number();
                    // Until the image comes, the style stays what it was, and so it does with a
                    // scale of 0, NaN or infinity, with which the pattern would fill nothing.
                    const found = scale !== 0 && isFinite(scale) ? pattern(url, scale) : null;
                    if (found !== null) {
                        style(target, found);
                    }
                    break;
                }
                case Op.LinearGradient: {
                    const target = data.getUint8(at++);
                    const [x0, y0, x1, y1] = [number(), number(), number(), number()];
                    gradient(target, () => context.createLinearGradient(x0, y0, x1, y1), x0 === x1 && y0 === y1);
                    break;
                }
                case Op.RadialGradient: {
                    const target = data.getUint8(at++);
                    const [x0, y0, r0, x1, y1, r1] = [number(), number(), number(), number(), number(), number()];
                    gradient(target, () => context.createRadialGradient(x0, y0, r0, x1, y1, r1),
                        x0 === x1 && y0 === y1 && r0 === r1);
                    break;
                }
                case Op.ConicGradient: {
                    const target = data.getUint8(at++);
                    const [angle, x, y] = [number(), number(), number()];
                    gradient(target, () => context.createConicGradient(angle, x, y), false);
                    break;
                }
                case Op.Font: context.font = text(); break;
                // From CSS pixels to units, under the transform at the time.
                case Op.FontPx: context.font = resized(text(), px()); break;
                case Op.TextAlign: keyword('textAlign'); break;
                case Op.TextBaseline: keyword('textBaseline'); break;
                case Op.Text: write(data.getUint8(at++)); break;
                case Op.Composite: keyword('globalCompositeOperation'); break;
                case Op.Shadow: {
                    // In bitmap pixels, whatever the transform: a CSS pixel is `ratio` of them.
                    context.shadowColor = text();
                    context.shadowBlur = number() * ratio;
                    context.shadowOffsetX = number() * ratio;
                    context.shadowOffsetY = number() * ratio;
                    break;
                }
                case Op.Filter: context.filter = resized(text(), ratio); break;
                case Op.ImageSmoothing: {
                    const level = data.getUint8(at++);
                    context.imageSmoothingEnabled = level > 0;
                    context.imageSmoothingQuality = QUALITIES[level - 1] ?? 'low';
                    break;
                }
                case Op.Save: context.save(); saved++; break;
                // A drawing appended is drawn as it would be alone, and leaves the state as it was.
                case Op.Append: context.save(); saved++; defaults(); break;
                case Op.Restore:
                case Op.AppendEnd:
                    if (saved > 0) {
                        context.restore();
                        saved--;
                        device = null;
                    }
                    break;
                case Op.Translate: context.translate(number(), number()); break;
                case Op.Rotate: context.rotate(number()); break;
                case Op.Scale: context.scale(number(), number()); device = null; break;
                case Op.Transform:
                    context.transform(number(), number(), number(), number(), number(), number());
                    device = null;
                    break;
                case Op.ResetTransform: context.setTransform(...base); device = scale; break;
                default:
                    throw new Error(`canvas: unknown operation ${op} at byte ${at - 1}`);
            }
        }

        return wrote;
    } finally {
        for (; saved > 0; saved--) {
            context.restore();
        }
        context.restore();
    }
};

// In device pixels, `ratio` of them to a CSS pixel.
type Size = { width: number, height: number, ratio: number };

// The images drawings ask for, by URL, each loaded once and kept while a drawing uses it. Of
// those no drawing uses any more, the last `kept` stay too, so that an animation going through a
// few, or a map scrolled back, has them at once. Older ones are let go, so that a map that has
// shown thousands of tiles doesn't hold them all, and load again if a drawing asks for them. One
// loaded beforehand stays until a drawing asks for it. One that fails to load stays left out
// while it is kept.
class Images {
    private readonly images: Map<string, HTMLImageElement> = new Map();
    // Canvases to draw again when the image at a URL comes.
    private readonly waiting: Map<string, Set<HTMLCanvasElement>> = new Map();
    // The URLs the last drawing of each canvas asked for.
    private readonly used: Map<HTMLCanvasElement, Set<string>> = new Map();
    // The images no drawing uses any more, the one used longest ago first.
    private readonly unused: Set<string> = new Set();

    constructor(
        private readonly kept: number,
        private readonly loaded: (canvas: HTMLCanvasElement) => void,
    ) {}

    // The image at `url`, loading unless it has started already.
    public load(url: string): HTMLImageElement {
        let image = this.images.get(url);

        if (image === undefined) {
            image = new Image();
            image.onload = () => {
                const canvases = this.waiting.get(url) ?? new Set();
                this.waiting.delete(url);
                for (const canvas of canvases) {
                    this.loaded(canvas);
                }
            };
            image.onerror = () => {
                this.waiting.delete(url);
                console.error(`canvas: the image ${url} failed to load`);
            };
            image.src = url;
            this.images.set(url, image);
        }

        return image;
    }

    // Loads the image at `url` for drawings to come, and keeps it until one asks for it.
    public preload(url: string) {
        this.load(url);
        this.unused.delete(url);
    }

    // The image at `url` for `canvas`, or `null` while it loads, when `canvas` waits for it.
    public get(url: string, canvas: HTMLCanvasElement): CanvasImageSource | null {
        const image = this.load(url);

        if (image.complete) {
            // A broken image is complete too, but has no size.
            return image.naturalWidth > 0 ? image : null;
        }

        const waiting = this.waiting.get(url) ?? new Set();
        waiting.add(canvas);
        this.waiting.set(url, waiting);
        return null;
    }

    // The last drawing of `canvas` asked for the images at `urls`, and for no others. The images
    // it asked for before and no drawing uses any more join the unused ones, of which those
    // beyond the last `kept` are let go.
    public use(canvas: HTMLCanvasElement, urls: Set<string>) {
        const before = this.used.get(canvas) ?? [];
        this.used.set(canvas, urls);

        for (const url of urls) {
            this.unused.delete(url);
        }

        // A page has a few canvases, so asking each is cheap.
        const all = [...this.used.values()];
        for (const url of before) {
            if (!all.some((other) => other.has(url))) {
                this.unused.add(url);
            }
        }

        for (const url of this.unused) {
            if (this.unused.size <= this.kept) {
                break;
            }
            this.unused.delete(url);
            const image = this.images.get(url);
            if (image !== undefined) {
                // Still loading, it would take the canvases waiting for a new image of the URL
                // when it comes, and that one would come to none.
                image.onload = null;
                image.onerror = null;
                this.images.delete(url);
            }
            this.waiting.delete(url);
        }
    }

    public forget(canvas: HTMLCanvasElement) {
        for (const canvases of this.waiting.values()) {
            canvases.delete(canvas);
        }
        this.use(canvas, new Set());
        this.used.delete(canvas);
    }
}

export class Canvases {
    private readonly drawings: WeakMap<HTMLCanvasElement, Uint8Array> = new WeakMap();
    private readonly sizes: WeakMap<HTMLCanvasElement, Size> = new WeakMap();
    // Canvases as big as their bitmaps, drawn a pixel to a CSS pixel, see `paint`.
    private readonly unscaled: WeakSet<HTMLCanvasElement> = new WeakSet();
    // Canvases with a new drawing in the batch being applied, see `paintDrawn`.
    private readonly drawn: Set<HTMLCanvasElement> = new Set();
    // Canvases whose last drawing wrote text, drawn again when a font loads, see `wrote`.
    private readonly writing: Set<HTMLCanvasElement> = new Set();
    private listening = false;
    private observer: ResizeObserver | null = null;
    private readonly images: Images;

    // `kept`: how many images no drawing uses any more stay loaded, see `Images`.
    public constructor(kept = 64) {
        this.images = new Images(kept, (canvas) => this.repaint(canvas));
    }

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

    // Loads the image at `url` for drawings to come, which then have it at once.
    public preload(url: string) {
        this.images.preload(url);
    }

    public forget(node: Element | Comment | Text) {
        if (node instanceof HTMLCanvasElement && this.drawings.has(node)) {
            this.observer?.unobserve(node);
            this.drawings.delete(node);
            this.sizes.delete(node);
            this.unscaled.delete(node);
            this.drawn.delete(node);
            this.writing.delete(node);
            this.images.forget(node);
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

    // A canvas wrote text, or didn't, with its last drawing. Text written before its font came
    // is in another font, so the canvases that wrote some are drawn again when a font loads.
    private wrote(canvas: HTMLCanvasElement, wrote: boolean) {
        if (!wrote) {
            this.writing.delete(canvas);
            return;
        }

        this.writing.add(canvas);

        if (!this.listening && typeof document !== 'undefined') {
            this.listening = true;
            document.fonts.addEventListener('loadingdone', () => {
                for (const writing of [...this.writing]) {
                    this.repaint(writing);
                }
            });
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
            // The images the drawing asks for, kept while it is the last one of the canvas.
            const urls: Set<string> = new Set();
            let wrote = false;
            try {
                wrote = play(context, drawing, size.width, size.height, size.ratio, (url) => {
                    urls.add(url);
                    return this.images.get(url, canvas);
                });
            } finally {
                this.images.use(canvas, urls);
                this.wrote(canvas, wrote);
            }
        }
    }
}
