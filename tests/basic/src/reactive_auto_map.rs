use vertigo::{bind_rc, component, dom, log, ReactiveAutoMap, Value};

#[component]
pub fn ReactiveAutoMapTest() {
    let data: ReactiveAutoMap<String, String> = ReactiveAutoMap::new(|_map, key| {
        log::info!("creating {} value", key);
        format!("{} Test Value", key)
    });
    let trigger = Value::new("default");

    let clear = bind_rc!(data, trigger, || {
        trigger.set("triggered");
        data.clear();
    });

    let item = data.get("First".to_string());

    dom! {
        <div>
            <div id="automaptest-item">{item}</div>
            <div id="automaptest-trigger-state">{trigger}</div>
            <button id="automaptest-clear-button" on_click={clear}>"Clear"</button>
        </div>
    }
}
