#include "llvm/TargetParser/Triple.h"
#include "llvm/MC/TargetRegistry.h"
#include "llvm/Target/NeoVM/NeoVMTargetInfo.h"
#include "llvm/IR/LLVMContext.h"
#include "llvm/IR/Module.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/Support/TargetSelect.h"
#include <iostream>

extern "C" void LLVMInitializeNeoVMTargetInfo();

int main() {
  std::cout << "Testing NeoVM target registration...\n";
  
  // Initialize target info
  LLVMInitializeNeoVMTargetInfo();
  std::cout << "Target info initialized\n";
  
  // Look up target
  llvm::Triple TT("neovm");
  std::string Error;
  const llvm::Target *T = llvm::TargetRegistry::lookupTarget("", TT, Error);
  if (!T) {
    std::cerr << "Target lookup failed: " << Error << "\n";
    return 1;
  }
  
  std::cout << "Target found: " << T->getName() << "\n";
  std::cout << "Target description: " << T->getShortDescription() << "\n";
  
  // Create a simple module
  llvm::LLVMContext Ctx;
  auto M = std::make_unique<llvm::Module>("test", Ctx);
  
  // Create a simple function
  auto *FuncTy = llvm::FunctionType::get(llvm::Type::getVoidTy(Ctx), false);
  auto *Func = llvm::Function::Create(FuncTy, llvm::GlobalValue::ExternalLinkage, "main", M.get());
  llvm::BasicBlock *Entry = llvm::BasicBlock::Create(Ctx, "entry", Func);
  llvm::IRBuilder<> Builder(Entry);
  
  // Add some simple instructions
  llvm::Value *A = Builder.getInt32(2);
  llvm::Value *B = Builder.getInt32(3);
  llvm::Value *Sum = Builder.CreateAdd(A, B);
  Builder.CreateRetVoid();
  
  std::cout << "Module created successfully\n";
  std::cout << "Function: " << Func->getName().str() << "\n";
  std::cout << "Basic blocks: " << Func->size() << "\n";
  
  return 0;
}
