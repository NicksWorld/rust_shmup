#!/bin/bash
cargo build --target x86_64-unknown-linux-gnu 
mv -b ./target/x86_64-unknown-linux-gnu/debug/*.so ./lib/x86_64-unknown-linux-gnu
godot --path godot/ -d