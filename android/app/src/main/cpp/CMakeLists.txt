cmake_minimum_required(VERSION 3.10)
project(hub_android)

# Map Android ABIs to Rust targets
if(ANDROID_ABI STREQUAL "arm64-v8a")
    set(RUST_TARGET "aarch64-linux-android")
elseif(ANDROID_ABI STREQUAL "armeabi-v7a")
    set(RUST_TARGET "armv7-linux-androideabi")
elseif(ANDROID_ABI STREQUAL "x86_64")
    set(RUST_TARGET "x86_64-linux-android")
elseif(ANDROID_ABI STREQUAL "x86")
    set(RUST_TARGET "i686-linux-android")
else()
    message(FATAL_ERROR "Unknown Android ABI: ${ANDROID_ABI}")
endif()

set(RUST_HUB_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../../../../native/hub")
set(RUST_OUT_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../../../../target/${RUST_TARGET}")

message(STATUS "CMAKE_C_COMPILER: ${CMAKE_C_COMPILER}")
message(STATUS "CMAKE_CXX_COMPILER: ${CMAKE_CXX_COMPILER}")
message(STATUS "CMAKE_SYSROOT: ${CMAKE_SYSROOT}")
message(STATUS "ANDROID_NDK: ${ANDROID_NDK}")
message(STATUS "CMAKE_ANDROID_NDK: ${CMAKE_ANDROID_NDK}")

if(CMAKE_BUILD_TYPE MATCHES "Debug")
    set(CARGO_BUILD_TYPE "debug")
    set(CARGO_FLAGS "")
else()
    set(CARGO_BUILD_TYPE "release")
    set(CARGO_FLAGS "--release")
endif()

set(LIB_NAME "libhub.so")
set(LIB_PATH "${RUST_OUT_DIR}/${CARGO_BUILD_TYPE}/${LIB_NAME}")

# Add the dummy source to satisfy CMake
add_library(hub SHARED hub.cpp)

# Custom command to call cargo
# Note: We rely on the environment having the necessary Rust targets installed

# Map Rust target to Cargo environment variable prefix
string(REPLACE "-" "_" RUST_TARGET_UPPER "${RUST_TARGET}")
string(TOUPPER "${RUST_TARGET_UPPER}" RUST_TARGET_UPPER)

add_custom_command(
    OUTPUT "${LIB_PATH}"
    COMMAND ${CMAKE_COMMAND} -E env 
            "CC=${CMAKE_C_COMPILER}" 
            "CXX=${CMAKE_CXX_COMPILER}" 
            "CARGO_TARGET_${RUST_TARGET_UPPER}_LINKER=${CMAKE_C_COMPILER}"
            "RUSTC_WRAPPER="
            cargo build --manifest-path "${RUST_HUB_DIR}/Cargo.toml" --target ${RUST_TARGET} ${CARGO_FLAGS}
    WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/../../../../.."
    COMMENT "Building Rust library for ${ANDROID_ABI}..."
)

# Define a target that represents the Rust library
add_custom_target(run_cargo_build DEPENDS "${LIB_PATH}")
add_dependencies(hub run_cargo_build)

# Tell CMake where to find the library we just built so it can bundle it
set_target_properties(hub PROPERTIES IMPORTED_LOCATION "${LIB_PATH}")
