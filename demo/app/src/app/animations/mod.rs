use std::rc::Rc;

use vertigo::{css_fn, css_fn_push, Value, bind, get_driver, struct_mut::ValueMut, DomElement, dom};

mod spinner;
use spinner::spinner;

#[derive(Clone)]
pub struct State {
    pub progress: Value<u32>,
    in_progress: Rc<ValueMut<bool>>,
}

impl State {
    pub fn new() -> State {
        State {
            progress: Value::new(0),
            in_progress: Rc::new(ValueMut::new(false)),
        }
    }

    pub async fn start_animation(self) {
        if self.in_progress.get() {
            return;
        }

        self.in_progress.set(true);

        for i in 0..50 {
            self.progress.set(i as u32);
            get_driver().sleep(20).await;
        }

        for i in (0..50).rev() {
            self.progress.set(i as u32);
            get_driver().sleep(10).await;
        }

        self.in_progress.set(false);
    }
}

css_fn! { css_bg, "
    border: 1px solid black;
    padding: 10px;
    background-color: #e0e0e0;
    margin-bottom: 10px;
" }

css_fn_push! { css_button, css_bg, "
    cursor: pointer;
" }

pub struct Animations { }

impl Animations {
    pub fn mount(&self) -> DomElement {
        let state = State::new();

        let ids = state.progress.map(|progress| {
            (0..progress).collect::<Vec<_>>()
        });

        let list = ids.render_list(
            |id| *id,
            |_id| dom!{
                <span>"."</span>
            }
        );

        let on_click_progress = bind!(|state| {
            let state = state.clone();
            get_driver().spawn(async move {
                state.start_animation().await;
            });
        });

        let on_mouse_enter = || {
            log::info!("mouse enter");
        };

        let on_mouse_leave = || {
            log::info!("mouse leave");
        };

        dom! {
            <div>
                <div css={css_bg()} on_mouse_enter={on_mouse_enter} on_mouse_leave={on_mouse_leave}>
                    { spinner() }
                </div>

                <button on_click={on_click_progress}>
                    <span>
                        "start the progress bar"
                    </span>
                    <span>
                        { list }
                    </span>
                </button>
            </div>
        }
    }
}
