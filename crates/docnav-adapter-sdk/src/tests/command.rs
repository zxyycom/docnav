use super::common::StubAdapter;
use crate::{run_command, AdapterExitCode, SdkCommand};

#[test]
fn manifest_and_probe_are_not_wrapped_in_invoke_envelope() {
    let mut manifest_stdout = Vec::new();
    let exit = run_command(
        &StubAdapter,
        SdkCommand::Manifest,
        std::io::empty(),
        &mut manifest_stdout,
        std::io::sink(),
    );
    assert_eq!(exit, AdapterExitCode::Success.code());
    let manifest: serde_json::Value =
        serde_json::from_slice(&manifest_stdout).expect("manifest JSON");
    assert!(manifest.get("manifest_version").is_some());
    assert!(manifest.get("protocol_version").is_none());
    assert!(manifest.get("protocol").is_none());
    assert!(manifest.get("recommended_parameters").is_none());
    assert!(manifest.get("ok").is_none());

    let mut probe_stdout = Vec::new();
    let exit = run_command(
        &StubAdapter,
        SdkCommand::Probe {
            path: "sample.stub".to_owned(),
        },
        std::io::empty(),
        &mut probe_stdout,
        std::io::sink(),
    );
    assert_eq!(exit, AdapterExitCode::Success.code());
    let probe: serde_json::Value = serde_json::from_slice(&probe_stdout).expect("probe JSON");
    assert!(probe.get("probe_version").is_some());
    assert!(probe.get("protocol_version").is_none());
    assert!(probe.get("ok").is_none());
}
