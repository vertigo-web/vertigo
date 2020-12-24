
use crate::computed::{
    Value,
    Dependencies,
    Computed,
};
use crate::computed::tests::{
    box_value::BoxValue,
};

struct AppState {
    value1: Value<i32>,
    value2: Value<i32>,
    value3: Value<i32>,
    suma: Computed<i32>,
}

impl AppState {
    pub fn new(root: &Dependencies) -> std::rc::Rc<AppState> {
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

        std::rc::Rc::new(AppState {
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


    let suma3_box: BoxValue<i32> = BoxValue::new(0);
    let suma_total_box: BoxValue<i32> = BoxValue::new(0);

    assert_eq!((suma3_box.get(), suma_total_box.get()), (0, 0));


    let suma3sub = {
        let suma3_box = suma3_box.clone();

        suma3.subscribe(move |value| {
            suma3_box.set(*value);
        })
    };

    app_state.value1.set_value(2);

    assert_eq!((suma3_box.get(), suma_total_box.get()), (5, 0));

    app_state.value1.set_value(3);
    assert_eq!((suma3_box.get(), suma_total_box.get()), (6, 0));

    app_state.value2.set_value(4);
    assert_eq!((suma3_box.get(), suma_total_box.get()), (6, 0));

    app_state.value2.set_value(5);
    assert_eq!((suma3_box.get(), suma_total_box.get()), (6, 0));

    app_state.value3.set_value(6);
    assert_eq!((suma3_box.get(), suma_total_box.get()), (9, 0));

    app_state.value3.set_value(7);
    assert_eq!((suma3_box.get(), suma_total_box.get()), (10, 0));

    suma3sub.off();
    app_state.value3.set_value(8);
    assert_eq!((suma3_box.get(), suma_total_box.get()), (10, 0));


    let suma_total = {
        let suma_total_box = suma_total_box.clone();
        app_state.suma.clone().subscribe(move |value| {
            suma_total_box.set(*value);
        })
    };

    assert_eq!((suma3_box.get(), suma_total_box.get()), (10, 16));

    app_state.value1.set_value(2);
    assert_eq!((suma3_box.get(), suma_total_box.get()), (10, 15));

    app_state.value2.set_value(3);
    assert_eq!((suma3_box.get(), suma_total_box.get()), (10, 13));

    app_state.value3.set_value(4);
    assert_eq!((suma3_box.get(), suma_total_box.get()), (10, 9));

    suma_total.off();
}