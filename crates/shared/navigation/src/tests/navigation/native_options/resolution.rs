use super::*;

#[test]
fn navigation_resolves_selected_catalog_option_and_dispatches() {
    let outcome = execute_loaded_navigation_command(
        navigation_command(vec![cli_value_candidate(
            "docnav.adapters.docnav-markdown.options.max_heading_level",
            "--max-heading-level",
            json!(4),
        )]),
        config_sources(
            json!({
                "defaults": {
                    "pagination": { "limit": 120 },
                    "output": "protocol-json"
                },
                "options": {
                    "docnav-markdown": {
                        "max_heading_level": 2
                    }
                }
            }),
            json!({
                "options": {
                    "docnav-markdown": {
                        "max_heading_level": 1
                    }
                }
            }),
        ),
        &crate::tests::support::document_parameter_catalog(),
        &StubRegistry,
    )
    .expect("navigation outcome");

    assert_eq!(outcome.output, NavigationOutputMode::ProtocolJson);
    match outcome.response {
        ProtocolResponse::Success(success) => {
            assert_eq!(success.operation, docnav_protocol::Operation::Outline);
            assert!(success.ok);
            let OperationResult::Outline(result) = success.result else {
                panic!("expected outline result");
            };
            let result = result.as_structured().expect("structured outline result");
            assert_eq!(result.entries[0].label, "Max 4");
        }
        ProtocolResponse::Failure(failure) => panic!("expected success, got {failure:?}"),
    }
}

#[test]
fn resolved_protocol_options_and_standard_input_share_resolution_value() {
    let command = navigation_command(vec![
        cli_value_candidate(
            "docnav.adapters.docnav-markdown.options.max_heading_level",
            "--max-heading-level",
            json!(4),
        ),
        cli_value_candidate("docnav.defaults.output", "--output", json!("protocol-json")),
    ]);
    let sources = config_sources(Value::Null, Value::Null);
    let registry = StubRegistry;
    let selected = registry
        .adapters()
        .into_iter()
        .next()
        .expect("selected adapter definition");

    let resolved = resolve_operation_input(
        &command,
        &sources,
        selected.id(),
        &crate::tests::support::document_parameter_catalog(),
    )
    .expect("resolved operation input");
    let protocol_value = resolved
        .options
        .as_ref()
        .and_then(|options| options.get("max_heading_level"))
        .expect("protocol max heading level");
    let StandardOperationInput::Outline(standard_input) = &resolved.standard_input else {
        panic!("expected outline standard input");
    };

    assert_eq!(resolved.output, NavigationOutputMode::ProtocolJson);
    assert_eq!(
        serde_json::to_value(resolved.options.as_ref().expect("protocol options")).unwrap(),
        json!({"max_heading_level": 4})
    );
    assert_eq!(protocol_value.as_i64(), standard_input.max_heading_level);
    assert_eq!(standard_input.page.get(), 1);
    assert_eq!(standard_input.limit.get(), 6000);
}

#[test]
fn pagination_disabled_normalizes_protocol_and_standard_input_limit() {
    let command = navigation_command(Vec::new());
    let sources = config_sources(
        json!({
            "defaults": {
                "pagination": {
                    "enabled": false,
                    "limit": 12
                }
            }
        }),
        Value::Null,
    );
    let registry = StubRegistry;
    let selected = registry
        .adapters()
        .into_iter()
        .next()
        .expect("selected adapter definition");

    let resolved = resolve_operation_input(
        &command,
        &sources,
        selected.id(),
        &crate::tests::support::document_parameter_catalog(),
    )
    .expect("resolved operation input");
    let StandardOperationInput::Outline(standard_input) = &resolved.standard_input else {
        panic!("expected outline standard input");
    };
    let protocol_limit = resolved.limit.expect("protocol limit");

    assert_eq!(protocol_limit.get(), u32::MAX);
    assert_eq!(standard_input.limit, protocol_limit);
}

#[test]
fn read_and_find_build_sibling_protocol_and_standard_input_facts() {
    let sources = config_sources(Value::Null, Value::Null);
    let registry = StubRegistry;
    let selected = registry
        .adapters()
        .into_iter()
        .next()
        .expect("selected adapter definition");
    let catalog = crate::tests::support::document_parameter_catalog();

    let mut read_command = navigation_command(Vec::new());
    read_command.operation = Operation::Read;
    read_command.ref_id = Some("stub:1".to_owned());
    let read = resolve_operation_input(&read_command, &sources, selected.id(), &catalog)
        .expect("resolved read input");
    let StandardOperationInput::Read(read_standard) = &read.standard_input else {
        panic!("expected read standard input");
    };
    assert_eq!(read.document_path, read_standard.document_path);
    assert_eq!(read.ref_id.as_deref(), Some(read_standard.ref_id.as_str()));
    assert_eq!(read.page, Some(read_standard.page));
    assert_eq!(read.limit, Some(read_standard.limit));
    assert!(read.options.is_none());

    let mut find_command = navigation_command(vec![cli_value_candidate(
        "docnav.adapters.docnav-markdown.options.max_heading_level",
        "--max-heading-level",
        json!(5),
    )]);
    find_command.operation = Operation::Find;
    find_command.query = Some("needle".to_owned());
    let find = resolve_operation_input(&find_command, &sources, selected.id(), &catalog)
        .expect("resolved find input");
    let StandardOperationInput::Find(find_standard) = &find.standard_input else {
        panic!("expected find standard input");
    };
    assert_eq!(find.document_path, find_standard.document_path);
    assert_eq!(find.query.as_deref(), Some(find_standard.query.as_str()));
    assert_eq!(find.page, Some(find_standard.page));
    assert_eq!(find.limit, Some(find_standard.limit));
    assert_eq!(find_standard.max_heading_level, Some(5));
    assert_eq!(
        serde_json::to_value(find.options.expect("find protocol options")).unwrap(),
        json!({"max_heading_level": 5})
    );
}
