# Note: FAAD2 is turned off because it causes issues in GitHub Actions and is not needed

# if apt exists, install the dependencies
if [ -x "$(command -v apt-get)" ]; then
sudo apt-get install autoconf automake git gzip make patch tar xz libtool cmake gcc
fi

mkdir build
cd build
git clone https://github.com/theori-io/nrsc5.git
cd nrsc5
git reset --hard a57dd5b5f93e08d9ccdeb5f6b670a16d7566f8f1
mkdir build
cd build
cmake ../
make
cd ../../
mkdir bin
mv ./nrsc5/build/src/nrsc5 ./bin/nrsc5-$(rustc -vV | sed -n 's|host: ||p')
cd ../