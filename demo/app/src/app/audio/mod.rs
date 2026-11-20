//! Sound: a beep played over itself, a tune played, paused, stopped and looped, the volume of all
//! of it, and whether the browser lets the page make sound yet.

use vertigo::{
    Value,
    audio::{self, Music, Sound},
    bind, component, dom, dom_element, include_static,
};

#[component]
pub fn AudioDemo() {
    let beep = Sound::new(include_static!("./beep.wav"));
    let tune = Music::new(include_static!("./tune.wav"));
    let ended = Value::new(0);
    let looping = Value::new(false);

    let allowed = audio::allowed().map(|allowed| format!("Sound allowed: {allowed}"));
    let ended_text = ended.map(|ended| format!("Tune ended: {ended} times"));

    let play_beep = bind!(beep, |_| beep.play());
    let play_quiet = bind!(beep, |_| beep.play_at(0.3));
    let play = bind!(tune, |_| tune.play());
    let pause = bind!(tune, |_| tune.pause());
    let stop = bind!(tune, |_| tune.stop());
    let set_looping = bind!(tune, looping, |now: bool| {
        looping.set(now);
        tune.set_looping(now);
    });
    let volume = |value: String| {
        if let Ok(percent) = value.parse::<f32>() {
            audio::set_volume(percent / 100.0);
        }
    };
    let muted = |now: bool| audio::set_muted(now);

    let element = dom_element! {
        <div>
            <p>{allowed}</p>
            <button on_click={play_beep}>"Beep"</button>
            <button on_click={play_quiet}>"Quiet beep"</button>
            <div>
                <button on_click={play}>"Play tune"</button>
                <button on_click={pause}>"Pause tune"</button>
                <button on_click={stop}>"Stop tune"</button>
                <label>
                    <input type="checkbox" on_change_checked={set_looping} />
                    "Loop"
                </label>
            </div>
            <p>{ended_text}</p>
            <label>
                "Volume "
                <input type="range" min="0" max="100" value="100" on_input={volume} />
            </label>
            <label>
                <input type="checkbox" on_change_checked={muted} />
                "Mute"
            </label>
        </div>
    };
    element.append_drop_resource(tune.on_end(bind!(ended, || {
        ended.change(|ended| *ended += 1);
    })));

    dom! { <div>{element}</div> }
}
