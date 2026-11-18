//! Drawings for `<canvas>` elements, see [`Drawing`].
//!
//! ## Format
//!
//! The browser reads the bytes in `driver_module/src_js/api/command/canvas.ts`, and
//! `canvas.test.ts` there checks it against bytes made here. Numbers are little-endian.
//!
//! ```text
//! drawing := units op*
//! units   := 0                              -- CSS pixels from the top left corner
//!          | 1 x:f32 y:f32 width:f32 height:f32  -- a view box, see `ViewBox`
//! op      := code:u8 args                   -- see `code`
//! target  := 0 | 1                          -- a style for fills, or for lines and outlines
//! rule    := 0 | 1                          -- nonzero or evenodd, see `FillRule`
//! text    := length:u16 utf8*
//! stops   := count:u16 (offset:f32 text)*   -- of a gradient
//! rect    := x:f32 y:f32 width:f32 height:f32
//! path    := svg:long count:u32 command*    -- see `Path2D`
//! long    := length:u32 utf8*
//! ```
//!
//! A pattern, a gradient or a text is one operation with its `target`, for fills and strokes
//! alike. A keyword, f. ex. of `Composite`, travels as its text, so the browser needs no table
//! of them.

use std::sync::Arc;

mod path;
pub use path::Path2D;

/// Operation codes. Append only, as the browser reads them by number.
mod code {
    pub const FILL_STYLE: u8 = 1;
    pub const STROKE_STYLE: u8 = 2;
    pub const GLOBAL_ALPHA: u8 = 3;
    pub const LINE_WIDTH: u8 = 4;
    pub const LINE_WIDTH_PX: u8 = 5;
    pub const LINE_CAP: u8 = 6;
    pub const FILL_RECT: u8 = 7;
    pub const FILL_CIRCLE: u8 = 8;
    pub const FILL_CIRCLE_PX: u8 = 9;
    pub const STROKE_CIRCLE: u8 = 10;
    pub const POLYLINE: u8 = 11;
    pub const LINE_JOIN: u8 = 12;
    pub const FILL_POLYGON: u8 = 13;
    pub const STROKE_POLYGON: u8 = 14;
    pub const DRAW_IMAGE: u8 = 15;
    pub const PATTERN: u8 = 16;
    pub const LINEAR_GRADIENT: u8 = 17;
    /// A drawing appended: the state of the context is saved, and the drawing starts from its
    /// defaults, until `APPEND_END` restores it.
    pub const APPEND: u8 = 18;
    pub const APPEND_END: u8 = 19;
    pub const SAVE: u8 = 20;
    pub const RESTORE: u8 = 21;
    pub const TRANSLATE: u8 = 22;
    pub const ROTATE: u8 = 23;
    pub const SCALE: u8 = 24;
    pub const TRANSFORM: u8 = 25;
    pub const RESET_TRANSFORM: u8 = 26;
    pub const FILL_PATH: u8 = 27;
    pub const STROKE_PATH: u8 = 28;
    pub const CLIP: u8 = 29;
    pub const STROKE_RECT: u8 = 30;
    pub const CLEAR_RECT: u8 = 31;
    pub const FONT: u8 = 32;
    pub const FONT_PX: u8 = 33;
    pub const TEXT_ALIGN: u8 = 34;
    pub const TEXT_BASELINE: u8 = 35;
    pub const TEXT: u8 = 36;
    pub const RADIAL_GRADIENT: u8 = 37;
    pub const CONIC_GRADIENT: u8 = 38;
    pub const LINE_DASH: u8 = 39;
    pub const MITER_LIMIT: u8 = 40;
    pub const COMPOSITE: u8 = 41;
    pub const SHADOW: u8 = 42;
    pub const FILTER: u8 = 43;
    pub const IMAGE_SMOOTHING: u8 = 44;
    pub const DRAW_PIXELS: u8 = 45;
}

const UNITS_PIXELS: u8 = 0;
const UNITS_VIEW_BOX: u8 = 1;

/// What a pattern or a gradient is for, see `target` in the format.
const FILL: u8 = 0;
const STROKE: u8 = 1;

/// The units of a [`Drawing`]: a rectangle fitted into the canvas as an SVG `viewBox` with
/// `preserveAspectRatio="xMidYMid meet"` is, as big as fits and in the middle. What lies
/// outside the box is drawn too, in the margins it leaves. A box without a width and a height
/// above zero, or with a number that is NaN or infinite, draws nothing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ViewBox {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// A rectangle: a part of an image in its pixels, or where it goes in a drawing's units, see
/// [`Drawing::draw_image`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// How the ends of lines look, as `lineCap` of the 2D context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

/// How lines turn a corner, as `lineJoin` of the 2D context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}

/// Which places a path that crosses itself, or has shapes inside one another, fills: see
/// [`Drawing::fill_path`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FillRule {
    /// Inside, wherever the path doesn't wind around as many times one way as the other: all of
    /// a shape inside another drawn the same way round.
    #[default]
    NonZero,
    /// Inside, wherever an odd number of the path's lines lie between it and the outside: a
    /// shape inside another is a hole in it.
    EvenOdd,
}

/// Where text is from the `x` it is written at, as `textAlign` of the 2D context.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextAlign {
    /// Left of it in a text from left to right, right of it in one from right to left.
    #[default]
    Start,
    End,
    Left,
    Right,
    Center,
}

/// Which line of text the `y` it is written at is, as `textBaseline` of the 2D context.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextBaseline {
    Top,
    Hanging,
    Middle,
    /// The line letters stand on.
    #[default]
    Alphabetic,
    Ideographic,
    Bottom,
}

/// How what is drawn is put together with what is there, as `globalCompositeOperation` of the
/// 2D context: f. ex. [`Composite::Multiply`] to shade, or [`Composite::DestinationOut`] to rub
/// out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Composite {
    /// Over what is there.
    #[default]
    SourceOver,
    SourceIn,
    SourceOut,
    SourceAtop,
    DestinationOver,
    DestinationIn,
    DestinationOut,
    DestinationAtop,
    Lighter,
    Copy,
    Xor,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl Composite {
    fn keyword(self) -> &'static str {
        match self {
            Self::SourceOver => "source-over",
            Self::SourceIn => "source-in",
            Self::SourceOut => "source-out",
            Self::SourceAtop => "source-atop",
            Self::DestinationOver => "destination-over",
            Self::DestinationIn => "destination-in",
            Self::DestinationOut => "destination-out",
            Self::DestinationAtop => "destination-atop",
            Self::Lighter => "lighter",
            Self::Copy => "copy",
            Self::Xor => "xor",
            Self::Multiply => "multiply",
            Self::Screen => "screen",
            Self::Overlay => "overlay",
            Self::Darken => "darken",
            Self::Lighten => "lighten",
            Self::ColorDodge => "color-dodge",
            Self::ColorBurn => "color-burn",
            Self::HardLight => "hard-light",
            Self::SoftLight => "soft-light",
            Self::Difference => "difference",
            Self::Exclusion => "exclusion",
            Self::Hue => "hue",
            Self::Saturation => "saturation",
            Self::Color => "color",
            Self::Luminosity => "luminosity",
        }
    }
}

impl TextAlign {
    fn keyword(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
            Self::Left => "left",
            Self::Right => "right",
            Self::Center => "center",
        }
    }
}

impl TextBaseline {
    fn keyword(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Hanging => "hanging",
            Self::Middle => "middle",
            Self::Alphabetic => "alphabetic",
            Self::Ideographic => "ideographic",
            Self::Bottom => "bottom",
        }
    }
}

/// How images, patterns and pixels are smoothed when they are drawn bigger or smaller than
/// they are, as `imageSmoothingEnabled` and `imageSmoothingQuality` of the 2D context.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageSmoothing {
    /// Not at all: each pixel a sharp square, f. ex. for pixel art.
    Off,
    #[default]
    Low,
    Medium,
    High,
}

