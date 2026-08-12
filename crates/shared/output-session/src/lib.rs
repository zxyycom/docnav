mod collector;
mod gate;
mod projection;
mod session;

pub use collector::{Collector, StringCollector, VecCollector};
pub use gate::{
    BoundedInputCost, BudgetSnapshot, FlowControl, Gate, GateReport, GateSnapshot, InputCost,
    InputDisposition, LimitedGate, OutputReport, PushOutcome, SessionPushError, SourceCompletion,
    StopReason, UnboundedGate,
};
pub use projection::{
    IdentityTextProjection, TextFragmentFlow, TextFragmentSink, TextInputCost, TextProjection,
};
pub use session::{OutputSession, SessionOutput};

#[cfg(test)]
mod tests;
