set -x
cargo build --release -p vertigo-cli
mkdir -p build/vertigo-cli-test
cp target/release/vertigo build/vertigo-cli-test/
cd build/vertigo-cli-test
./vertigo new some_app
cd some_app
# Use local version of vertigo
sed "s/vertigo = .*/vertigo = { path = \"..\/..\/..\/crates\/vertigo\" }/" Cargo.toml > Cargo.toml.new
# Intercept workspace detection
echo "[workspace]" >> Cargo.toml.new
mv Cargo.toml.new Cargo.toml
../vertigo build
du -sh build/*
cd ../..
rm -r vertigo-cli-test