/// A picture for a `<canvas>`: operations of its 2D context, in order, recorded in Rust and
/// played back in the browser.
///
/// A canvas gets one through its `draw` attribute in [`dom!`](crate::dom!), as a plain
/// `Drawing`, a [`Value`](crate::Value) or a [`Computed`](crate::Computed):
///
/// ```rust
/// use vertigo::{Drawing, ViewBox, Value, dom};
///
/// let angle = Value::new(0.0f32);
///
/// let drawing = angle.map(|angle| {
///     let mut drawing = Drawing::new(ViewBox::new(-100.0, -100.0, 200.0, 200.0));
///     drawing
///         .fill_style("#ffd27f")
///         .fill_circle(80.0 * angle.cos(), 80.0 * angle.sin(), 10.0);
///     drawing
/// });
///
/// dom! { <canvas style="width: 100%; height: 300px;" draw={drawing} /> };
/// ```
///
/// Every new drawing clears the canvas and is drawn in its place. It travels in the same batch
/// of DOM changes as the rest of the transaction, so it never arrives before its canvas. A
/// drawing equal to the one before is not sent again.
///
/// The canvas takes its size from CSS, and the driver keeps its bitmap as big as that, times
/// `devicePixelRatio`, so the drawing stays sharp. When the size changes, the canvas draws its
/// last drawing again by itself. The driver owns the `width` and `height` of a canvas it draws
/// on. A canvas that CSS gives no size is as big as its bitmap: it keeps the size it has and is
/// drawn a pixel to a CSS pixel.
///
/// Each drawing starts from the context's defaults: black fill and stroke, full opacity, solid
/// lines one unit wide with butt ends and mitered corners, text in `10px sans-serif` of the
/// drawing's units, no shadow or filter, and nothing clipped. Nothing it changes lasts into the
/// next drawing. Clones are cheap and share the operations until one of them draws more, and so
/// does the browser's copy of a drawing on its way there.
#[derive(Clone, Debug, PartialEq)]
pub struct Drawing {
    bytes: Arc<Vec<u8>>,
    /// Saves not restored yet, which a drawing appended to another restores at its end.
    saves: u32,
}

impl Drawing {
    /// A drawing in the units of `view_box`.
    pub fn new(view_box: ViewBox) -> Self {
        let mut bytes = vec![UNITS_VIEW_BOX];
        for value in [view_box.x, view_box.y, view_box.width, view_box.height] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }

