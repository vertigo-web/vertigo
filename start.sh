set -e

cargo-watch --watch ./app --delay 0.5 -s "rm -Rf ./build && \
wasm-pack build app --no-typescript --target web --out-dir ../build --out-name app && \
cp ./app/index.html ./build && \
rm ./build/.gitignore && \
rm ./build/package.json && \
basic-http-server --addr 127.0.0.1:3000 ./build
"
