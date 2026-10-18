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
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
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

    /// The color, gradient keyword or anything else CSS takes, that shapes are filled with.
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
        let bytes = self.bytes_mut();
        bytes.push(code::POLYLINE);
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

    /// The drawing as the browser reads it.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn op(&mut self, code: u8, args: &[f32]) -> &mut Self {
        let bytes = self.bytes_mut();
        bytes.push(code);
        for arg in args {
            bytes.extend_from_slice(&arg.to_le_bytes());
        }
        self
    }

    /// A style is a CSS text, which no style needs to be longer than its two length bytes
    /// allow; a longer one is cut at a character boundary.
    fn text(&mut self, code: u8, value: &str) -> &mut Self {
        let mut end = value.len().min(u16::MAX as usize);
        while !value.is_char_boundary(end) {
            end -= 1;
        }
        let bytes = self.bytes_mut();
        bytes.push(code);
        bytes.extend_from_slice(&(end as u16).to_le_bytes());
        bytes.extend_from_slice(&value.as_bytes()[..end]);
        self
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
            .polyline([(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)]);
        drawing
    }

    const FIXTURE: &[u8] = &[
        1, 0, 0, 0, 193, 0, 0, 160, 192, 0, 0, 128, 65, 0, 0, 32, 65, 1, 7, 0, 35, 102, 102, 100,
        50, 55, 102, 2, 8, 0, 197, 188, 195, 179, 197, 130, 196, 135, 3, 0, 0, 0, 63, 4, 0, 0, 0,
        64, 5, 0, 0, 192, 63, 6, 1, 12, 2, 7, 0, 0, 0, 193, 0, 0, 160, 192, 0, 0, 128, 65, 0, 0,
        32, 65, 8, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 9, 0, 0, 128, 191, 0, 0, 0, 192, 0, 0,
        128, 64, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192, 64, 11, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 128, 63, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 0, 0,
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
