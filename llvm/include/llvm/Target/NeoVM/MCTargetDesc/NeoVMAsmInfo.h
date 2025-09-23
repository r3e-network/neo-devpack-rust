#pragma once

#include "llvm/MC/MCAsmInfo.h"

namespace llvm {

class Triple;

class NeoVMMCAsmInfo : public MCAsmInfo {
public:
  explicit NeoVMMCAsmInfo(const Triple &TT);
};

} // namespace llvm

