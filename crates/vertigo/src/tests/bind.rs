#[test]
fn test_bind() {
    use crate as vertigo;
    use crate::{bind, bind_rc, bind_spawn};

    let state = 2;

    let on_click_progress = bind_spawn!(state, async move {
        println!("state: = {state}");
    });

    on_click_progress();

    let on_click = bind!(state, || -> i32 { state + 100 });

    assert_eq!(on_click(), 102);

    let on_click2 = bind!(state, || -> i32 { state + 100 });

    assert_eq!(on_click2(), 102);

    let on_click3: std::rc::Rc<dyn Fn()> = bind_rc!(state, || {
        let _aaa = state + 100;
    });

    on_click3();
}

#[test]
fn test_bind_namespaced_params() {
    use crate as vertigo;
    use crate::{bind, bind_rc, bind_spawn};

    pub struct State {
        pub value: i32,
    }

    let state = State { value: 2 };

    let on_click_progress = bind_spawn!(state.value, async move {
        println!("state: = {value}");
    });

    on_click_progress();

    let on_click = bind!(state.value, || value + 100);

    assert_eq!(on_click(), 102);

    let on_click2 = bind!(state.value, || value + 100);

    assert_eq!(on_click2(), 102);

    let on_click3: std::rc::Rc<dyn Fn()> = bind_rc!(state.value, || {
        let _aaa = value + 100;
    });

    on_click3();
}
