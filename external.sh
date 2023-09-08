#!/usr/bin/env bash
set -e
rm -rf pinmame
git clone https://github.com/vpinball/pinmame.git
cd pinmame
if [[ $OSTYPE == 'darwin'* ]]; then
  cp cmake/libpinmame/CMakeLists_osx-arm64.txt CMakeLists.txt
elif [[ $OSTYPE == 'linux-gnu'* ]]; then
  cp cmake/libpinmame/CMakeLists_linux-x64.txt CMakeLists.txt
fi
cmake -DCMAKE_BUILD_TYPE=Release -B build/Release
cmake --build build/Release -- -j$(sysctl -n hw.ncpu)
# if [[ "${{ matrix.platform }}" == "linux-x64" ]]; then
#   upx --best --lzma build/Release/${{ matrix.libpinmame }}
# fi