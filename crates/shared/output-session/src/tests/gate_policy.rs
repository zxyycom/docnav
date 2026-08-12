use super::*;
use std::cell::Cell;
use std::convert::Infallible;
use std::rc::Rc;

#[derive(Default)]
struct FailingCost;

impl InputCost<String> for FailingCost {
    type Error = TestCostError;

    fn measure(
        &mut self,
        _input: &String,
        _unit: CostUnit,
        _threshold: u64,
    ) -> Result<BoundedInputCost, Self::Error> {
        Err(TestCostError)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TestCostError;

impl std::fmt::Display for TestCostError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("test cost failure")
    }
}

impl std::error::Error for TestCostError {}

#[test]
fn input_cost_failure_does_not_commit_input() {
    let mut session =
        OutputSession::limited(CostUnit::Tokens, limit(4), FailingCost, VecCollector::new());

    assert_eq!(
        session.push("ignored".to_owned()).unwrap_err(),
        SessionPushError::InputCost(TestCostError)
    );
    let output = session.finish(SourceCompletion::NotExhausted).unwrap();
    assert!(output.output.is_empty());
    assert_eq!(
        output.report.gate,
        GateReport::Limited {
            budget: BudgetSnapshot {
                unit: CostUnit::Tokens,
                limit: 4,
                used: 0,
                remaining: 4,
            },
            stop_reason: None,
        }
    );
}

#[derive(Default)]
struct InvalidCost;

impl InputCost<String> for InvalidCost {
    type Error = Infallible;

    fn measure(
        &mut self,
        _input: &String,
        _unit: CostUnit,
        threshold: u64,
    ) -> Result<BoundedInputCost, Self::Error> {
        Ok(BoundedInputCost::Fits(threshold + 1))
    }
}

#[test]
fn invalid_fitting_cost_does_not_commit_input_or_budget() {
    let mut session =
        OutputSession::limited(CostUnit::Bytes, limit(4), InvalidCost, VecCollector::new());

    assert_eq!(
        session.push("ignored".to_owned()).unwrap_err(),
        SessionPushError::InvalidMeasuredCost {
            measured: 5,
            remaining: 4,
        }
    );
    let output = session.finish(SourceCompletion::NotExhausted).unwrap();
    assert!(output.output.is_empty());
    assert_eq!(
        output.report.gate,
        GateReport::Limited {
            budget: BudgetSnapshot {
                unit: CostUnit::Bytes,
                limit: 4,
                used: 0,
                remaining: 4,
            },
            stop_reason: None,
        }
    );
}

struct RecordingCost {
    observed: Rc<Cell<Option<(CostUnit, u64)>>>,
}

impl InputCost<String> for RecordingCost {
    type Error = Infallible;

    fn measure(
        &mut self,
        _input: &String,
        unit: CostUnit,
        threshold: u64,
    ) -> Result<BoundedInputCost, Self::Error> {
        self.observed.set(Some((unit, threshold)));
        Ok(BoundedInputCost::Fits(1))
    }
}

#[test]
fn limited_gate_supplies_selected_unit_and_remaining_threshold() {
    let observed = Rc::new(Cell::new(None));
    let policy = RecordingCost {
        observed: Rc::clone(&observed),
    };
    let mut session =
        OutputSession::limited(CostUnit::Tokens, limit(4), policy, VecCollector::new());

    let first = session.push("one".to_owned()).unwrap();
    assert_eq!(observed.get(), Some((CostUnit::Tokens, 4)));
    assert_eq!(
        first.gate,
        GateSnapshot::Limited(BudgetSnapshot {
            unit: CostUnit::Tokens,
            limit: 4,
            used: 1,
            remaining: 3,
        })
    );

    let second = session.push("two".to_owned()).unwrap();

    assert_eq!(observed.get(), Some((CostUnit::Tokens, 3)));
    assert_eq!(
        second.gate,
        GateSnapshot::Limited(BudgetSnapshot {
            unit: CostUnit::Tokens,
            limit: 4,
            used: 2,
            remaining: 2,
        })
    );
}
