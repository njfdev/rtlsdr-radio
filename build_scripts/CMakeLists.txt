cmake_minimum_required(VERSION 3.15)
project(ExternalDependencies)

include(ExternalProject)

# Set build directories
# set(BUILD_DIR ${CMAKE_BINARY_DIR})
# set(LIB_DIR ${BUILD_DIR}/lib)
# set(INCLUDE_DIR ${BUILD_DIR}/include)
# set(SHARE_DIR ${BUILD_DIR}/share)

# Set platform-specific library extensions
# if(WIN32)
#     set(LIB_EXT ".dll")
# elseif(APPLE)
#     set(LIB_EXT ".dylib")
# else()
#     set(LIB_EXT ".so")
# endif()

# Create directories
# file(MAKE_DIRECTORY ${LIB_DIR} ${INCLUDE_DIR} ${SHARE_DIR})

# ExternalProject_Add for SoapySDR
# ExternalProject_Add(
#     SoapySDR
#     GIT_REPOSITORY https://github.com/pothosware/SoapySDR.git
#     GIT_TAG ab6260680c37a80ed4a23719bebc854248290c8b
#     PREFIX SoapySDR
#     CMAKE_ARGS -DCMAKE_INSTALL_PREFIX=${BUILD_DIR}
#     BUILD_COMMAND ${CMAKE_COMMAND} --build . --target all
#     INSTALL_COMMAND ""
# )

# ExternalProject_Add for libusb
ExternalProject_Add(
  libusb
  PREFIX ${CMAKE_BINARY_DIR}/libusb
  GIT_REPOSITORY https://github.com/libusb/libusb.git
  GIT_TAG v1.0.26  # Specify a tag or branch
  UPDATE_COMMAND ""
  PATCH_COMMAND ""
  CONFIGURE_COMMAND <SOURCE_DIR>/autogen.sh && <SOURCE_DIR>/configure
  BUILD_COMMAND make <SOURCE_DIR>
  INSTALL_COMMAND make install DESTDIR=${CMAKE_CURRENT_SOURCE_DIR}/../build
)


# ExternalProject_Add for librtlsdr
# ExternalProject_Add(
#     rtl-sdr
#     GIT_REPOSITORY https://github.com/osmocom/rtl-sdr.git
#     GIT_TAG 619ac3186ea0ffc092615e1f59f7397e5e6f668c
#     PREFIX rtl-sdr
#     CMAKE_ARGS 
#         -DCMAKE_INSTALL_PREFIX=${BUILD_DIR}
#         -DLIBUSB_INCLUDE_DIR=${INCLUDE_DIR}/libusb-1.0
#         -DLIBUSB_LIBRARY=${LIB_DIR}/libusb-1.0${LIB_EXT}
#     BUILD_COMMAND ${CMAKE_COMMAND} --build . --target all
#     INSTALL_COMMAND ""
# )

# ExternalProject_Add for SoapyRTLSDR
# ExternalProject_Add(
#     SoapyRTLSDR
#     GIT_REPOSITORY https://github.com/pothosware/SoapyRTLSDR.git
#     PREFIX SoapyRTLSDR
#     CMAKE_ARGS 
#         -DRTLSDR_INCLUDE_DIR=${INCLUDE_DIR}
#         -DRTLSDR_LIBRARY=${LIB_DIR}/librtlsdr${LIB_EXT}
#         -DSoapySDR_DIR=${SHARE_DIR}/cmake/SoapySDR
#         -DCMAKE_INSTALL_PREFIX=${BUILD_DIR}
#     BUILD_COMMAND ${CMAKE_COMMAND} --build . --target all
#     INSTALL_COMMAND ""
# )
