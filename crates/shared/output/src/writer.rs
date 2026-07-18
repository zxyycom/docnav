use std::io::Write;

use docnav_json_io::write_json_value_pretty;
use docnav_protocol::ProtocolResponse;

use crate::{DocumentOutputError, OutputPlan};

pub fn write_document_response<W: Write>(
    response: &ProtocolResponse,
    plan: OutputPlan,
    stdout: &mut W,
) -> Result<(), DocumentOutputError> {
    match plan {
        OutputPlan::ProtocolJson => {
            write_json_value_pretty(response, stdout).map_err(DocumentOutputError::StdoutJson)
        }
        OutputPlan::Rendered(renderer) => {
            let rendered = renderer(response).map_err(DocumentOutputError::Render)?;
            stdout
                .write_all(rendered.as_bytes())
                .map_err(DocumentOutputError::StdoutWrite)
        }
    }
}
