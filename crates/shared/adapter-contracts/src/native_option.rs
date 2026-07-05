mod descriptions;
mod issue;
mod spec;
mod spec_error;

pub use issue::NativeOptionIssue;
pub use spec::{AdapterOptionProcessStrategy, AdapterOptionSpec, AdapterOptionSpecBuilder};
pub use spec_error::AdapterOptionSpecError;
