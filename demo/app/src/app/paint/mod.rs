//! A paper to draw on with a mouse, a pen or a finger.
//!
//! The paper is two `<canvas>` elements, one over the other, each drawn from the
//! `Computed<Drawing>` in its `draw` attribute. The lower one shows the strokes done, and is
//! drawn anew only when one is added or taken away. The upper one shows the stroke being
//! drawn, and is drawn anew at every move of the pointer.

mod state;

use vertigo::{Css, DomNode, PointerEvent, bind, component, css, dom};

use state::{BRUSHES, PALETTE, Paint, State, state_paint};

#[component]
pub fn PaintDemo() {
    let state = state_paint();

    let down = bind!(state, |event: PointerEvent| {
        if state.press(&event) {
            // The stroke goes on outside the paper, and ends wherever the pointer is released.
            event.capture_pointer();
        }
    });
    let moved = bind!(state, |event: PointerEvent| state.move_to(&event));
    let up = bind!(state, |event: PointerEvent| state.release(&event));

    let blank = state
        .strokes
        .map(|strokes| strokes.is_empty().then(|| "disabled".to_string()));
    let undo = bind!(state, |_| state.undo());
    let clear = bind!(state, |_| state.clear());

    let paints: Vec<_> = PALETTE.iter().map(|paint| swatch(&state, *paint)).collect();
    let brushes: Vec<_> = BRUSHES.iter().map(|width| brush(&state, *width)).collect();

    dom! {
        <div css={css_wrapper()}>
            <div css={css_toolbar()}>
                <div css={css_group()}>{..paints}</div>
                <div css={css_group()}>{..brushes}</div>
                <div css={css_group()}>
                    <button on_click={undo} disabled={blank.clone()}>"Undo"</button>
                    <button on_click={clear} disabled={blank}>"Clear"</button>
                </div>
            </div>
            <div
                css={css_paper()}
                on_pointer_down={down}
                on_pointer_move={moved}
                on_pointer_up={up.clone()}
                on_pointer_cancel={up}
            >
                <canvas css={css_layer()} draw={state.picture()} />
                <canvas css={css_layer()} draw={state.brush()} />
            </div>
            <p>"Draw with a mouse, a pen or a finger."</p>
        </div>
    }
}

fn swatch(state: &State, paint: Paint) -> DomNode {
    let picked = state.color.map(move |color| color == paint.color);
    let pick = bind!(state, |_| state.color.set(paint.color));

    dom! {
        <button
            css={picked.map(move |picked| css_swatch(paint.color, picked))}
            title={paint.name}
            aria-pressed={picked.map(|picked| picked.to_string())}
            on_click={pick}
        />
    }
}

fn brush(state: &State, width: f32) -> DomNode {
    let picked = state.width.map(move |current| current == width);
    let pick = bind!(state, |_| state.width.set(width));
    // As wide as the brush draws on a paper shown at its own size, as far as the button fits it.
    let dot = format!("width: {0}px; height: {0}px;", width.min(22.0));

    dom! {
        <button
            css={picked.map(css_tool)}
            title={format!("Brush {width}")}
            aria-pressed={picked.map(|picked| picked.to_string())}
            on_click={pick}
        >
            <span css={css_dot()} style={dot} />
        </button>
    }
}

fn css_wrapper() -> Css {
    css! {"
        margin: 10px 0;
    "}
}

fn css_toolbar() -> Css {
    css! {"
        display: flex;
        flex-wrap: wrap;
        gap: 10px 20px;
        margin-bottom: 10px;
    "}
}

fn css_group() -> Css {
    css! {"
        display: flex;
        flex-wrap: wrap;
        align-items: center;
    "}
}

/// The paper's edge is a shadow rather than a border: the pointer is found from the edge of
/// the element's border, and the canvases fill what is inside it.
fn css_paper() -> Css {
    css! {"
        position: relative;
        width: 100%;
        max-width: 800px;
        aspect-ratio: 8 / 5;
        background-color: white;
        box-shadow: 0 0 0 1px #333, 0 2px 6px rgba(0, 0, 0, 0.3);
        cursor: crosshair;
        touch-action: none;
        user-select: none;
    "}
}

fn css_layer() -> Css {
    css! {"
        position: absolute;
        left: 0;
        top: 0;
        width: 100%;
        height: 100%;
    "}
}

fn css_swatch(color: &'static str, picked: bool) -> Css {
    let outline = match picked {
        true => "3px solid #1e88e5",
        false => "none",
    };

    css! {"
        width: 30px;
        height: 30px;
        background-color: {color};
        outline: {outline};
        outline-offset: 1px;
        cursor: pointer;
    "}
}

fn css_tool(picked: bool) -> Css {
    let background = match picked {
        true => "lightblue",
        false => "white",
    };

    css! {"
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 36px;
        height: 30px;
        padding: 0;
        background-color: {background};
        cursor: pointer;
    "}
}

fn css_dot() -> Css {
    css! {"
        display: block;
        flex-shrink: 0;
        border-radius: 50%;
        background-color: #333;
    "}
}
