/*===- TableGen'erated file -------------------------------------*- C++ -*-===*\
|*                                                                            *|
|* Subtarget Enumeration Source Fragment                                      *|
|*                                                                            *|
|* Automatically generated file, do not edit!                                 *|
|*                                                                            *|
\*===----------------------------------------------------------------------===*/


#ifdef GET_SUBTARGETINFO_ENUM
#undef GET_SUBTARGETINFO_ENUM

namespace llvm {
namespace NeoVM {
enum {
  FeatureBigIntegers = 0,
  FeatureManifestHints = 1,
  FeatureNeoVM = 2,
  FeatureTryCatch = 3,
  NumSubtargetFeatures = 4
};
} // end namespace NeoVM
} // end namespace llvm

#endif // GET_SUBTARGETINFO_ENUM


#ifdef GET_SUBTARGETINFO_MACRO
GET_SUBTARGETINFO_MACRO(HasBigIntegers, false, hasBigIntegers)
GET_SUBTARGETINFO_MACRO(HasManifestHints, false, hasManifestHints)
GET_SUBTARGETINFO_MACRO(HasNeoVM, false, hasNeoVM)
GET_SUBTARGETINFO_MACRO(HasTryCatch, false, hasTryCatch)
#undef GET_SUBTARGETINFO_MACRO
#endif // GET_SUBTARGETINFO_MACRO


#ifdef GET_SUBTARGETINFO_MC_DESC
#undef GET_SUBTARGETINFO_MC_DESC

namespace llvm {
// Sorted (by key) array of values for CPU features.
extern const llvm::SubtargetFeatureKV NeoVMFeatureKV[] = {
  { "big-integers", "Enable 256-bit arithmetic support", NeoVM::FeatureBigIntegers, { { { 0x0ULL, 0x0ULL, 0x0ULL, 0x0ULL, 0x0ULL, } } } },
  { "manifest-hints", "Enable manifest hint support", NeoVM::FeatureManifestHints, { { { 0x0ULL, 0x0ULL, 0x0ULL, 0x0ULL, 0x0ULL, } } } },
  { "neovm", "Enable base NeoVM feature set", NeoVM::FeatureNeoVM, { { { 0x0ULL, 0x0ULL, 0x0ULL, 0x0ULL, 0x0ULL, } } } },
  { "try-catch", "Enable TRY/ENDTRY opcodes", NeoVM::FeatureTryCatch, { { { 0x0ULL, 0x0ULL, 0x0ULL, 0x0ULL, 0x0ULL, } } } },
};

#ifdef DBGFIELD
#error "<target>GenSubtargetInfo.inc requires a DBGFIELD macro"
#endif
#if !defined(NDEBUG) || defined(LLVM_ENABLE_DUMP)
#define DBGFIELD(x) x,
#else
#define DBGFIELD(x)
#endif

// ===============================================================
// Data tables for the new per-operand machine model.

// {ProcResourceIdx, ReleaseAtCycle, AcquireAtCycle}
extern const llvm::MCWriteProcResEntry NeoVMWriteProcResTable[] = {
  { 0,  0,  0 }, // Invalid
}; // NeoVMWriteProcResTable

// {Cycles, WriteResourceID}
extern const llvm::MCWriteLatencyEntry NeoVMWriteLatencyTable[] = {
  { 0,  0}, // Invalid
}; // NeoVMWriteLatencyTable

// {UseIdx, WriteResourceID, Cycles}
extern const llvm::MCReadAdvanceEntry NeoVMReadAdvanceTable[] = {
  {0,  0,  0}, // Invalid
}; // NeoVMReadAdvanceTable

#undef DBGFIELD

static const llvm::MCSchedModel NoSchedModel = {
  MCSchedModel::DefaultIssueWidth,
  MCSchedModel::DefaultMicroOpBufferSize,
  MCSchedModel::DefaultLoopMicroOpBufferSize,
  MCSchedModel::DefaultLoadLatency,
  MCSchedModel::DefaultHighLatency,
  MCSchedModel::DefaultMispredictPenalty,
  false, // PostRAScheduler
  false, // CompleteModel
  false, // EnableIntervals
  0, // Processor ID
  nullptr, nullptr, 0, 0, // No instruction-level machine model.
  nullptr, // No Itinerary
  nullptr // No extra processor descriptor
};

// Sorted (by key) array of values for CPU subtype.
extern const llvm::SubtargetSubTypeKV NeoVMSubTypeKV[] = {
 { "neovm", { { { 0xfULL, 0x0ULL, 0x0ULL, 0x0ULL, 0x0ULL, } } }, { { { 0x0ULL, 0x0ULL, 0x0ULL, 0x0ULL, 0x0ULL, } } }, &NoSchedModel },
 { "neovm-v1", { { { 0xfULL, 0x0ULL, 0x0ULL, 0x0ULL, 0x0ULL, } } }, { { { 0x0ULL, 0x0ULL, 0x0ULL, 0x0ULL, 0x0ULL, } } }, &NoSchedModel },
 { "neovm-v2", { { { 0xfULL, 0x0ULL, 0x0ULL, 0x0ULL, 0x0ULL, } } }, { { { 0x0ULL, 0x0ULL, 0x0ULL, 0x0ULL, 0x0ULL, } } }, &NoSchedModel },
};

namespace NeoVM_MC {
unsigned resolveVariantSchedClassImpl(unsigned SchedClass,
    const MCInst *MI, const MCInstrInfo *MCII, unsigned CPUID) {
  // Don't know how to resolve this scheduling class.
  return 0;
}
} // end namespace NeoVM_MC

struct NeoVMGenMCSubtargetInfo : public MCSubtargetInfo {
  NeoVMGenMCSubtargetInfo(const Triple &TT,
    StringRef CPU, StringRef TuneCPU, StringRef FS,
    ArrayRef<SubtargetFeatureKV> PF,
    ArrayRef<SubtargetSubTypeKV> PD,
    const MCWriteProcResEntry *WPR,
    const MCWriteLatencyEntry *WL,
    const MCReadAdvanceEntry *RA, const InstrStage *IS,
    const unsigned *OC, const unsigned *FP) :
      MCSubtargetInfo(TT, CPU, TuneCPU, FS, PF, PD,
                      WPR, WL, RA, IS, OC, FP) { }

