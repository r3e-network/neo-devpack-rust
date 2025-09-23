# NEF (Neo Executable Format) Specification

## Overview
NEF is the binary format for Neo N3 smart contracts. It contains the compiled bytecode, metadata, and manifest information.

## NEF File Structure

```
NEF Header (8 bytes)
├── Magic: "NEF" (3 bytes)
├── Version: 0x01 (1 byte)
├── Reserved: 0x00 (4 bytes)

Script Section
├── Length: 4 bytes (little-endian)
├── Script Data: variable length

Manifest Section (Optional)
├── Length: 4 bytes (little-endian)
├── Manifest JSON: variable length

Checksum (4 bytes)
└── CRC32 of all data except checksum
```

## Implementation Plan

### 1. NEF Container Class
```cpp
class NEFContainer {
public:
  struct Header {
    char magic[3] = {'N', 'E', 'F'};
    uint8_t version = 0x01;
    uint32_t reserved = 0;
  };
  
  std::vector<uint8_t> script;
  std::string manifest;
  
  std::vector<uint8_t> serialize() const;
  static NEFContainer deserialize(const std::vector<uint8_t>& data);
};
```

### 2. Manifest Generation
```json
{
  "name": "ContractName",
  "version": "1.0.0",
  "author": "Developer",
  "email": "dev@example.com",
  "description": "Contract description",
  "abi": {
    "methods": [
      {
        "name": "main",
        "parameters": [],
        "returnType": "Void"
      }
    ],
    "events": []
  },
  "permissions": [],
  "trusts": [],
  "supportedstandards": [],
  "sources": {},
  "compiler": {
    "name": "neo-llvm",
    "version": "1.0.0"
  }
}
```

### 3. Integration with AsmPrinter
- Modify `NeoVMAsmPrinter` to emit NEF format
- Add manifest generation
- Implement checksum calculation
- Add debug information support

## Usage Example

```cpp
// Create NEF container
NEFContainer nef;
nef.script = {0x00, 0x01, 0x02, ...}; // Compiled bytecode
nef.manifest = R"({
  "name": "HelloWorld",
  "version": "1.0.0",
  "abi": {
    "methods": [{"name": "main", "parameters": [], "returnType": "Void"}]
  }
})";

// Serialize to file
auto data = nef.serialize();
std::ofstream file("contract.nef", std::ios::binary);
file.write(reinterpret_cast<const char*>(data.data()), data.size());
```

## Testing Strategy

1. **Unit Tests**: Test NEF serialization/deserialization
2. **Integration Tests**: Test with NeoVM emulator
3. **Round-trip Tests**: Verify NEF -> NeoVM -> NEF
4. **Validation Tests**: Test with official Neo N3 tools

## Implementation Priority

1. **Basic NEF Container** - Core serialization/deserialization
2. **Manifest Generation** - JSON manifest creation
3. **AsmPrinter Integration** - NEF emission in LLVM
4. **Validation** - NeoVM emulator integration
5. **Debug Support** - Source mapping, debug info
