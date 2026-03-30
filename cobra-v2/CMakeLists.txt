cmake_minimum_required(VERSION 3.20)
project(cobra-v2 LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

find_package(LLVM REQUIRED CONFIG)
message(STATUS "Found LLVM ${LLVM_PACKAGE_VERSION}")
message(STATUS "Using LLVMConfig.cmake in: ${LLVM_DIR}")

include_directories(${LLVM_INCLUDE_DIRS})
separate_arguments(LLVM_DEFINITIONS_LIST NATIVE_COMMAND ${LLVM_DEFINITIONS})
add_definitions(${LLVM_DEFINITIONS_LIST})

include_directories(${CMAKE_SOURCE_DIR}/include)

add_executable(cobra-v2
    src/main.cpp
    src/PassPipeline.cpp
    src/passes/ConstantUnfold.cpp
    src/passes/InsnSubstitution.cpp
    src/passes/MBA.cpp
    src/passes/BogusCF.cpp
    src/passes/DeadCode.cpp
    src/passes/JunkInsertion.cpp
    src/passes/CFF.cpp
    src/utils/OpaquePredicates.cpp
)

llvm_map_components_to_libnames(LLVM_LIBS
    core
    irreader
    bitreader
    bitwriter
    support
    passes
)

target_link_libraries(cobra-v2 PRIVATE ${LLVM_LIBS})

# z3 is an optional LLVM dependency; add its lib dir if present so the linker can find it
find_library(Z3_LIB z3 HINTS /opt/homebrew/lib /opt/homebrew/Cellar/z3/4.15.0/lib)
if(Z3_LIB)
    target_link_libraries(cobra-v2 PRIVATE ${Z3_LIB})
endif()
