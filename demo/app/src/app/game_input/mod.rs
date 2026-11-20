//! What a game reads of the keyboard and the mouse: the keys held and the presses repeated, the
//! window's focus and the page's visibility, the pointer locked to look around, and the whole
//! screen.

use std::collections::BTreeSet;

use vertigo::{
    ClickEvent, Computed, KeyboardEvent, PointerEvent, Value, bind, component, css, dom,
    dom_element, get_driver, transaction,
};

#[component]
pub fn GameInputDemo() {
    let held = Value::new(BTreeSet::<String>::new());
    let presses = Value::new(0);
    let repeats = Value::new(0);
    let focused = Value::new(true);
    let visible = Value::new(true);

    // Not handled: the arrows go on to the menu's, which switch the tabs.
    let key_down = bind!(held, presses, repeats, |event: KeyboardEvent| {
        if event.repeat {
            repeats.change(|repeats| *repeats += 1);
        } else {
            presses.change(|presses| *presses += 1);
        }
        held.change(|held| {
            held.insert(event.code);
        });
        false
    });
    let key_up = bind!(held, |event: KeyboardEvent| {
        held.change(|held| {
            held.remove(&event.code);
        });
        false
    });
    // The keys let go elsewhere never come up here.
    let window_focus = bind!(held, focused, |now: bool| {
        focused.set(now);
        if !now {
            held.set(BTreeSet::new());
        }
    });
    let visibility = bind!(visible, |now: bool| visible.set(now));

    let held_text = held.map(|held| {
        if held.is_empty() {
            "Keys held: none".to_string()
        } else {
            let held = held.into_iter().collect::<Vec<_>>().join(" ");
            format!("Keys held: {held}")
        }
    });
    let presses_text = Computed::from(move |ctx| {
        format!(
            "Presses: {}, repeated: {}",
            presses.get(ctx),
            repeats.get(ctx)
        )
    });
    let focused_text = focused.map(|focused| format!("Window focused: {focused}"));
    let visible_text = visible.map(|visible| format!("Page visible: {visible}"));

    dom! {
        <div
            hook_key_down={key_down}
            hook_key_up={key_up}
            hook_window_focus={window_focus}
            hook_visibility={visibility}
        >
            <p>"Press keys anywhere on the page, and hold some."</p>
            <div>{held_text}</div>
            <div>{presses_text}</div>
            <div>{focused_text}</div>
            <div>{visible_text}</div>
            <LookAround />
            <WholeScreen />
        </div>
    }
}

/// A box which locks the pointer when clicked, adding up how far the mouse moves over it. L lets
/// the pointer go, as a click while it's locked goes to the box, wherever the mouse is.
#[component]
fn LookAround() {
    let locked = Value::new(false);
    let moved = Value::new((0.0, 0.0));

    let area = dom_element! {
        <div css={css! {"
            width: 300px;
            height: 120px;
            background: lightsteelblue;
            display: flex;
            align-items: center;
            justify-content: center;
        "}}>
            "Click to look around, L or Esc to stop"
        </div>
    };
    let area_ref = area.get_ref();
    let area = area
        .on_click(move |_: ClickEvent| area_ref.request_pointer_lock())
        .hook_pointer_lock(bind!(locked, |now: bool| locked.set(now)))
        .on_pointer_move(bind!(moved, |event: PointerEvent| {
            moved.change(|(x, y)| {
                *x += event.movement_x;
                *y += event.movement_y;
            });
        }));

    let let_go = |event: KeyboardEvent| {
        if event.code == "KeyL" {
            get_driver().exit_pointer_lock();
        }
        false
    };
    let locked_text = locked.map(|locked| format!("Pointer locked: {locked}"));
    let moved_text = moved.map(|(x, y)| format!("Moved: {x}, {y}"));

    dom! {
        <div hook_key_down={let_go}>
            {area}
            <div>{locked_text}</div>
            <div>{moved_text}</div>
        </div>
    }
}

/// A box shown on the whole screen and back by its button, which is in it.
#[component]
fn WholeScreen() {
    let whole = Value::new(false);
    let whole_text = whole.map(|whole| format!("Whole screen: {whole}"));

    let screen = dom_element! {
        <div css={css! {"padding: 10px; background: lightyellow;"}}>
            <div>{whole_text}</div>
        </div>
    };
    let screen_ref = screen.get_ref();
    let toggle = bind!(whole, screen_ref, |_: ClickEvent| {
        if transaction(|ctx| whole.get(ctx)) {
            get_driver().exit_fullscreen();
        } else {
            screen_ref.request_fullscreen();
        }
    });
    let screen = screen
        .hook_fullscreen(bind!(whole, |now: bool| whole.set(now)))
        .child(dom! { <button on_click={toggle}>"Whole screen"</button> });

    dom! { <div>{screen}</div> }
}
