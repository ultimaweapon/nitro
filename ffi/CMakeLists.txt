cmake_minimum_required(VERSION 3.12)

project(ffi)

if(WIN32)
    add_compile_options(/W4)
else()
    add_compile_options(-Wall -Wextra)
endif()

find_package(LLD REQUIRED CONFIG)

add_library(ffi STATIC
    lld.cpp)

target_compile_features(ffi PRIVATE cxx_std_20)
target_include_directories(ffi PRIVATE ${LLD_INCLUDE_DIRS})

install(TARGETS ffi)
