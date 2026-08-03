use crate::{Adapter, AdapterResult, FindInput, InfoInput, OutlineInput, ReadInput};
use docnav_protocol::{
    AdapterIdentity, Cost, FindResult, FormatDescriptor, InfoResult, Manifest, OutlineResult,
    ReadResult, MANIFEST_VERSION,
};

pub(super) struct NoHookAdapter;

pub(super) fn no_hook_manifest() -> Manifest {
    Manifest {
        manifest_version: MANIFEST_VERSION.to_owned(),
        adapter: AdapterIdentity {
            id: "no-hook".to_owned(),
            name: "No Hook".to_owned(),
            version: "0.1.0".to_owned(),
        },
        formats: vec![FormatDescriptor {
            id: "stub".to_owned(),
            extensions: vec![".stub".to_owned()],
            filenames: vec![],
            content_types: vec!["text/stub".to_owned()],
        }],
    }
}

impl Adapter for NoHookAdapter {
    fn outline(&self, _input: &OutlineInput) -> AdapterResult<OutlineResult> {
        Ok(OutlineResult::structured(Vec::new(), None))
    }

    fn read(&self, input: &ReadInput) -> AdapterResult<ReadResult> {
        Ok(ReadResult {
            ref_id: input.ref_id.clone(),
            content: String::new(),
            content_type: "text/stub".to_owned(),
            cost: Cost {
                measurements: Vec::new(),
            },
            page: None,
        })
    }

    fn find(&self, _input: &FindInput) -> AdapterResult<FindResult> {
        Ok(FindResult::new(Vec::new(), None))
    }

    fn info(&self, _input: &InfoInput) -> AdapterResult<InfoResult> {
        Ok(InfoResult {
            document: None,
            adapter: None,
            metadata: None,
        })
    }
}
