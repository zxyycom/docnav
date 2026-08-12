use super::*;
use docnav_protocol::CostUnit;
use std::num::NonZeroU64;

fn limit(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).expect("test limit is positive")
}

fn text_cost() -> TextInputCost<IdentityTextProjection> {
    TextInputCost::new(IdentityTextProjection)
}

#[test]
fn limited_session_commits_only_accepted_inputs() {
    let mut session =
        OutputSession::limited(CostUnit::Bytes, limit(5), text_cost(), VecCollector::new());

    let first = session.push("abc".to_owned()).unwrap();
    assert_eq!(first.input, InputDisposition::Accepted);
    assert_eq!(first.flow, FlowControl::Continue);
    assert_eq!(
        first.gate,
        GateSnapshot::Limited(BudgetSnapshot {
            unit: CostUnit::Bytes,
            limit: 5,
            used: 3,
            remaining: 2,
        })
    );

    let rejected = session.push("界".to_owned()).unwrap();
    assert_eq!(rejected.input, InputDisposition::Rejected);
    assert_eq!(
        rejected.flow,
        FlowControl::Stop(StopReason::InputDoesNotFit)
    );

    let output = session.finish(SourceCompletion::NotExhausted).unwrap();
    assert_eq!(output.output, vec!["abc"]);
    assert_eq!(
        output.report,
        OutputReport {
            gate: GateReport::Limited {
                budget: BudgetSnapshot {
                    unit: CostUnit::Bytes,
                    limit: 5,
                    used: 3,
                    remaining: 2,
                },
                stop_reason: Some(StopReason::InputDoesNotFit),
            },
            complete: false,
        }
    );
}

#[test]
fn exact_limit_accepts_input_and_stops() {
    let mut session = OutputSession::limited(
        CostUnit::Bytes,
        limit(3),
        text_cost(),
        StringCollector::new(),
    );

    let outcome = session.push("abc".to_owned()).unwrap();
    assert_eq!(outcome.input, InputDisposition::Accepted);
    assert_eq!(outcome.flow, FlowControl::Stop(StopReason::LimitExhausted));

    let output = session.finish(SourceCompletion::Exhausted).unwrap();
    assert_eq!(output.output, "abc");
    assert!(output.report.complete);
}

#[test]
fn exact_limit_is_incomplete_when_source_has_more_input() {
    let mut session = OutputSession::limited(
        CostUnit::Bytes,
        limit(3),
        text_cost(),
        StringCollector::new(),
    );

    let outcome = session.push("abc".to_owned()).unwrap();
    assert_eq!(outcome.flow, FlowControl::Stop(StopReason::LimitExhausted));

    let output = session.finish(SourceCompletion::NotExhausted).unwrap();
    assert_eq!(output.output, "abc");
    assert!(!output.report.complete);
}

#[test]
fn empty_limited_session_finishes_complete() {
    let session: OutputSession<String, _, _> = OutputSession::limited(
        CostUnit::Bytes,
        limit(3),
        text_cost(),
        StringCollector::new(),
    );

    let output = session.finish(SourceCompletion::Exhausted).unwrap();
    assert_eq!(output.output, "");
    assert_eq!(
        output.report,
        OutputReport {
            gate: GateReport::Limited {
                budget: BudgetSnapshot {
                    unit: CostUnit::Bytes,
                    limit: 3,
                    used: 0,
                    remaining: 3,
                },
                stop_reason: None,
            },
            complete: true,
        }
    );
}

#[test]
fn stopped_session_returns_error_without_committing() {
    let mut session =
        OutputSession::limited(CostUnit::Bytes, limit(1), text_cost(), VecCollector::new());

    let _ = session.push("a".to_owned()).unwrap();
    let error = session.push("b".to_owned()).unwrap_err();
    assert_eq!(error, SessionPushError::Stopped(StopReason::LimitExhausted));

    let output = session.finish(SourceCompletion::NotExhausted).unwrap();
    assert_eq!(output.output, vec!["a"]);
}

mod composition;
mod gate_policy;
