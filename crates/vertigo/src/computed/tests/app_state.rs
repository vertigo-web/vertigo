use alloc::rc::Rc;
use crate::computed::{
    Value,
    Dependencies,
    Computed,
};
use crate::computed::tests::{
    box_value_version::SubscribeValueVer,
};

struct AppState {
    value1: Value<i32>,
    value2: Value<i32>,
    value3: Value<i32>,
    suma: Computed<i32>,
}

impl AppState {
    pub fn new(root: &Dependencies) -> Rc<AppState> {
        let value1 = root.new_value(1);
        let value2 = root.new_value(2);
        let value3 = root.new_value(3);

        let suma = {
            let com1 = value1.to_computed();
            let com2 = value2.to_computed();
            let com3 = value3.to_computed();

            root.from(move || {
                let val1 = com1.get_value();
                let val2 = com2.get_value();
                let val3 = com3.get_value();

                *val1 + *val2 + *val3
            })
        };

        Rc::new(AppState {
            value1,
            value2,
            value3,
            suma
        })
    }
}

#[test]
fn test_app_state() {
    let root = Dependencies::default();

    let app_state = AppState::new(&root);

    let suma3 = {
        let app_state = app_state.clone();

        root.from(move || -> i32 {
            let val1 = app_state.value1.get_value();
            let val3 = app_state.value3.get_value();

            *val1 + *val3
        })
    };


    let mut suma3_box = SubscribeValueVer::new(suma3);

    assert_eq!(suma3_box.get(), (4, 1));    //1 _ 3

    app_state.value1.set_value(2);          //2 _ 3

    assert_eq!(suma3_box.get(), (5, 2));

    app_state.value1.set_value(3);          //3 _ 3
    assert_eq!(suma3_box.get(), (6, 3));

    app_state.value2.set_value(4);          //3 _ 3
    assert_eq!(suma3_box.get(), (6, 3));

    app_state.value2.set_value(5);          //3 _ 3
    assert_eq!(suma3_box.get(), (6, 3));

    app_state.value3.set_value(6);          //3 _ 6
    assert_eq!(suma3_box.get(), (9, 4));

    app_state.value3.set_value(7);          //3 _ 7
    assert_eq!(suma3_box.get(), (10, 5));

    suma3_box.off();


    app_state.value3.set_value(8);

    assert_eq!(suma3_box.get(), (10, 5));

    let mut suma_total = SubscribeValueVer::new(app_state.suma.clone());

    assert_eq!((suma3_box.get(), suma_total.get()), ((10, 5), (16, 1)));

    app_state.value1.set_value(2);
    assert_eq!((suma3_box.get(), suma_total.get()), ((10, 5), (15, 2)));

    app_state.value2.set_value(3);
    assert_eq!((suma3_box.get(), suma_total.get()), ((10, 5), (13, 3)));

    app_state.value3.set_value(4);
    assert_eq!((suma3_box.get(), suma_total.get()), ((10, 5), (9, 4)));

    app_state.value3.set_value(4);
    assert_eq!((suma3_box.get(), suma_total.get()), ((10, 5), (9, 4)));

    suma_total.off();
}