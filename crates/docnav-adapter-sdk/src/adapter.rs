use docnav_protocol::{
    FindArguments, FindResult, InfoArguments, InfoResult, Manifest, Operation, OutlineArguments,
    OutlineResult, ProbeResult, ReadArguments, ReadResult, RequestEnvelope, StableError,
};

use crate::AdapterError;

pub type AdapterResult<T> = Result<T, AdapterError>;

pub trait Adapter {
    fn adapter_id(&self) -> &str;

    fn manifest(&self) -> Manifest;

    fn probe(&self, path: &str) -> ProbeResult;

    fn outline(
        &self,
        _request: &RequestEnvelope,
        _arguments: &OutlineArguments,
    ) -> AdapterResult<OutlineResult> {
        Err(self.unsupported(Operation::Outline))
    }

    fn read(
        &self,
        _request: &RequestEnvelope,
        _arguments: &ReadArguments,
    ) -> AdapterResult<ReadResult> {
        Err(self.unsupported(Operation::Read))
    }

    fn find(
        &self,
        _request: &RequestEnvelope,
        _arguments: &FindArguments,
    ) -> AdapterResult<FindResult> {
        Err(self.unsupported(Operation::Find))
    }

    fn info(
        &self,
        _request: &RequestEnvelope,
        _arguments: &InfoArguments,
    ) -> AdapterResult<InfoResult> {
        Err(self.unsupported(Operation::Info))
    }

    fn unsupported(&self, operation: Operation) -> AdapterError {
        StableError::capability_unsupported(operation, self.adapter_id()).into()
    }
}
