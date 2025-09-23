#!/bin/bash

# NeoVM LLVM Backend Build Script
# This script builds the complete NeoVM LLVM backend and development framework

set -e

echo "Building NeoVM LLVM Backend..."
echo "================================="

# Check if we're in the right directory
if [ ! -f "../CMakeLists.txt" ]; then
    echo "Error: Please run this script from the build directory"
    exit 1
fi

# Clean previous build
echo "Cleaning previous build..."
make clean || true

# Configure CMake
echo "Configuring CMake..."
cmake .. -DCMAKE_BUILD_TYPE=Release \
         -DLLVM_TARGETS_TO_BUILD="NeoVM" \
         -DLLVM_ENABLE_PROJECTS="clang" \
         -DLLVM_ENABLE_ASSERTIONS=ON \
         -DLLVM_ENABLE_EXPENSIVE_CHECKS=ON

# Build the backend
echo "Building NeoVM backend..."
make -j$(nproc) NeoVMCodeGen

# Build test programs
echo "Building test programs..."
g++ -std=c++17 test_end_to_end.cpp -o test_end_to_end
g++ -std=c++17 test_target_registration.cpp -o test_target_registration
g++ -std=c++17 minimal_test.cpp -o minimal_test

# Build NEF integration test
echo "Building NEF integration test..."
g++ -std=c++17 test_nef_integration.cpp -o test_nef_integration

# Build Rust codegen backend
echo "Building Rust codegen backend..."
cd ../rust-codegen-backend
cargo build --release
cd ../build

# Run tests
echo "Running tests..."
echo "================="

echo "1. Testing target registration..."
./test_target_registration

echo -e "\n2. Testing end-to-end pipeline..."
./test_end_to_end

echo -e "\n3. Testing minimal functionality..."
./minimal_test

echo -e "\n4. Testing NEF integration..."
./test_nef_integration

echo -e "\n5. Testing Rust codegen backend..."
cd ../rust-codegen-backend
cargo test
cd ../build

echo -e "\nBuild completed successfully!"
echo "NeoVM LLVM backend is ready for development."
echo ""
echo "Next steps:"
echo "1. Test with real NeoVM emulator"
echo "2. Implement remaining syscall registry features"
echo "3. Add more comprehensive test cases"
echo "4. Create documentation and examples"
