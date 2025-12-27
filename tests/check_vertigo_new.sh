#!/bin/bash

# Checks all templates are working
# This should be run from main directory of the repository

set -xe

cargo build --release --bin vertigo
mkdir -p build/vertigo-cli-test
cp target/release/vertigo build/vertigo-cli-test/
cd build/vertigo-cli-test

echo "*** Testing frontend template***"

./vertigo new frontend_app --template frontend
cd frontend_app
# Use local version of vertigo
sed -i "s/vertigo = .*/vertigo = { path = \"..\/..\/..\/crates\/vertigo\" }/" Cargo.toml
# Intercept workspace detection (fullstack template already got it)
echo "[workspace]" >> Cargo.toml
../vertigo build
du -sh build/*
cd ..

echo "*** Testing fullstack template***"

./vertigo new fullstack_app --template fullstack
cd fullstack_app
sed -i "s/vertigo = .*/vertigo = { path = \"..\/..\/..\/..\/crates\/vertigo\" }/" frontend/Cargo.toml
sed -i "s/vertigo = .*/vertigo = { path = \"..\/..\/..\/..\/crates\/vertigo\" }/" backend/Cargo.toml
sed -i "s/vertigo-cli = .*/vertigo-cli = { path = \"..\/..\/..\/..\/crates\/vertigo-cli\" }/" backend/Cargo.toml
../vertigo build
cargo check
du -sh build/*
cd ..

cd ..
rm -r vertigo-cli-test
cd ..
