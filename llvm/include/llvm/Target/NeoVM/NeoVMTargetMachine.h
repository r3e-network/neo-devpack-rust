#pragma once

#include "llvm/Target/TargetMachine.h"
#include "llvm/Support/CodeGen.h"

// Skeleton interface tracked in docs/neo-n3-backend.md (NeoVMTargetMachine Responsibilities).

namespace llvm {

class Function;
class NeoVMSubtarget;
class TargetOptions;
class TargetPassConfig;
class PassManagerBase;

class NeoVMTargetMachine final : public LLVMTargetMachine {
public:
  NeoVMTargetMachine(const Target &T, const Triple &TT, StringRef CPU,
                     StringRef FS, const TargetOptions &Options,
                     std::optional<Reloc::Model> RM,
                     std::optional<CodeModel::Model> CM,
                     CodeGenOptLevel OL, bool JIT);

  ~NeoVMTargetMachine() override;

  const NeoVMSubtarget &getNeoVMSubtarget(const Function &) const;
  const TargetSubtargetInfo *getSubtargetImpl(const Function &) const override;

  TargetPassConfig *createPassConfig(PassManagerBase &PM) override;

  TargetLoweringObjectFile *getObjFileLowering() const override {
    return ObjFileLowering.get();
  }

private:
  std::unique_ptr<NeoVMSubtarget> DefaultSubtarget;
  std::unique_ptr<TargetLoweringObjectFile> ObjFileLowering;
};

} // namespace llvm
