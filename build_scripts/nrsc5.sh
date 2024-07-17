# Note: FAAD2 is turned off because it causes issues in GitHub Actions and is not needed

# if pacman exists, install the dependencies
if [ -x "$(command -v pacman)" ]; then
pacman -S --noconfirm autoconf automake git gzip make mingw-w64-x86_64-gcc mingw-w64-x86_64-cmake mingw-w64-x86_64-libtool patch tar xz libtool cmake gcc
fi

mkdir build
cd build
git clone https://github.com/theori-io/nrsc5.git
cd nrsc5
git reset --hard a57dd5b5f93e08d9ccdeb5f6b670a16d7566f8f1
mkdir build
cd build
cmake -DUSE_FAAD2=OFF ../
make
cd ../../
mkdir bin
mv ./nrsc5/build/src/nrsc5 ./bin/nrsc5-$(rustc -vV | sed -n 's|host: ||p')
cd ../