import sourcemaps from 'rollup-plugin-sourcemaps';
import typescript from '@rollup/plugin-typescript';

export default [
    {
        input: 'crates/vertigo/src/driver_module/src_js/api/command/dom/hydration.test.ts',
        output: [
            {
                sourcemap: true,
                file: 'build/hydration.test.js',
                format: 'cjs',
            }
        ],
        plugins: [
            typescript({
                sourceMap: true,
                inlineSources: true,
            }),
            sourcemaps(),
        ],
    },
    {
        input: 'crates/vertigo/src/driver_module/src_js/api/command/dom/dom_wire.test.ts',
        output: [
            {
                sourcemap: true,
                file: 'build/dom_wire.test.js',
                format: 'cjs',
            }
        ],
        plugins: [
            typescript({
                sourceMap: true,
                inlineSources: true,
            }),
            sourcemaps(),
        ],
    },
    {
        input: 'crates/vertigo/src/driver_module/src_js/api/command/dom/injects.test.ts',
        output: [
            {
                sourcemap: true,
                file: 'build/injects.test.js',
                format: 'cjs',
            }
        ],
        plugins: [
            typescript({
                sourceMap: true,
                inlineSources: true,
            }),
            sourcemaps(),
        ],
    },
    {
        input: 'crates/vertigo/src/driver_module/src_js/api/command/canvas.test.ts',
        output: [
            {
                sourcemap: true,
                file: 'build/canvas.test.js',
                format: 'cjs',
            }
        ],
        plugins: [
            typescript({
                sourceMap: true,
                inlineSources: true,
            }),
            sourcemaps(),
        ],
    },
    {
        input: 'crates/vertigo/src/driver_module/src_js/api/command/languages.test.ts',
        output: [
            {
                sourcemap: true,
                file: 'build/languages.test.js',
                format: 'cjs',
            }
        ],
        plugins: [
            typescript({
                sourceMap: true,
                inlineSources: true,
            }),
            sourcemaps(),
        ],
    },
    {
        input: 'crates/vertigo/src/driver_module/src_js/api/command/animationFrames.test.ts',
        output: [
            {
                sourcemap: true,
                file: 'build/animationFrames.test.js',
                format: 'cjs',
            }
        ],
        plugins: [
            typescript({
                sourceMap: true,
                inlineSources: true,
            }),
            sourcemaps(),
        ],
    },
    {
        input: 'crates/vertigo/src/driver_module/src_js/api/command/dom/callbackManager.test.ts',
        output: [
            {
                sourcemap: true,
                file: 'build/callbackManager.test.js',
                format: 'cjs',
            }
        ],
        plugins: [
            typescript({
                sourceMap: true,
                inlineSources: true,
            }),
            sourcemaps(),
        ],
    },
    {
        input: 'crates/vertigo/src/driver_module/src_js/api/command/fetchExec.test.ts',
        output: [
            {
                sourcemap: true,
                file: 'build/fetchExec.test.js',
                format: 'cjs',
            }
        ],
        plugins: [
            typescript({
                sourceMap: true,
                inlineSources: true,
            }),
            sourcemaps(),
        ],
    },
    {
        input: 'crates/vertigo/src/driver_module/src_js/api/websocket/connection.test.ts',
        output: [
            {
                sourcemap: true,
                file: 'build/connection.test.js',
                format: 'cjs',
            }
        ],
        plugins: [
            typescript({
                sourceMap: true,
                inlineSources: true,
            }),
            sourcemaps(),
        ],
    },
    {
        input: 'crates/vertigo/src/driver_module/src_js/api/websocket/websocket.test.ts',
        output: [
            {
                sourcemap: true,
                file: 'build/websocket.test.js',
                format: 'cjs',
            }
        ],
        plugins: [
            typescript({
                sourceMap: true,
                inlineSources: true,
            }),
            sourcemaps(),
        ],
    },
    {
        input: 'crates/vertigo/src/driver_module/src_js/jsjson.test.ts',
        output: [
            {
                sourcemap: true,
                file: 'build/jsjson.test.js',
                format: 'cjs',
            }
        ],
        plugins: [
            typescript({
                sourceMap: true,
                inlineSources: true,
            }),
            sourcemaps(),
        ],
    }
];
