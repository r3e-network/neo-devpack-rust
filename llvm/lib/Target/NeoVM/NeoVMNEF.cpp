#include "llvm/Target/NeoVM/NeoVMNEF.h"
#include "llvm/Support/Endian.h"
#include "llvm/Support/raw_ostream.h"
#include <cstring>
#include <fstream>
#include <sstream>
#include <cstdint>

using namespace llvm;

std::vector<uint8_t> NEFContainer::serialize() const {
  std::vector<uint8_t> data;
  
  // NEF Header
  Header header;
  data.insert(data.end(), header.magic, header.magic + 3);
  data.push_back(header.version);
  
  // Reserved field (4 bytes)
  uint32_t reserved = 0;
  data.insert(data.end(), reinterpret_cast<uint8_t*>(&reserved), 
              reinterpret_cast<uint8_t*>(&reserved) + 4);
  
  // Script section
  uint32_t scriptLength = static_cast<uint32_t>(script.size());
  data.insert(data.end(), reinterpret_cast<uint8_t*>(&scriptLength), 
              reinterpret_cast<uint8_t*>(&scriptLength) + 4);
  data.insert(data.end(), script.begin(), script.end());
  
  // Manifest section (optional)
  if (!manifest.empty()) {
    uint32_t manifestLength = static_cast<uint32_t>(manifest.size());
    data.insert(data.end(), reinterpret_cast<uint8_t*>(&manifestLength), 
                reinterpret_cast<uint8_t*>(&manifestLength) + 4);
    data.insert(data.end(), manifest.begin(), manifest.end());
  } else {
    uint32_t manifestLength = 0;
    data.insert(data.end(), reinterpret_cast<uint8_t*>(&manifestLength), 
                reinterpret_cast<uint8_t*>(&manifestLength) + 4);
  }
  
  // Calculate and append checksum
  uint32_t checksum = calculateCRC32(data.data(), data.size());
  data.insert(data.end(), reinterpret_cast<uint8_t*>(&checksum), 
              reinterpret_cast<uint8_t*>(&checksum) + 4);
  
  return data;
}

NEFContainer NEFContainer::deserialize(const std::vector<uint8_t>& data) {
  NEFContainer container;
  
  if (data.size() < 8) {
    return container; // Invalid NEF
  }
  
  size_t offset = 0;
  
  // Read header
  Header header;
  std::memcpy(header.magic, data.data() + offset, 3);
  offset += 3;
  header.version = data[offset++];
  std::memcpy(&header.reserved, data.data() + offset, 4);
  offset += 4;
  
  // Validate magic
  if (std::strncmp(header.magic, "NEF", 3) != 0) {
    return container; // Invalid magic
  }
  
  // Read script section
  if (offset + 4 > data.size()) {
    return container; // Invalid NEF
  }
  uint32_t scriptLength;
  std::memcpy(&scriptLength, data.data() + offset, 4);
  offset += 4;
  
  if (offset + scriptLength > data.size()) {
    return container; // Invalid NEF
  }
  container.script.assign(data.begin() + offset, data.begin() + offset + scriptLength);
  offset += scriptLength;
  
  // Read manifest section
  if (offset + 4 > data.size()) {
    return container; // Invalid NEF
  }
  uint32_t manifestLength;
  std::memcpy(&manifestLength, data.data() + offset, 4);
  offset += 4;
  
  if (manifestLength > 0) {
    if (offset + manifestLength > data.size()) {
      return container; // Invalid NEF
    }
    container.manifest.assign(data.begin() + offset, data.begin() + offset + manifestLength);
    offset += manifestLength;
  }
  
  // Verify checksum
  if (offset + 4 != data.size()) {
    return container; // Invalid NEF
  }
  
  uint32_t storedChecksum;
  std::memcpy(&storedChecksum, data.data() + offset, 4);
  
  uint32_t calculatedChecksum = calculateCRC32(data.data(), offset);
  if (storedChecksum != calculatedChecksum) {
    return container; // Invalid checksum
  }
  
  return container;
}

std::string NEFContainer::createBasicManifest(StringRef contractName, 
                                            StringRef version,
                                            StringRef author,
                                            StringRef description) {
  NEFManifestGenerator generator;
  generator.contractName = contractName.str();
  generator.version = version.str();
  generator.author = author.str();
  generator.description = description.str();
  
  // Add default main method
  generator.addMethod("main");
  
  return generator.generate();
}

