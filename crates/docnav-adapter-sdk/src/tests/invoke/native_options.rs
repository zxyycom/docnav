use super::*;

const REQUIRED_BY_TEST_OPERATIONS: &[Operation] = &[Operation::Outline];
const REQUIRED_BY_TEST_OPTION: NativeOptionSpec = NativeOptionSpec {
    flag: "--required-by-test",
    option_key: "required_by_test",
    operations: REQUIRED_BY_TEST_OPERATIONS,
    value: NativeOptionValueSpec::IntegerRange { min: 1, max: 1 },
    default: None,
};

#[test]
fn invoke_standard_parameter_normalization_preserves_options_passthrough() {
    let input = br#"{
          "protocol_version": "0.1",
          "request_id": "req-opts",
          "operation": "outline",
          "document": { "path": "sample.stub" },
          "arguments": {
            "limit": 80,
            "page": 1,
            "options": { "required_by_test": 1 }
          }
        }"#;
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = invoke_once_with_standard_parameter_config(
        &OptionsRequiredAdapter,
        invoke_config_with_native_options(6000, None, None, &[REQUIRED_BY_TEST_OPTION]),
        &input[..],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, AdapterExitCode::Success.code());
    assert!(stderr.is_empty());
    let response: ProtocolResponse =
        serde_json::from_slice(&stdout).expect("stdout is one JSON response");
    response.validate().expect("response validates");
}

#[test]
fn invoke_rejects_undeclared_native_options() {
    let input = br#"{
          "protocol_version": "0.1",
          "request_id": "req-unknown-option",
          "operation": "outline",
          "document": { "path": "sample.stub" },
          "arguments": {
            "limit": 80,
            "page": 1,
            "options": { "future_option": 1 }
          }
        }"#;
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = invoke_once(&StubAdapter, &input[..], &mut stdout, &mut stderr);

    assert_eq!(exit, AdapterExitCode::ProtocolError.code());
    assert!(stderr.is_empty());
    let response = failure_response_from_stdout(&stdout);
    assert_eq!(
        response.error.code(),
        ProtocolDiagnosticCode::InvalidRequest
    );
    assert_eq!(
        response.error.details()["field"],
        "arguments.options.future_option"
    );
}

#[test]
fn invoke_rejects_operation_inapplicable_native_options() {
    let input = br#"{
          "protocol_version": "0.1",
          "request_id": "req-info-option",
          "operation": "info",
          "document": { "path": "sample.stub" },
          "arguments": {
            "options": { "required_by_test": 1 }
          }
        }"#;
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = invoke_once_with_standard_parameter_config(
        &StubAdapter,
        invoke_config_with_native_options(6000, None, None, &[REQUIRED_BY_TEST_OPTION]),
        &input[..],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, AdapterExitCode::ProtocolError.code());
    assert!(stderr.is_empty());
    let response = failure_response_from_stdout(&stdout);
    assert_eq!(
        response.error.code(),
        ProtocolDiagnosticCode::InvalidRequest
    );
    assert_eq!(
        response.error.details()["field"],
        "arguments.options.required_by_test"
    );
}

struct OptionsRequiredAdapter;

impl Adapter for OptionsRequiredAdapter {
    fn adapter_id(&self) -> &str {
        "options-required"
    }

    fn manifest(&self) -> Manifest {
        Manifest {
            manifest_version: MANIFEST_VERSION.to_owned(),
            adapter: AdapterIdentity {
                id: "options-required".to_owned(),
                name: "Options Required".to_owned(),
                version: "0.1.0".to_owned(),
            },
            formats: vec![FormatDescriptor {
                id: "stub".to_owned(),
                extensions: vec![".stub".to_owned()],
                content_types: vec!["text/stub".to_owned()],
            }],
            capabilities: vec![Operation::Outline],
        }
    }

    fn probe(&self, path: &str) -> ProbeResult {
        ProbeResult {
            probe_version: PROBE_VERSION.to_owned(),
            adapter_id: "options-required".to_owned(),
            path: path.to_owned(),
            supported: true,
            format: Some("stub".to_owned()),
            confidence: 1.0,
            reasons: Vec::new(),
        }
    }

    fn outline(
        &self,
        _request: &RequestEnvelope,
        arguments: &OutlineArguments,
    ) -> AdapterResult<OutlineResult> {
        let Some(options) = &arguments.options else {
            return Err(crate::AdapterError::invalid_request(
                "arguments.options",
                "missing options",
            ));
        };
        if options
            .get("required_by_test")
            .and_then(serde_json::Value::as_u64)
            != Some(1)
        {
            return Err(crate::AdapterError::invalid_request(
                "arguments.options.required_by_test",
                "missing required_by_test option",
            ));
        }
        Ok(OutlineResult {
            entries: vec![Entry {
                ref_id: "L1:Options".to_owned(),
                label: "options preserved".to_owned(),
                kind: None,
                location: None,
                summary: None,
                excerpt: None,
                rank: None,
                cost: None,
                metadata: None,
            }],
            page: None,
        })
    }
}
