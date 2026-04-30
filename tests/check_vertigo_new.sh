#!/bin/bash

# Checks all templates are working
# This should be run from main directory of the repository

set -xe

export MAIN_TARGET_DIR="${PWD}/target"

cargo build --locked --bin vertigo
mkdir -p build/vertigo-cli-test
cp target/debug/vertigo build/vertigo-cli-test/
cd build/vertigo-cli-test

echo "*** Testing frontend template***"

./vertigo new frontend_app --template frontend
cd frontend_app
# Use local version of vertigo
sed -i "s/vertigo = .*/vertigo = { path = \"..\/..\/..\/crates\/vertigo\" }/" Cargo.toml
# Intercept workspace detection (fullstack template already got it)
echo "[workspace]" >> Cargo.toml
ln -s "${MAIN_TARGET_DIR}" target
../vertigo build --release-mode false --wasm-opt true
du -sh build/*
cd ..

echo "*** Testing fullstack template***"

./vertigo new fullstack_app --template fullstack
cd fullstack_app
sed -i "s/vertigo = .*/vertigo = { path = \"..\/..\/..\/..\/crates\/vertigo\" }/" frontend/Cargo.toml
sed -i "s/vertigo = .*/vertigo = { path = \"..\/..\/..\/..\/crates\/vertigo\" }/" backend/Cargo.toml
sed -i "s/vertigo-cli = .*/vertigo-cli = { path = \"..\/..\/..\/..\/crates\/vertigo-cli\" }/" backend/Cargo.toml
ln -s "${MAIN_TARGET_DIR}" target
../vertigo build --release-mode false --wasm-opt true
cargo check --locked
du -sh build/*
cd ..

cd ..
rm -r vertigo-cli-test
cd ..
