// @case WB-CORE-STDPARAMS-001
use std::path::PathBuf;

use docnav_protocol::Operation;
use serde_json::json;

use super::*;
use crate::cli::DocumentCommand;
use crate::config::CoreConfig;
use crate::project_context::ProjectContext;

#[test]
fn core_document_parameters_use_standard_resolution_sources() {
    let command = DocumentCommand {
        operation: Operation::Outline,
        path: "doc.md".to_owned(),
        ref_id: None,
        query: None,
        page: None,
        pagination_enabled: None,
        limit: None,
        output: None,
        adapter: Some("direct-adapter".to_owned()),
    };
    let context = ConfigContext {
        project: project_context(),
        project_config: serde_json::from_value(json!({
            "defaults": {
                "adapter": "project-adapter",
                "pagination": {
                    "enabled": true,
                    "limit": 321
                },
                "output": "protocol-json"
            }
        }))
        .unwrap(),
        user_config: serde_json::from_value(json!({
            "defaults": {
                "pagination": {
                    "enabled": false,
                    "limit": 111
                },
                "output": "readable-json"
            }
        }))
        .unwrap(),
    };

    let resolved = resolve_core_document_parameters(&command, &context).unwrap();

    assert_eq!(resolved.path, "doc.md");
    assert_eq!(resolved.adapter.as_deref(), Some("direct-adapter"));
    assert_eq!(resolved.limit.unwrap().get(), 321);
    assert_eq!(resolved.page.unwrap().get(), 1);
    assert_eq!(resolved.output, OutputMode::ProtocolJson);
    assert_eq!(resolved.defaults.adapter.source, "explicit");
    let pagination = resolved.defaults.pagination.unwrap();
    assert_eq!(pagination.enabled.source, "project");
    assert_eq!(pagination.limit.source, "project");
    assert_eq!(resolved.defaults.output.source, "project");
    assert_eq!(resolved.defaults.page.unwrap().source, "built_in");
}

#[test]
fn core_pagination_disabled_finalizes_operation_limit() {
    let command = DocumentCommand {
        operation: Operation::Outline,
        path: "doc.md".to_owned(),
        ref_id: None,
        query: None,
        page: None,
        pagination_enabled: Some(false),
        limit: Some(docnav_protocol::try_positive(12).unwrap()),
        output: None,
        adapter: None,
    };
    let context = ConfigContext {
        project: project_context(),
        project_config: CoreConfig::default(),
        user_config: CoreConfig::default(),
    };

    let resolved = resolve_core_document_parameters(&command, &context).unwrap();

    assert_eq!(resolved.limit.unwrap().get(), u32::MAX);
    let pagination = resolved.defaults.pagination.unwrap();
    assert_eq!(pagination.enabled.source, "explicit");
    assert_eq!(pagination.limit.source, "explicit");
}

fn project_context() -> ProjectContext {
    let root = PathBuf::from("D:/docnav-test");
    ProjectContext {
        cwd: root.clone(),
        project_root: root.clone(),
        project_config_path: root.join(".docnav").join("docnav.json"),
        user_config_path: root.join("user").join("docnav.json"),
    }
}

fn _core_config_type_is_reexported_for_tests(_: CoreConfig) {}
