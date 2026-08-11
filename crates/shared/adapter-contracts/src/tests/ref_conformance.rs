use std::sync::Mutex;

use docnav_protocol::{Cost, Measurement, ReadResult};

use crate::{assert_ref_round_trip, Adapter, AdapterDefinition, AdapterDocument, ReadInput};

use super::support::no_hook_manifest;

struct RecordingFactory {
    pages: Mutex<Vec<u32>>,
    behavior: ReadBehavior,
}

#[derive(Clone, Copy)]
enum ReadBehavior {
    Valid,
    MismatchedRef,
    InvalidContentType,
}

impl Adapter for RecordingFactory {
    fn create_document(&self, _document_path: String) -> Box<dyn AdapterDocument + '_> {
        Box::new(RecordingDocument { factory: self })
    }
}

struct RecordingDocument<'a> {
    factory: &'a RecordingFactory,
}

impl AdapterDocument for RecordingDocument<'_> {
    fn outline(
        &mut self,
        _input: &crate::OutlineInput,
    ) -> crate::AdapterResult<docnav_protocol::OutlineResult> {
        Ok(docnav_protocol::OutlineResult::structured(Vec::new(), None))
    }

    fn read(&mut self, input: &ReadInput) -> crate::AdapterResult<ReadResult> {
        self.factory.pages.lock().unwrap().push(input.page.get());
        Ok(ReadResult {
            ref_id: match self.factory.behavior {
                ReadBehavior::MismatchedRef => "opaque:different".to_owned(),
                ReadBehavior::Valid | ReadBehavior::InvalidContentType => input.ref_id.clone(),
            },
            content: "selected".to_owned(),
            content_type: match self.factory.behavior {
                ReadBehavior::InvalidContentType => String::new(),
                ReadBehavior::Valid | ReadBehavior::MismatchedRef => "text/plain".to_owned(),
            },
            cost: Cost {
                measurements: vec![Measurement {
                    unit: "bytes".to_owned(),
                    value: 8,
                    scope: None,
                }],
            },
            page: None,
        })
    }

    fn find(
        &mut self,
        _input: &crate::FindInput,
    ) -> crate::AdapterResult<docnav_protocol::FindResult> {
        unreachable!("ref harness only reads")
    }

    fn info(
        &mut self,
        _input: &crate::InfoInput,
    ) -> crate::AdapterResult<docnav_protocol::InfoResult> {
        unreachable!("ref harness only reads")
    }
}

#[test]
fn ref_conformance_reads_opaque_ref_on_same_and_fresh_documents_at_page_one() {
    let factory = RecordingFactory {
        pages: Mutex::new(Vec::new()),
        behavior: ReadBehavior::Valid,
    };
    let definition =
        AdapterDefinition::new(no_hook_manifest(), &factory, None).expect("valid definition");
    let mut document = definition.create_document("doc.stub".to_owned());
    let input = read_input();

    let (same, fresh) = assert_ref_round_trip(&definition, document.as_mut(), &input);

    assert_eq!(same.ref_id, "opaque:unchanged");
    assert_eq!(fresh.ref_id, "opaque:unchanged");
    assert_eq!(*factory.pages.lock().unwrap(), vec![1, 1]);
}

#[test]
fn ref_conformance_rejects_mismatched_read_results() {
    for behavior in [
        ReadBehavior::MismatchedRef,
        ReadBehavior::InvalidContentType,
    ] {
        let factory = RecordingFactory {
            pages: Mutex::new(Vec::new()),
            behavior,
        };
        let definition =
            AdapterDefinition::new(no_hook_manifest(), &factory, None).expect("valid definition");
        let mut document = definition.create_document("doc.stub".to_owned());

        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            assert_ref_round_trip(&definition, document.as_mut(), &read_input());
        }));

        assert!(panic.is_err());
    }
}

fn read_input() -> ReadInput {
    ReadInput {
        document_path: "doc.stub".to_owned(),
        ref_id: "opaque:unchanged".to_owned(),
        page: docnav_protocol::positive_result(7).unwrap(),
        limit: docnav_protocol::positive_result(80).unwrap(),
    }
}
