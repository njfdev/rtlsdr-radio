#!/bin/bash

# Detect the operating system
OS=$(uname)

# Set the shared library extension based on the operating system
if [[ "$OS" == "Linux" ]]; then
    LIB_EXT=".so"
elif [[ "$OS" == "Darwin" ]]; then
    LIB_EXT=".dylib"

    # I spent 1.5 hours figuring out that this ENV variable was messing
    # up libusb build script on Apple Silicon.
    unset IPHONEOS_DEPLOYMENT_TARGET
elif [[ "$OS" == "CYGWIN"* || "$OS" == "MINGW"* || "$OS" == "MSYS"* ]]; then
    LIB_EXT=".dll"
else
    echo "Unsupported OS: $OS"
    exit 1
fi

ORIG_DIR="$PWD"

# go to build folder
mkdir build
cd build

# make lib, include, and share folder
mkdir lib
mkdir include
mkdir share

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
mkdir -p $BUILD_DIR/share/cmake/SoapySDR
cp lib/libSoapySDR* $BUILD_DIR/lib/
cp lib/SoapySDR.pc $BUILD_DIR/lib/pkgconfig/SoapySDR.pc
cp ../include/SoapySDR/* $BUILD_DIR/include/SoapySDR/
cp ../cmake/Modules/SoapySDR*.cmake $BUILD_DIR/share/cmake/SoapySDR/
cp ./SoapySDR*.cmake $BUILD_DIR/share/cmake/SoapySDR/
cp ./lib/CMakeFiles/Export/*/SoapySDRExport*.cmake $BUILD_DIR/share/cmake/SoapySDR/

# build libusb if libs don't already exist
if [ ! -f "$BUILD_DIR/lib/libusb-1.0$LIB_EXT" ]; then
  cd "$BUILD_DIR"
  git clone https://github.com/libusb/libusb.git
  cd libusb
  git reset --hard d52e355daa09f17ce64819122cb067b8a2ee0d4b
  ./autogen.sh
  ./configure
  make
  mkdir $BUILD_DIR/include/libusb-1.0
  cp ./libusb/libusb.h $BUILD_DIR/include/libusb-1.0/libusb.h
  cp ./libusb/.libs/libusb*$LIB_EXT $BUILD_DIR/lib
  cp ./libusb-1.0.pc $BUILD_DIR/lib/pkgconfig/libusb-1.0.pc
  sed -i '' "1s|.*|prefix=$BUILD_DIR|" $BUILD_DIR/lib/pkgconfig/libusb-1.0.pc
fi

# build librtlsdr
cd "$BUILD_DIR"
git clone https://github.com/osmocom/rtl-sdr.git
cd rtl-sdr
git reset --hard 619ac3186ea0ffc092615e1f59f7397e5e6f668c
# patch cmake file with custom libusb installation path
git checkout -- CMakeLists.txt
PATCH_DATA="/if(PKG_CONFIG_FOUND AND NOT LIBUSB_FOUND)/i\\
set(LIBUSB_LIBRARIES \"$BUILD_DIR/lib/libusb-1.0.0$LIB_EXT\")\\
set(LIBUSB_INCLUDE_DIRS \"$BUILD_DIR/include/libusb-1.0\")\\
set(LIBUSB_FOUND TRUE)\\
"
sed -i '' "$PATCH_DATA" CMakeLists.txt
mkdir build
cd build
cmake ../
make
cp src/librtlsdr* $BUILD_DIR/lib
cp ../include/{rtl-sdr.h,rtl-sdr_export.h} $BUILD_DIR/include

# build SoapySDR RTL-SDR Hardware Support libs
cd "$BUILD_DIR"
git clone https://github.com/pothosware/SoapyRTLSDR.git
cd SoapyRTLSDR
mkdir build
cd build
cmake -DRTLSDR_INCLUDE_DIR=$BUILD_DIR/include -DRTLSDR_LIBRARY=$BUILD_DIR/lib/librtlsdr$LIB_EXT -DSoapySDR_DIR=$BUILD_DIR/share/cmake/SoapySDR ..
make
mkdir -p $BUILD_DIR/lib/SoapySDR/modules0.8
cp librtlsdrSupport.so $BUILD_DIR/lib/SoapySDR/modules0.8/librtlsdrSupport.so

cd "$ORIG_DIR"