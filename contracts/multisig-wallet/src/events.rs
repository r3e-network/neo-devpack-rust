use neo_devpack::prelude::*;

#[neo_event]
pub struct ProposalCreated {
    pub proposal_id: NeoInteger,
    pub proposer: NeoByteString,
    pub target: NeoByteString,
    pub method: NeoString,
}

#[neo_event]
pub struct ProposalExecuted {
    pub proposal_id: NeoInteger,
}
