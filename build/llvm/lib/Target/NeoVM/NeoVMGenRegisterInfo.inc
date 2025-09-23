/*===- TableGen'erated file -------------------------------------*- C++ -*-===*\
|*                                                                            *|
|* Target Register Enum Values                                                *|
|*                                                                            *|
|* Automatically generated file, do not edit!                                 *|
|*                                                                            *|
\*===----------------------------------------------------------------------===*/


#ifdef GET_REGINFO_ENUM
#undef GET_REGINFO_ENUM

namespace llvm {

class MCRegisterClass;
extern const MCRegisterClass NeoVMMCRegisterClasses[];

namespace NeoVM {
enum {
  NoRegister,
  STACK = 1,
  NUM_TARGET_REGS // 2
};
} // end namespace NeoVM

// Register classes

namespace NeoVM {
enum {
  GR32RegClassID = 0,

};
} // end namespace NeoVM

// Register pressure sets enum.
namespace NeoVM {
enum RegisterPressureSets {
  GR32 = 0,
};
} // end namespace NeoVM

} // end namespace llvm

#endif // GET_REGINFO_ENUM

/*===- TableGen'erated file -------------------------------------*- C++ -*-===*\
|*                                                                            *|
|* MC Register Information                                                    *|
|*                                                                            *|
|* Automatically generated file, do not edit!                                 *|
|*                                                                            *|
\*===----------------------------------------------------------------------===*/


#ifdef GET_REGINFO_MC_DESC
#undef GET_REGINFO_MC_DESC

namespace llvm {

extern const int16_t NeoVMRegDiffLists[] = {
  /* 0 */ 0,
};

extern const LaneBitmask NeoVMLaneMaskLists[] = {
  /* 0 */ LaneBitmask(0xFFFFFFFFFFFFFFFF), LaneBitmask::getAll(),
};

extern const uint16_t NeoVMSubRegIdxLists[] = {
  /* 0 */ 0,
};

extern const MCRegisterInfo::SubRegCoveredBits NeoVMSubRegIdxRanges[] = {
  { 65535, 65535 },
};


#ifdef __GNUC__
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Woverlength-strings"
#endif
extern const char NeoVMRegStrings[] = {
  /* 0 */ "STACK\0"
};
#ifdef __GNUC__
#pragma GCC diagnostic pop
#endif

extern const MCRegisterDesc NeoVMRegDesc[] = { // Descriptors
  { 5, 0, 0, 0, 0, 0 },
  { 0, 0, 0, 0, 0, 0 },
};

extern const MCPhysReg NeoVMRegUnitRoots[][2] = {
  { NeoVM::STACK },
};

namespace {     // Register classes...
  // GR32 Register Class...
  const MCPhysReg GR32[] = {
    NeoVM::STACK, 
  };

  // GR32 Bit set.
  const uint8_t GR32Bits[] = {
    0x02, 
  };

} // end anonymous namespace


#ifdef __GNUC__
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Woverlength-strings"
#endif
extern const char NeoVMRegClassStrings[] = {
  /* 0 */ "GR32\0"
};
#ifdef __GNUC__
#pragma GCC diagnostic pop
#endif

extern const MCRegisterClass NeoVMMCRegisterClasses[] = {
  { GR32, GR32Bits, 0, 1, sizeof(GR32Bits), NeoVM::GR32RegClassID, 32, 1, true },
};

extern const uint16_t NeoVMRegEncodingTable[] = {
  0,
  0,
};
static inline void InitNeoVMMCRegisterInfo(MCRegisterInfo *RI, unsigned RA, unsigned DwarfFlavour = 0, unsigned EHFlavour = 0, unsigned PC = 0) {
  RI->InitMCRegisterInfo(NeoVMRegDesc, 2, RA, PC, NeoVMMCRegisterClasses, 1, NeoVMRegUnitRoots, 1, NeoVMRegDiffLists, NeoVMLaneMaskLists, NeoVMRegStrings, NeoVMRegClassStrings, NeoVMSubRegIdxLists, 1,
NeoVMSubRegIdxRanges, NeoVMRegEncodingTable);

}

} // end namespace llvm

#endif // GET_REGINFO_MC_DESC

/*===- TableGen'erated file -------------------------------------*- C++ -*-===*\
|*                                                                            *|
|* Register Information Header Fragment                                       *|
|*                                                                            *|
|* Automatically generated file, do not edit!                                 *|
|*                                                                            *|
\*===----------------------------------------------------------------------===*/


