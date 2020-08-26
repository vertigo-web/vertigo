const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

const outPath = path.resolve(__dirname, "../build_www");

console.info('outPath', outPath);


module.exports = {
  entry: "./bootstrap.js",
  output: {
    path: outPath,
    filename: "bootstrap.js",
  },
  mode: "development",
  plugins: [
    new CopyWebpackPlugin(['index.html'])
  ],
};
