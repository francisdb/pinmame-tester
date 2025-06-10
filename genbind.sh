#!/usr/bin/env bash
set -e
if ! cargo install --list | grep -q "bindgen-cli"; then
    cargo install bindgen-cli
fi
~/.cargo/bin/bindgen ./pinmame/src/libpinmame/libpinmame.h -o src/libpinmame.rs -- -x c++
