//! `draw`: the drawing of a `<canvas>` as the DOM commands carry it.

use crate::{self as vertigo, AttrGroup, Computed, Drawing, Value, ViewBox, component, dom};
use crate::{
    DomId,
    dev::{
        command::DriverDomCommand,
        inspect::{DomDebugFragment, log_start, log_take},
    },
    transaction,
};

/// A dot at `x`, in a box of 100 x 100 units.
fn dot(x: f32) -> Drawing {
    let mut drawing = Drawing::new(ViewBox::new(0.0, 0.0, 100.0, 100.0));
    drawing.fill_style("#fff").fill_circle(x, 50.0, 2.0);
    drawing
}

/// The canvases created, and every drawing sent to one, in order.
fn canvases_and_drawings(commands: Vec<DriverDomCommand>) -> (Vec<DomId>, Vec<(DomId, Vec<u8>)>) {
    let mut canvases = Vec::new();
    let mut drawings = Vec::new();

    for command in commands {
        match command {
            DriverDomCommand::CreateNode { id, name } if name.as_str() == "canvas" => {
                canvases.push(id);
            }
            DriverDomCommand::CanvasDraw { id, drawing } => {
                drawings.push((id, drawing));
            }
            _ => {}
        }
    }

    (canvases, drawings)
}

#[test]
fn a_drawing_follows_its_canvas_in_the_same_batch() {
    log_start();
    let _canvas = dom! { <canvas draw={dot(10.0)} /> };
    let commands = log_take();

    let created = commands.iter().position(|command| {
        matches!(command, DriverDomCommand::CreateNode { name, .. } if name.as_str() == "canvas")
    });
    let drawn = commands
        .iter()
        .position(|command| matches!(command, DriverDomCommand::CanvasDraw { .. }));
    assert!(matches!((created, drawn), (Some(created), Some(drawn)) if created < drawn));

    let (canvases, drawings) = canvases_and_drawings(commands);
    assert_eq!(drawings, vec![(canvases[0], dot(10.0).as_bytes().to_vec())]);
}

#[test]
fn a_new_drawing_is_sent_and_an_equal_one_is_not() {
    let x = Value::new(10.0);
    // Whole units only, so 20.0 and 20.4 draw the same.
    let drawing = x.map(|x: f32| dot(x.round()));

    log_start();
    let _canvas = dom! { <canvas draw={drawing} /> };
    x.set(20.0);
    x.set(20.4);
    x.set(30.0);
    let (canvases, drawings) = canvases_and_drawings(log_take());

    let expected: Vec<_> = [10.0, 20.0, 30.0]
        .into_iter()
        .map(|x| (canvases[0], dot(x).as_bytes().to_vec()))
        .collect();
    assert_eq!(drawings, expected);
}

#[test]
fn a_transaction_sends_the_last_drawing_only() {
    let x = Value::new(10.0);
    let drawing = x.map(dot);

    log_start();
    let _canvas = dom! { <canvas draw={drawing} /> };
    transaction(|_| {
        x.set(20.0);
        x.set(30.0);
    });
    let (_, drawings) = canvases_and_drawings(log_take());

    let sent: Vec<_> = drawings.into_iter().map(|(_, drawing)| drawing).collect();
    assert_eq!(
        sent,
        vec![dot(10.0).as_bytes().to_vec(), dot(30.0).as_bytes().to_vec()]
    );
}

#[test]
fn a_canvas_gone_gets_no_more_drawings() {
    let x = Value::new(10.0);
    let drawing = x.map(dot);
    let canvas = dom! { <canvas draw={drawing} /> };

    drop(canvas);
    log_start();
    x.set(20.0);
    let (_, drawings) = canvases_and_drawings(log_take());

    assert!(drawings.is_empty());
}

#[test]
fn a_component_passes_a_drawing_on() {
    #[component]
    fn Picture(inner: AttrGroup) {
        dom! { <canvas {..inner} /> }
    }

    let drawing = Computed::from(|_| dot(10.0));

    log_start();
    let _picture = dom! { <Picture inner:draw={drawing} /> };
    let html = DomDebugFragment::from_log().to_pseudo_html();

    let bytes = dot(10.0).as_bytes().len();
    assert_eq!(
        html,
        format!("<canvas v-component='Picture' v-drawing='{bytes} bytes' />")
    );
}
