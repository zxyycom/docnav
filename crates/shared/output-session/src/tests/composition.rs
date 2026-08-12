use super::*;
use std::cell::Cell;
use std::convert::Infallible;
use std::rc::Rc;

#[derive(Default)]
struct SumCollector(u64);

impl Collector<u8> for SumCollector {
    type Output = u64;
    type Error = Infallible;

    fn accept(&mut self, input: u8) {
        self.0 += u64::from(input);
    }

    fn finish(self) -> Result<Self::Output, Self::Error> {
        Ok(self.0)
    }
}

#[test]
fn unbounded_reuses_session_and_custom_collector_without_input_cost() {
    let mut session = OutputSession::unbounded(SumCollector::default());
    for value in [1_u8, 2, 3] {
        let outcome = session.push(value).unwrap();
        assert_eq!(outcome.input, InputDisposition::Accepted);
        assert_eq!(outcome.flow, FlowControl::Continue);
        assert_eq!(outcome.gate, GateSnapshot::Unbounded);
    }

    let output = session.finish(SourceCompletion::Exhausted).unwrap();
    assert_eq!(output.output, 6);
    assert_eq!(
        output.report,
        OutputReport {
            gate: GateReport::Unbounded,
            complete: true,
        }
    );
}

#[test]
fn unbounded_report_uses_caller_owned_source_completion() {
    let session: OutputSession<String, _, _> = OutputSession::unbounded(VecCollector::new());

    let output = session.finish(SourceCompletion::NotExhausted).unwrap();
    assert!(output.output.is_empty());
    assert_eq!(
        output.report,
        OutputReport {
            gate: GateReport::Unbounded,
            complete: false,
        }
    );
}

#[derive(Default)]
struct FailingFinishCollector;

impl Collector<String> for FailingFinishCollector {
    type Output = String;
    type Error = FinishError;

    fn accept(&mut self, _input: String) {}

    fn finish(self) -> Result<Self::Output, Self::Error> {
        Err(FinishError)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FinishError;

#[test]
fn collector_finish_failure_ends_session_without_output() {
    let mut session = OutputSession::unbounded(FailingFinishCollector);
    let _ = session.push("accepted".to_owned()).unwrap();

    assert_eq!(
        session.finish(SourceCompletion::Exhausted),
        Err(FinishError)
    );
}

struct RecordingProjection {
    fragments: Rc<Cell<usize>>,
}

impl TextProjection<Vec<String>> for RecordingProjection {
    type Error = Infallible;

    fn project(
        &mut self,
        input: &Vec<String>,
        sink: &mut dyn TextFragmentSink,
    ) -> Result<(), Self::Error> {
        for fragment in input {
            self.fragments.set(self.fragments.get() + 1);
            if sink.push(fragment).should_stop() {
                break;
            }
        }
        Ok(())
    }
}

#[test]
fn projection_stops_after_meter_proves_exceed() {
    let fragments = Rc::new(Cell::new(0));
    let policy = TextInputCost::new(RecordingProjection {
        fragments: Rc::clone(&fragments),
    });
    let mut session =
        OutputSession::limited(CostUnit::Bytes, limit(2), policy, VecCollector::new());
    let outcome = session
        .push(vec!["abc".to_owned(), "must-not-project".to_owned()])
        .unwrap();

    assert_eq!(outcome.input, InputDisposition::Rejected);
    assert_eq!(fragments.get(), 1);
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TestEntry {
    ref_id: String,
    label: String,
    summary: Option<String>,
    rank: u64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct TestEntryProjection;

impl TextProjection<TestEntry> for TestEntryProjection {
    type Error = Infallible;

    fn project(
        &mut self,
        input: &TestEntry,
        sink: &mut dyn TextFragmentSink,
    ) -> Result<(), Self::Error> {
        for (index, field) in [
            Some(input.ref_id.as_str()),
            Some(input.label.as_str()),
            input.summary.as_deref(),
        ]
        .into_iter()
        .flatten()
        .enumerate()
        {
            if index > 0 && sink.push("\n").should_stop() {
                return Ok(());
            }
            if sink.push(field).should_stop() {
                return Ok(());
            }
        }
        Ok(())
    }
}

#[test]
fn caller_owned_projection_composes_with_vec_collector() {
    let policy = TextInputCost::new(TestEntryProjection);
    let mut session =
        OutputSession::limited(CostUnit::Lines, limit(3), policy, VecCollector::new());
    let value = TestEntry {
        ref_id: "H:L1:H1".to_owned(),
        label: "Title".to_owned(),
        summary: Some("Summary".to_owned()),
        rank: 1,
    };

    let outcome = session.push(value.clone()).unwrap();
    assert_eq!(outcome.flow, FlowControl::Stop(StopReason::LimitExhausted));
    let output = session.finish(SourceCompletion::Exhausted).unwrap();
    assert_eq!(output.output, vec![value]);
    assert!(output.report.complete);
}

#[test]
fn canonical_loop_does_not_request_tail_after_stop() {
    let limited =
        OutputSession::limited(CostUnit::Bytes, limit(3), text_cost(), VecCollector::new());
    let (limited_output, limited_stats) = drive_string_session(limited, 100);
    assert_eq!(limited_stats, RunStats::new(3, 3, 3));
    assert_eq!(limited_output.output, vec!["0", "1", "2"]);
    assert!(!limited_output.report.complete);

    let unbounded = OutputSession::unbounded(VecCollector::new());
    let (unbounded_output, unbounded_stats) = drive_string_session(unbounded, 100);
    assert_eq!(unbounded_stats, RunStats::new(100, 100, 100));
    assert_eq!(unbounded_output.output.len(), 100);
    assert!(unbounded_output.report.complete);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RunStats {
    produced: usize,
    pushed: usize,
    accepted: usize,
}

impl RunStats {
    const fn new(produced: usize, pushed: usize, accepted: usize) -> Self {
        Self {
            produced,
            pushed,
            accepted,
        }
    }
}

fn drive_string_session<G, C>(
    mut session: OutputSession<String, G, C>,
    input_count: usize,
) -> (SessionOutput<C::Output>, RunStats)
where
    G: Gate<String>,
    G::Error: std::fmt::Debug,
    C: Collector<String>,
    C::Error: std::fmt::Debug,
{
    let mut stats = RunStats::new(0, 0, 0);
    let mut stopped = false;
    for value in 0..input_count {
        stats.produced += 1;
        let outcome = session.push(value.to_string()).unwrap();
        stats.pushed += 1;
        if outcome.input == InputDisposition::Accepted {
            stats.accepted += 1;
        }
        if !outcome.should_continue() {
            stopped = true;
            break;
        }
    }

    let completion = if stopped {
        SourceCompletion::NotExhausted
    } else {
        SourceCompletion::Exhausted
    };
    (session.finish(completion).unwrap(), stats)
}
