use crate::error::NeoResult;
use crate::value::NeoValue;

/// Neo N3 Contract trait
pub trait NeoContract {
    fn name() -> &'static str;
    fn version() -> &'static str;
    fn author() -> &'static str;
    fn description() -> &'static str;
}

/// Neo N3 Contract Entry Point
pub trait NeoContractEntry {
    fn deploy() -> NeoResult<()>;
    fn update() -> NeoResult<()>;
    fn destroy() -> NeoResult<()>;
}

/// Neo N3 Contract Method trait
pub trait NeoContractMethodTrait {
    fn name() -> &'static str;
    fn parameters() -> &'static [&'static str];
    fn return_type() -> &'static str;
    fn execute(args: &[NeoValue]) -> NeoResult<NeoValue>;
}
