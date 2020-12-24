
use crate::computed::{
    Value,
    Dependencies,
    Computed,
};
use crate::computed::tests::{
    BoxValue::BoxValue,
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

    let appState = AppState::new(&root);

    let suma3 = {
        let appState = appState.clone();

        root.from(move || -> i32 {
            let val1 = appState.value1.get_value();
            let val3 = appState.value3.get_value();

            *val1 + *val3
        })
    };


    let suma3Box: BoxValue<i32> = BoxValue::new(0);
    let sumaTotalBox: BoxValue<i32> = BoxValue::new(0);

    assert_eq!((suma3Box.get(), sumaTotalBox.get()), (0, 0));


    let suma3sub = {
        let suma3Box = suma3Box.clone();

        suma3.subscribe(move |value| {
            suma3Box.set(*value);
        })
    };

    appState.value1.set_value(2);

    assert_eq!((suma3Box.get(), sumaTotalBox.get()), (5, 0));

    appState.value1.set_value(3);
    assert_eq!((suma3Box.get(), sumaTotalBox.get()), (6, 0));

    appState.value2.set_value(4);
    assert_eq!((suma3Box.get(), sumaTotalBox.get()), (6, 0));

    appState.value2.set_value(5);
    assert_eq!((suma3Box.get(), sumaTotalBox.get()), (6, 0));

    appState.value3.set_value(6);
    assert_eq!((suma3Box.get(), sumaTotalBox.get()), (9, 0));

    appState.value3.set_value(7);
    assert_eq!((suma3Box.get(), sumaTotalBox.get()), (10, 0));

    suma3sub.off();
    appState.value3.set_value(8);
    assert_eq!((suma3Box.get(), sumaTotalBox.get()), (10, 0));


    let sumaTotal = {
        let sumaTotalBox = sumaTotalBox.clone();
        appState.suma.clone().subscribe(move |value| {
            sumaTotalBox.set(*value);
        })
    };

    assert_eq!((suma3Box.get(), sumaTotalBox.get()), (10, 16));

    appState.value1.set_value(2);
    assert_eq!((suma3Box.get(), sumaTotalBox.get()), (10, 15));

    appState.value2.set_value(3);
    assert_eq!((suma3Box.get(), sumaTotalBox.get()), (10, 13));

    appState.value3.set_value(4);
    assert_eq!((suma3Box.get(), sumaTotalBox.get()), (10, 9));

    sumaTotal.off();
}