  unsigned resolveVariantSchedClass(unsigned SchedClass,
      const MCInst *MI, const MCInstrInfo *MCII,
      unsigned CPUID) const override {
    return NeoVM_MC::resolveVariantSchedClassImpl(SchedClass, MI, MCII, CPUID);
  }
};

static inline MCSubtargetInfo *createNeoVMMCSubtargetInfoImpl(const Triple &TT, StringRef CPU, StringRef TuneCPU, StringRef FS) {
  return new NeoVMGenMCSubtargetInfo(TT, CPU, TuneCPU, FS, NeoVMFeatureKV, NeoVMSubTypeKV, 
                      NeoVMWriteProcResTable, NeoVMWriteLatencyTable, NeoVMReadAdvanceTable, 
                      nullptr, nullptr, nullptr);
}

} // end namespace llvm

#endif // GET_SUBTARGETINFO_MC_DESC


#ifdef GET_SUBTARGETINFO_TARGET_DESC
#undef GET_SUBTARGETINFO_TARGET_DESC

#include "llvm/Support/Debug.h"
#include "llvm/Support/raw_ostream.h"

// ParseSubtargetFeatures - Parses features string setting specified
// subtarget options.
void llvm::NeoVMSubtarget::ParseSubtargetFeatures(StringRef CPU, StringRef TuneCPU, StringRef FS) {
  LLVM_DEBUG(dbgs() << "\nFeatures:" << FS);
  LLVM_DEBUG(dbgs() << "\nCPU:" << CPU);
  LLVM_DEBUG(dbgs() << "\nTuneCPU:" << TuneCPU << "\n\n");
  InitMCProcessorInfo(CPU, TuneCPU, FS);
  const FeatureBitset &Bits = getFeatureBits();
  if (Bits[NeoVM::FeatureBigIntegers]) HasBigIntegers = true;
  if (Bits[NeoVM::FeatureManifestHints]) HasManifestHints = true;
  if (Bits[NeoVM::FeatureNeoVM]) HasNeoVM = true;
  if (Bits[NeoVM::FeatureTryCatch]) HasTryCatch = true;
}
#endif // GET_SUBTARGETINFO_TARGET_DESC


