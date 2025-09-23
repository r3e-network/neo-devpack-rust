#include "llvm/Target/NeoVM/MCTargetDesc/NeoVMAsmInfo.h"

#include "llvm/TargetParser/Triple.h"
#include "llvm/CodeGen/Passes.h"

using namespace llvm;

NeoVMMCAsmInfo::NeoVMMCAsmInfo(const Triple &) {
  CodePointerSize = 1;
  CalleeSaveStackSlotSize = 1;
  MinInstAlignment = 1;
  Data8bitsDirective = "\t.byte";
  CommentString = ";";
  SupportsDebugInformation = false;
  UseIntegratedAssembler = true;
}
