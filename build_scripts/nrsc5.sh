# Note: FAAD2 is turned off because it causes issues in GitHub Actions and is not needed

# if apt exists, install the dependencies
if [ -x "$(command -v apt-get)" ]; then
sudo apt-get install autoconf automake git gzip make patch tar libtool cmake gcc
fi

mkdir build
cd build
git clone https://github.com/njfdev/nrsc5.git
cd nrsc5
mkdir build
cd build
cmake ../
make
cd ../../
mkdir bin
mv ./nrsc5/build/src/nrsc5 ./bin/nrsc5-$(rustc -vV | sed -n 's|host: ||p')
cd ../