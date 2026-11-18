//! Lines and curves to fill, stroke or clip with, see [`Path2D`].

use std::f32::consts::TAU;

/// Commands of a path, as the browser reads them by number: append only.
pub(super) mod command {
    pub const MOVE_TO: u8 = 0;
    pub const LINE_TO: u8 = 1;
    pub const QUAD_TO: u8 = 2;
    pub const CUBIC_TO: u8 = 3;
    pub const ARC_TO: u8 = 4;
    pub const ELLIPSE: u8 = 5;
    pub const CLOSE: u8 = 6;
}

/// Lines and curves, one shape or many, as `Path2D` of the browser has them: for
/// [`Drawing::fill_path`](crate::Drawing::fill_path),
/// [`Drawing::stroke_path`](crate::Drawing::stroke_path) and
/// [`Drawing::clip`](crate::Drawing::clip). Its numbers are in the units of the drawing it is
/// drawn on, and its angles in radians, clockwise from the right.
///
/// Shapes inside one another make holes with [`FillRule::EvenOdd`](crate::FillRule::EvenOdd):
///
/// ```rust
/// use vertigo::{Drawing, FillRule, Path2D, ViewBox};
///
/// let mut frame = Path2D::new();
/// frame.rect(0.0, 0.0, 10.0, 10.0).circle(5.0, 5.0, 3.0);
///
/// let mut drawing = Drawing::new(ViewBox::new(0.0, 0.0, 10.0, 10.0));
/// drawing.fill_style("#8db360").fill_path(&frame, FillRule::EvenOdd);
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Path2D {
    svg: String,
    commands: Vec<u8>,
    count: u32,
}

impl Path2D {
    /// A path of nothing yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// A path from SVG path data, the `d` of a `<path>`: f. ex. an icon, in the units of the
    /// drawing. What is added to it comes after. Data with an error in it is drawn up to the
    /// error.
    pub fn svg(data: &str) -> Self {
        Self {
            svg: data.into(),
            ..Self::default()
        }
    }

