use docnav_protocol::CostUnit;
use std::convert::Infallible;
use std::fmt;
use std::num::NonZeroU64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundedInputCost {
    Fits(u64),
    ExceedsThreshold,
}

pub trait InputCost<I> {
    type Error;

    fn measure(
        &mut self,
        input: &I,
        unit: CostUnit,
        threshold: u64,
    ) -> Result<BoundedInputCost, Self::Error>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputDisposition {
    Accepted,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StopReason {
    LimitExhausted,
    InputDoesNotFit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlowControl {
    Continue,
    Stop(StopReason),
}

impl FlowControl {
    const fn should_continue(self) -> bool {
        matches!(self, Self::Continue)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BudgetSnapshot {
    pub unit: CostUnit,
    pub limit: u64,
    pub used: u64,
    pub remaining: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GateSnapshot {
    Limited(BudgetSnapshot),
    Unbounded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use = "the push outcome determines whether the producer may generate another input"]
pub struct PushOutcome {
    pub input: InputDisposition,
    pub flow: FlowControl,
    pub gate: GateSnapshot,
}

impl PushOutcome {
    pub const fn should_continue(self) -> bool {
        self.flow.should_continue()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceCompletion {
    Exhausted,
    NotExhausted,
}

impl SourceCompletion {
    const fn is_exhausted(self) -> bool {
        matches!(self, Self::Exhausted)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GateReport {
    Limited {
        budget: BudgetSnapshot,
        stop_reason: Option<StopReason>,
    },
    Unbounded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutputReport {
    pub gate: GateReport,
    pub complete: bool,
}

pub trait Gate<I> {
    type Error;

    fn push(&mut self, input: &I) -> Result<PushOutcome, Self::Error>;
    fn finish(self, source_completion: SourceCompletion) -> OutputReport;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionPushError<E> {
    InputCost(E),
    InvalidMeasuredCost { measured: u64, remaining: u64 },
    Stopped(StopReason),
}

impl<E: fmt::Display> fmt::Display for SessionPushError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputCost(error) => write!(formatter, "input cost failed: {error}"),
            Self::InvalidMeasuredCost {
                measured,
                remaining,
            } => write!(
                formatter,
                "input cost returned {measured} as fitting within remaining budget {remaining}"
            ),
            Self::Stopped(reason) => write!(formatter, "output session already stopped: {reason}"),
        }
    }
}

impl<E> std::error::Error for SessionPushError<E>
where
    E: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InputCost(error) => Some(error),
            Self::InvalidMeasuredCost { .. } | Self::Stopped(_) => None,
        }
    }
}

impl fmt::Display for StopReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LimitExhausted => formatter.write_str("limit exhausted"),
            Self::InputDoesNotFit => formatter.write_str("input does not fit"),
        }
    }
}

#[derive(Debug)]
pub struct LimitedGate<P> {
    input_cost: P,
    unit: CostUnit,
    limit: u64,
    used: u64,
    stop_reason: Option<StopReason>,
}

impl<P> LimitedGate<P> {
    pub const fn new(unit: CostUnit, limit: NonZeroU64, input_cost: P) -> Self {
        Self {
            input_cost,
            unit,
            limit: limit.get(),
            used: 0,
            stop_reason: None,
        }
    }
}

impl<I, P> Gate<I> for LimitedGate<P>
where
    P: InputCost<I>,
{
    type Error = SessionPushError<P::Error>;

    fn push(&mut self, input: &I) -> Result<PushOutcome, Self::Error> {
        if let Some(reason) = self.stop_reason {
            return Err(SessionPushError::Stopped(reason));
        }

        let remaining = self.limit - self.used;
        let measured = self
            .input_cost
            .measure(input, self.unit, remaining)
            .map_err(SessionPushError::InputCost)?;

        match measured {
            BoundedInputCost::ExceedsThreshold => {
                self.stop_reason = Some(StopReason::InputDoesNotFit);
                Ok(self.outcome(
                    InputDisposition::Rejected,
                    FlowControl::Stop(StopReason::InputDoesNotFit),
                ))
            }
            BoundedInputCost::Fits(cost) if cost > remaining => {
                Err(SessionPushError::InvalidMeasuredCost {
                    measured: cost,
                    remaining,
                })
            }
            BoundedInputCost::Fits(cost) => {
                self.used = self
                    .used
                    .checked_add(cost)
                    .expect("accepted input cost is no greater than the remaining budget");
                if cost == remaining {
                    self.stop_reason = Some(StopReason::LimitExhausted);
                    Ok(self.outcome(
                        InputDisposition::Accepted,
                        FlowControl::Stop(StopReason::LimitExhausted),
                    ))
                } else {
                    Ok(self.outcome(InputDisposition::Accepted, FlowControl::Continue))
                }
            }
        }
    }

    fn finish(self, source_completion: SourceCompletion) -> OutputReport {
        let complete = source_completion.is_exhausted()
            && self.stop_reason != Some(StopReason::InputDoesNotFit);
        OutputReport {
            gate: GateReport::Limited {
                budget: self.snapshot(),
                stop_reason: self.stop_reason,
            },
            complete,
        }
    }
}

impl<P> LimitedGate<P> {
    fn snapshot(&self) -> BudgetSnapshot {
        BudgetSnapshot {
            unit: self.unit,
            limit: self.limit,
            used: self.used,
            remaining: self.limit - self.used,
        }
    }

    fn outcome(&self, input: InputDisposition, flow: FlowControl) -> PushOutcome {
        PushOutcome {
            input,
            flow,
            gate: GateSnapshot::Limited(self.snapshot()),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UnboundedGate;

impl<I> Gate<I> for UnboundedGate {
    type Error = Infallible;

    fn push(&mut self, _input: &I) -> Result<PushOutcome, Self::Error> {
        Ok(PushOutcome {
            input: InputDisposition::Accepted,
            flow: FlowControl::Continue,
            gate: GateSnapshot::Unbounded,
        })
    }

    fn finish(self, source_completion: SourceCompletion) -> OutputReport {
        OutputReport {
            gate: GateReport::Unbounded,
            complete: source_completion.is_exhausted(),
        }
    }
}
