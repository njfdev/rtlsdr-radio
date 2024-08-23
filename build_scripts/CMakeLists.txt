cmake_minimum_required(VERSION 3.15)
project(ExternalDependencies)

include(ExternalProject)

set(RTLSDR "librtlsdr")

# Set platform-specific library extensions
if(WIN32)
    set(LIB_EXT ".lib")

    set(RTLSDR_DEPENDS_ON pthreads)

    # ExternalProject_Add for pthreads
    ExternalProject_Add(
        pthreads
        PREFIX ${CMAKE_CURRENT_BINARY_DIR}/pthreads
        GIT_REPOSITORY https://github.com/GerHobbelt/pthread-win32.git
        GIT_TAG master
        UPDATE_COMMAND ""
        PATCH_COMMAND ""
        CONFIGURE_COMMAND ${CMAKE_COMMAND} -DCMAKE_INSTALL_PREFIX=${CMAKE_CURRENT_BINARY_DIR}/dist -S <SOURCE_DIR> -B <BINARY_DIR>
        BUILD_COMMAND ${CMAKE_COMMAND} --build <BINARY_DIR> --config Release
        INSTALL_COMMAND ${CMAKE_COMMAND} -P <BINARY_DIR>/cmake_install.cmake --prefix ${CMAKE_CURRENT_BINARY_DIR}/dist
    )

    list(APPEND PTHREAD_INCLUDE_ARGUMENTS 
        -DTHREADS_PTHREADS_INCLUDE_DIR=${CMAKE_CURRENT_BINARY_DIR}/dist/include/
        -DTHREADS_PTHREADS_LIBRARY=${CMAKE_CURRENT_BINARY_DIR}/dist/lib/pthreadVSE3${LIB_EXT}
    )
elseif(APPLE)
    set(LIB_EXT ".dylib")
else()
    set(LIB_EXT ".so")
endif()

if(WIN32)
    set(RTLSDR_LIBRARIES "${CMAKE_CURRENT_BINARY_DIR}/dist/lib/rtlsdr.lib;${CMAKE_CURRENT_BINARY_DIR}/dist/bin/rtlsdr.dll")
else()
    set(RTLSDR_LIBRARIES ${CMAKE_CURRENT_BINARY_DIR}/dist/lib/librtlsdr${LIB_EXT})
endif()

ExternalProject_Add(
  libusb
  PREFIX ${CMAKE_CURRENT_BINARY_DIR}/libusb
  GIT_REPOSITORY https://github.com/libusb/libusb-cmake.git
  GIT_TAG v1.0.27-1  # Specify a tag or branch
  UPDATE_COMMAND ""
  PATCH_COMMAND ""
  CONFIGURE_COMMAND ${CMAKE_COMMAND} -DCMAKE_INSTALL_PREFIX=${CMAKE_CURRENT_BINARY_DIR}/dist -S <SOURCE_DIR> -B <BINARY_DIR>
  BUILD_COMMAND ${CMAKE_COMMAND} --build <BINARY_DIR> --config Release
  INSTALL_COMMAND ${CMAKE_COMMAND} --build <BINARY_DIR> --target install
)

# ExternalProject_Add for librtlsdr
ExternalProject_Add(
    rtl-sdr
    PREFIX ${CMAKE_CURRENT_BINARY_DIR}/rtl-sdr
    GIT_REPOSITORY https://github.com/osmocom/rtl-sdr.git
    GIT_TAG v2.0.2
    DEPENDS ${RTLSDR_DEPENDS_ON} libusb
    UPDATE_COMMAND ""
    PATCH_COMMAND ""
    CONFIGURE_COMMAND ${CMAKE_COMMAND} -S <SOURCE_DIR> -B <BINARY_DIR> -DCMAKE_PREFIX_PATH=${CMAKE_CURRENT_BINARY_DIR}/dist -DLIBUSB_INCLUDE_DIRS=${CMAKE_CURRENT_BINARY_DIR}/dist/include/libusb-1.0 -DLIBUSB_LIBRARIES=${CMAKE_CURRENT_BINARY_DIR}/dist/lib/libusb-1.0${LIB_EXT} ${PTHREAD_INCLUDE_ARGUMENTS}
    BUILD_COMMAND ${CMAKE_COMMAND} --build <BINARY_DIR> --config Release
    INSTALL_COMMAND ${CMAKE_COMMAND} --install <BINARY_DIR> --prefix ${CMAKE_CURRENT_BINARY_DIR}/dist
)

# ExternalProject_Add for SoapySDR
ExternalProject_Add(
    SoapySDR
    PREFIX ${CMAKE_CURRENT_BINARY_DIR}/soapysdr
    GIT_REPOSITORY https://github.com/pothosware/SoapySDR.git
    GIT_TAG soapy-sdr-0.8.1  # Specify a tag or branch
    UPDATE_COMMAND ""
    PATCH_COMMAND ""
    CONFIGURE_COMMAND ${CMAKE_COMMAND} -DCMAKE_INSTALL_PREFIX=${CMAKE_CURRENT_BINARY_DIR}/dist -S <SOURCE_DIR> -B <BINARY_DIR>
    BUILD_COMMAND ${CMAKE_COMMAND} --build <BINARY_DIR> --config Release
    INSTALL_COMMAND ${CMAKE_COMMAND} --build <BINARY_DIR> --target install
)

# ExternalProject_Add for SoapyRTLSDR
ExternalProject_Add(
    SoapyRTLSDR
    PREFIX ${CMAKE_CURRENT_BINARY_DIR}/soapyrtlsdr
    GIT_REPOSITORY https://github.com/pothosware/SoapyRTLSDR.git
    GIT_TAG soapy-rtl-sdr-0.3.3
    DEPENDS SoapySDR rtl-sdr
    UPDATE_COMMAND ""
    PATCH_COMMAND ""
    CONFIGURE_COMMAND ${CMAKE_COMMAND} -S <SOURCE_DIR> -B <BINARY_DIR> -DCMAKE_INSTALL_PREFIX=${CMAKE_CURRENT_BINARY_DIR}/dist -DCMAKE_PREFIX_PATH=${CMAKE_CURRENT_BINARY_DIR}/dist -DRTLSDR_INCLUDE_DIRS=${CMAKE_CURRENT_BINARY_DIR}/dist/include -DRTLSDR_LIBRARIES=${RTLSDR_LIBRARIES} -DSoapySDR_DIR=${CMAKE_CURRENT_BINARY_DIR}/dist/lib/cmake/SoapySDR
    BUILD_COMMAND ${CMAKE_COMMAND} --build <BINARY_DIR> --config Release
    INSTALL_COMMAND ${CMAKE_COMMAND} --build <BINARY_DIR> --target install
)