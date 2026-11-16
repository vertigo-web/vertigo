//! What the Paint tab keeps: the marks on the paper, and the one being made.

use std::rc::Rc;

use vertigo::{
    Computed, Drawing, LineCap, LineJoin, PointerEvent, Rect, Value, ViewBox, include_static,
    store, transaction,
};

/// The paper, in its own units. The canvases fit it in whole, as an SVG `viewBox` is fitted,
/// so a picture keeps its shape whatever the width of the page - also when the window changes
/// after it was drawn.
pub const PAPER: ViewBox = ViewBox::new(0.0, 0.0, 800.0, 500.0);

/// A paint on the palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Paint {
    pub name: &'static str,
    /// Anything CSS takes for a color.
    pub color: &'static str,
}

pub const PALETTE: &[Paint] = &[
    Paint {
        name: "Black",
        color: "#222222",
    },
    Paint {
        name: "Gray",
        color: "#9e9e9e",
    },
    Paint {
        name: "Red",
        color: "#e53935",
    },
    Paint {
        name: "Orange",
        color: "#fb8c00",
    },
    Paint {
        name: "Yellow",
        color: "#fdd835",
    },
    Paint {
        name: "Green",
        color: "#43a047",
    },
    Paint {
        name: "Blue",
        color: "#1e88e5",
    },
    Paint {
        name: "Violet",
        color: "#8e24aa",
    },
    // The paper's own color, so also the eraser.
    Paint {
        name: "White",
        color: "#ffffff",
    },
];

/// Widths of the brushes, in the units of the paper.
pub const BRUSHES: &[f32] = &[2.0, 6.0, 14.0, 30.0];

/// A point closer than this to the last one of its stroke adds nothing to see.
const STEP: f32 = 1.0;

/// A picture to paste on the paper, cut out of `stamps.webp`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stamp {
    pub name: &'static str,
    /// Where it is in the atlas, in the atlas's pixels.
    pub part: Rect,
}

/// The atlas is 2 x 2 cells of this many pixels. Each picture keeps clear of its cell's edges,
/// so a scaled stamp doesn't bring in a smoothed edge of its neighbour.
const CELL: f32 = 128.0;

const fn cell(column: f32, row: f32) -> Rect {
    Rect::new(column * CELL, row * CELL, CELL, CELL)
}

pub const STAMPS: &[Stamp] = &[
    Stamp {
        name: "Star",
        part: cell(0.0, 0.0),
    },
    Stamp {
        name: "Heart",
        part: cell(1.0, 0.0),
    },
    Stamp {
        name: "Flower",
        part: cell(0.0, 1.0),
    },
    Stamp {
        name: "Smile",
        part: cell(1.0, 1.0),
    },
];

/// How big a stamp is on the paper, in its units.
const STAMP_SIZE: f32 = 96.0;

/// Where the browser gets the atlas. It loads it once, for the paper and all the buttons.
fn atlas() -> String {
    include_static!("./stamps.webp")
}

/// One line of paint: where a pointer went, from being pressed to being released.
#[derive(Clone, Debug, PartialEq)]
pub struct Stroke {
    pub color: &'static str,
    pub width: f32,
    pub points: Vec<(f32, f32)>,
}

/// What one press of a pointer leaves on the paper.
#[derive(Clone, Debug, PartialEq)]
pub enum Mark {
    Stroke(Stroke),
    /// A stamp with its middle `at` the point the pointer was released.
    Stamp {
        stamp: Stamp,
        at: (f32, f32),
    },
}

/// The marks on one canvas, the oldest at the bottom.
pub fn picture<'a>(marks: impl IntoIterator<Item = &'a Mark>) -> Drawing {
    let mut drawing = Drawing::new(PAPER);
    drawing.line_cap(LineCap::Round).line_join(LineJoin::Round);
    let mut atlas_url = None;

    for mark in marks {
        match mark {
            Mark::Stroke(stroke) => match stroke.points.as_slice() {
                [] => {}
                // A click. A line through one point draws nothing, so a dot as wide as the brush.
                [(x, y)] => {
                    drawing
                        .fill_style(stroke.color)
                        .fill_circle(*x, *y, stroke.width / 2.0);
                }
                points => {
                    drawing
                        .stroke_style(stroke.color)
                        .line_width(stroke.width)
                        .polyline(points.iter().copied());
                }
            },
            Mark::Stamp { stamp, at: (x, y) } => {
                let target = Rect::new(
                    x - STAMP_SIZE / 2.0,
                    y - STAMP_SIZE / 2.0,
                    STAMP_SIZE,
                    STAMP_SIZE,
                );
                drawing.draw_image(atlas_url.get_or_insert_with(atlas), stamp.part, target);
            }
        }
    }

    drawing
}

