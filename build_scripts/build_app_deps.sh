# go to build folder
mkdir build
cd build

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