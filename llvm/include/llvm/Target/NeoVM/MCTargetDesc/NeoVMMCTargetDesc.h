#pragma once

#include "llvm/Support/Compiler.h"
#include "llvm/ADT/StringRef.h"
#include <memory>

namespace llvm {

class MCAsmInfo;
class MCContext;
class MCInstrInfo;
class MCRegisterInfo;
class MCSubtargetInfo;
class Target;
class Triple;
struct MCTargetOptions;

MCAsmInfo *createNeoVMMCAsmInfo(const MCRegisterInfo &MRI, const Triple &TT,
                                const MCTargetOptions &Options);
MCInstrInfo *createNeoVMMCInstrInfo();
MCRegisterInfo *createNeoVMMCRegisterInfo(const Triple &TT);
MCSubtargetInfo *createNeoVMMCSubtargetInfo(const Triple &TT, StringRef CPU,
                                            StringRef FS);

} // namespace llvm
