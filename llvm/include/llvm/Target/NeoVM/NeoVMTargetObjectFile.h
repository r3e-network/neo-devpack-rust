#pragma once

#include "llvm/CodeGen/TargetLoweringObjectFileImpl.h"

namespace llvm {

class NeoVMTargetObjectFile final : public TargetLoweringObjectFile {
public:
  NeoVMTargetObjectFile();
  ~NeoVMTargetObjectFile() override = default;

  void Initialize(MCContext &Ctx, const TargetMachine &TM) override;

  MCSection *getExplicitSectionGlobal(const GlobalObject *GO, SectionKind Kind,
                                      const TargetMachine &TM) const override;

  MCSection *SelectSectionForGlobal(const GlobalObject *GO, SectionKind Kind,
                                    const TargetMachine &TM) const override;

private:
  mutable MCSection *TextSection = nullptr;
};

} // namespace llvm
