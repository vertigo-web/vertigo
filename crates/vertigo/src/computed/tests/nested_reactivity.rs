use crate::{Computed, Value, transaction};

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

#[test]
fn test_nested_computed_subscription() {
    let token_value = Value::new("token1".to_string());
    let token_computed = token_value.to_computed();

    // bearer_auth equivalent: Computed<Option<Computed<String>>>
    let bearer_auth = Computed::from({
        let token_computed = token_computed.clone();
        move |_ctx| Some(token_computed.clone())
    });

    let counter = std::rc::Rc::new(std::cell::Cell::new(0));

    // The "flattened" revalidate_trigger equivalent
    let revalidate_trigger = Computed::from({
        let bearer_auth = bearer_auth.clone();
        move |ctx| bearer_auth.get(ctx).map(|c| c.get(ctx))
    });

    let _drop = revalidate_trigger.subscribe({
        let counter = counter.clone();
        move |_| {
            counter.set(counter.get() + 1);
        }
    });

    assert_eq!(counter.get(), 1); // Initial subscription fire

    token_value.set("token2".to_string());
    assert_eq!(counter.get(), 2); // Should fire because of flattening
}

#[test]
fn test_nested_computed_subscription_no_flattening() {
    let token_value = Value::new("token1".to_string());
    let token_computed = token_value.to_computed();

    let bearer_auth = Computed::from({
        let token_computed = token_computed.clone();
        move |_ctx| Some(token_computed.clone())
    });

    let counter = std::rc::Rc::new(std::cell::Cell::new(0));

    // NO flattening: subcribing directly to bearer_auth
    let _drop = bearer_auth.subscribe({
        let counter = counter.clone();
        move |_| {
            counter.set(counter.get() + 1);
        }
    });

    assert_eq!(counter.get(), 1); // Initial fire

    token_value.set("token2".to_string());
    assert_eq!(counter.get(), 1); // Does NOT fire because bearer_auth didn't change (the Option<Computed> is the same instance)
}