/// `stamp` filling its canvas, for its button.
pub fn icon(stamp: Stamp) -> Drawing {
    let Rect { width, height, .. } = stamp.part;
    let mut drawing = Drawing::new(ViewBox::new(0.0, 0.0, width, height));
    drawing.draw_image(&atlas(), stamp.part, Rect::new(0.0, 0.0, width, height));
    drawing
}

/// Where `event`'s pointer is on the paper, in its units. The canvases fill the element with
/// the listener, so the paper is fitted into it as into them: as big as fits, in the middle.
pub fn on_paper(event: &PointerEvent) -> Option<(f32, f32)> {
    let (width, height) = (f64::from(PAPER.width), f64::from(PAPER.height));
    let scale = (event.element_width / width).min(event.element_height / height);
    // Not on the screen.
    if scale.is_nan() || scale <= 0.0 {
        return None;
    }

    let x = (event.pointer_x - (event.element_width - width * scale) / 2.0) / scale;
    let y = (event.pointer_y - (event.element_height - height * scale) / 2.0) / scale;
    Some((PAPER.x + x as f32, PAPER.y + y as f32))
}

/// A mark being made, and the pointer making it.
#[derive(Clone, Debug, PartialEq)]
struct Making {
    pointer: i32,
    mark: Mark,
}

#[derive(Clone)]
pub struct State {
    /// What is on the paper, oldest first.
    pub marks: Value<Rc<Vec<Mark>>>,
    /// The mark being made, until its pointer is released.
    making: Value<Option<Making>>,
    pub color: Value<&'static str>,
    pub width: Value<f32>,
    /// The stamp a press pastes, or `None` for the brush.
    pub stamp: Value<Option<Stamp>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            marks: Value::default(),
            making: Value::default(),
            color: Value::new(PALETTE[0].color),
            width: Value::new(BRUSHES[1]),
            stamp: Value::default(),
        }
    }

    /// The marks on the paper, drawn anew only when one is added or taken away.
    pub fn picture(&self) -> Computed<Drawing> {
        self.marks.map(|marks| picture(marks.iter()))
    }

    /// The mark being made, drawn anew at every move of its pointer. It has a canvas of its own
    /// over the paper's, which would otherwise draw all the marks again at every move.
    pub fn making(&self) -> Computed<Drawing> {
        self.making
            .map(|making| picture(making.as_ref().map(|making| &making.mark)))
    }

    /// Whether the brush of `width` is the tool.
    pub fn is_brush(&self, width: f32) -> Computed<bool> {
        let state = self.clone();
        Computed::from(move |context| {
            state.stamp.get(context).is_none() && state.width.get(context) == width
        })
    }

    /// Picking a paint takes the brush, as a stamp has its own colors.
    pub fn pick_paint(&self, color: &'static str) {
        self.color.set(color);
        self.stamp.set(None);
    }

    pub fn pick_brush(&self, width: f32) {
        self.width.set(width);
        self.stamp.set(None);
    }

    /// Starts a mark where `event` presses: the main button of a mouse, a pen, or the first
    /// finger to touch. Says whether it did.
    pub fn press(&self, event: &PointerEvent) -> bool {
        if !event.is_primary || event.button != 0 {
            return false;
        }
        let Some(point) = on_paper(event) else {
            return false;
        };

        let mark = transaction(|context| match self.stamp.get(context) {
            Some(stamp) => Mark::Stamp { stamp, at: point },
            None => Mark::Stroke(Stroke {
                color: self.color.get(context),
                width: self.width.get(context),
                points: vec![point],
            }),
        });
        self.making.set(Some(Making {
            pointer: event.pointer_id,
            mark,
        }));
        true
    }

    /// Takes the mark of `event`'s pointer on to where it moved: a stroke grows, a stamp goes
    /// along.
    pub fn move_to(&self, event: &PointerEvent) {
        // Released where the paper did not hear it: the mark stays where it got to.
        if event.buttons & 1 == 0 {
            self.release(event);
            return;
        }
        let Some((x, y)) = on_paper(event) else {
            return;
        };

        self.making.change(|making| {
            let Some(Making { pointer, mark }) = making else {
                return;
            };
            if *pointer != event.pointer_id {
                return;
            }

            match mark {
                Mark::Stroke(stroke) => {
                    if stroke
                        .points
                        .last()
                        .is_none_or(|(last_x, last_y)| (x - last_x).hypot(y - last_y) >= STEP)
                    {
                        stroke.points.push((x, y));
                    }
                }
                Mark::Stamp { at, .. } => *at = (x, y),
            }
        });
    }

    /// Ends the mark of `event`'s pointer, which joins the others on the paper.
    pub fn release(&self, event: &PointerEvent) {
        let making = transaction(|context| self.making.get(context));

        if let Some(Making { pointer, mark }) = making
            && pointer == event.pointer_id
        {
            // In the transaction of the event, both, so the mark goes from one canvas to the
            // other in one batch of DOM changes: it is never on both, nor on neither.
            self.making.set(None);
            self.marks.change(|marks| Rc::make_mut(marks).push(mark));
        }
    }

    /// Takes the last mark off the paper.
    pub fn undo(&self) {
        self.marks.change(|marks| {
            Rc::make_mut(marks).pop();
        });
    }

    pub fn clear(&self) {
        self.marks.set(Rc::default());
    }
}

