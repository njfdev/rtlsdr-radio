ORIG_DIR="$PWD"

# go to build folder
mkdir build
cd build

# make lib and include folder
mkdir lib
mkdir include

BUILD_DIR="$PWD"

# build SoapySDR libs
if [ -d "./SoapySDR" ]; then
  # if already built
  cd SoapySDR
  git pull origin master
  cd build
  make -j4
else
  # if building for first time
  git clone https://github.com/pothosware/SoapySDR.git
  cd SoapySDR
  mkdir build
  cd build
  cmake ..
  make -j`nproc`
fi
mkdir $BUILD_DIR/include/SoapySDR
cp lib/*SoapySDR* $BUILD_DIR/lib/
cp ../include/SoapySDR/* $BUILD_DIR/include/SoapySDR/

# build SoapySDR RTL-SDR Hardware Support libs
cd "$BUILD_DIR"
git clone https://github.com/pothosware/SoapyRTLSDR.git
cd SoapyRTLSDR
mkdir build
cd build
cmake ..
make
mkdir $BUILD_DIR/lib/modules0.8
cp librtlsdrSupport.so $BUILD_DIR/lib/modules0.8/librtlsdrSupport.so

cd "$ORIG_DIR"