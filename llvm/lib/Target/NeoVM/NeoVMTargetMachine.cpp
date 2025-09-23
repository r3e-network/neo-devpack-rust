#include "llvm/Target/NeoVM/NeoVMTargetMachine.h"

#include "llvm/CodeGen/TargetPassConfig.h"
#include "llvm/IR/Function.h"
#include "llvm/Support/Compiler.h"
#include "llvm/MC/TargetRegistry.h"
#include "llvm/MC/MCStreamer.h"
#include "llvm/Target/NeoVM/NeoVMAsmPrinter.h"
#include "llvm/Target/NeoVM/NeoVMInstructionSelector.h"
#include "llvm/Target/NeoVM/NeoVMStackify.h"
#include "llvm/Target/NeoVM/NeoVMSubtarget.h"
#include "llvm/Target/NeoVM/NeoVMTargetInfo.h"
#include "llvm/Target/NeoVM/NeoVMTargetObjectFile.h"
#include "llvm/Target/TargetOptions.h"
#include "llvm/IR/LegacyPassManager.h"

// Implementation stub aligned with docs/neo-n3-backend.md (NeoVMTargetMachine Responsibilities).

using namespace llvm;

namespace {
class NeoVMPassConfig : public TargetPassConfig {
public:
  NeoVMPassConfig(NeoVMTargetMachine &TM, PassManagerBase &PM)
      : TargetPassConfig(TM, PM) {}

  NeoVMTargetMachine &getNeoVMTargetMachine() const {
    return getTM<NeoVMTargetMachine>();
  }

  void addIRPasses() override {
    addPass(createNeoVMIntrinsicLoweringPass().release());
  }

  bool addInstSelector() override {
    addPass(createNeoVMInstructionSelectPass().release());
    return false;
  }

  void addMachinePasses() override {
    addPass(createNeoVMStackifyPass().release());
    addPass(createNeoVMStackHeightVerifierPass().release());
  }
};
} // namespace

NeoVMTargetMachine::NeoVMTargetMachine(const Target &T, const Triple &TT,
                                       StringRef CPU, StringRef FS,
                                       const TargetOptions &Options,
                                       std::optional<Reloc::Model> RM,
                                       std::optional<CodeModel::Model> CM,
                                       CodeGenOptLevel OL, bool JIT)
    : LLVMTargetMachine(T, "", TT, CPU, FS, Options,
                        RM.value_or(Reloc::PIC_), CM.value_or(CodeModel::Small),
                        OL) {
  this->Options.ExceptionModel = ExceptionHandling::None;
  ObjFileLowering = std::make_unique<NeoVMTargetObjectFile>();
  (void)JIT;
  DefaultSubtarget = std::make_unique<NeoVMSubtarget>(TT, CPU, FS, *this);
}

NeoVMTargetMachine::~NeoVMTargetMachine() = default;

const NeoVMSubtarget &
NeoVMTargetMachine::getNeoVMSubtarget(const Function &) const {
  return *DefaultSubtarget;
}

const TargetSubtargetInfo *
NeoVMTargetMachine::getSubtargetImpl(const Function &) const {
  return DefaultSubtarget.get();
}

TargetPassConfig *
NeoVMTargetMachine::createPassConfig(PassManagerBase &PM) {
  return new NeoVMPassConfig(*this, PM);
}

extern "C" LLVM_EXTERNAL_VISIBILITY void LLVMInitializeNeoVMTarget() {
  RegisterTargetMachine<NeoVMTargetMachine> X(getTheNeoVMTarget());
}

extern "C" LLVM_EXTERNAL_VISIBILITY void LLVMInitializeNeoVMAsmPrinter() {
  Target &T = getTheNeoVMTarget();
  RegisterAsmPrinter<NeoVMAsmPrinter> X(T);
}
