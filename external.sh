#!/usr/bin/env bash
set -e
if [[ $OSTYPE == 'darwin'* ]]; then
  brew install sdl2 sdl2_gfx sdl2_ttf
elif [[ $OSTYPE == 'linux-gnu'* ]]; then
  sudo apt-get update
  sudo apt-get install -y libsdl2-dev libsdl2-gfx-dev libsdl2-ttf-dev
fi
rm -rf pinmame
# Pinned: later libpinmame.h revisions include the C++ stdlib headers (<cstdint>
# etc.) in C++ mode, which leak libc++ templates into the bindgen output, and
# changed the callback userData from `const void*` to `void* const`. This tag
# matches the API the Rust source is written against.
PINMAME_TAG=v3.7.0-46-2b48173
git clone --depth 1 --branch "$PINMAME_TAG" https://github.com/vpinball/pinmame.git pinmame
rm -rf pinmame/.git
cd pinmame
cp cmake/libpinmame/CMakeLists.txt .
if [[ "$(uname)" == "Darwin" ]]; then
  NUM_PROCS=$(sysctl -n hw.ncpu)
else
  NUM_PROCS=$(nproc)
fi
PLATFORM=unknown
ARCH=unknown
if [[ $OSTYPE == 'darwin'* ]]; then
  PLATFORM=macos
  if [[ $(uname -m) == 'arm64' ]]; then
    ARCH=arm64
  else
    ARCH=x64
  fi
elif [[ $OSTYPE == 'linux-gnu'* ]]; then
  PLATFORM=linux
  ARCH=x64
fi
echo "Building pinmame for platform: ${PLATFORM}, arch: ${ARCH} with ${NUM_PROCS} threads"
cmake -DCMAKE_BUILD_TYPE=Release -DPLATFORM=${PLATFORM} -DARCH=${ARCH} -B build/Release
cmake --build build/Release -- -j${NUM_PROCS}

# remove the dylib files (to make sure the rust linker does not use them)
rm -rf build/Release/*.dylib
