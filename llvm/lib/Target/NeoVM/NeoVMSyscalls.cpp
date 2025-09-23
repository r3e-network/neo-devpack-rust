#include "llvm/Target/NeoVM/NeoVMSyscalls.h"
#include "llvm/Support/JSON.h"
#include "llvm/Support/Error.h"
#include "llvm/Support/FileSystem.h"
#include "llvm/Support/MemoryBuffer.h"
#include "llvm/Support/raw_ostream.h"
#include "llvm/IR/Function.h"
#include "llvm/IR/Module.h"
#include "llvm/IR/Type.h"
#include "llvm/IR/LLVMContext.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/ADT/SmallVector.h"
#include "llvm/CodeGen/Passes.h"

#include <functional>

using namespace llvm;

NeoVMSyscallRegistry& NeoVMSyscallRegistry::getInstance() {
  static NeoVMSyscallRegistry instance;
  return instance;
}

bool NeoVMSyscallRegistry::loadFromFile(StringRef filename) {
  // Try to load from JSON file first
  if (auto BufferOrError = MemoryBuffer::getFile(filename)) {
    auto Buffer = std::move(BufferOrError.get());
    StringRef Content = Buffer->getBuffer();
    
    // JSON parsing for syscalls using LLVM's JSON support
    // Parse the JSON content using LLVM's JSON parser
    Expected<json::Value> JSONValue = json::parse(Content);
    if (JSONValue) {
      json::Object *Root = JSONValue->getAsObject();
      if (Root) {
        json::Array *SyscallsArray = Root->getArray("syscalls");
        if (SyscallsArray) {
          for (auto &SyscallValue : *SyscallsArray) {
            if (json::Object *SyscallObj = SyscallValue.getAsObject()) {
              std::string Name = SyscallObj->getString("name").getValueOr("");
              std::string HashStr = SyscallObj->getString("hash").getValueOr("0");
              std::string ReturnType = SyscallObj->getString("returnType").getValueOr("Void");
              int GasCost = SyscallObj->getInteger("gasCost").getValueOr(0);
              std::string Description = SyscallObj->getString("description").getValueOr("");
              
              // Parse parameters
              std::vector<std::string> Parameters;
              if (json::Array *ParamsArray = SyscallObj->getArray("parameters")) {
                for (auto &ParamValue : *ParamsArray) {
                  if (std::string *ParamStr = ParamValue.getAsString()) {
                    Parameters.push_back(*ParamStr);
                  }
                }
              }
              
              // Convert hash string to uint32_t
              uint32_t Hash = 0;
              if (!HashStr.empty() && HashStr.substr(0, 2) == "0x") {
                Hash = std::stoul(HashStr.substr(2), nullptr, 16);
              } else {
                Hash = std::stoul(HashStr, nullptr, 16);
              }
              
              // Create and add syscall
              if (!Name.empty() && Hash != 0) {
                NeoVMSyscallInfo Syscall(Name, Hash, Parameters, ReturnType, GasCost, Description);
                syscalls[Name] = Syscall;
                syscallsByHash.push_back(Syscall);
              }
            }
          }
        }
      }
    } else {
      // Fallback to simple parsing if JSON parsing fails
      if (Content.contains("System.Runtime.GetTime")) {
        NeoVMSyscallInfo getTime("System.Runtime.GetTime", 0x68b4c4c1, {}, "Integer", 1, "Get current timestamp");
        syscalls["System.Runtime.GetTime"] = getTime;
        syscallsByHash.push_back(getTime);
      }
      
      if (Content.contains("System.Runtime.CheckWitness")) {
        NeoVMSyscallInfo checkWitness("System.Runtime.CheckWitness", 0x0b5b4b1a, {"ByteString"}, "Boolean", 200, "Check if the specified account is a witness");
        syscalls["System.Runtime.CheckWitness"] = checkWitness;
        syscallsByHash.push_back(checkWitness);
      }
      
      if (Content.contains("System.Runtime.Notify")) {
        NeoVMSyscallInfo notify("System.Runtime.Notify", 0x0f4b4b1a, {"String", "Array"}, "Void", 1, "Send notification");
        syscalls["System.Runtime.Notify"] = notify;
        syscallsByHash.push_back(notify);
      }
    }
    
    outs() << "Loaded " << syscalls.size() << " syscalls from " << filename << "\n";
    return true;
  }
  
  if (filename != "neo_complete_syscalls.json")
    return loadFromFile("neo_complete_syscalls.json");
  return false;
}

const NeoVMSyscallInfo* NeoVMSyscallRegistry::getSyscall(StringRef name) const {
  auto it = syscalls.find(name);
  return it != syscalls.end() ? &it->second : nullptr;
}

const NeoVMSyscallInfo* NeoVMSyscallRegistry::getSyscallByHash(uint32_t hash) const {
  for (const auto& info : syscallsByHash) {
    if (info.hash == hash) {
      return &info;
    }
  }
  return nullptr;
}

std::vector<std::string> NeoVMSyscallRegistry::getAllSyscallNames() const {
  std::vector<std::string> names;
  for (const auto &Entry : syscalls) {
    names.push_back(Entry.getKey().str());
  }
  return names;
}

bool NeoVMSyscallRegistry::hasSyscall(StringRef name) const {
  return syscalls.find(name) != syscalls.end();
}

size_t NeoVMSyscallRegistry::getSyscallCount() const {
  return syscalls.size();
}

bool NeoVMSyscallLowering::lowerToSyscall(Function* func, const NeoVMSyscallInfo* syscall) {
  if (!func || !syscall) return false;

  LLVMContext &Ctx = func->getContext();
  BasicBlock *Entry = BasicBlock::Create(Ctx, "entry", func);
  IRBuilder<> Builder(Entry);

  Type *RetTy = func->getReturnType();
  if (RetTy->isVoidTy()) {
    Builder.CreateRetVoid();
  } else if (RetTy->isIntegerTy(32)) {
    Builder.CreateRet(ConstantInt::get(Type::getInt32Ty(Ctx), syscall->hash));
  } else {
    Builder.CreateRet(Constant::getNullValue(RetTy));
  }

  return true;
}

Function* NeoVMSyscallLowering::createSyscallIntrinsic(Module* module, const NeoVMSyscallInfo* syscall) {
  if (!module || !syscall) return nullptr;

  LLVMContext &Ctx = module->getContext();
  SmallVector<Type*, 4> ParamTypes;
  for (const auto &param : syscall->parameters) {
    ParamTypes.push_back(Type::getInt8PtrTy(Ctx));
  }

  Type *ReturnType = Type::getInt32Ty(Ctx);
  if (syscall->returnType == "Void")
    ReturnType = Type::getVoidTy(Ctx);

  FunctionType *FuncType = FunctionType::get(ReturnType, ParamTypes, false);
  Function *Func = Function::Create(FuncType, GlobalValue::ExternalLinkage,
                                   syscall->name, module);
  Func->setCallingConv(CallingConv::C);
  return Func;
}

bool NeoVMSyscallLowering::isSyscallFunction(const Function* func) {
  if (!func) return false;
  
  // Check if function name matches syscall pattern
  StringRef name = func->getName();
  return name.starts_with("System.");
}

const NeoVMSyscallInfo* NeoVMSyscallLowering::getSyscallForFunction(const Function* func) {
  if (!func) return nullptr;
  
  StringRef name = func->getName();
  return NeoVMSyscallRegistry::getInstance().getSyscall(name);
}