/// One paper for the whole visit, so a picture is still there when the visitor comes back
/// from another tab.
#[store]
pub fn state_paint() -> State {
    State::new()
}

#[cfg(test)]
mod tests {
    use vertigo::{PointerEvent, PointerType, transaction};

    use super::*;

    /// A pointer of the mouse at (`x`, `y`) over a paper drawn twice its size.
    fn mouse(x: f64, y: f64, buttons: u16) -> PointerEvent {
        let mut event = PointerEvent::default();
        event.pointer_id = 1;
        event.is_primary = true;
        event.button = 0;
        event.buttons = buttons;
        event.pointer_x = x;
        event.pointer_y = y;
        event.element_width = 1600.0;
        event.element_height = 1000.0;
        event
    }

    fn marks(state: &State) -> Vec<Mark> {
        transaction(|context| state.marks.get(context).to_vec())
    }

    /// The marks, for a test that makes only strokes.
    fn strokes(state: &State) -> Vec<Stroke> {
        marks(state)
            .into_iter()
            .map(|mark| match mark {
                Mark::Stroke(stroke) => stroke,
                other => panic!("not a stroke: {other:?}"),
            })
            .collect()
    }

    #[test]
    fn the_pointer_is_found_on_the_paper_as_the_canvases_show_it() {
        assert_eq!(on_paper(&mouse(400.0, 250.0, 0)), Some((200.0, 125.0)));

        // Wider than the paper, so in the middle with a margin on each side.
        let mut wide = mouse(150.0, 100.0, 0);
        wide.element_width = 1000.0;
        wide.element_height = 500.0;
        assert_eq!(on_paper(&wide), Some((50.0, 100.0)));

        let mut hidden = mouse(1.0, 1.0, 0);
        hidden.element_width = 0.0;
        hidden.element_height = 0.0;
        assert_eq!(on_paper(&hidden), None);
    }

    #[test]
    fn a_click_leaves_a_dot_and_a_drag_a_line() {
        let dot = Stroke {
            color: "red",
            width: 6.0,
            points: vec![(1.0, 2.0)],
        };
        let line = Stroke {
            color: "blue",
            width: 2.0,
            points: vec![(1.0, 2.0), (3.0, 4.0)],
        };

        let mut expected = Drawing::new(PAPER);
        expected
            .line_cap(LineCap::Round)
            .line_join(LineJoin::Round)
            .fill_style("red")
            .fill_circle(1.0, 2.0, 3.0)
            .stroke_style("blue")
            .line_width(2.0)
            .polyline([(1.0, 2.0), (3.0, 4.0)]);

        assert_eq!(picture([&Mark::Stroke(dot), &Mark::Stroke(line)]), expected);
    }

    #[test]
    fn a_stamp_is_drawn_with_its_middle_at_its_point() {
        let star = Mark::Stamp {
            stamp: STAMPS[0],
            at: (100.0, 200.0),
        };

        let mut expected = Drawing::new(PAPER);
        expected
            .line_cap(LineCap::Round)
            .line_join(LineJoin::Round)
            .draw_image(
                &atlas(),
                Rect::new(0.0, 0.0, 128.0, 128.0),
                Rect::new(52.0, 152.0, 96.0, 96.0),
            );

        assert_eq!(picture([&star]), expected);
    }

