use serde::Serialize;

use crate::record::{DiagnosticRecord, DiagnosticRecordDraft, DiagnosticRecordError};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct DiagnosticId(u64);

impl DiagnosticId {
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DiagnosticMark {
    index: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DiagnosticStack {
    records: Vec<DiagnosticRecord>,
    next_id: u64,
}

impl DiagnosticStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(
        &mut self,
        draft: DiagnosticRecordDraft,
    ) -> Result<DiagnosticId, DiagnosticRecordError> {
        let id = DiagnosticId(self.next_id);
        self.next_id += 1;
        let record = draft.into_record(id)?;
        self.records.push(record);
        Ok(id)
    }

    pub fn get(&self, id: DiagnosticId) -> Option<&DiagnosticRecord> {
        self.records.iter().find(|record| record.id == id)
    }

    pub fn mark(&self) -> DiagnosticMark {
        DiagnosticMark {
            index: self.records.len(),
        }
    }

    pub fn peek_recent(&self) -> Option<&DiagnosticRecord> {
        self.records.last()
    }

    pub fn pop_recent(&mut self) -> Option<DiagnosticRecord> {
        self.records.pop()
    }

    pub fn drain_after(&mut self, mark: DiagnosticMark) -> Vec<DiagnosticRecord> {
        let start = mark.index.min(self.records.len());
        let mut records = self.records.split_off(start);
        records.reverse();
        records
    }

    pub fn drain_after_event(
        &mut self,
        id: DiagnosticId,
        include_anchor: bool,
    ) -> Vec<DiagnosticRecord> {
        let Some(anchor_index) = self.records.iter().position(|record| record.id == id) else {
            return Vec::new();
        };
        let start = if include_anchor {
            anchor_index
        } else {
            anchor_index + 1
        };
        let mut records = self.records.split_off(start);
        records.reverse();
        records
    }

    pub fn snapshot(&self) -> Vec<DiagnosticRecord> {
        let mut records = self.records.clone();
        records.reverse();
        records
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}
