fn test() {
    let css_pszczola = Computed::from(bind_ctx!(|context, pos_x, pos_y| {
        let left = pos_x.get(context) * pszczola;
        let top  = pos_y.get(context) * pszczola;

        css!("
            position: absolute;
            width: {pszczola}px;
            left: {left}px;
            top: {top}px;

            transition-property: top left;
            transition-duration: 1s;
        ")
    });


    let css_pszczola = Computed::from({
        let pos_x = pos_x.clone();
        let pos_y = pos_y.clone();

        move |context| {
            let left = pos_x.get(context) * pszczola;
            let top  = pos_y.get(context) * pszczola;

            css!("
                position: absolute;
                width: {pszczola}px;
                left: {left}px;
                top: {top}px;

                transition-property: top left;
                transition-duration: 1s;
            ")
        }
    });

}

    /*
        https://crates.io/crates/clone-macro

        https://crates.io/crates/enclose

        https://gtk-rs.org/gtk-rs-core/stable/latest/docs/glib/macro.clone.html

        https://github.com/rust-webplatform/rust-todomvc/blob/master/src/main.rs#L34
        
        bind(|| {

        })

        bind_ctx(|| {

        })

        bind(async || {

        })

        bind_ctx(async || {

        })

        jak rozwiązać kwestię częściowego wiązania

    let on_set = || bind!(|cell, possible_last_value| {
        cell.number.value.set(Some(*possible_last_value));
    })};


    let on_set = |param| bind!(|cell, possible_last_value, param| {
        cell.number.value.set(Some(*possible_last_value));
    })};

    klonowanie wewnątrz moze byc za późno
    */