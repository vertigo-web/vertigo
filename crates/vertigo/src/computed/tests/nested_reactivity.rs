use crate::{transaction, Computed, Value};

#[test]
fn test_nested_reactivity() {
    let val = Value::new(1);

    // Generate computed from computed
    let computed = Computed::from({
        let val = val.clone();
        move |ctx| val.to_computed().get(ctx)
    });

    let mut result = 0;
    transaction(|ctx| {
        result = computed.get(ctx);
    });
    assert_eq!(result, 1);

    val.set(2);

    transaction(|ctx| {
        result = computed.get(ctx);
    });
    assert_eq!(result, 2);
}
