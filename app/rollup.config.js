import rust from "@wasm-tool/rollup-plugin-rust";

export default {
    input: {
        main: "Cargo.toml",
    },
    output: {
        dir: "dist/js",
        format: "iife",
        sourcemap: true,
    },
    plugins: [
        rust({
            serverPath: "js/",
            verbose: true,
        }),
    ],
};
