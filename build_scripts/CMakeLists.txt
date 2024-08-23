cmake_minimum_required(VERSION 3.15)
project(ExternalDependencies)

include(ExternalProject)

# Set platform-specific library extensions
if(WIN32)
    set(LIB_EXT ".dll")
elseif(APPLE)
    set(LIB_EXT ".dylib")
else()
    set(LIB_EXT ".so")
endif()

# ExternalProject_Add for libusb
ExternalProject_Add(
  libusb
  PREFIX ${CMAKE_BINARY_DIR}/libusb
  GIT_REPOSITORY https://github.com/libusb/libusb-cmake.git
  GIT_TAG v1.0.27-1  # Specify a tag or branch
  UPDATE_COMMAND ""
  PATCH_COMMAND ""
  CONFIGURE_COMMAND ${CMAKE_COMMAND} <SOURCE_DIR>
  BUILD_COMMAND ${CMAKE_COMMAND} --build <SOURCE_DIR>
  INSTALL_COMMAND ${CMAKE_COMMAND} --install ${CMAKE_BINARY_DIR}
)

# ExternalProject_Add for librtlsdr
ExternalProject_Add(
    rtl-sdr
    PREFIX ${CMAKE_BINARY_DIR}/rtl-sdr
    GIT_REPOSITORY https://github.com/osmocom/rtl-sdr.git
    GIT_TAG v2.0.2
    UPDATE_COMMAND ""
    PATCH_COMMAND ""
    CONFIGURE_COMMAND ${CMAKE_COMMAND} -DLIBUSB_INCLUDE_DIR=${CMAKE_BINARY_DIR}/usr/local/include/libusb-1.0 -DLIBUSB_LIBRARY=${CMAKE_BINARY_DIR}/usr/local/lib/libusb-1.0${LIB_EXT} <SOURCE_DIR>
    BUILD_COMMAND ${CMAKE_COMMAND} --build <SOURCE_DIR>
    INSTALL_COMMAND ${CMAKE_COMMAND} --install ${CMAKE_BINARY_DIR}
)

# ExternalProject_Add for SoapySDR
ExternalProject_Add(
    SoapySDR
    PREFIX ${CMAKE_BINARY_DIR}/soapysdr
    GIT_REPOSITORY https://github.com/pothosware/SoapySDR.git
    GIT_TAG soapy-sdr-0.8.1  # Specify a tag or branch
    UPDATE_COMMAND ""
    PATCH_COMMAND ""
    CONFIGURE_COMMAND ${CMAKE_COMMAND} <SOURCE_DIR>
    BUILD_COMMAND ${CMAKE_COMMAND} --build <SOURCE_DIR>
    INSTALL_COMMAND ${CMAKE_COMMAND} --install ${CMAKE_BINARY_DIR}
)

# ExternalProject_Add for SoapyRTLSDR
ExternalProject_Add(
    SoapyRTLSDR
    PREFIX ${CMAKE_BINARY_DIR}/soapyrtlsdr
    GIT_REPOSITORY https://github.com/pothosware/SoapyRTLSDR.git
    GIT_TAG soapy-rtl-sdr-0.3.3
    UPDATE_COMMAND ""
    PATCH_COMMAND ""
    CONFIGURE_COMMAND ${CMAKE_COMMAND} -DRTLSDR_INCLUDE_DIR=${CMAKE_BINARY_DIR}/usr/local/include -DRTLSDR_LIBRARY=${CMAKE_BINARY_DIR}/usr/local/lib/librtlsdr${LIB_EXT} -DSoapySDR_DIR=${CMAKE_BINARY_DIR}/usr/local/share/cmake/SoapySDR <SOURCE_DIR>
    BUILD_COMMAND ${CMAKE_COMMAND} --build <SOURCE_DIR>
    INSTALL_COMMAND ${CMAKE_COMMAND} --install ${CMAKE_BINARY_DIR}
)
