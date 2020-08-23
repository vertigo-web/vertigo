Reactive webassembly
===================

Basic commands
--------------
- `cargo run` - Run project
- `cargo build` - Build project
- `cargo doc` - Build build documentation
- `cargo doc --open` - Open the documentation in the browser
- `cargo update --dry-run` - Checking for updates

Links
--------------
- https://rustwasm.github.io/docs/book/reference/crates.html
- https://webassembly.org/

Struktura aplikacji, szkic
--------------

AppDataState --- stan dotyczący danych                              (konstruktor tworzy Rc<AppDataState>)

AppViewState (wstrzyknięty AppDataState) - stan dotyczący widoku    (konstruktor tworzy Rc<AppDataState>)

AppState {
    data: AppDataState,
    view: AppViewState,
}