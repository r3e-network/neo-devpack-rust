#include "llvm/Target/NeoVM/NeoVMTargetObjectFile.h"

#include "llvm/MC/MCContext.h"
#include "llvm/MC/MCSection.h"
#include "llvm/MC/SectionKind.h"
#include "llvm/Target/TargetMachine.h"
#include "llvm/Target/TargetLoweringObjectFile.h"
#include "llvm/BinaryFormat/ELF.h"
#include "llvm/CodeGen/Passes.h"

using namespace llvm;

NeoVMTargetObjectFile::NeoVMTargetObjectFile() = default;

void NeoVMTargetObjectFile::Initialize(MCContext &Ctx, const TargetMachine &TM) {
  TargetLoweringObjectFile::Initialize(Ctx, TM);
  TextSection = Ctx.getObjectFileInfo()->getTextSection();
}

MCSection *NeoVMTargetObjectFile::getExplicitSectionGlobal(
    const GlobalObject *, SectionKind, const TargetMachine &) const {
  return TextSection;
}

MCSection *NeoVMTargetObjectFile::SelectSectionForGlobal(
    const GlobalObject *, SectionKind, const TargetMachine &) const {
  return TextSection;
}
