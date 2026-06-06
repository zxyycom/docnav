use super::common::{
    BadConfidenceProbeAdapter, EmptyReasonsProbeAdapter, InvalidManifestAdapter,
    ManifestAdapterIdDriftAdapter, ProbeAdapterIdDriftAdapter,
};
use crate::{run_command, Adapter, AdapterExitCode, SdkCommand};

#[test]
fn invalid_manifest_is_not_written_to_stdout() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = run_command(
        &InvalidManifestAdapter,
        SdkCommand::Manifest,
        std::io::empty(),
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, AdapterExitCode::ProtocolError.code());
    assert!(stdout.is_empty());
    assert!(!stderr.is_empty());
}

#[test]
fn manifest_adapter_id_drift_is_not_written_to_stdout() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = run_command(
        &ManifestAdapterIdDriftAdapter,
        SdkCommand::Manifest,
        std::io::empty(),
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, AdapterExitCode::ProtocolError.code());
    assert!(stdout.is_empty());
    let stderr = String::from_utf8(stderr).expect("stderr is UTF-8");
    assert!(stderr.contains("manifest adapter id mismatch"));
    assert!(stderr.contains("\"stub\""));
    assert!(stderr.contains("\"drift\""));
}

#[test]
fn invalid_probe_is_not_written_to_stdout() {
    fn assert_invalid_probe_not_written(adapter: &impl Adapter) {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let exit = run_command(
            adapter,
            SdkCommand::Probe {
                path: "sample.stub".to_owned(),
            },
            std::io::empty(),
            &mut stdout,
            &mut stderr,
        );

        assert_eq!(exit, AdapterExitCode::ProtocolError.code());
        assert!(stdout.is_empty());
        assert!(!stderr.is_empty());
    }

    assert_invalid_probe_not_written(&EmptyReasonsProbeAdapter);
    assert_invalid_probe_not_written(&BadConfidenceProbeAdapter);
}

#[test]
fn probe_adapter_id_drift_is_not_written_to_stdout() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = run_command(
        &ProbeAdapterIdDriftAdapter,
        SdkCommand::Probe {
            path: "sample.stub".to_owned(),
        },
        std::io::empty(),
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, AdapterExitCode::ProtocolError.code());
    assert!(stdout.is_empty());
    let stderr = String::from_utf8(stderr).expect("stderr is UTF-8");
    assert!(stderr.contains("probe result adapter id mismatch"));
    assert!(stderr.contains("probe.adapter_id"));
    assert!(stderr.contains("\"stub\""));
    assert!(stderr.contains("\"drift\""));
}
