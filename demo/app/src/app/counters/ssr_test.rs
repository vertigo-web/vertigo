use vertigo::{component, dom, get_driver};

#[component]
pub fn SsrTest() {
    let driver = get_driver();

    if driver.is_browser() {
        return dom! {
            <div>
                <div>
                    <div>
                        "aaaa"
                    </div>
                </div>
                <a>
                    "link 1 ddd"
                </a>
                <a>
                    "link 2"
                </a>
                <div>
                    "Content from browser"
                </div>
                <input type="text" value="ttttt2"/>
                <input type="text" value="ttttt1"/>
            </div>
        };
    }

    dom! {
        <div>
            <div>
                "Content from server"
            </div>

            <hr/>

            <a>
                "inny link"
            </a>

            <input type="text" value="ttttt1"/>
            <input type="text" value="ttttt2"/>

            <a href="ffff1">
                "link 1"
            </a>
            <a href="ffff2">
                "link 2"
            </a>
        </div>
    }
}
