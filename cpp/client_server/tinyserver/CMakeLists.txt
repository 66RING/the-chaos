cmake_minimum_required(VERSION 3.18)

if (NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release)
endif()
set(CMAKE_CXX_STANDARD 20)

SET(PROJECT_NAME server)
project(${PROJECT_NAME} LANGUAGES CXX)

SET(LIBRARIES pthread event)
add_executable(${PROJECT_NAME} main.cpp server.cpp)
target_include_directories(${PROJECT_NAME} PUBLIC .)
TARGET_LINK_LIBRARIES(${PROJECT_NAME} ${LIBRARIES})

