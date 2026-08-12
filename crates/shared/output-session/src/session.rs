use crate::{Collector, Gate, LimitedGate, OutputReport, SourceCompletion, UnboundedGate};
use docnav_protocol::CostUnit;
use std::marker::PhantomData;
use std::num::NonZeroU64;

#[derive(Debug)]
pub struct OutputSession<I, G, C> {
    gate: G,
    collector: C,
    input: PhantomData<fn(I)>,
}

impl<I, G, C> OutputSession<I, G, C> {
    pub const fn new(gate: G, collector: C) -> Self {
        Self {
            gate,
            collector,
            input: PhantomData,
        }
    }
}

impl<I, P, C> OutputSession<I, LimitedGate<P>, C>
where
    P: crate::InputCost<I>,
{
    pub const fn limited(unit: CostUnit, limit: NonZeroU64, input_cost: P, collector: C) -> Self {
        Self::new(LimitedGate::new(unit, limit, input_cost), collector)
    }
}

impl<I, C> OutputSession<I, UnboundedGate, C> {
    pub const fn unbounded(collector: C) -> Self {
        Self::new(UnboundedGate, collector)
    }
}

impl<I, G, C> OutputSession<I, G, C>
where
    G: Gate<I>,
    C: Collector<I>,
{
    pub fn push(&mut self, input: I) -> Result<crate::PushOutcome, G::Error> {
        let outcome = self.gate.push(&input)?;
        if outcome.input == crate::InputDisposition::Accepted {
            self.collector.accept(input);
        }
        Ok(outcome)
    }

    pub fn finish(
        self,
        source_completion: SourceCompletion,
    ) -> Result<SessionOutput<C::Output>, C::Error> {
        let Self {
            gate, collector, ..
        } = self;
        let report = gate.finish(source_completion);
        let output = collector.finish()?;
        Ok(SessionOutput { output, report })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionOutput<O> {
    pub output: O,
    pub report: OutputReport,
}