#ifdef GET_REGINFO_HEADER
#undef GET_REGINFO_HEADER

#include "llvm/CodeGen/TargetRegisterInfo.h"

namespace llvm {

class NeoVMFrameLowering;

struct NeoVMGenRegisterInfo : public TargetRegisterInfo {
  explicit NeoVMGenRegisterInfo(unsigned RA, unsigned D = 0, unsigned E = 0,
      unsigned PC = 0, unsigned HwMode = 0);
  const RegClassWeight &getRegClassWeight(const TargetRegisterClass *RC) const override;
  unsigned getRegUnitWeight(unsigned RegUnit) const override;
  unsigned getNumRegPressureSets() const override;
  const char *getRegPressureSetName(unsigned Idx) const override;
  unsigned getRegPressureSetLimit(const MachineFunction &MF, unsigned Idx) const override;
  const int *getRegClassPressureSets(const TargetRegisterClass *RC) const override;
  const int *getRegUnitPressureSets(unsigned RegUnit) const override;
  ArrayRef<const char *> getRegMaskNames() const override;
  ArrayRef<const uint32_t *> getRegMasks() const override;
  bool isGeneralPurposeRegister(const MachineFunction &, MCRegister) const override;
  bool isFixedRegister(const MachineFunction &, MCRegister) const override;
  bool isArgumentRegister(const MachineFunction &, MCRegister) const override;
  bool isConstantPhysReg(MCRegister PhysReg) const override final;
  /// Devirtualized TargetFrameLowering.
  static const NeoVMFrameLowering *getFrameLowering(
      const MachineFunction &MF);
};

namespace NeoVM { // Register classes
  extern const TargetRegisterClass GR32RegClass;
} // end namespace NeoVM

} // end namespace llvm

#endif // GET_REGINFO_HEADER

/*===- TableGen'erated file -------------------------------------*- C++ -*-===*\
|*                                                                            *|
|* Target Register and Register Classes Information                           *|
|*                                                                            *|
|* Automatically generated file, do not edit!                                 *|
|*                                                                            *|
\*===----------------------------------------------------------------------===*/


#ifdef GET_REGINFO_TARGET_DESC
#undef GET_REGINFO_TARGET_DESC

