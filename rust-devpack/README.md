# Neo N3 Rust Development Pack

A complete Rust SDK for Neo N3 smart contract development, providing a well-designed syntax and comprehensive functionality for building production-ready smart contracts.

## 🚀 Features

- **Complete Neo N3 Type System**: Full support for all Neo N3 data types
- **System Call Integration**: Direct access to all Neo N3 system calls
- **Storage Operations**: Comprehensive storage management with type safety
- **Event System**: Built-in event emission and handling
- **Macro System**: Powerful procedural macros for contract development
- **Runtime Integration**: Complete runtime environment for smart contracts
- **Testing Framework**: Built-in testing and benchmarking support

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
neo-devpack = "0.1.0"
```

## 🎯 Quick Start

### Basic Contract

```rust
use neo_devpack::prelude::*;

#[neo_contract]
pub struct HelloWorld {
    greeting: NeoString,
}

impl HelloWorld {
    #[neo_method]
    pub fn say_hello(&self) -> NeoResult<NeoString> {
        Ok(self.greeting.clone())
    }
    
    #[neo_method]
    pub fn set_greeting(&mut self, greeting: NeoString) -> NeoResult<()> {
        self.greeting = greeting;
        Ok(())
    }
}

#[neo_entry]
pub fn deploy() -> NeoResult<()> {
    // Contract deployment logic
    Ok(())
}
```

### Token Contract

```rust
use neo_devpack::prelude::*;

#[neo_contract]
pub struct TokenContract {
    name: NeoString,
    symbol: NeoString,
    total_supply: NeoInteger,
}

#[neo_storage]
pub struct TokenStorage {
    balances: NeoMap<NeoByteString, NeoInteger>,
    allowances: NeoMap<NeoByteString, NeoMap<NeoByteString, NeoInteger>>,
}

impl TokenContract {
    #[neo_method]
    pub fn transfer(&mut self, to: &NeoByteString, amount: NeoInteger) -> NeoResult<NeoBoolean> {
        // Transfer logic with storage operations
        let mut storage = TokenStorage::load(&NeoRuntime::get_storage_context()?)?;
        
        // Implementation details...
        Ok(NeoBoolean::TRUE)
    }
}
```

## 🏗️ Architecture

### Core Components

1. **neo-types**: Core Neo N3 data types and structures
2. **neo-syscalls**: System call bindings and wrappers
3. **neo-runtime**: Runtime environment and utilities
4. **neo-macros**: Procedural macros for contract development

### Type System

```rust
// Primitive types
let int_value = NeoInteger::new(42);
let bool_value = NeoBoolean::new(true);
let string_value = NeoString::from_str("Hello, Neo!");
let byte_string = NeoByteString::from_slice(b"data");

// Collection types
let mut array = NeoArray::new();
array.push(NeoValue::from(int_value));

let mut map = NeoMap::new();
map.insert(NeoValue::from(string_value), NeoValue::from(int_value));

// Complex types
let mut struct_data = NeoStruct::new();
struct_data.set_field("name", NeoValue::from(string_value));
```

### Storage Operations

```rust
#[neo_storage]
pub struct MyStorage {
    data: NeoMap<NeoString, NeoValue>,
    counters: NeoMap<NeoString, NeoInteger>,
}

impl MyStorage {
    pub fn load(context: &NeoStorageContext) -> NeoResult<Self> {
        // Load from storage
    }
    
    pub fn save(&self, context: &NeoStorageContext) -> NeoResult<()> {
        // Save to storage
    }
}
```

### System Calls

```rust
// Runtime operations
let timestamp = NeoRuntime::get_time()?;
let gas_left = NeoRuntime::get_gas_left()?;
let caller = NeoRuntime::get_calling_script_hash()?;

// Storage operations
let context = NeoRuntime::get_storage_context()?;
let value = NeoStorage::get(&context, &key)?;
NeoStorage::put(&context, &key, &value)?;

// Crypto operations
let hash = NeoCrypto::sha256(&data)?;
let signature_valid = NeoCrypto::verify_signature(&message, &signature, &public_key)?;
```

### Events

```rust
#[neo_event]
pub struct TransferEvent {
    pub from: NeoByteString,
    pub to: NeoByteString,
    pub amount: NeoInteger,
}

// Emit event
let event = TransferEvent {
    from: sender,
    to: receiver,
    amount: value,
};
event.emit()?;
```

To describe manifest metadata (events, permissions, supported standards, trusts, etc.) use the supplied macros instead of maintaining JSON by hand:

```rust
use neo_devpack::prelude::*;

#[neo_event]
pub struct ApprovalEvent {
    pub owner: NeoByteString,
    pub spender: NeoByteString,
    pub amount: NeoInteger,
}

