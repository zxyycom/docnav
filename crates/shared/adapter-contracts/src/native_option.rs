mod descriptions;
mod handoff;
mod issue;
mod spec;
mod spec_error;

pub use handoff::{NativeOptionHandoff, NativeOptionValue};
pub use issue::NativeOptionIssue;
pub use spec::{AdapterOptionProcessStrategy, AdapterOptionSpec, AdapterOptionSpecBuilder};
pub use spec_error::AdapterOptionSpecError;
