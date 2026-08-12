use crate::{BoundedInputCost, InputCost};
use docnav_protocol::CostUnit;
use docnav_text_cost::{BoundedTextCost, TextMeter, TextMeterFlow};
use std::convert::Infallible;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use = "the fragment flow determines whether the projection should provide another fragment"]
pub enum TextFragmentFlow {
    Continue,
    Stop,
}

impl TextFragmentFlow {
    pub const fn should_stop(self) -> bool {
        matches!(self, Self::Stop)
    }
}

pub trait TextFragmentSink {
    fn push(&mut self, fragment: &str) -> TextFragmentFlow;
}

pub trait TextProjection<I> {
    type Error;

    fn project(&mut self, input: &I, sink: &mut dyn TextFragmentSink) -> Result<(), Self::Error>;
}

#[derive(Clone, Debug)]
pub struct TextInputCost<P> {
    projection: P,
}

impl<P> TextInputCost<P> {
    pub const fn new(projection: P) -> Self {
        Self { projection }
    }
}

impl<I, P> InputCost<I> for TextInputCost<P>
where
    P: TextProjection<I>,
{
    type Error = P::Error;

    fn measure(
        &mut self,
        input: &I,
        unit: CostUnit,
        threshold: u64,
    ) -> Result<BoundedInputCost, Self::Error> {
        let mut sink = MeterSink::new(unit, threshold);
        self.projection.project(input, &mut sink)?;
        Ok(match sink.finish() {
            BoundedTextCost::Fits(cost) => BoundedInputCost::Fits(cost),
            BoundedTextCost::ExceedsThreshold => BoundedInputCost::ExceedsThreshold,
        })
    }
}

struct MeterSink {
    meter: TextMeter,
}

impl MeterSink {
    fn new(unit: CostUnit, threshold: u64) -> Self {
        Self {
            meter: TextMeter::new(unit, threshold),
        }
    }

    fn finish(self) -> BoundedTextCost {
        self.meter.finish()
    }
}

impl TextFragmentSink for MeterSink {
    fn push(&mut self, fragment: &str) -> TextFragmentFlow {
        match self.meter.consume(fragment) {
            TextMeterFlow::Continue => TextFragmentFlow::Continue,
            TextMeterFlow::ProvenExceedsThreshold => TextFragmentFlow::Stop,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IdentityTextProjection;

impl TextProjection<String> for IdentityTextProjection {
    type Error = Infallible;

    fn project(
        &mut self,
        input: &String,
        sink: &mut dyn TextFragmentSink,
    ) -> Result<(), Self::Error> {
        let _ = sink.push(input);
        Ok(())
    }
}
