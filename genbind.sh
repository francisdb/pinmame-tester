#!/usr/bin/env bash
set -e
cargo install bindgen-cli
~/.cargo/bin/bindgen ./pinmame/src/libpinmame/libpinmame.h -o src/libpinmame.rs -- -x c++