    /// Starts a new shape at (`x`, `y`).
    pub fn move_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.command(command::MOVE_TO, &[x, y])
    }

    /// A straight line from where the path is to (`x`, `y`).
    pub fn line_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.command(command::LINE_TO, &[x, y])
    }

    /// A quadratic Bézier curve to (`x`, `y`), pulled toward (`cx`, `cy`).
    pub fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) -> &mut Self {
        self.command(command::QUAD_TO, &[cx, cy, x, y])
    }

    /// A cubic Bézier curve to (`x`, `y`), leaving toward (`c1x`, `c1y`) and coming from
    /// (`c2x`, `c2y`).
    pub fn cubic_to(
        &mut self,
        c1x: f32,
        c1y: f32,
        c2x: f32,
        c2y: f32,
        x: f32,
        y: f32,
    ) -> &mut Self {
        self.command(command::CUBIC_TO, &[c1x, c1y, c2x, c2y, x, y])
    }

    /// A corner rounded by `radius`: a line from where the path is toward (`x1`, `y1`), turning
    /// along an arc toward (`x2`, `y2`), as `arcTo` of the 2D context. A radius below zero
    /// counts as zero.
    pub fn arc_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, radius: f32) -> &mut Self {
        self.command(command::ARC_TO, &[x1, y1, x2, y2, radius])
    }

    /// The arc of the circle around (`x`, `y`) from the angle `start` to `end`, clockwise, or
    /// the other way when `counterclockwise`. A line joins it to where the path was. A radius
    /// below zero counts as zero.
    pub fn arc(
        &mut self,
        x: f32,
        y: f32,
        radius: f32,
        start: f32,
        end: f32,
        counterclockwise: bool,
    ) -> &mut Self {
        self.ellipse((x, y), (radius, radius), 0.0, start, end, counterclockwise)
    }

    /// As [`Self::arc`], of the ellipse around `center` with its radii along x and y, turned by
    /// `rotation`.
    pub fn ellipse(
        &mut self,
        center: (f32, f32),
        radii: (f32, f32),
        rotation: f32,
        start: f32,
        end: f32,
        counterclockwise: bool,
    ) -> &mut Self {
        let (x, y) = center;
        let (radius_x, radius_y) = radii;
        self.command(
            command::ELLIPSE,
            &[x, y, radius_x, radius_y, rotation, start, end],
        );
        self.commands.push(u8::from(counterclockwise));
        self
    }

    /// Closes the shape with a line back to where it started.
    pub fn close(&mut self) -> &mut Self {
        self.command(command::CLOSE, &[])
    }

    /// A rectangle, a closed shape of its own.
    pub fn rect(&mut self, x: f32, y: f32, width: f32, height: f32) -> &mut Self {
        self.move_to(x, y)
            .line_to(x + width, y)
            .line_to(x + width, y + height)
            .line_to(x, y + height)
            .close()
    }

    /// A rectangle with its corners rounded by `radius`, at most half its shorter side.
    pub fn round_rect(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radius: f32,
    ) -> &mut Self {
        // NaN, as a radius below zero, counts as zero: `max` takes the other number.
        let radius = radius
            .max(0.0)
            .min(width.abs() / 2.0)
            .min(height.abs() / 2.0);
        let (right, bottom) = (x + width, y + height);
        self.move_to(x + radius.copysign(width), y)
            .arc_to(right, y, right, bottom, radius)
            .arc_to(right, bottom, x, bottom, radius)
            .arc_to(x, bottom, x, y, radius)
            .arc_to(x, y, right, y, radius)
            .close()
    }

    /// A circle, a closed shape of its own.
    pub fn circle(&mut self, x: f32, y: f32, radius: f32) -> &mut Self {
        self.move_to(x + radius, y)
            .arc(x, y, radius, 0.0, TAU, false)
            .close()
    }

    /// The shape `points` enclose, closed. No points add nothing.
    pub fn polygon(&mut self, points: impl IntoIterator<Item = (f32, f32)>) -> &mut Self {
        let mut points = points.into_iter();
        if let Some((x, y)) = points.next() {
            self.move_to(x, y);
            for (x, y) in points {
                self.line_to(x, y);
            }
            self.close();
        }
        self
    }

    /// The path as the browser reads it: `svg:long_text count:u32 command*`.
    pub(super) fn write(&self, bytes: &mut Vec<u8>) {
        bytes.extend_from_slice(&(self.svg.len() as u32).to_le_bytes());
        bytes.extend_from_slice(self.svg.as_bytes());
        bytes.extend_from_slice(&self.count.to_le_bytes());
        bytes.extend_from_slice(&self.commands);
    }

    fn command(&mut self, command: u8, args: &[f32]) -> &mut Self {
        self.commands.push(command);
        for arg in args {
            self.commands.extend_from_slice(&arg.to_le_bytes());
        }
        self.count += 1;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn written(path: &Path2D) -> Vec<u8> {
        let mut bytes = Vec::new();
        path.write(&mut bytes);
        bytes
    }

    #[test]
    fn a_path_writes_its_svg_then_its_commands() {
        let mut path = Path2D::svg("M0 0");
        path.line_to(1.0, 2.0).close();

        let bytes = written(&path);
        assert_eq!(&bytes[..4], 4u32.to_le_bytes());
        assert_eq!(&bytes[4..8], b"M0 0");
        assert_eq!(&bytes[8..12], 2u32.to_le_bytes());
        assert_eq!(bytes[12], command::LINE_TO);
        assert_eq!(&bytes[13..21], [1.0f32, 2.0].map(f32::to_le_bytes).concat());
        assert_eq!(bytes[21..], [command::CLOSE]);
    }

    #[test]
    fn an_ellipse_says_which_way_it_turns() {
        let mut path = Path2D::new();
        path.ellipse((1.0, 2.0), (3.0, 4.0), 0.5, 0.0, 1.0, true);

        let bytes = written(&path);
        assert_eq!(bytes[8], command::ELLIPSE);
        assert_eq!(
            &bytes[9..37],
            [1.0f32, 2.0, 3.0, 4.0, 0.5, 0.0, 1.0]
                .map(f32::to_le_bytes)
                .concat()
        );
        assert_eq!(bytes[37..], [1]);
    }

    #[test]
    fn a_round_rect_keeps_its_radius_within_half_its_shorter_side() {
        let mut path = Path2D::new();
        path.round_rect(0.0, 0.0, 10.0, 4.0, 5.0);

        let mut expected = Path2D::new();
        expected
            .move_to(2.0, 0.0)
            .arc_to(10.0, 0.0, 10.0, 4.0, 2.0)
            .arc_to(10.0, 4.0, 0.0, 4.0, 2.0)
            .arc_to(0.0, 4.0, 0.0, 0.0, 2.0)
            .arc_to(0.0, 0.0, 10.0, 0.0, 2.0)
            .close();
        assert_eq!(path, expected);
    }

    #[test]
    fn a_polygon_of_no_points_adds_nothing() {
        let mut path = Path2D::new();
        path.polygon([]);

        assert_eq!(path, Path2D::new());
    }
}
