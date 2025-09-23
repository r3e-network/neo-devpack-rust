#include "llvm/TargetParser/Triple.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/LLVMContext.h"
#include "llvm/IR/LegacyPassManager.h"
#include "llvm/IR/Module.h"
#include "llvm/MC/TargetRegistry.h"
#include "llvm/Support/TargetSelect.h"
#include "llvm/Target/TargetMachine.h"
#include "llvm/Target/NeoVM/NeoVMTargetInfo.h"

#include <iostream>
#include <memory>

extern "C" void LLVMInitializeNeoVMTargetInfo();
extern "C" void LLVMInitializeNeoVMTarget();
extern "C" void LLVMInitializeNeoVMTargetMC();
extern "C" void LLVMInitializeNeoVMAsmPrinter();

int main() {
  LLVMInitializeNeoVMTargetInfo();
  LLVMInitializeNeoVMTarget();
  LLVMInitializeNeoVMTargetMC();
  LLVMInitializeNeoVMAsmPrinter();

  llvm::LLVMContext Ctx;
  auto M = std::make_unique<llvm::Module>("test", Ctx);
  llvm::Triple TT("neovm");

  std::string Error;
  const llvm::Target *T = llvm::TargetRegistry::lookupTarget("", TT, Error);
  if (!T) {
    std::cerr << "Target lookup failed: " << Error << "\n";
    return 1;
  }

  llvm::TargetOptions Options;
  auto TM = std::unique_ptr<llvm::TargetMachine>(
      T->createTargetMachine(TT.str(), "", "", Options, std::nullopt,
                             std::nullopt, llvm::CodeGenOptLevel::None, false));
  if (!TM) {
    std::cerr << "Failed to create target machine\n";
    return 1;
  }

  M->setDataLayout(TM->createDataLayout());

  auto *FuncTy = llvm::FunctionType::get(llvm::Type::getVoidTy(Ctx), false);
  auto *Func = llvm::Function::Create(FuncTy, llvm::GlobalValue::ExternalLinkage,
                                      "main", M.get());
  llvm::BasicBlock *Entry = llvm::BasicBlock::Create(Ctx, "entry", Func);
  llvm::IRBuilder<> Builder(Entry);
  llvm::Value *A = Builder.getInt32(2);
  llvm::Value *B = Builder.getInt32(3);
  Builder.CreateAdd(A, B);
  Builder.CreateRetVoid();

  llvm::SmallVector<char, 0> Buffer;
  llvm::raw_svector_ostream OS(Buffer);

  llvm::legacy::PassManager PM;
  if (TM->addPassesToEmitFile(PM, OS, nullptr,
                              llvm::CodeGenFileType::AssemblyFile)) {
    std::cerr << "NeoVM backend cannot emit this file type yet\n";
    return 1;
  }

  PM.run(*M);

  std::cout.write(Buffer.data(), Buffer.size());
  return 0;
}
