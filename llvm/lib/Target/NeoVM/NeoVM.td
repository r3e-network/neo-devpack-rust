// TableGen root file for the NeoVM backend.

include "NeoVMBase.td"
include "NeoVMRegisterInfo.td"
include "NeoVMInstrInfo.td"
include "NeoVMSubtarget.td"

// Anchor generated tables.
def NeoVMInstrInfo : InstrInfo;

def NeoVM : Target {
  let InstructionSet = NeoVMInstrInfo;
}
