# Neo LLVM: LLVM Backend for Neo N3 Smart Contracts

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Neo N3](https://img.shields.io/badge/Neo-N3-blue.svg)](https://neo.org)
[![LLVM](https://img.shields.io/badge/LLVM-16.0-orange.svg)](https://llvm.org)
[![Rust](https://img.shields.io/badge/Rust-1.70+-red.svg)](https://rust-lang.org)

A complete LLVM backend and Rust development framework for building Neo N3 smart contracts. This project enables developers to write high-performance smart contracts in Rust and compile them directly to NeoVM bytecode using LLVM's powerful optimization pipeline.

## 🚀 **Features**

### **Complete Neo N3 Support**
- ✅ **189+ Neo N3 Opcodes**: Full support for all Neo N3 virtual machine instructions
- ✅ **50+ System Calls**: Complete syscall bindings for Neo N3 runtime
- ✅ **NEF Generation**: Native Neo Executable Format (NEF) output with manifest
- ✅ **Stack-Based Translation**: Automatic conversion from LLVM's register-based IR to NeoVM's stack-based execution

### **Rust Development Framework**
- ✅ **Type-Safe APIs**: Comprehensive Neo N3 data types and runtime bindings
- ✅ **Procedural Macros**: `#[neo_contract]`, `#[neo_method]`, `#[neo_storage]` for easy contract development
- ✅ **Storage Abstractions**: High-level storage operations with type safety
- ✅ **Testing Framework**: Built-in testing utilities for smart contract development

### **Developer Experience**
- ✅ **Familiar Syntax**: Write contracts in standard Rust
- ✅ **Rich Tooling**: Full LLVM toolchain support for debugging and optimization
- ✅ **Gas Optimization**: Automatic optimization for minimal gas consumption
- ✅ **Documentation**: Comprehensive guides and examples

## 📋 **Quick Start**

### **Prerequisites**
- Rust 1.70+
- LLVM 16.0+
- CMake 3.20+

### **Installation**
```bash
# Clone the repository
git clone https://github.com/your-org/neo-llvm.git
cd neo-llvm

# Build the LLVM backend
mkdir build && cd build
cmake .. -DLLVM_ENABLE_PROJECTS="clang;lld"
make -j$(nproc)

# Build the Rust development framework
cd ../rust-devpack
cargo build --release
```

### **Your First Contract**
```rust
use neo_devpack::prelude::*;

#[neo_contract]
struct HelloWorld {
    message: NeoString,
}

impl HelloWorld {
    pub fn new() -> Self {
        Self {
            message: NeoString::from_str("Hello, Neo N3!"),
        }
    }
    
    #[neo_method]
    pub fn get_message(&self) -> NeoResult<NeoString> {
        Ok(self.message.clone())
    }
    
    #[neo_method]
    pub fn set_message(&mut self, new_message: NeoString) -> NeoResult<()> {
        self.message = new_message;
        Ok(())
    }
}
```

### **Compilation**
```bash
# Compile to NeoVM bytecode
cargo build --target neovm-unknown-unknown --release

# Generate NEF file with manifest
neo-llvm-tool generate-nef target/neovm-unknown-unknown/release/contract
```

## 🏗️ **Architecture**

### **LLVM Backend**
The Neo LLVM backend translates LLVM's register-based intermediate representation (IR) to NeoVM's stack-based execution model through a sophisticated stackification pass.

```
Rust Code → LLVM IR → Stackification → NeoVM Bytecode
```

**Key Components:**
- **Stackification Pass**: Converts SSA form to stack operations
- **Stack Height Tracking**: Maintains consistency across control flow
- **Instruction Selection**: Maps LLVM instructions to NeoVM opcodes
- **NEF Generation**: Creates Neo Executable Format files

### **Rust Framework**
The Rust development framework provides a complete SDK for Neo N3 smart contract development.

**Core Crates:**
- **`neo-types`**: Neo N3 data types and value representations
- **`neo-syscalls`**: System call bindings and runtime interface
- **`neo-runtime`**: Runtime utilities and storage operations
- **`neo-macros`**: Procedural macros for contract development
- **`neo-devpack`**: Main development package with prelude

## 📚 **Documentation**

### **Architecture & Design**
- **[Project Roadmap](docs/neo-llvm-roadmap.md)** - Project goals, architecture, and development roadmap
- **[Implementation Plan](docs/implementation-plan.md)** - Detailed technical implementation plan
- **[Neo N3 Backend](docs/neo-n3-backend.md)** - LLVM backend architecture and design

### **Technical Specifications**
- **[LLVM to NeoVM Translation](docs/llvm-to-neovm-translation.md)** - Technical explanation of register-based to stack-based translation
- **[NEF Format Specification](docs/nef-format-specification.md)** - Neo Executable Format specification
- **[Complete Neo N3 Support](docs/complete-neon3-support.md)** - Complete opcode and syscall documentation

### **Rust Integration**
- **[Rust Framework](docs/rust-framework.md)** - Rust development framework design
- **[Rust Integration](docs/rust-integration.md)** - Rust integration with LLVM backend

## 🎯 **Examples**

### **Storage Contract**
```rust
#[neo_contract]
struct StorageContract {
    owner: NeoByteString,
    data: NeoMap<NeoString, NeoValue>,
}

#[neo_storage]
struct ContractStorage {
    users: NeoMap<NeoByteString, UserData>,
    settings: NeoMap<NeoString, NeoValue>,
}
```

### **Token Contract**
```rust
#[neo_contract]
struct TokenContract {
    name: NeoString,
    symbol: NeoString,
    decimals: NeoInteger,
    total_supply: NeoInteger,
}

impl TokenContract {
    #[neo_method]
    pub fn transfer(&mut self, from: NeoByteString, to: NeoByteString, amount: NeoInteger) -> NeoResult<NeoBoolean> {
        // Implementation
    }
}
```

## 🧪 **Testing**

```bash
# Run all tests
cargo test

# Run examples
cargo run --example hello_world
cargo run --example storage_contract
cargo run --example token_contract

# Run comprehensive test suite
cargo test --test comprehensive_test_suite
```

## 🔧 **Development**

### **Building from Source**
```bash
# Build LLVM backend
cd llvm
mkdir build && cd build
cmake .. -DLLVM_ENABLE_PROJECTS="clang;lld" -DCMAKE_BUILD_TYPE=Release
make -j$(nproc)

# Build Rust framework
cd ../../rust-devpack
cargo build --release
```

### **Running Tests**
```bash
# LLVM backend tests
cd llvm/build
make check-llvm-codegen-neovm

# Rust framework tests
cd ../../rust-devpack
cargo test --all-features
```

## 🤝 **Contributing**

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

### **Development Workflow**
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## 📄 **License**

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 **Acknowledgments**

- **Neo Foundation** for the Neo N3 platform and virtual machine
- **LLVM Project** for the powerful compiler infrastructure
- **Rust Community** for the excellent language and ecosystem

## 📞 **Support**

- **Documentation**: [docs/](docs/)
- **Issues**: [GitHub Issues](https://github.com/your-org/neo-llvm/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-org/neo-llvm/discussions)

---

**Neo LLVM** - Bringing the power of LLVM and Rust to Neo N3 smart contract development.