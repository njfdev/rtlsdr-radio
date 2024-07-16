# Note: FAAD2 is turned off because it causes issues in GitHub Actions and is not needed

mkdir build
cd build
git clone https://github.com/theori-io/nrsc5.git
cd nrsc5
git reset --hard a57dd5b5f93e08d9ccdeb5f6b670a16d7566f8f1
mkdir build
cd build
if [ "$TAURI_PLATFORM" = "macos" ]; then
  if [ "$TAURI_ARCH" = "aarch64" ]; then
    cmake -DCMAKE_OSX_ARCHITECTURES=arm64 -DUSE_FAAD2=OFF ../
  else
    cmake -DCMAKE_OSX_ARCHITECTURES=$TAURI_ARCH -DUSE_FAAD2=OFF ../
  fi
    BINARY_BUILD_TARGET_NAME="$TAURI_ARCH-apple-darwin"
else
  cmake -DUSE_FAAD2=OFF ../
  BINARY_BUILD_TARGET_NAME=$(rustc -vV | sed -n 's|host: ||p')
fi
make
cd ../../
mkdir bin
mv ./nrsc5/build/src/nrsc5 ./bin/nrsc5-$BINARY_BUILD_TARGET_NAME