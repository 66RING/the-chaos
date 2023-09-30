cmake_minimum_required(VERSION 3.18)

if (NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release)
endif()
set(CMAKE_CXX_STANDARD 20)

SET(PROJECT_NAME client)
project(${PROJECT_NAME} LANGUAGES CXX)

add_executable(${PROJECT_NAME} client.cpp)
target_include_directories(${PROJECT_NAME} PUBLIC .)


