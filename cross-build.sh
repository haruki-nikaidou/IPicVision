#!/bin/bash

# This script is used to compile the project to
# Linux x86_64
# Linux aarch64
# Windows x86_64
# apple M1

PROJECT_DIR=$(pwd)
PROJECT_NAME="ipic_vision"
BUILD_DIR=$PROJECT_DIR/build
export OPENSSL_STATIC=1

pushd "$PROJECT_DIR" || exit 1
if [ ! -d "src" ]; then
  echo "src directory not found"
  exit 1
fi

echo "Downloading rustup..."
rustup target add x86_64-unknown-linux-gnu   # Linux x86
rustup target add aarch64-unknown-linux-gnu  # Linux ARM64
rustup target add x86_64-pc-windows-gnu      # Windows x64
rustup target add aarch64-apple-darwin       # macOS ARM64 (M1)

echo "Cleaning up old builds..."
cargo clean

if [ ! -d "build" ]; then
  mkdir build
else
  rm -rf build/*
fi

echo "Building for Linux x86..."
cargo build --target=x86_64-unknown-linux-gnu --release
cp target/x86_64-unknown-linux-gnu/release/$PROJECT_NAME "$BUILD_DIR"/$PROJECT_NAME-linux-x86_64

echo "Building for Linux aarch64..."
cargo build --target=aarch64-unknown-linux-gnu --release
cp target/aarch64-unknown-linux-gnu/release/$PROJECT_NAME "$BUILD_DIR"/$PROJECT_NAME-linux-aarch64

echo "Building for Windows x86_64..."
cargo build --target=x86_64-pc-windows-gnu --release
cp target/x86_64-pc-windows-gnu/release/$PROJECT_NAME.exe "$BUILD_DIR"/$PROJECT_NAME-windows-x86_64.exe

echo "Building for macOS aarch64..."
cargo build --target=aarch64-apple-darwin --release
cp target/aarch64-apple-darwin/release/$PROJECT_NAME "$BUILD_DIR"/$PROJECT_NAME-macos-aarch64

echo "Done!"

popd || exit 1