#ifdef GET_SUBTARGETINFO_HEADER
#undef GET_SUBTARGETINFO_HEADER

namespace llvm {
class DFAPacketizer;
namespace NeoVM_MC {
unsigned resolveVariantSchedClassImpl(unsigned SchedClass, const MCInst *MI, const MCInstrInfo *MCII, unsigned CPUID);
} // end namespace NeoVM_MC

struct NeoVMGenSubtargetInfo : public TargetSubtargetInfo {
  explicit NeoVMGenSubtargetInfo(const Triple &TT, StringRef CPU, StringRef TuneCPU, StringRef FS);
public:
  unsigned resolveSchedClass(unsigned SchedClass,  const MachineInstr *DefMI, const TargetSchedModel *SchedModel) const override;
  unsigned resolveVariantSchedClass(unsigned SchedClass, const MCInst *MI, const MCInstrInfo *MCII, unsigned CPUID) const override;
  DFAPacketizer *createDFAPacketizer(const InstrItineraryData *IID) const;
};
} // end namespace llvm

#endif // GET_SUBTARGETINFO_HEADER


#ifdef GET_SUBTARGETINFO_CTOR
#undef GET_SUBTARGETINFO_CTOR

#include "llvm/CodeGen/TargetSchedule.h"

namespace llvm {
extern const llvm::SubtargetFeatureKV NeoVMFeatureKV[];
extern const llvm::SubtargetSubTypeKV NeoVMSubTypeKV[];
extern const llvm::MCWriteProcResEntry NeoVMWriteProcResTable[];
extern const llvm::MCWriteLatencyEntry NeoVMWriteLatencyTable[];
extern const llvm::MCReadAdvanceEntry NeoVMReadAdvanceTable[];
NeoVMGenSubtargetInfo::NeoVMGenSubtargetInfo(const Triple &TT, StringRef CPU, StringRef TuneCPU, StringRef FS)
  : TargetSubtargetInfo(TT, CPU, TuneCPU, FS, ArrayRef(NeoVMFeatureKV, 4), ArrayRef(NeoVMSubTypeKV, 3), 
                        NeoVMWriteProcResTable, NeoVMWriteLatencyTable, NeoVMReadAdvanceTable, 
                        nullptr, nullptr, nullptr) {}

unsigned NeoVMGenSubtargetInfo
::resolveSchedClass(unsigned SchedClass, const MachineInstr *MI, const TargetSchedModel *SchedModel) const {
  report_fatal_error("Expected a variant SchedClass");
} // NeoVMGenSubtargetInfo::resolveSchedClass

unsigned NeoVMGenSubtargetInfo
::resolveVariantSchedClass(unsigned SchedClass, const MCInst *MI, const MCInstrInfo *MCII, unsigned CPUID) const {
  return NeoVM_MC::resolveVariantSchedClassImpl(SchedClass, MI, MCII, CPUID);
} // NeoVMGenSubtargetInfo::resolveVariantSchedClass

} // end namespace llvm

#endif // GET_SUBTARGETINFO_CTOR


#ifdef GET_STIPREDICATE_DECLS_FOR_MC_ANALYSIS
#undef GET_STIPREDICATE_DECLS_FOR_MC_ANALYSIS

#endif // GET_STIPREDICATE_DECLS_FOR_MC_ANALYSIS


#ifdef GET_STIPREDICATE_DEFS_FOR_MC_ANALYSIS
#undef GET_STIPREDICATE_DEFS_FOR_MC_ANALYSIS

#endif // GET_STIPREDICATE_DEFS_FOR_MC_ANALYSIS

