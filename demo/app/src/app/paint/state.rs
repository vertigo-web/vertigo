//! What the Paint tab keeps: the strokes on the paper, and the one being drawn.

use std::rc::Rc;

use vertigo::{
    Computed, Drawing, LineCap, LineJoin, PointerEvent, Value, ViewBox, store, transaction,
};

/// The paper, in its own units. The canvases fit it in whole, as an SVG `viewBox` is fitted,
/// so a picture keeps its shape whatever the width of the page - also when the window changes
/// after it was drawn.
pub const PAPER: ViewBox = ViewBox {
    x: 0.0,
    y: 0.0,
    width: 800.0,
    height: 500.0,
};

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

/// One line of paint: where a pointer went, from being pressed to being released.
#[derive(Clone, Debug, PartialEq)]
pub struct Stroke {
    pub color: &'static str,
    pub width: f32,
    pub points: Vec<(f32, f32)>,
}

/// The strokes on one canvas, the oldest at the bottom.
pub fn picture<'a>(strokes: impl IntoIterator<Item = &'a Stroke>) -> Drawing {
    let mut drawing = Drawing::new(PAPER);
    drawing.line_cap(LineCap::Round).line_join(LineJoin::Round);

    for stroke in strokes {
        match stroke.points.as_slice() {
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
        }
    }

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

/// A stroke being drawn, and the pointer drawing it.
#[derive(Clone, Debug, PartialEq)]
struct Brushing {
    pointer: i32,
    stroke: Stroke,
}

#[derive(Clone)]
pub struct State {
    /// What is on the paper, oldest first.
    pub strokes: Value<Rc<Vec<Stroke>>>,
    /// The stroke being drawn, until its pointer is released.
    brushing: Value<Option<Brushing>>,
    pub color: Value<&'static str>,
    pub width: Value<f32>,
}

impl State {
    pub fn new() -> Self {
        Self {
            strokes: Value::default(),
            brushing: Value::default(),
            color: Value::new(PALETTE[0].color),
            width: Value::new(BRUSHES[1]),
        }
    }

    /// The strokes on the paper, drawn anew only when one is added or taken away.
    pub fn picture(&self) -> Computed<Drawing> {
        self.strokes.map(|strokes| picture(strokes.iter()))
    }

    /// The stroke being drawn, drawn anew at every move of its pointer. It has a canvas of its
    /// own over the paper's, which would otherwise draw all the strokes again at every move.
    pub fn brush(&self) -> Computed<Drawing> {
        self.brushing
            .map(|brushing| picture(brushing.as_ref().map(|brushing| &brushing.stroke)))
    }

    /// Starts a stroke where `event` presses: the main button of a mouse, a pen, or the first
    /// finger to touch. Says whether it did.
    pub fn press(&self, event: &PointerEvent) -> bool {
        if !event.is_primary || event.button != 0 {
            return false;
        }
        let Some(point) = on_paper(event) else {
            return false;
        };

        let (color, width) =
            transaction(|context| (self.color.get(context), self.width.get(context)));
        self.brushing.set(Some(Brushing {
            pointer: event.pointer_id,
            stroke: Stroke {
                color,
                width,
                points: vec![point],
            },
        }));
        true
    }

    /// Takes the stroke of `event`'s pointer on to where it moved.
    pub fn move_to(&self, event: &PointerEvent) {
        // Released where the paper did not hear it: the stroke ends where it got to.
        if event.buttons & 1 == 0 {
            self.release(event);
            return;
        }
        let Some((x, y)) = on_paper(event) else {
            return;
        };

        self.brushing.change(|brushing| {
            if let Some(Brushing { pointer, stroke }) = brushing
                && *pointer == event.pointer_id
                && stroke
                    .points
                    .last()
                    .is_none_or(|(last_x, last_y)| (x - last_x).hypot(y - last_y) >= STEP)
            {
                stroke.points.push((x, y));
            }
        });
    }

    /// Ends the stroke of `event`'s pointer, which joins the others on the paper.
    pub fn release(&self, event: &PointerEvent) {
        let brushing = transaction(|context| self.brushing.get(context));

        if let Some(Brushing { pointer, stroke }) = brushing
            && pointer == event.pointer_id
        {
            // In the transaction of the event, both, so the stroke goes from one canvas to the
            // other in one batch of DOM changes: it is never on both, nor on neither.
            self.brushing.set(None);
            self.strokes
                .change(|strokes| Rc::make_mut(strokes).push(stroke));
        }
    }

    /// Takes the last stroke off the paper.
    pub fn undo(&self) {
        self.strokes.change(|strokes| {
            Rc::make_mut(strokes).pop();
        });
    }

    pub fn clear(&self) {
        self.strokes.set(Rc::default());
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

    fn strokes(state: &State) -> Vec<Stroke> {
        transaction(|context| state.strokes.get(context).to_vec())
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

        assert_eq!(picture([&dot, &line]), expected);
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
    fn clearing_takes_every_stroke_off() {
        let state = State::new();
        for x in [0.0, 100.0] {
            assert!(state.press(&mouse(x, 0.0, 1)));
            state.release(&mouse(x, 0.0, 0));
        }
        assert_eq!(strokes(&state).len(), 2);

        state.clear();
        assert_eq!(strokes(&state), vec![]);
    }
}