neo_permission!("0xff", ["balanceOf"]);
neo_supported_standards!(["NEP-17"]);
neo_trusts!(["*"]);
```

Each invocation emits a `neo.manifest` custom section that `wasm-neovm` merges during translation, keeping your NEF manifest aligned with the code without extra tooling.

## 🧪 Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_contract_creation() {
        let contract = MyContract::new();
        assert_eq!(contract.get_value().unwrap().as_i32(), 0);
    }
    
    #[test]
    fn test_contract_operations() {
        let mut contract = MyContract::new();
        contract.set_value(NeoInteger::new(42)).unwrap();
        assert_eq!(contract.get_value().unwrap().as_i32(), 42);
    }
}
```

### Integration Tests

```rust
#[neo_test]
pub fn test_contract_integration() {
    let contract = MyContract::new();
    // Test contract functionality
}
```

### Benchmarks

```rust
#[neo_bench]
pub fn bench_contract_operations(b: &mut Bencher) {
    b.iter(|| {
        let contract = MyContract::new();
        contract.perform_operation().unwrap()
    });
}
```

## 🔧 Configuration

### Contract Manifest

```rust
#[neo_contract]
pub struct MyContract {
    // Contract fields
}

impl NeoContract for MyContract {
    fn name() -> &'static str { "MyContract" }
    fn version() -> &'static str { "1.0.0" }
    fn author() -> &'static str { "Developer" }
    fn description() -> &'static str { "My Neo N3 Contract" }
}
```

### Storage Configuration

```rust
#[neo_storage]
pub struct MyStorage {
    // Storage fields
}

impl MyStorage {
    pub fn default() -> Self {
        // Default values
    }
}
```

## 📚 Examples

### Hello World Contract
- Basic contract structure
- Method definitions
- Entry points

### Token Contract
- ERC-20 like functionality
- Transfer operations
- Balance management
- Approval system

### Storage Contract
- Data persistence
- User management
- Settings and counters
- Data serialization

## 🚀 Deployment

### Build Configuration

```toml
[package]
name = "my-neo-contract"
version = "0.1.0"
edition = "2021"

[dependencies]
neo-devpack = "0.1.0"

[[bin]]
name = "contract"
path = "src/main.rs"
```

### Target Configuration

```json
{
  "arch": "neovm",
  "llvm-target": "neovm-unknown-neo3",
  "target-endian": "little",
  "target-pointer-width": "32",
  "panic-strategy": "abort",
  "relocation-model": "static"
}
```

## 🔍 Debugging

### Debug Information

```rust
#[neo_method]
pub fn debug_info(&self) -> NeoResult<NeoString> {
    let info = format!(
        "Gas: {}, Time: {}, Caller: {}",
        NeoRuntime::get_gas_left()?.as_i32(),
        NeoRuntime::get_time()?.as_i32(),
        NeoRuntime::get_calling_script_hash()?.len()
    );
    Ok(NeoString::from_str(&info))
}
```

### Error Handling

```rust
#[neo_method]
pub fn safe_operation(&self, input: NeoInteger) -> NeoResult<NeoInteger> {
    if &input < &NeoInteger::zero() {
        return Err(NeoError::InvalidArgument);
    }
    
    if &input > &NeoInteger::max_i32() {
        return Err(NeoError::Overflow);
    }
    
    Ok(input * NeoInteger::new(2))
}
```

## 📖 API Reference

### Core Types
- `NeoInteger`: 32-bit integer type
- `NeoBoolean`: Boolean type
- `NeoByteString`: Byte array type
- `NeoString`: String type
- `NeoArray<T>`: Dynamic array type
- `NeoMap<K, V>`: Key-value map type
- `NeoStruct`: Structure type
- `NeoValue`: Union type for all Neo types

### Runtime Operations
- `NeoRuntime`: Runtime environment operations
- `NeoStorage`: Storage operations
- `NeoCrypto`: Cryptographic operations
- `NeoJSON`: JSON serialization/deserialization

### Macros
- `#[neo_contract]`: Contract definition
- `#[neo_method]`: Method definition
- `#[neo_event]`: Event definition
- `#[neo_storage]`: Storage definition
- `#[neo_entry]`: Entry point definition

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📄 License

MIT License - see LICENSE file for details.

## 🆘 Support

- Documentation: [docs.rs/neo-devpack](https://docs.rs/neo-devpack)
- Issues: [GitHub Issues](https://github.com/neo-project/neo-devpack/issues)
- Discussions: [GitHub Discussions](https://github.com/neo-project/neo-devpack/discussions)

## 🎉 Acknowledgments

- Neo N3 Development Team
- Rust Community
- LLVM Project
- All contributors and supporters