namespace llvm {

extern const MCRegisterClass NeoVMMCRegisterClasses[];

static const MVT::SimpleValueType VTLists[] = {
  /* 0 */ MVT::i32, MVT::Other,
};

static const char *SubRegIndexNameTable[] = { "" };


static const LaneBitmask SubRegIndexLaneMaskTable[] = {
  LaneBitmask::getAll(),
 };



static const TargetRegisterInfo::RegClassInfo RegClassInfos[] = {
  // Mode = 0 (Default)
  { 32, 32, 32, /*VTLists+*/0 },    // GR32
};

static const TargetRegisterClass *const NullRegClasses[] = { nullptr };

static const uint32_t GR32SubClassMask[] = {
  0x00000001, 
};

static const uint16_t SuperRegIdxSeqs[] = {
  /* 0 */ 0,
};


namespace NeoVM {   // Register class instances
  extern const TargetRegisterClass GR32RegClass = {
    &NeoVMMCRegisterClasses[GR32RegClassID],
    GR32SubClassMask,
    SuperRegIdxSeqs + 0,
    LaneBitmask(0x0000000000000001),
    0,
    false,
    0x00, /* TSFlags */
    false, /* HasDisjunctSubRegs */
    false, /* CoveredBySubRegs */
    NullRegClasses,
    nullptr
  };

} // end namespace NeoVM

namespace {
  const TargetRegisterClass *const RegisterClasses[] = {
    &NeoVM::GR32RegClass,
  };
} // end anonymous namespace

static const uint8_t CostPerUseTable[] = { 
0, 0, };


static const bool InAllocatableClassTable[] = { 
false, true, };


static const TargetRegisterInfoDesc NeoVMRegInfoDesc = { // Extra Descriptors
CostPerUseTable, 1, InAllocatableClassTable};

/// Get the weight in units of pressure for this register class.
const RegClassWeight &NeoVMGenRegisterInfo::
getRegClassWeight(const TargetRegisterClass *RC) const {
  static const RegClassWeight RCWeightTable[] = {
    {1, 1},  	// GR32
  };
  return RCWeightTable[RC->getID()];
}

/// Get the weight in units of pressure for this register unit.
unsigned NeoVMGenRegisterInfo::
getRegUnitWeight(unsigned RegUnit) const {
  assert(RegUnit < 1 && "invalid register unit");
  // All register units have unit weight.
  return 1;
}


// Get the number of dimensions of register pressure.
unsigned NeoVMGenRegisterInfo::getNumRegPressureSets() const {
  return 1;
}

// Get the name of this register unit pressure set.
const char *NeoVMGenRegisterInfo::
getRegPressureSetName(unsigned Idx) const {
  static const char *PressureNameTable[] = {
    "GR32",
  };
  return PressureNameTable[Idx];
}

// Get the register unit pressure limit for this dimension.
// This limit must be adjusted dynamically for reserved registers.
unsigned NeoVMGenRegisterInfo::
getRegPressureSetLimit(const MachineFunction &MF, unsigned Idx) const {
  static const uint8_t PressureLimitTable[] = {
    1,  	// 0: GR32
  };
  return PressureLimitTable[Idx];
}

/// Table of pressure sets per register class or unit.
static const int RCSetsTable[] = {
  /* 0 */ 0, -1,
};

/// Get the dimensions of register pressure impacted by this register class.
/// Returns a -1 terminated array of pressure set IDs
const int *NeoVMGenRegisterInfo::
getRegClassPressureSets(const TargetRegisterClass *RC) const {
  static const uint8_t RCSetStartTable[] = {
    0,};
  return &RCSetsTable[RCSetStartTable[RC->getID()]];
}

/// Get the dimensions of register pressure impacted by this register unit.
/// Returns a -1 terminated array of pressure set IDs
const int *NeoVMGenRegisterInfo::
getRegUnitPressureSets(unsigned RegUnit) const {
  assert(RegUnit < 1 && "invalid register unit");
  static const uint8_t RUSetStartTable[] = {
    0,};
  return &RCSetsTable[RUSetStartTable[RegUnit]];
}

extern const MCRegisterDesc NeoVMRegDesc[];
extern const int16_t NeoVMRegDiffLists[];
extern const LaneBitmask NeoVMLaneMaskLists[];
extern const char NeoVMRegStrings[];
extern const char NeoVMRegClassStrings[];
extern const MCPhysReg NeoVMRegUnitRoots[][2];
extern const uint16_t NeoVMSubRegIdxLists[];
extern const MCRegisterInfo::SubRegCoveredBits NeoVMSubRegIdxRanges[];
extern const uint16_t NeoVMRegEncodingTable[];
NeoVMGenRegisterInfo::
NeoVMGenRegisterInfo(unsigned RA, unsigned DwarfFlavour, unsigned EHFlavour,
      unsigned PC, unsigned HwMode)
  : TargetRegisterInfo(&NeoVMRegInfoDesc, RegisterClasses, RegisterClasses+1,
             SubRegIndexNameTable, SubRegIndexLaneMaskTable,
             LaneBitmask(0xFFFFFFFFFFFFFFFF), RegClassInfos, VTLists, HwMode) {
  InitMCRegisterInfo(NeoVMRegDesc, 2, RA, PC,
                     NeoVMMCRegisterClasses, 1,
                     NeoVMRegUnitRoots,
                     1,
                     NeoVMRegDiffLists,
                     NeoVMLaneMaskLists,
                     NeoVMRegStrings,
                     NeoVMRegClassStrings,
                     NeoVMSubRegIdxLists,
                     1,
                     NeoVMSubRegIdxRanges,
                     NeoVMRegEncodingTable);

}



ArrayRef<const uint32_t *> NeoVMGenRegisterInfo::getRegMasks() const {
  return std::nullopt;
}

bool NeoVMGenRegisterInfo::
isGeneralPurposeRegister(const MachineFunction &MF, MCRegister PhysReg) const {
  return
      false;
}

bool NeoVMGenRegisterInfo::
isFixedRegister(const MachineFunction &MF, MCRegister PhysReg) const {
  return
      false;
}

bool NeoVMGenRegisterInfo::
isArgumentRegister(const MachineFunction &MF, MCRegister PhysReg) const {
  return
      false;
}

bool NeoVMGenRegisterInfo::
isConstantPhysReg(MCRegister PhysReg) const {
  return
      false;
}

ArrayRef<const char *> NeoVMGenRegisterInfo::getRegMaskNames() const {
  return std::nullopt;
}

const NeoVMFrameLowering *
NeoVMGenRegisterInfo::getFrameLowering(const MachineFunction &MF) {
  return static_cast<const NeoVMFrameLowering *>(
      MF.getSubtarget().getFrameLowering());
}

} // end namespace llvm

#endif // GET_REGINFO_TARGET_DESC

