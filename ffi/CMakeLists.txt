cmake_minimum_required(VERSION 3.12)

project(ffi)

if(WIN32)
    add_compile_options(/W4)
else()
    add_compile_options(-Wall -Wextra -Wno-unused-parameter)
endif()

find_package(LLD REQUIRED CONFIG)

add_library(ffi STATIC
    lld.cpp
    llvm.cpp
    llvm_block.cpp
    llvm_builder.cpp
    llvm_context.cpp
    llvm_function.cpp
    llvm_layout.cpp
    llvm_module.cpp
    llvm_target.cpp
    llvm_type.cpp)

target_compile_features(ffi PRIVATE cxx_std_20)
target_include_directories(ffi PRIVATE ${LLD_INCLUDE_DIRS})

install(TARGETS ffi)
