#include "llvm/Target/NeoVM/NeoVMNEF.h"
#include <iostream>
#include <fstream>

int main() {
  std::cout << "Testing NEF container functionality...\n";
  
  // Create a simple NEF container
  llvm::NEFContainer nef;
  
  // Add some test bytecode
  nef.script = {0x00, 0x01, 0x02, 0x03, 0x04, 0x05};
  
  // Create a basic manifest
  nef.manifest = llvm::NEFContainer::createBasicManifest("TestContract", "1.0.0", "neo-llvm", "Test contract");
  
  std::cout << "Created NEF container with " << nef.script.size() << " bytes of script\n";
  std::cout << "Manifest:\n" << nef.manifest << "\n";
  
  // Serialize NEF
  auto data = nef.serialize();
  std::cout << "Serialized NEF size: " << data.size() << " bytes\n";
  
  // Write to file
  std::ofstream file("test.nef", std::ios::binary);
  file.write(reinterpret_cast<const char*>(data.data()), data.size());
  file.close();
  std::cout << "Written NEF to test.nef\n";
  
  // Test deserialization
  auto deserialized = llvm::NEFContainer::deserialize(data);
  if (deserialized.isValid()) {
    std::cout << "NEF deserialization successful\n";
    std::cout << "Deserialized script size: " << deserialized.script.size() << " bytes\n";
  } else {
    std::cout << "NEF deserialization failed\n";
    return 1;
  }
  
  return 0;
}
