cmake_minimum_required(VERSION 3.20)
project(win32_hooks LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_library(win32_hooks SHARED
    src/dllmain.cpp
    src/workerw_hook.cpp
)

target_include_directories(win32_hooks PUBLIC include)

if(WIN32)
    target_link_libraries(win32_hooks PRIVATE user32 gdi32 dwmapi)
endif()
