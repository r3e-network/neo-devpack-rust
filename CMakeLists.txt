cmake_minimum_required(VERSION 3.20)

project(NeoLLVM LANGUAGES CXX)

# Use the host LLVM installation for core infrastructure. The NeoVM backend is
# built out-of-tree, so we rely on LLVM's exported CMake package to supply the
# common macros (AddLLVM, TableGen, etc.) along with include paths and
# toolchain metadata like `llvm-tblgen`.
find_package(LLVM REQUIRED CONFIG)

message(STATUS "Found LLVM ${LLVM_PACKAGE_VERSION} at ${LLVM_DIR}")

list(APPEND CMAKE_MODULE_PATH "${LLVM_CMAKE_DIR}")
include(AddLLVM)
include(TableGen)

if (NOT LLVM_TABLEGEN_EXE)
  find_program(LLVM_TABLEGEN_EXE NAMES llvm-tblgen
    HINTS "${LLVM_TOOLS_BINARY_DIR}" "${LLVM_BINARY_DIR}/bin")
  if (NOT LLVM_TABLEGEN_EXE)
    message(FATAL_ERROR "Unable to locate llvm-tblgen; set LLVM_TABLEGEN_EXE")
  endif()
endif()
set(LLVM_TABLEGEN_EXE "${LLVM_TABLEGEN_EXE}" CACHE FILEPATH "Path to llvm-tblgen" FORCE)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

add_definitions(${LLVM_DEFINITIONS})

include_directories(SYSTEM ${LLVM_INCLUDE_DIRS})
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/llvm/include)

if (NOT DEFINED LLVM_VERSION_MAJOR)
  string(REGEX MATCH "^[0-9]+" LLVM_VERSION_MAJOR "${LLVM_PACKAGE_VERSION}")
endif()

find_path(LLVM_EXTRA_INCLUDE_DIR
  NAMES llvm/ADT/Triple.h
  HINTS ${LLVM_INCLUDE_DIRS}
        ${LLVM_INCLUDE_DIR}
        /usr/include/llvm-${LLVM_VERSION_MAJOR}
        /usr/include/llvm
        /usr/include
        /usr/local/include)
if (NOT LLVM_EXTRA_INCLUDE_DIR)
  set(_llvm_default_include "/usr/include/llvm-${LLVM_VERSION_MAJOR}")
  if (EXISTS "${_llvm_default_include}")
    set(LLVM_EXTRA_INCLUDE_DIR "${_llvm_default_include}")
  endif()
endif()
if (LLVM_EXTRA_INCLUDE_DIR)
  set(LLVM_EXTRA_INCLUDE_DIR "${LLVM_EXTRA_INCLUDE_DIR}" CACHE PATH "Additional LLVM include dir" FORCE)
  include_directories(SYSTEM ${LLVM_EXTRA_INCLUDE_DIR})
endif()

if (NOT LLVM_TARGETS_TO_BUILD)
  set(LLVM_TARGETS_TO_BUILD NeoVM)
else()
  list(FIND LLVM_TARGETS_TO_BUILD NeoVM _neovm_index)
  if (_neovm_index EQUAL -1)
    list(APPEND LLVM_TARGETS_TO_BUILD NeoVM)
  endif()
endif()
set(LLVM_TARGETS_TO_BUILD "${LLVM_TARGETS_TO_BUILD}" CACHE STRING "Targets to build" FORCE)

add_subdirectory(llvm/lib/Target)
