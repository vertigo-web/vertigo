use vertigo::{component, dom, tw};

#[component]
pub fn Tailwind() {
    let mut my_class = tw!("py-10 bg-green-500");

    my_class += tw!("flex text-red-800");

    dom! {
        <div tw="m-10">
            <div class="some-external-class" tw={my_class}>
                "Some tailwind-styled elements"
            </div>
        </div>
    }
}
