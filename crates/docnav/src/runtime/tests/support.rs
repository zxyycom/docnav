use std::fs;
use std::path::{Path, PathBuf};

mod commands;
mod logging;
mod output;
mod workspace;

pub(super) use commands::{cli_source, outline_command, read_command, set_cli_value};
pub(super) use logging::{
    event_named, is_lower_sha256, read_jsonl_events, test_sha256_hex, FailingWriter,
    LogAbsentWriter,
};
pub(super) use output::{
    assert_no_invocation_event_text, entry_labels, first_entry_label, parse_single_json_value,
    write_document_result, write_outcome_text_with_exit, write_protocol_json,
    write_protocol_json_with_exit,
};
pub(super) use workspace::{
    default_context, markdown_project, project_context, temp_workspace, write_config_file,
    write_native_option_config,
};

pub(super) struct TempWorkspace {
    path: PathBuf,
}

impl TempWorkspace {
    pub(super) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
