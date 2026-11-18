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
//! text    := length:u16 utf8*
//! stops   := count:u16 (offset:f32 text)*   -- of a gradient
//! rect    := x:f32 y:f32 width:f32 height:f32
//! ```

use std::rc::Rc;

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
    pub const FILL_PATTERN: u8 = 16;
    pub const FILL_LINEAR_GRADIENT: u8 = 17;
}

const UNITS_PIXELS: u8 = 0;
const UNITS_VIEW_BOX: u8 = 1;

/// The units of a [`Drawing`]: a rectangle fitted into the canvas as an SVG `viewBox` with
/// `preserveAspectRatio="xMidYMid meet"` is, as big as fits and in the middle. What lies
/// outside the box is drawn too, in the margins it leaves.
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
/// Each drawing starts from the context's defaults: black fill and stroke, full opacity, lines
/// one unit wide with butt ends and mitered corners. Clones are cheap and share the operations
/// until one of them draws more.
#[derive(Clone, Debug, PartialEq)]
pub struct Drawing {
    bytes: Rc<Vec<u8>>,
}

impl Drawing {
    /// A drawing in the units of `view_box`.
    pub fn new(view_box: ViewBox) -> Self {
        let mut bytes = vec![UNITS_VIEW_BOX];
        for value in [view_box.x, view_box.y, view_box.width, view_box.height] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }

        Self {
            bytes: Rc::new(bytes),
        }
    }

    /// A drawing in CSS pixels, from the top left corner of the canvas.
    pub fn pixels() -> Self {
        Self {
            bytes: Rc::new(vec![UNITS_PIXELS]),
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

    pub fn fill_rect(&mut self, x: f32, y: f32, width: f32, height: f32) -> &mut Self {
        self.op(code::FILL_RECT, &[x, y, width, height])
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

    /// The `part` of the image at `url`, in the image's pixels, drawn into `target`, in the
    /// drawing's units, with the global alpha: f. ex. one picture out of many in an atlas.
    ///
    /// `url` is read as the `src` of an `<img>`, so a relative one is relative to the page. The
    /// browser loads an image once, the first time a drawing asks for it, and the canvas draws
    /// its last drawing again when the image comes. Until then, and if it fails to load, the
    /// drawing goes on without it.
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

    /// Fills shapes from now on with the image at `url` repeated, each of its pixels `scale` of
    /// the drawing's units wide: f. ex. a seamless texture tile. The pattern lies on the
    /// drawing, starting at its point (0, 0), so it moves and grows with the view box, and goes
    /// on unbroken from a shape to the one beside it. Each shape is smoothed along its edges on
    /// its own, though, so a hairline of what lies beneath can show where two meet: paint a
    /// similar color beneath them first.
    ///
    /// The image loads as for [`Self::draw_image`]. Until it comes, and if it fails to load, the
    /// fill style stays what it was, so a color set before stands in for the pattern:
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
        self.text(code::FILL_PATTERN, url);
        self.bytes_mut().extend_from_slice(&scale.to_le_bytes());
        self
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
        let stops = &stops[..stops.len().min(u16::MAX as usize)];
        self.op(
            code::FILL_LINEAR_GRADIENT,
            &[start.0, start.1, end.0, end.1],
        );
        self.bytes_mut()
            .extend_from_slice(&(stops.len() as u16).to_le_bytes());
        for (offset, color) in stops {
            self.bytes_mut().extend_from_slice(&offset.to_le_bytes());
            self.text_bytes(color);
        }
        self
    }

    /// The drawing as the browser reads it.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
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
        Rc::make_mut(&mut self.bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shared with `canvas.test.ts`, which plays these bytes back on a recording context.
    /// Change the format and one of the two tests fails, whichever side was changed.
    pub(crate) fn fixture() -> Drawing {
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
            .fill_rect(0.0, 0.0, 2.0, 2.0)
            .fill_linear_gradient((0.0, -1.0), (2.0, 1.0), &[(0.0, "red"), (1.0, "#00f")])
            .fill_rect(0.0, 0.0, 2.0, 2.0);
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
        128, 64, 0, 0, 128, 191, 0, 0, 0, 192, 0, 0, 0, 64, 0, 0, 128, 63, 16, 7, 0, 99, 97, 116,
        46, 112, 110, 103, 0, 0, 0, 63, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 64, 17, 0,
        0, 0, 0, 0, 0, 128, 191, 0, 0, 0, 64, 0, 0, 128, 63, 2, 0, 0, 0, 0, 0, 3, 0, 114, 101, 100,
        0, 0, 128, 63, 4, 0, 35, 48, 48, 102, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 64,
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
    fn a_pattern_takes_its_url_and_scale() {
        let mut drawing = Drawing::pixels();
        drawing.fill_pattern("grass.webp", 0.25);

        let bytes = drawing.as_bytes();
        assert_eq!(bytes[1], code::FILL_PATTERN);
        assert_eq!(u16::from_le_bytes([bytes[2], bytes[3]]), 10);
        assert_eq!(&bytes[4..14], b"grass.webp");
        assert_eq!(&bytes[14..], 0.25f32.to_le_bytes());
    }

    #[test]
    fn a_gradient_takes_its_ends_and_stops() {
        let mut drawing = Drawing::pixels();
        drawing.fill_linear_gradient((1.0, 2.0), (3.0, 4.0), &[(0.0, "red"), (0.5, "#0f0")]);

        let bytes = drawing.as_bytes();
        assert_eq!(bytes[1], code::FILL_LINEAR_GRADIENT);
        assert_eq!(
            &bytes[2..18],
            [1.0f32, 2.0, 3.0, 4.0].map(f32::to_le_bytes).concat()
        );
        assert_eq!(u16::from_le_bytes([bytes[18], bytes[19]]), 2);
        assert_eq!(&bytes[20..24], 0.0f32.to_le_bytes());
        assert_eq!(&bytes[24..29], b"\x03\x00red");
        assert_eq!(&bytes[29..33], 0.5f32.to_le_bytes());
        assert_eq!(&bytes[33..], b"\x04\x00#0f0");
    }

    #[test]
    fn a_gradient_of_too_many_stops_is_cut() {
        let stops = vec![(0.5f32, "red"); 70_000];
        let mut drawing = Drawing::pixels();
        drawing.fill_linear_gradient((0.0, 0.0), (1.0, 0.0), &stops);

        let bytes = drawing.as_bytes();
        assert_eq!(u16::from_le_bytes([bytes[18], bytes[19]]), u16::MAX);
        assert_eq!(
            bytes.len(),
            1 + 1 + 16 + 2 + usize::from(u16::MAX) * (4 + 2 + 3)
        );
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
