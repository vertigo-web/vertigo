use vertigo::{Computed, Value, bind, component, dom, tw};

#[component]
pub fn Tailwind() {
    let toogle_bg = Value::new(false);

    let my_class = Computed::from(bind!(toogle_bg, |context| {
        let mut my_class = tw!("p-[10px]");

        my_class += tw!("flex");

        if toogle_bg.get(context) {
            my_class + tw!("bg-green-500 text-red-800")
        } else {
            my_class + tw!("bg-green-900 text-[white]")
        }
    }));

    dom! {
        <div tw="m-10 flex flex-col gap-[10px]">
            <button
                type="button"
                tw="bg-[orange]/25 cursor-pointer p-[10px]"
                value={toogle_bg.clone()}
                on_click={bind!(toogle_bg, |_| {
                    toogle_bg.change(|inner| {
                        log::info!("current {}", inner);

                        *inner = !*inner;
                    });
                })}
            >
                "Switch background dd"
            </button>
            <div class="some-external-class" tw={my_class}>
                "Some tailwind-styled elements"
            </div>
            <div tw="bg-blue-400 w-full md:w-30 sm:w-20 p-[10px]">"Tailwind CSS 4 test"</div>
        </div>
    }
}
