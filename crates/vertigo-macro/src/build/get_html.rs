use super::config::EnvConfigIn;


pub fn get_html(config: &EnvConfigIn, wasm_run: impl Into<String>, wasm_name: impl Into<String>) -> String {
    let wasm_run = wasm_run.into();
    let wasm_name = wasm_name.into();

    let wasm_run = config.get_path_to_static(wasm_run);
    let wasm_bin = config.get_path_to_static(wasm_name);

    let left = "{";
    let right = "}";

    format!(r#"<!DOCTYPE html>
<html>
    <head>
        <meta charset="utf-8"/>
        <style type="text/css">
            * {left}
                box-sizing: border-box;
            {right}
            html, body {left}
                width: 100%;
                height: 100%;
                margin: 0;
                padding: 0;
                border: 0;
            {right}
        </style>
        <script type="module">
            import {left} runModule {right} from "{wasm_run}";
            runModule("{wasm_bin}");
        </script>
    </head>
    <body>
    </body>
</html>
"#)
}
