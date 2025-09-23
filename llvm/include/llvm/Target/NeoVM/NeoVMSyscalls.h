#pragma once

#include "llvm/ADT/StringMap.h"
#include "llvm/ADT/StringRef.h"
#include "llvm/IR/Function.h"
#include "llvm/IR/Module.h"
#include <string>
#include <vector>

namespace llvm {

/// NeoVM syscall information
struct NeoVMSyscallInfo {
  std::string name;
  uint32_t hash;
  std::vector<std::string> parameters;
  std::string returnType;
  uint32_t gasCost;
  std::string description;
  
  NeoVMSyscallInfo() = default;
  NeoVMSyscallInfo(StringRef name, uint32_t hash, 
                   const std::vector<std::string>& params,
                   StringRef returnType, uint32_t gasCost,
                   StringRef description)
    : name(name.str()), hash(hash), parameters(params),
      returnType(returnType.str()), gasCost(gasCost),
      description(description.str()) {}
};

/// NeoVM syscall registry
class NeoVMSyscallRegistry {
public:
  static NeoVMSyscallRegistry& getInstance();
  
  /// Load syscalls from JSON file
  bool loadFromFile(StringRef filename);
  
  /// Get syscall info by name
  const NeoVMSyscallInfo* getSyscall(StringRef name) const;
  
  /// Get syscall info by hash
  const NeoVMSyscallInfo* getSyscallByHash(uint32_t hash) const;
  
  /// Get all syscall names
  std::vector<std::string> getAllSyscallNames() const;
  
  /// Check if syscall exists
  bool hasSyscall(StringRef name) const;
  
  /// Get syscall count
  size_t getSyscallCount() const;

private:
  NeoVMSyscallRegistry() = default;
  StringMap<NeoVMSyscallInfo> syscalls;
  std::vector<NeoVMSyscallInfo> syscallsByHash;
};

/// NeoVM syscall intrinsic lowering
class NeoVMSyscallLowering {
public:
  /// Lower a function call to syscall
  static bool lowerToSyscall(Function* func, const NeoVMSyscallInfo* syscall);
  
  /// Create syscall intrinsic
  static Function* createSyscallIntrinsic(Module* module, const NeoVMSyscallInfo* syscall);
  
  /// Check if function is a syscall
  static bool isSyscallFunction(const Function* func);
  
  /// Get syscall info for function
  static const NeoVMSyscallInfo* getSyscallForFunction(const Function* func);
};

} // namespace llvm