        Self {
            bytes: Arc::new(bytes),
            saves: 0,
        }
    }

    /// A drawing in CSS pixels, from the top left corner of the canvas.
    pub fn pixels() -> Self {
        Self {
            bytes: Arc::new(vec![UNITS_PIXELS]),
            saves: 0,
        }
    }

    /// The color shapes are filled with, as CSS takes it: a name, `#rgb`, `rgba(..)` and so on.
    /// A text that is no color leaves the fill style as it was.
    pub fn fill_style(&mut self, color: &str) -> &mut Self {
        self.text(code::FILL_STYLE, color)
    }

    /// What lines and outlines are drawn with, as [`Self::fill_style`].
    pub fn stroke_style(&mut self, color: &str) -> &mut Self {
        self.text(code::STROKE_STYLE, color)
    }

    /// Opacity of everything drawn from now on, from 0 to 1.
    pub fn global_alpha(&mut self, alpha: f32) -> &mut Self {
        self.op(code::GLOBAL_ALPHA, &[alpha])
    }

    /// Width of lines in the drawing's units, so it grows with the drawing.
    pub fn line_width(&mut self, width: f32) -> &mut Self {
        self.op(code::LINE_WIDTH, &[width])
    }

    /// Width of lines in CSS pixels, however big the drawing is shown, like SVG's
    /// `vector-effect: non-scaling-stroke`.
    pub fn line_width_px(&mut self, width: f32) -> &mut Self {
        self.op(code::LINE_WIDTH_PX, &[width])
    }

    /// How the ends of lines look from now on.
    pub fn line_cap(&mut self, cap: LineCap) -> &mut Self {
        let value = match cap {
            LineCap::Butt => 0,
            LineCap::Round => 1,
            LineCap::Square => 2,
        };
        let bytes = self.bytes_mut();
        bytes.push(code::LINE_CAP);
        bytes.push(value);
        self
    }

    /// How lines and outlines turn their corners from now on.
    pub fn line_join(&mut self, join: LineJoin) -> &mut Self {
        let value = match join {
            LineJoin::Miter => 0,
            LineJoin::Round => 1,
            LineJoin::Bevel => 2,
        };
        let bytes = self.bytes_mut();
        bytes.push(code::LINE_JOIN);
        bytes.push(value);
        self
    }

    /// How far a mitered corner may reach out, in line widths, before it is cut flat as a bevel
    /// is: 10 at first. A limit not above zero leaves it as it was.
    pub fn miter_limit(&mut self, limit: f32) -> &mut Self {
        self.op(code::MITER_LIMIT, &[limit])
    }

    /// Dashes lines and outlines from now on: `segments` are the lengths of a dash and a gap in
    /// turn, in the drawing's units, and the dashes start `offset` into them. An odd number of
    /// them goes round twice, and none draws lines whole again. A length below zero, NaN or
    /// infinite leaves the dashes as they were.
    ///
    /// ```rust
    /// use vertigo::{Drawing, ViewBox};
    ///
    /// let mut drawing = Drawing::new(ViewBox::new(0.0, 0.0, 100.0, 100.0));
    /// drawing
    ///     .line_dash(&[4.0, 2.0], 0.0)
    ///     .polyline([(0.0, 50.0), (100.0, 50.0)])
    ///     .line_dash(&[], 0.0)
    ///     .polyline([(0.0, 60.0), (100.0, 60.0)]);
    /// ```
    pub fn line_dash(&mut self, segments: &[f32], offset: f32) -> &mut Self {
        self.dash(false, segments, offset)
    }

    /// As [`Self::line_dash`], with the lengths and the offset in CSS pixels, however big the
    /// drawing is shown.
    pub fn line_dash_px(&mut self, segments: &[f32], offset: f32) -> &mut Self {
        self.dash(true, segments, offset)
    }

    /// The rectangle from (`x`, `y`), `width` wide and `height` high, filled with the fill style.
    pub fn fill_rect(&mut self, x: f32, y: f32, width: f32, height: f32) -> &mut Self {
        self.op(code::FILL_RECT, &[x, y, width, height])
    }

    /// The outline of the rectangle from (`x`, `y`), with the stroke style and the line width.
    pub fn stroke_rect(&mut self, x: f32, y: f32, width: f32, height: f32) -> &mut Self {
        self.op(code::STROKE_RECT, &[x, y, width, height])
    }

    /// The rectangle made transparent again, whatever was drawn there: f. ex. a window in a
    /// layer of fog.
    pub fn clear_rect(&mut self, x: f32, y: f32, width: f32, height: f32) -> &mut Self {
        self.op(code::CLEAR_RECT, &[x, y, width, height])
    }

    /// A disc around (`x`, `y`). A radius below zero draws nothing.
    pub fn fill_circle(&mut self, x: f32, y: f32, radius: f32) -> &mut Self {
        self.op(code::FILL_CIRCLE, &[x, y, radius])
    }

    /// A disc around (`x`, `y`) with its radius in CSS pixels, so it is as big however big the
    /// drawing is shown: a dot.
    pub fn fill_circle_px(&mut self, x: f32, y: f32, radius: f32) -> &mut Self {
        self.op(code::FILL_CIRCLE_PX, &[x, y, radius])
    }

    /// The outline of a circle, with the stroke style and the line width.
    pub fn stroke_circle(&mut self, x: f32, y: f32, radius: f32) -> &mut Self {
        self.op(code::STROKE_CIRCLE, &[x, y, radius])
    }

    /// A line through `points` in order, with the stroke style, the line width and the line
    /// cap. Fewer than two points draw nothing.
    pub fn polyline(&mut self, points: impl IntoIterator<Item = (f32, f32)>) -> &mut Self {
        self.points(code::POLYLINE, points)
    }

    /// The shape `points` enclose, filled with the fill style. The last point joins the first.
    /// Fewer than three points draw nothing.
    pub fn fill_polygon(&mut self, points: impl IntoIterator<Item = (f32, f32)>) -> &mut Self {
        self.points(code::FILL_POLYGON, points)
    }

    /// The closed outline through `points`, with the stroke style, the line width and the line
    /// join, which also turns the corner where the last point joins the first. Fewer than three
    /// points draw nothing.
    pub fn stroke_polygon(&mut self, points: impl IntoIterator<Item = (f32, f32)>) -> &mut Self {
        self.points(code::STROKE_POLYGON, points)
    }

    /// The shapes of `path` filled with the fill style, inside them as `rule` says: curves,
    /// arcs and holes, which a polygon has not.
    pub fn fill_path(&mut self, path: &Path2D, rule: FillRule) -> &mut Self {
        let bytes = self.bytes_mut();
        bytes.extend_from_slice(&[code::FILL_PATH, rule as u8]);
        path.write(bytes);
        self
    }

    /// The lines of `path`, with the stroke style, the line width, the cap, the join and the
    /// dashes.
    pub fn stroke_path(&mut self, path: &Path2D) -> &mut Self {
        let bytes = self.bytes_mut();
        bytes.push(code::STROKE_PATH);
        path.write(bytes);
        self
    }

    /// Draws from now on only inside the shapes of `path`, as `rule` says, and inside any clip
    /// before it, until a [`Self::restore`] takes it off: f. ex. a picture in a round frame.
    ///
    /// ```rust
    /// use vertigo::{Drawing, FillRule, Path2D, Rect, ViewBox};
    ///
    /// let mut frame = Path2D::new();
    /// frame.circle(50.0, 50.0, 40.0);
    ///
    /// let mut drawing = Drawing::new(ViewBox::new(0.0, 0.0, 100.0, 100.0));
    /// drawing
    ///     .save()
    ///     .clip(&frame, FillRule::NonZero)
    ///     .draw_image(
    ///         "face.webp",
    ///         Rect::new(0.0, 0.0, 256.0, 256.0),
    ///         Rect::new(10.0, 10.0, 80.0, 80.0),
    ///     )
    ///     .restore()
    ///     .stroke_path(&frame);
    /// ```
    pub fn clip(&mut self, path: &Path2D, rule: FillRule) -> &mut Self {
        let bytes = self.bytes_mut();
        bytes.extend_from_slice(&[code::CLIP, rule as u8]);
        path.write(bytes);
        self
    }

    /// The font of text from now on, as CSS writes it: `"bold 12px Georgia, serif"`, with its
    /// size in the drawing's units, so it grows with the drawing. `10px sans-serif` at first. A
    /// text that is no font leaves it as it was.
    pub fn font(&mut self, font: &str) -> &mut Self {
        self.text(code::FONT, font)
    }

    /// As [`Self::font`], with its size in CSS pixels, however big the drawing is shown: f. ex.
    /// the labels of a map that zooms.
    ///
    /// ```rust
    /// use vertigo::{Drawing, TextAlign, TextBaseline, ViewBox};
    ///
    /// let mut drawing = Drawing::new(ViewBox::new(0.0, 0.0, 1.0, 1.0));
    /// drawing
    ///     .font_px("bold 14px sans-serif")
    ///     .text_align(TextAlign::Center)
    ///     .text_baseline(TextBaseline::Middle)
    ///     .fill_text("Warszawa", 0.5, 0.5);
    /// ```
    pub fn font_px(&mut self, font: &str) -> &mut Self {
        self.text(code::FONT_PX, font)
    }

    /// Where text is from the `x` it is written at, from now on.
    pub fn text_align(&mut self, align: TextAlign) -> &mut Self {
        self.text(code::TEXT_ALIGN, align.keyword())
    }

    /// Which line of text the `y` it is written at is, from now on.
    pub fn text_baseline(&mut self, baseline: TextBaseline) -> &mut Self {
        self.text(code::TEXT_BASELINE, baseline.keyword())
    }

    /// `text` at (`x`, `y`), in the font, filled with the fill style. It is drawn as big as the
    /// font says however small the drawing's units are, where a browser would leave a font
    /// below a pixel out.
    pub fn fill_text(&mut self, text: &str, x: f32, y: f32) -> &mut Self {
        self.write(FILL, text, x, y)
    }

    /// The outlines of the letters of `text` at (`x`, `y`), with the stroke style and the line
    /// width, as [`Self::fill_text`] fills them.
    pub fn stroke_text(&mut self, text: &str, x: f32, y: f32) -> &mut Self {
        self.write(STROKE, text, x, y)
    }

    /// The `part` of the image at `url`, in the image's pixels, drawn into `target`, in the
    /// drawing's units, with the global alpha: f. ex. one picture out of many in an atlas.
    ///
    /// `url` is read as the `src` of an `<img>`, so a relative one is relative to the page. The
    /// browser loads an image once, the first time a drawing asks for it, and the canvas draws
    /// its last drawing again when the image comes. Until then, and if it fails to load, the
    /// drawing goes on without it.
    ///
    /// The browser keeps an image while the last drawing of a canvas uses it, and of the images
    /// no drawing uses any more, the last 64: an animation going through a few, or a map scrolled
    /// back, has them at once. An older one is let go, so a map that has shown thousands of tiles
    /// doesn't hold them all, and loads again when a drawing asks for it.
    pub fn draw_image(&mut self, url: &str, part: Rect, target: Rect) -> &mut Self {
        self.text(code::DRAW_IMAGE, url);
        let bytes = self.bytes_mut();
        for rect in [part, target] {
            for value in [rect.x, rect.y, rect.width, rect.height] {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        self
    }

    /// An image made of `pixels`, drawn into `target` in the drawing's units, with the global
    /// alpha and the image smoothing: f. ex. a heat map worked out in Rust. The pixels are rows
    /// of `width` from the top, each pixel four bytes, red, green, blue and alpha. Pixels of
    /// another length than `width` × `height` × 4 draw nothing.
    ///
    /// ```rust
    /// use vertigo::{Drawing, ImageSmoothing, Rect, ViewBox};
    ///
    /// // Two pixels, red and blue, as two sharp squares.
    /// let pixels = [255, 0, 0, 255, 0, 0, 255, 255];
    ///
    /// let mut drawing = Drawing::new(ViewBox::new(0.0, 0.0, 2.0, 1.0));
    /// drawing
    ///     .image_smoothing(ImageSmoothing::Off)
    ///     .draw_pixels(&pixels, 2, 1, Rect::new(0.0, 0.0, 2.0, 1.0));
    /// ```
    pub fn draw_pixels(
        &mut self,
        pixels: &[u8],
        width: u32,
        height: u32,
        target: Rect,
    ) -> &mut Self {
        let length = (width as usize)
            .checked_mul(height as usize)
            .and_then(|count| count.checked_mul(4));
        if length == Some(pixels.len()) && !pixels.is_empty() {
            let bytes = self.bytes_mut();
            bytes.push(code::DRAW_PIXELS);
            bytes.extend_from_slice(&width.to_le_bytes());
            bytes.extend_from_slice(&height.to_le_bytes());
            for value in [target.x, target.y, target.width, target.height] {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            bytes.extend_from_slice(pixels);
        }
        self
    }

    /// Fills shapes from now on with the image at `url` repeated, each of its pixels `scale` of
    /// the drawing's units wide: f. ex. a seamless texture tile. The pattern lies on the
    /// drawing, starting at its point (0, 0), so it moves and grows with the view box, and goes
    /// on unbroken from a shape to the one beside it. Each shape is smoothed along its edges on
    /// its own, though, so a hairline of what lies beneath can show where two meet: paint a
    /// similar color beneath them first.
    ///
    /// The image loads as for [`Self::draw_image`]. Until it comes, if it fails to load, and with
    /// a `scale` of 0, NaN or infinity, with which a pattern would fill nothing, the fill style
    /// stays what it was, so a color set before stands in for the pattern:
    ///
    /// ```rust
    /// use vertigo::{Drawing, ViewBox};
    ///
    /// let mut drawing = Drawing::new(ViewBox::new(0.0, 0.0, 100.0, 100.0));
    /// drawing
    ///     .fill_style("#8db360")
    ///     .fill_pattern("grass.webp", 0.25)
    ///     .fill_polygon([(0.0, 50.0), (50.0, 25.0), (100.0, 50.0), (50.0, 75.0)]);
    /// ```
    pub fn fill_pattern(&mut self, url: &str, scale: f32) -> &mut Self {
        self.pattern(FILL, url, scale)
    }

    /// Strokes lines and outlines from now on with the image at `url` repeated, as
    /// [`Self::fill_pattern`] fills shapes with it: f. ex. a road of cobbles. Until the image
    /// comes, if it fails to load, and with a `scale` of 0, NaN or infinity, the stroke style
    /// stays what it was.
    pub fn stroke_pattern(&mut self, url: &str, scale: f32) -> &mut Self {
        self.pattern(STROKE, url, scale)
    }

    /// Fills shapes from now on with a linear gradient from `start` to `end`, in the drawing's
    /// units, so it moves and grows with the view box. Each of `stops` is an offset along the
    /// way, from 0 at `start` to 1 at `end`, and the color there, as [`Self::fill_style`] takes
    /// it. Between the stops the colors blend, before the first and after the last the color of
    /// that stop goes on. Along each line across the way the color is the same.
    ///
    /// A triangle shaded evenly between two colors is one gradient of them, which puts each
    /// corner at the offset of its shade. It starts at a corner of offset 0 but seldom ends at
    /// the one of offset 1, as it runs across the lines of equal shade: `gradient_end` below
    /// finds where. Triangles sharing an edge then agree on its colors, so a mesh of them is
    /// shaded smoothly:
    ///
    /// ```rust
    /// use vertigo::{Drawing, ViewBox};
    ///
    /// // Where the gradient over the triangle `a`, `b`, `c` ends, with `a` at offset 0, `b` at
    /// // `t` and `c` at 1: at `a + g / |g|²`, where `g · (b - a) = t` and `g · (c - a) = 1`.
    /// // `None` for a triangle with no area.
    /// fn gradient_end(a: (f32, f32), b: (f32, f32), c: (f32, f32), t: f32) -> Option<(f32, f32)> {
    ///     let (ab, ac) = ((b.0 - a.0, b.1 - a.1), (c.0 - a.0, c.1 - a.1));
    ///     let cross = ab.0 * ac.1 - ab.1 * ac.0;
    ///     let g = ((t * ac.1 - ab.1) / cross, (ab.0 - t * ac.0) / cross);
    ///     let squared = g.0 * g.0 + g.1 * g.1;
    ///     let end = (a.0 + g.0 / squared, a.1 + g.1 / squared);
    ///     (end.0.is_finite() && end.1.is_finite()).then_some(end)
    /// }
    ///
    /// // Light at the top left, dark at the top right, halfway between at the bottom left.
    /// let (a, b, c) = ((0.0, 0.0), (0.0, 100.0), (100.0, 0.0));
    /// let mut drawing = Drawing::new(ViewBox::new(0.0, 0.0, 100.0, 100.0));
    /// if let Some(end) = gradient_end(a, b, c, 0.5) {
    ///     // (80, 40), not the dark corner
    ///     drawing
    ///         .fill_linear_gradient(a, end, &[(0.0, "#8db360"), (1.0, "#2d4a1e")])
    ///         .fill_polygon([a, b, c]);
    /// }
    /// ```
    ///
    /// Each shape is smoothed along its edges on its own, though, so a hairline of what lies
    /// beneath can show where two meet: paint a similar color beneath the mesh first.
    ///
    /// A stop whose offset is outside 0 to 1 counts as the nearer end, and one whose offset is
    /// NaN or that has no color is left out. Without stops the gradient is transparent and fills
    /// nothing; more than `u16::MAX` are cut. When `start` and `end` are one point, shapes are
    /// filled with the color of the last stop, as in SVG, where a canvas would fill nothing. An
    /// end that is NaN or infinite leaves the fill style as it was.
    pub fn fill_linear_gradient(
        &mut self,
        start: (f32, f32),
        end: (f32, f32),
        stops: &[(f32, &str)],
    ) -> &mut Self {
        self.linear_gradient(FILL, start, end, stops)
    }

    /// Strokes lines and outlines from now on with a linear gradient from `start` to `end`, as
    /// [`Self::fill_linear_gradient`] fills shapes with one. An end that is NaN or infinite
    /// leaves the stroke style as it was.
    pub fn stroke_linear_gradient(
        &mut self,
        start: (f32, f32),
        end: (f32, f32),
        stops: &[(f32, &str)],
    ) -> &mut Self {
        self.linear_gradient(STROKE, start, end, stops)
    }

    /// Fills shapes from now on with a radial gradient, from the circle around `start` to the
    /// one around `end`, in the drawing's units: f. ex. a glow, from a point to a circle around
    /// it. The offsets of `stops` go from 0 on the first circle to 1 on the second, as those of
    /// [`Self::fill_linear_gradient`] go along its way, and so do the rules for them. When the
    /// two circles are one, shapes are filled with the color of the last stop. A radius below
    /// zero, or a number that is NaN or infinite, leaves the fill style as it was.
    ///
    /// ```rust
    /// use vertigo::{Drawing, ViewBox};
    ///
    /// let mut drawing = Drawing::new(ViewBox::new(0.0, 0.0, 100.0, 100.0));
    /// drawing
    ///     .fill_radial_gradient(
    ///         (50.0, 50.0),
    ///         0.0,
    ///         (50.0, 50.0),
    ///         40.0,
    ///         &[(0.0, "#ffd27f"), (1.0, "rgba(255, 210, 127, 0)")],
    ///     )
    ///     .fill_circle(50.0, 50.0, 40.0);
    /// ```
    pub fn fill_radial_gradient(
        &mut self,
        start: (f32, f32),
        start_radius: f32,
        end: (f32, f32),
        end_radius: f32,
        stops: &[(f32, &str)],
    ) -> &mut Self {
        let geometry = [start.0, start.1, start_radius, end.0, end.1, end_radius];
        self.gradient(code::RADIAL_GRADIENT, FILL, &geometry, stops)
    }

    /// Strokes lines and outlines from now on with a radial gradient, as
    /// [`Self::fill_radial_gradient`] fills shapes with one.
    pub fn stroke_radial_gradient(
        &mut self,
        start: (f32, f32),
        start_radius: f32,
        end: (f32, f32),
        end_radius: f32,
        stops: &[(f32, &str)],
    ) -> &mut Self {
        let geometry = [start.0, start.1, start_radius, end.0, end.1, end_radius];
        self.gradient(code::RADIAL_GRADIENT, STROKE, &geometry, stops)
    }

    /// Fills shapes from now on with a conic gradient around `center`: the colors of `stops`
    /// go round it clockwise, from 0 at the angle `angle`, in radians from the right, to 1 a
    /// full turn later, f. ex. a pie or a color wheel. The rules for the stops are those of
    /// [`Self::fill_linear_gradient`]. A number that is NaN or infinite, or a browser too old for
    /// conic gradients, leaves the fill style as it was.
    pub fn fill_conic_gradient(
        &mut self,
        center: (f32, f32),
        angle: f32,
        stops: &[(f32, &str)],
    ) -> &mut Self {
        let geometry = [angle, center.0, center.1];
        self.gradient(code::CONIC_GRADIENT, FILL, &geometry, stops)
    }

    /// Strokes lines and outlines from now on with a conic gradient, as
    /// [`Self::fill_conic_gradient`] fills shapes with one.
    pub fn stroke_conic_gradient(
        &mut self,
        center: (f32, f32),
        angle: f32,
        stops: &[(f32, &str)],
    ) -> &mut Self {
        let geometry = [angle, center.0, center.1];
        self.gradient(code::CONIC_GRADIENT, STROKE, &geometry, stops)
    }

    /// How what is drawn from now on is put together with what is there already.
    pub fn composite(&mut self, composite: Composite) -> &mut Self {
        self.text(code::COMPOSITE, composite.keyword())
    }

    /// A shadow under what is drawn from now on, of `color`, blurred by `blur` and moved by
    /// `offset_x` and `offset_y`, all three in CSS pixels, as CSS `box-shadow` has them. A
    /// transparent color, as at first, draws no shadow.
    pub fn shadow(&mut self, color: &str, blur: f32, offset_x: f32, offset_y: f32) -> &mut Self {
        self.text(code::SHADOW, color);
        let bytes = self.bytes_mut();
        for value in [blur, offset_x, offset_y] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        self
    }

    /// CSS filters over what is drawn from now on: f. ex. `"blur(4px) grayscale(1)"`, with
    /// lengths in CSS pixels. `"none"`, as at first, takes them off. A browser without filters
    /// on a canvas, as Safari before 18, draws without them.
    pub fn filter(&mut self, filter: &str) -> &mut Self {
        self.text(code::FILTER, filter)
    }

    /// How images, patterns and pixels drawn from now on are smoothed.
    pub fn image_smoothing(&mut self, smoothing: ImageSmoothing) -> &mut Self {
        let bytes = self.bytes_mut();
        bytes.extend_from_slice(&[code::IMAGE_SMOOTHING, smoothing as u8]);
        self
    }

    /// Saves the styles, the transform and the clip, for the next [`Self::restore`] to bring
    /// back.
    pub fn save(&mut self) -> &mut Self {
        self.saves += 1;
        self.bytes_mut().push(code::SAVE);
        self
    }

    /// Brings back the styles, the transform and the clip of the last save not restored yet. A
    /// restore without one does nothing.
    pub fn restore(&mut self) -> &mut Self {
        if self.saves > 0 {
            self.saves -= 1;
            self.bytes_mut().push(code::RESTORE);
        }
        self
    }

    /// Moves what is drawn from now on by `x` and `y` units.
    pub fn translate(&mut self, x: f32, y: f32) -> &mut Self {
        self.op(code::TRANSLATE, &[x, y])
    }

    /// Turns what is drawn from now on by `angle`, in radians clockwise, around the point
    /// (0, 0) of the units at the time.
    pub fn rotate(&mut self, angle: f32) -> &mut Self {
        self.op(code::ROTATE, &[angle])
    }

    /// Makes what is drawn from now on `x` times wider and `y` times higher, from the point
    /// (0, 0) of the units at the time. Sizes in CSS pixels stay as they are.
    pub fn scale(&mut self, x: f32, y: f32) -> &mut Self {
        self.op(code::SCALE, &[x, y])
    }

    /// Transforms what is drawn from now on by the matrix `[a c e; b d f; 0 0 1]`, on top of
    /// the transform there is, as `transform` of the 2D context.
    pub fn transform(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> &mut Self {
        self.op(code::TRANSFORM, &[a, b, c, d, e, f])
    }

    /// Takes off every transform: what is drawn from now on is in the drawing's units again.
    pub fn reset_transform(&mut self) -> &mut Self {
        self.bytes_mut().push(code::RESET_TRANSFORM);
        self
    }

    /// Draws `other` after what this drawing has drawn, as it would be drawn alone: from the
    /// context's defaults, and leaving the styles of what comes after it as they were. It is
    /// drawn in the units of this drawing, its own are left out, so the two are best made in the
    /// same. The transform and the clip at the time hold for it, so a part drawn once can be
    /// put in many places, each moved by [`Self::translate`] between a save and a restore.
    ///
    /// A part that changes seldom, f. ex. a layer of a map, kept as a drawing of its own, is put
    /// together with the others this way without being recorded again:
    ///
    /// ```rust
    /// use vertigo::{Drawing, ViewBox};
    ///
    /// const VIEW: ViewBox = ViewBox::new(0.0, 0.0, 100.0, 100.0);
    ///
    /// let mut terrain = Drawing::new(VIEW);
    /// terrain.fill_style("#8db360").fill_rect(0.0, 0.0, 100.0, 100.0);
    ///
    /// let mut frame = Drawing::new(VIEW);
    /// frame
    ///     .fill_style("#c33")
    ///     .append(&terrain)
    ///     // Red: the green of the terrain stays in it.
    ///     .fill_circle(50.0, 50.0, 5.0);
    /// ```
    pub fn append(&mut self, other: &Drawing) -> &mut Self {
        let ops = other.ops();
        if !ops.is_empty() {
            let bytes = self.bytes_mut();
            bytes.push(code::APPEND);
            bytes.extend_from_slice(ops);
            // Saves it left open close with it, so the end brings back what was before it.
            bytes.extend(std::iter::repeat_n(code::RESTORE, other.saves as usize));
            bytes.push(code::APPEND_END);
        }
        self
    }

    /// The drawing as the browser reads it.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// The bytes, shared with the command that takes them to the browser rather than copied.
    pub(crate) fn into_shared_bytes(self) -> Arc<Vec<u8>> {
        self.bytes
    }

    /// The operations, without the units in front of them.
    fn ops(&self) -> &[u8] {
        let units = if self.bytes.first() == Some(&UNITS_VIEW_BOX) {
            1 + 4 * 4
        } else {
            1
        };
        &self.bytes[units..]
    }

    fn pattern(&mut self, target: u8, url: &str, scale: f32) -> &mut Self {
        self.bytes_mut().extend_from_slice(&[code::PATTERN, target]);
        self.text_bytes(url);
        self.bytes_mut().extend_from_slice(&scale.to_le_bytes());
        self
    }

    fn linear_gradient(
        &mut self,
        target: u8,
        start: (f32, f32),
        end: (f32, f32),
        stops: &[(f32, &str)],
    ) -> &mut Self {
        let geometry = [start.0, start.1, end.0, end.1];
        self.gradient(code::LINEAR_GRADIENT, target, &geometry, stops)
    }

    fn gradient(
        &mut self,
        code: u8,
        target: u8,
        geometry: &[f32],
        stops: &[(f32, &str)],
    ) -> &mut Self {
        let bytes = self.bytes_mut();
        bytes.extend_from_slice(&[code, target]);
        for value in geometry {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        self.stops(stops)
    }

    fn dash(&mut self, px: bool, segments: &[f32], offset: f32) -> &mut Self {
        let segments = &segments[..segments.len().min(u16::MAX as usize)];
        let bytes = self.bytes_mut();
        bytes.extend_from_slice(&[code::LINE_DASH, u8::from(px)]);
        bytes.extend_from_slice(&(segments.len() as u16).to_le_bytes());
        for value in segments.iter().chain([&offset]) {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        self
    }

    fn write(&mut self, target: u8, text: &str, x: f32, y: f32) -> &mut Self {
        self.bytes_mut().extend_from_slice(&[code::TEXT, target]);
        self.text_bytes(text);
        let bytes = self.bytes_mut();
        for value in [x, y] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        self
    }

    /// The stops of a gradient, as many as two bytes count.
    fn stops(&mut self, stops: &[(f32, &str)]) -> &mut Self {
        let stops = &stops[..stops.len().min(u16::MAX as usize)];
        self.bytes_mut()
            .extend_from_slice(&(stops.len() as u16).to_le_bytes());
        for (offset, color) in stops {
            self.bytes_mut().extend_from_slice(&offset.to_le_bytes());
            self.text_bytes(color);
        }
        self
    }

    fn points(&mut self, code: u8, points: impl IntoIterator<Item = (f32, f32)>) -> &mut Self {
        let bytes = self.bytes_mut();
        bytes.push(code);
        let count_at = bytes.len();
        bytes.extend_from_slice(&0u32.to_le_bytes());

        let mut count: u32 = 0;
        for (x, y) in points {
            bytes.extend_from_slice(&x.to_le_bytes());
            bytes.extend_from_slice(&y.to_le_bytes());
            count += 1;
        }
        bytes[count_at..count_at + 4].copy_from_slice(&count.to_le_bytes());
        self
    }

    fn op(&mut self, code: u8, args: &[f32]) -> &mut Self {
        let bytes = self.bytes_mut();
        bytes.push(code);
        for arg in args {
            bytes.extend_from_slice(&arg.to_le_bytes());
        }
        self
    }

    fn text(&mut self, code: u8, value: &str) -> &mut Self {
        self.bytes_mut().push(code);
        self.text_bytes(value);
        self
    }

    /// A style or a URL is a text, which none needs to be longer than its two length bytes
    /// allow; a longer one is cut at a character boundary.
    fn text_bytes(&mut self, value: &str) {
        let mut end = value.len().min(u16::MAX as usize);
        while !value.is_char_boundary(end) {
            end -= 1;
        }
        let bytes = self.bytes_mut();
        bytes.extend_from_slice(&(end as u16).to_le_bytes());
        bytes.extend_from_slice(&value.as_bytes()[..end]);
    }

    fn bytes_mut(&mut self) -> &mut Vec<u8> {
        Arc::make_mut(&mut self.bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shared with `canvas.test.ts`, which plays these bytes back on a recording context.
    /// Change the format and one of the two tests fails, whichever side was changed.
    pub(crate) fn fixture() -> Drawing {
        // In pixels, which the drawing it is appended to leaves out, and with a save left open,
        // which the append closes.
        let mut appended = Drawing::pixels();
        appended.save().fill_rect(1.0, 1.0, 1.0, 1.0);

        // Every command, after SVG data.
        let mut path = Path2D::svg("M0 0 H1");
        path.move_to(0.0, 0.0)
            .line_to(1.0, 0.0)
            .quad_to(1.0, 1.0, 0.0, 1.0)
            .cubic_to(0.0, 2.0, 1.0, 2.0, 1.0, 3.0)
            .arc_to(2.0, 3.0, 2.0, 4.0, 0.5)
            .ellipse((1.0, 1.0), (2.0, 1.0), 0.5, 0.0, 1.0, true)
            .close();

        let mut drawing = Drawing::new(ViewBox::new(-8.0, -5.0, 16.0, 10.0));
        drawing
            .fill_style("#ffd27f")
            .stroke_style("żółć")
            .global_alpha(0.5)
            .line_width(2.0)
            .line_width_px(1.5)
            .line_cap(LineCap::Round)
            .line_join(LineJoin::Bevel)
            .fill_rect(-8.0, -5.0, 16.0, 10.0)
            .fill_circle(1.0, 2.0, 3.0)
            .fill_circle_px(-1.0, -2.0, 4.0)
            .stroke_circle(0.0, 0.0, 6.0)
            .polyline([(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)])
            .fill_polygon([(0.0, -1.0), (2.0, 0.0), (0.0, 1.0), (-2.0, 0.0)])
            .stroke_polygon([(0.0, -1.0), (2.0, 0.0), (0.0, 1.0)])
            .draw_image(
                "cat.png",
                Rect::new(1.0, 2.0, 3.0, 4.0),
                Rect::new(-1.0, -2.0, 2.0, 1.0),
            )
            .fill_pattern("cat.png", 0.5)
            .stroke_pattern("cat.png", 2.0)
            .fill_rect(0.0, 0.0, 2.0, 2.0)
            .fill_linear_gradient((0.0, -1.0), (2.0, 1.0), &[(0.0, "red"), (1.0, "#00f")])
            .stroke_linear_gradient((0.0, 0.0), (0.0, 1.0), &[(0.5, "lime")])
            .fill_rect(0.0, 0.0, 2.0, 2.0)
            .append(&appended)
            .save()
            .translate(1.0, 2.0)
            .rotate(0.5)
            .scale(2.0, 2.0)
            .transform(1.0, 0.0, 0.0, 1.0, 3.0, 4.0)
            .line_width_px(1.0)
            .reset_transform()
            .restore()
            // One too many: nothing.
            .restore()
            .fill_path(&path, FillRule::EvenOdd)
            .stroke_path(&path)
            .clip(&path, FillRule::NonZero)
            .stroke_rect(0.0, 0.0, 1.0, 1.0)
            .clear_rect(0.0, 0.0, 1.0, 1.0)
            .font("bold 2px serif")
            .font_px("12px serif")
            .text_align(TextAlign::Center)
            .text_baseline(TextBaseline::Middle)
            .fill_text("Hi", 1.0, 2.0)
            .stroke_text("Hi", 1.0, 2.0)
            .fill_radial_gradient((0.0, 0.0), 0.0, (0.0, 0.0), 1.0, &[(0.0, "red")])
            .stroke_conic_gradient((1.0, 1.0), 0.5, &[(0.0, "red"), (1.0, "blue")])
            .line_dash(&[1.0, 2.0], 0.5)
            .line_dash_px(&[4.0], 2.0)
            .miter_limit(4.0)
            .composite(Composite::Multiply)
            .shadow("black", 2.0, 1.0, -1.0)
            .filter("blur(2px)")
            .image_smoothing(ImageSmoothing::Off)
            .draw_pixels(&[255, 0, 0, 255], 1, 1, Rect::new(0.0, 0.0, 1.0, 1.0))
            // Left open: the end of the drawing restores it.
            .save();
        drawing
    }

    const FIXTURE: &[u8] = &[
        1, 0, 0, 0, 193, 0, 0, 160, 192, 0, 0, 128, 65, 0, 0, 32, 65, 1, 7, 0, 35, 102, 102, 100,
        50, 55, 102, 2, 8, 0, 197, 188, 195, 179, 197, 130, 196, 135, 3, 0, 0, 0, 63, 4, 0, 0, 0,
        64, 5, 0, 0, 192, 63, 6, 1, 12, 2, 7, 0, 0, 0, 193, 0, 0, 160, 192, 0, 0, 128, 65, 0, 0,
        32, 65, 8, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 9, 0, 0, 128, 191, 0, 0, 0, 192, 0, 0,
        128, 64, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192, 64, 11, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 128, 63, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 0, 0, 13, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        128, 191, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 63, 0, 0, 0, 192, 0, 0, 0, 0, 14,
        3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 191, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 63,
        15, 7, 0, 99, 97, 116, 46, 112, 110, 103, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 0, 0,
        128, 64, 0, 0, 128, 191, 0, 0, 0, 192, 0, 0, 0, 64, 0, 0, 128, 63, 16, 0, 7, 0, 99, 97,
        116, 46, 112, 110, 103, 0, 0, 0, 63, 16, 1, 7, 0, 99, 97, 116, 46, 112, 110, 103, 0, 0, 0,
        64, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 64, 17, 0, 0, 0, 0, 0, 0, 0, 128, 191,
        0, 0, 0, 64, 0, 0, 128, 63, 2, 0, 0, 0, 0, 0, 3, 0, 114, 101, 100, 0, 0, 128, 63, 4, 0, 35,
        48, 48, 102, 17, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 63, 1, 0, 0, 0, 0, 63,
        4, 0, 108, 105, 109, 101, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 64, 18, 20, 7,
        0, 0, 128, 63, 0, 0, 128, 63, 0, 0, 128, 63, 0, 0, 128, 63, 21, 19, 20, 22, 0, 0, 128, 63,
        0, 0, 0, 64, 23, 0, 0, 0, 63, 24, 0, 0, 0, 64, 0, 0, 0, 64, 25, 0, 0, 128, 63, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 128, 63, 0, 0, 64, 64, 0, 0, 128, 64, 5, 0, 0, 128, 63, 26, 21, 27, 1, 7,
        0, 0, 0, 77, 48, 32, 48, 32, 72, 49, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 128,
        63, 0, 0, 0, 0, 2, 0, 0, 128, 63, 0, 0, 128, 63, 0, 0, 0, 0, 0, 0, 128, 63, 3, 0, 0, 0, 0,
        0, 0, 0, 64, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 128, 63, 0, 0, 64, 64, 4, 0, 0, 0, 64, 0, 0,
        64, 64, 0, 0, 0, 64, 0, 0, 128, 64, 0, 0, 0, 63, 5, 0, 0, 128, 63, 0, 0, 128, 63, 0, 0, 0,
        64, 0, 0, 128, 63, 0, 0, 0, 63, 0, 0, 0, 0, 0, 0, 128, 63, 1, 6, 28, 7, 0, 0, 0, 77, 48,
        32, 48, 32, 72, 49, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 128, 63, 0, 0, 0, 0, 2,
        0, 0, 128, 63, 0, 0, 128, 63, 0, 0, 0, 0, 0, 0, 128, 63, 3, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0,
        128, 63, 0, 0, 0, 64, 0, 0, 128, 63, 0, 0, 64, 64, 4, 0, 0, 0, 64, 0, 0, 64, 64, 0, 0, 0,
        64, 0, 0, 128, 64, 0, 0, 0, 63, 5, 0, 0, 128, 63, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 128,
        63, 0, 0, 0, 63, 0, 0, 0, 0, 0, 0, 128, 63, 1, 6, 29, 0, 7, 0, 0, 0, 77, 48, 32, 48, 32,
        72, 49, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 128, 63, 0, 0, 0, 0, 2, 0, 0, 128,
        63, 0, 0, 128, 63, 0, 0, 0, 0, 0, 0, 128, 63, 3, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 128, 63, 0,
        0, 0, 64, 0, 0, 128, 63, 0, 0, 64, 64, 4, 0, 0, 0, 64, 0, 0, 64, 64, 0, 0, 0, 64, 0, 0,
        128, 64, 0, 0, 0, 63, 5, 0, 0, 128, 63, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 128, 63, 0, 0, 0,
        63, 0, 0, 0, 0, 0, 0, 128, 63, 1, 6, 30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 63, 0, 0, 128,
        63, 31, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 63, 0, 0, 128, 63, 32, 14, 0, 98, 111, 108, 100,
        32, 50, 112, 120, 32, 115, 101, 114, 105, 102, 33, 10, 0, 49, 50, 112, 120, 32, 115, 101,
        114, 105, 102, 34, 6, 0, 99, 101, 110, 116, 101, 114, 35, 6, 0, 109, 105, 100, 100, 108,
        101, 36, 0, 2, 0, 72, 105, 0, 0, 128, 63, 0, 0, 0, 64, 36, 1, 2, 0, 72, 105, 0, 0, 128, 63,
        0, 0, 0, 64, 37, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128,
        63, 1, 0, 0, 0, 0, 0, 3, 0, 114, 101, 100, 38, 1, 0, 0, 0, 63, 0, 0, 128, 63, 0, 0, 128,
        63, 2, 0, 0, 0, 0, 0, 3, 0, 114, 101, 100, 0, 0, 128, 63, 4, 0, 98, 108, 117, 101, 39, 0,
        2, 0, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 0, 63, 39, 1, 1, 0, 0, 0, 128, 64, 0, 0, 0, 64, 40,
        0, 0, 128, 64, 41, 8, 0, 109, 117, 108, 116, 105, 112, 108, 121, 42, 5, 0, 98, 108, 97, 99,
        107, 0, 0, 0, 64, 0, 0, 128, 63, 0, 0, 128, 191, 43, 9, 0, 98, 108, 117, 114, 40, 50, 112,
        120, 41, 44, 0, 45, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 63, 0, 0,
        128, 63, 255, 0, 0, 255, 20,
    ];

    #[test]
    fn matches_the_cross_language_fixture() {
        assert_eq!(
            fixture().as_bytes(),
            FIXTURE,
            "the drawing format changed - update canvas.test.ts to match, or this is a bug"
        );
    }

    #[test]
    fn a_drawing_in_pixels_has_no_view_box() {
        let mut drawing = Drawing::pixels();
        drawing.fill_circle(1.0, 2.0, 3.0);

        assert_eq!(drawing.as_bytes()[0], UNITS_PIXELS);
        assert_eq!(drawing.as_bytes().len(), 1 + 1 + 12);
    }

    #[test]
    fn drawings_are_equal_when_they_draw_the_same() {
        let draw = || {
            let mut drawing = Drawing::pixels();
            drawing.fill_style("red").fill_circle(1.0, 2.0, 3.0);
            drawing
        };
        let mut moved = draw();
        moved.fill_circle(1.0, 2.0, 3.0);

        assert_eq!(draw(), draw());
        assert_ne!(draw(), moved);
    }

    #[test]
    fn a_clone_draws_on_alone() {
        let mut drawing = Drawing::pixels();
        drawing.fill_circle(1.0, 2.0, 3.0);
        let kept = drawing.clone();

        drawing.fill_circle(4.0, 5.0, 6.0);

        assert_eq!(kept.as_bytes().len(), 1 + 13);
        assert_eq!(drawing.as_bytes().len(), 1 + 13 + 13);
    }

    #[test]
    fn a_polyline_counts_its_points() {
        let mut drawing = Drawing::pixels();
        drawing.polyline((0..300).map(|step| (step as f32, 0.0)));

        let bytes = drawing.as_bytes();
        assert_eq!(bytes[1], code::POLYLINE);
        assert_eq!(
            u32::from_le_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]),
            300
        );
        assert_eq!(bytes.len(), 1 + 1 + 4 + 300 * 8);
    }

    #[test]
    fn an_image_takes_its_url_and_two_rectangles() {
        let mut drawing = Drawing::pixels();
        drawing.draw_image(
            "atlas.webp",
            Rect::new(0.0, 0.0, 32.0, 16.0),
            Rect::new(10.0, 20.0, 64.0, 32.0),
        );

        let bytes = drawing.as_bytes();
        assert_eq!(bytes[1], code::DRAW_IMAGE);
        assert_eq!(u16::from_le_bytes([bytes[2], bytes[3]]), 10);
        assert_eq!(&bytes[4..14], b"atlas.webp");
        assert_eq!(bytes.len(), 1 + 1 + 2 + 10 + 8 * 4);
    }

    #[test]
    fn a_pattern_takes_its_target_url_and_scale() {
        let mut drawing = Drawing::pixels();
        drawing.fill_pattern("grass.webp", 0.25);

        let bytes = drawing.as_bytes();
        assert_eq!(&bytes[1..3], [code::PATTERN, FILL]);
        assert_eq!(u16::from_le_bytes([bytes[3], bytes[4]]), 10);
        assert_eq!(&bytes[5..15], b"grass.webp");
        assert_eq!(&bytes[15..], 0.25f32.to_le_bytes());

        let mut stroked = Drawing::pixels();
        stroked.stroke_pattern("grass.webp", 0.25);
        assert_eq!(&stroked.as_bytes()[1..3], [code::PATTERN, STROKE]);
        assert_eq!(stroked.as_bytes()[3..], bytes[3..]);
    }

    #[test]
    fn a_gradient_takes_its_target_ends_and_stops() {
        let mut drawing = Drawing::pixels();
        drawing.fill_linear_gradient((1.0, 2.0), (3.0, 4.0), &[(0.0, "red"), (0.5, "#0f0")]);

        let bytes = drawing.as_bytes();
        assert_eq!(&bytes[1..3], [code::LINEAR_GRADIENT, FILL]);
        assert_eq!(
            &bytes[3..19],
            [1.0f32, 2.0, 3.0, 4.0].map(f32::to_le_bytes).concat()
        );
        assert_eq!(u16::from_le_bytes([bytes[19], bytes[20]]), 2);
        assert_eq!(&bytes[21..25], 0.0f32.to_le_bytes());
        assert_eq!(&bytes[25..30], b"\x03\x00red");
        assert_eq!(&bytes[30..34], 0.5f32.to_le_bytes());
        assert_eq!(&bytes[34..], b"\x04\x00#0f0");

        let mut stroked = Drawing::pixels();
        stroked.stroke_linear_gradient((1.0, 2.0), (3.0, 4.0), &[(0.0, "red"), (0.5, "#0f0")]);
        assert_eq!(&stroked.as_bytes()[1..3], [code::LINEAR_GRADIENT, STROKE]);
        assert_eq!(stroked.as_bytes()[3..], bytes[3..]);
    }

    #[test]
    fn a_gradient_of_too_many_stops_is_cut() {
        let stops = vec![(0.5f32, "red"); 70_000];
        let mut drawing = Drawing::pixels();
        drawing.fill_linear_gradient((0.0, 0.0), (1.0, 0.0), &stops);

        let bytes = drawing.as_bytes();
        assert_eq!(u16::from_le_bytes([bytes[19], bytes[20]]), u16::MAX);
        assert_eq!(
            bytes.len(),
            1 + 2 + 16 + 2 + usize::from(u16::MAX) * (4 + 2 + 3)
        );
    }

    #[test]
    fn an_appended_drawing_comes_between_a_save_and_a_restore_without_its_units() {
        let mut layer = Drawing::new(ViewBox::new(0.0, 0.0, 10.0, 10.0));
        layer.fill_circle(1.0, 2.0, 3.0);
        let mut drawing = Drawing::pixels();
        drawing.fill_style("red").append(&layer);

        let bytes = drawing.as_bytes();
        let circle = &layer.as_bytes()[1 + 16..];
        assert_eq!(bytes[1..7], *b"\x01\x03\x00red");
        assert_eq!(bytes[7], code::APPEND);
        assert_eq!(&bytes[8..8 + circle.len()], circle);
        assert_eq!(bytes[8 + circle.len()..], [code::APPEND_END]);
    }

    #[test]
    fn an_appended_drawing_restores_the_saves_it_left_open() {
        let mut layer = Drawing::pixels();
        layer.save().save().fill_circle(1.0, 2.0, 3.0);
        let mut drawing = Drawing::pixels();
        drawing.append(&layer);

        let bytes = drawing.as_bytes();
        assert_eq!(bytes[1..4], [code::APPEND, code::SAVE, code::SAVE]);
        assert_eq!(
            bytes[bytes.len() - 3..],
            [code::RESTORE, code::RESTORE, code::APPEND_END]
        );
        assert_eq!(drawing.saves, 0);
    }

    #[test]
    fn a_restore_without_a_save_does_nothing() {
        let mut drawing = Drawing::pixels();
        drawing.save().restore().restore();

        assert_eq!(
            drawing.as_bytes(),
            [UNITS_PIXELS, code::SAVE, code::RESTORE]
        );
    }

    #[test]
    fn pixels_of_another_length_than_their_size_draw_nothing() {
        let target = Rect::new(0.0, 0.0, 1.0, 1.0);
        let mut drawing = Drawing::pixels();
        drawing
            .draw_pixels(&[0; 8], 1, 1, target)
            .draw_pixels(&[], 0, 0, target)
            .draw_pixels(&[0; 4], u32::MAX, u32::MAX, target);

        assert_eq!(drawing.as_bytes(), [UNITS_PIXELS]);

        drawing.draw_pixels(&[1, 2, 3, 4, 5, 6, 7, 8], 2, 1, target);
        let bytes = drawing.as_bytes();
        assert_eq!(bytes[1], code::DRAW_PIXELS);
        assert_eq!(&bytes[2..10], [2u32, 1].map(u32::to_le_bytes).concat());
        assert_eq!(&bytes[26..], [1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn keywords_travel_as_their_text() {
        let mut drawing = Drawing::pixels();
        drawing
            .composite(Composite::DestinationOut)
            .text_align(TextAlign::Right)
            .text_baseline(TextBaseline::Hanging);

        assert_eq!(
            &drawing.as_bytes()[1..],
            [
                &[code::COMPOSITE, 15, 0][..],
                b"destination-out",
                &[code::TEXT_ALIGN, 5, 0],
                b"right",
                &[code::TEXT_BASELINE, 7, 0],
                b"hanging",
            ]
            .concat()
        );
    }

    #[test]
    fn dashes_say_whether_they_are_in_pixels() {
        let mut drawing = Drawing::pixels();
        drawing.line_dash_px(&[1.0, 2.0], 3.0);

        let bytes = drawing.as_bytes();
        assert_eq!(bytes[1..5], [code::LINE_DASH, 1, 2, 0]);
        assert_eq!(
            &bytes[5..],
            [1.0f32, 2.0, 3.0].map(f32::to_le_bytes).concat()
        );
    }

    #[test]
    fn a_text_takes_its_target_and_place() {
        let mut drawing = Drawing::pixels();
        drawing.stroke_text("ż", 1.0, 2.0);

        let bytes = drawing.as_bytes();
        assert_eq!(bytes[1..5], [code::TEXT, STROKE, 2, 0]);
        assert_eq!(&bytes[5..7], "ż".as_bytes());
        assert_eq!(&bytes[7..], [1.0f32, 2.0].map(f32::to_le_bytes).concat());
    }

    #[test]
    fn a_path_follows_its_rule() {
        let mut path = Path2D::new();
        path.rect(0.0, 0.0, 1.0, 1.0);
        let mut drawing = Drawing::pixels();
        drawing.fill_path(&path, FillRule::EvenOdd);

        let mut written = Vec::new();
        path.write(&mut written);
        let bytes = drawing.as_bytes();
        assert_eq!(bytes[1..3], [code::FILL_PATH, 1]);
        assert_eq!(&bytes[3..], written);
    }

    #[test]
    fn appending_a_drawing_of_nothing_adds_nothing() {
        let mut drawing = Drawing::pixels();
        drawing.fill_circle(1.0, 2.0, 3.0);
        let before = drawing.clone();

        drawing.append(&Drawing::new(ViewBox::new(0.0, 0.0, 1.0, 1.0)));

        assert_eq!(drawing, before);
    }

    #[test]
    fn a_drawing_appended_to_its_clone_stays_as_it_was() {
        let mut layer = Drawing::pixels();
        layer.fill_circle(1.0, 2.0, 3.0);
        let mut doubled = layer.clone();

        doubled.append(&layer);

        assert_eq!(layer.as_bytes().len(), 1 + 13);
        assert_eq!(doubled.as_bytes().len(), 1 + 13 + 1 + 13 + 1);
    }

    #[test]
    fn a_style_too_long_is_cut_between_characters() {
        let long = "ż".repeat(40_000);
        let mut drawing = Drawing::pixels();
        drawing.fill_style(&long);

        let bytes = drawing.as_bytes();
        let length = u16::from_le_bytes([bytes[2], bytes[3]]) as usize;
        assert_eq!(length, 65_534);
        assert!(std::str::from_utf8(&bytes[4..4 + length]).is_ok());
    }
}
