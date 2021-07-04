import tsPlugin from '@rollup/plugin-typescript';

export default {
    input: './src/driver.ts',
    output: {
        file: './out/driver.js',
        format: 'es'
    },
    plugins: [
        tsPlugin(),
    ]
};