uint32_t NEFContainer::calculateCRC32(const uint8_t* data, size_t length) {
  // Simple CRC32 implementation
  uint32_t crc = 0xFFFFFFFF;
  for (size_t i = 0; i < length; ++i) {
    crc ^= data[i];
    for (int j = 0; j < 8; ++j) {
      if (crc & 1) {
        crc = (crc >> 1) ^ 0xEDB88320;
      } else {
        crc >>= 1;
      }
    }
  }
  return crc ^ 0xFFFFFFFF;
}

bool NEFContainer::isValid() const {
  return !script.empty() && !manifest.empty();
}

size_t NEFContainer::getSize() const {
  return 8 + 4 + script.size() + 4 + manifest.size() + 4; // Header + script + manifest + checksum
}

std::string NEFManifestGenerator::generate() const {
  std::ostringstream json;
  json << "{\n";
  json << "  \"name\": \"" << contractName << "\",\n";
  json << "  \"version\": \"" << version << "\",\n";
  json << "  \"author\": \"" << author << "\",\n";
  if (!email.empty()) {
    json << "  \"email\": \"" << email << "\",\n";
  }
  json << "  \"description\": \"" << description << "\",\n";
  
  // ABI
  json << "  \"abi\": {\n";
  json << "    \"methods\": [\n";
  for (size_t i = 0; i < methods.size(); ++i) {
    const auto& method = methods[i];
    json << "      {\n";
    json << "        \"name\": \"" << method.name << "\",\n";
    json << "        \"parameters\": [";
    for (size_t j = 0; j < method.parameters.size(); ++j) {
      if (j > 0) json << ", ";
      json << "\"" << method.parameters[j] << "\"";
    }
    json << "],\n";
    json << "        \"returnType\": \"" << method.returnType << "\"\n";
    json << "      }";
    if (i < methods.size() - 1) json << ",";
    json << "\n";
  }
  json << "    ],\n";
  
  json << "    \"events\": [\n";
  for (size_t i = 0; i < events.size(); ++i) {
    const auto& event = events[i];
    json << "      {\n";
    json << "        \"name\": \"" << event.name << "\",\n";
    json << "        \"parameters\": [";
    for (size_t j = 0; j < event.parameters.size(); ++j) {
      if (j > 0) json << ", ";
      json << "\"" << event.parameters[j] << "\"";
    }
    json << "]\n";
    json << "      }";
    if (i < events.size() - 1) json << ",";
    json << "\n";
  }
  json << "    ]\n";
  json << "  },\n";
  
  // Permissions
  json << "  \"permissions\": [";
  for (size_t i = 0; i < permissions.size(); ++i) {
    if (i > 0) json << ", ";
    json << "\"" << permissions[i] << "\"";
  }
  json << "],\n";
  
  // Trusts
  json << "  \"trusts\": [";
  for (size_t i = 0; i < trusts.size(); ++i) {
    if (i > 0) json << ", ";
    json << "\"" << trusts[i] << "\"";
  }
  json << "],\n";
  
  // Supported standards
  json << "  \"supportedstandards\": [";
  for (size_t i = 0; i < supportedStandards.size(); ++i) {
    if (i > 0) json << ", ";
    json << "\"" << supportedStandards[i] << "\"";
  }
  json << "],\n";
  
  // Compiler info
  json << "  \"compiler\": {\n";
  json << "    \"name\": \"neo-llvm\",\n";
  json << "    \"version\": \"1.0.0\"\n";
  json << "  }\n";
  
  json << "}\n";
  return json.str();
}

void NEFManifestGenerator::addMethod(StringRef name, 
                                    const std::vector<std::string>& parameters,
                                    StringRef returnType) {
  Method method;
  method.name = name.str();
  method.parameters = parameters;
  method.returnType = returnType.str();
  methods.push_back(method);
}

void NEFManifestGenerator::addEvent(StringRef name, 
                                   const std::vector<std::string>& parameters) {
  Event event;
  event.name = name.str();
  event.parameters = parameters;
  events.push_back(event);
}
