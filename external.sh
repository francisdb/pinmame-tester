#!/usr/bin/env bash
set -e
if [[ $OSTYPE == 'darwin'* ]]; then
  brew install sdl2 sdl2_gfx sdl2_ttf
  brew link sdl2 sdl2_gfx sdl2_ttf
elif [[ $OSTYPE == 'linux-gnu'* ]]; then
  sudo apt-get update
  sudo apt-get install -y libsdl2-dev libsdl2-gfx-dev libsdl2-ttf-dev
fi
rm -rf pinmame
git clone --depth 1 https://github.com/vpinball/pinmame.git pinmame
rm -rf pinmame/.git
cd pinmame
if [[ $OSTYPE == 'darwin'* ]]; then
  if [[ $(uname -m) == 'arm64' ]]; then
    cp cmake/libpinmame/CMakeLists_osx-arm64.txt CMakeLists.txt
  else
    cp cmake/libpinmame/CMakeLists_osx-x64.txt CMakeLists.txt
  fi
elif [[ $OSTYPE == 'linux-gnu'* ]]; then
  cp cmake/libpinmame/CMakeLists_linux-x64.txt CMakeLists.txt
fi
cmake -DCMAKE_BUILD_TYPE=Release -B build/Release
cmake --build build/Release -- -j$(sysctl -n hw.ncpu)
# if [[ "${{ matrix.platform }}" == "linux-x64" ]]; then
#   upx --best --lzma build/Release/${{ matrix.libpinmame }}
# fi

# remove the dylib files (to make sure the rust linker does not use them)
rm -rf build/Release/*.dylib
