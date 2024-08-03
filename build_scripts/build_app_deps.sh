ORIG_DIR="$PWD"

# go to build folder
mkdir build
cd build

# make lib and include folder
mkdir lib
mkdir include

BUILD_DIR="$PWD"

# build SoapySDR libs
git clone https://github.com/pothosware/SoapySDR.git
cd SoapySDR
git reset --hard ab6260680c37a80ed4a23719bebc854248290c8b
mkdir build
cd build
cmake ..
make -j`nproc`
# install libs
mkdir $BUILD_DIR/include/SoapySDR
mkdir $BUILD_DIR/lib/pkgconfig
cp lib/libSoapySDR* $BUILD_DIR/lib/
cp lib/SoapySDR.pc $BUILD_DIR/lib/pkgconfig/SoapySDR.pc
cp ../include/SoapySDR/* $BUILD_DIR/include/SoapySDR/

# build SoapySDR RTL-SDR Hardware Support libs
cd "$BUILD_DIR"
git clone https://github.com/pothosware/SoapyRTLSDR.git
cd SoapyRTLSDR
mkdir build
cd build
cmake ..
make
mkdir -p $BUILD_DIR/lib/SoapySDR/modules0.8
cp librtlsdrSupport.so $BUILD_DIR/lib/SoapySDR/modules0.8/librtlsdrSupport.so

cd "$ORIG_DIR"