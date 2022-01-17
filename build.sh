#!/bin/bash
cargo build --target x86_64-unknown-linux-gnu
mkdir -p ./godot/libshmup_rust
mv -b ./target/x86_64-unknown-linux-gnu/debug/*.so ./godot/libshmup_rust/
