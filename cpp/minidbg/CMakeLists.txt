cmake_minimum_required(VERSION 3.0)
project(minidbg)

add_compile_options(-std=c++14)

include_directories(ext/libelfin ext/linenoise include)
add_executable(minidbg src/main.cpp ext/linenoise/linenoise.c)