    #[test]
    fn a_stamp_goes_with_the_pointer_and_stays_where_it_is_released() {
        let state = State::new();
        state.stamp.set(Some(STAMPS[1]));

        assert!(state.press(&mouse(0.0, 0.0, 1)));
        state.move_to(&mouse(100.0, 50.0, 1));
        state.move_to(&mouse(101.0, 50.0, 1));
        assert_eq!(marks(&state), vec![], "not before the release");

        state.release(&mouse(101.0, 50.0, 0));
        assert_eq!(
            marks(&state),
            vec![Mark::Stamp {
                stamp: STAMPS[1],
                at: (50.5, 25.0),
            }]
        );
    }

    #[test]
    fn a_paint_or_a_brush_puts_the_stamp_down() {
        let state = State::new();
        let brush = state.is_brush(BRUSHES[1]);
        let is_brush = || transaction(|context| brush.get(context));
        assert!(is_brush());

        state.stamp.set(Some(STAMPS[0]));
        assert!(!is_brush());
        state.pick_paint("red");
        assert!(is_brush());

        state.stamp.set(Some(STAMPS[0]));
        state.pick_brush(BRUSHES[0]);
        assert!(!is_brush(), "another brush");
        assert!(transaction(|context| state
            .is_brush(BRUSHES[0])
            .get(context)));

        assert!(state.press(&mouse(0.0, 0.0, 1)));
        state.release(&mouse(0.0, 0.0, 0));
        assert_eq!(strokes(&state)[0].color, "red");
    }

    #[test]
    fn a_stroke_goes_on_the_paper_when_its_pointer_is_released() {
        let state = State::new();
        state.color.set("red");

        assert!(state.press(&mouse(0.0, 0.0, 1)));
        state.move_to(&mouse(20.0, 0.0, 1));
        // Too close to the last point to add one.
        state.move_to(&mouse(21.0, 0.0, 1));
        state.move_to(&mouse(20.0, 40.0, 1));
        assert_eq!(strokes(&state), vec![], "not before the release");

        state.release(&mouse(20.0, 40.0, 0));
        let drawn = Stroke {
            color: "red",
            width: BRUSHES[1],
            points: vec![(0.0, 0.0), (10.0, 0.0), (10.0, 20.0)],
        };
        assert_eq!(strokes(&state), vec![drawn.clone()]);

        // The pointer goes on moving, but it draws no more.
        state.move_to(&mouse(100.0, 100.0, 0));
        assert_eq!(strokes(&state), vec![drawn]);

        state.undo();
        assert_eq!(strokes(&state), vec![]);
    }

    #[test]
    fn only_the_pointer_that_pressed_draws() {
        let state = State::new();
        assert!(state.press(&mouse(0.0, 0.0, 1)));

        // A second finger.
        let mut other = mouse(50.0, 50.0, 1);
        other.pointer_id = 2;
        other.pointer_type = PointerType::Touch;
        other.is_primary = false;
        assert!(!state.press(&other));
        state.move_to(&other);
        state.release(&other);
        assert_eq!(strokes(&state), vec![]);

        state.release(&mouse(0.0, 0.0, 0));
        assert_eq!(strokes(&state)[0].points, vec![(0.0, 0.0)]);
    }

    #[test]
    fn the_secondary_button_does_not_draw() {
        let state = State::new();
        let mut right = mouse(0.0, 0.0, 2);
        right.button = 2;

        assert!(!state.press(&right));
    }

    #[test]
    fn a_release_the_paper_did_not_hear_ends_the_stroke_at_the_next_move() {
        let state = State::new();
        assert!(state.press(&mouse(0.0, 0.0, 1)));
        state.move_to(&mouse(20.0, 0.0, 1));

        // Back over the paper with no button held.
        state.move_to(&mouse(100.0, 100.0, 0));

        assert_eq!(strokes(&state)[0].points, vec![(0.0, 0.0), (10.0, 0.0)]);
    }

    #[test]
    fn clearing_takes_every_mark_off() {
        let state = State::new();
        for stamp in [None, Some(STAMPS[2])] {
            state.stamp.set(stamp);
            assert!(state.press(&mouse(0.0, 0.0, 1)));
            state.release(&mouse(0.0, 0.0, 0));
        }
        assert_eq!(marks(&state).len(), 2);

        state.clear();
        assert_eq!(marks(&state), vec![]);
    }
}
