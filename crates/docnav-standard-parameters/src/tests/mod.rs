#![allow(dead_code)]

// @case WB-STDPARAMS-RESOLVE-001
use std::fs;
use std::path::{Path, PathBuf};

use docnav_typed_fields::{
    ExtractStrategy, ExtractionStrategyId, FieldBound, FieldDef, FieldDefs, FieldIdentity,
    FieldValidation, JsonValue,
};

use super::*;

mod construction;
mod pipeline;
mod resolution;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputMode {
    ReadableView,
    ReadableJson,
}

impl docnav_typed_fields::FieldStringEnum for OutputMode {
    fn variants() -> &'static [Self] {
        &[Self::ReadableView, Self::ReadableJson]
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::ReadableView => "readable-view",
            Self::ReadableJson => "readable-json",
        }
    }
}

const CONFIG_STRATEGY: &str = "config";
const DIRECT_STRATEGY: &str = "direct";

fn config_json_path<I, S>(segments: I) -> ExtractStrategy
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    ExtractStrategy::json_path(segments)
}

#[derive(Debug, FieldDefs)]
struct Params {
    #[field(
        FieldDef::builder("docnav.defaults.limit_chars")
            .extract(DIRECT_STRATEGY, config_json_path(["limit_chars"]))
            .extract(CONFIG_STRATEGY, config_json_path(["defaults", "limit_chars"]))
            .validation(FieldValidation::int().between(
                FieldBound::closed(1),
                FieldBound::closed(100_000),
            ))
            .default_static(20_000)
    )]
    limit_chars: Option<i64>,

    #[field(
        FieldDef::builder("docnav.defaults.output")
            .extract(DIRECT_STRATEGY, config_json_path(["output"]))
            .extract(CONFIG_STRATEGY, config_json_path(["defaults", "output"]))
            .validation(FieldValidation::string_enum::<OutputMode>())
    )]
    output: OutputMode,
}

fn catalog_entry(identity: &str) -> StandardParameterCatalogEntry {
    let metadata = Params::field_defs()
        .unwrap()
        .schema_metadata()
        .into_iter()
        .find(|metadata| metadata.identity.as_str() == identity)
        .unwrap();
    StandardParameterCatalogEntry::new(metadata)
}

fn parameter_catalog() -> StandardParameterCatalog {
    let definitions = Params::field_defs().unwrap();
    derive_standard_parameter_catalog(
        &definitions,
        &ExtractionStrategyId::from(DIRECT_STRATEGY),
        &ExtractionStrategyId::from(CONFIG_STRATEGY),
    )
    .unwrap()
}

fn source_with_value(identity: &FieldIdentity, value: JsonValue) -> StandardParameterSource {
    StandardParameterSource::default().with_value(identity.clone(), value)
}

fn identity(value: &str) -> FieldIdentity {
    FieldIdentity::new(value).unwrap()
}

fn path<const N: usize>(segments: [&str; N]) -> StandardParameterPath {
    StandardParameterPath::new(segments).unwrap()
}

fn passthrough_at(
    resolution: &StandardParameterResolution,
    path: StandardParameterPath,
) -> &PassthroughValue {
    resolution
        .passthrough()
        .iter()
        .find(|value| value.path == path)
        .unwrap()
}

fn temp_path(file_name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "docnav-standard-parameters-{}-{file_name}",
        std::process::id()
    ));
    let _ = fs::remove_file(&path);
    path
}

fn temp_file(file_name: &str, content: &str) -> PathBuf {
    let path = temp_path(file_name);
    write_file(&path, content);
    path
}

fn temp_dir(dir_name: &str) -> PathBuf {
    let path = temp_path(dir_name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir(&path).unwrap();
    path
}

#[cfg(windows)]
struct UnreadableFileGuard {
    _file: fs::File,
}

#[cfg(unix)]
struct UnreadableFileGuard {
    path: PathBuf,
    permissions: fs::Permissions,
}

#[cfg(not(any(unix, windows)))]
struct UnreadableFileGuard;

#[cfg(windows)]
fn unreadable_file(file_name: &str) -> Option<(PathBuf, UnreadableFileGuard)> {
    use std::os::windows::fs::OpenOptionsExt;

    let path = temp_file(file_name, "{}");
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .share_mode(0)
        .open(&path)
        .ok()?;
    Some((path, UnreadableFileGuard { _file: file }))
}

#[cfg(unix)]
fn unreadable_file(file_name: &str) -> Option<(PathBuf, UnreadableFileGuard)> {
    use std::os::unix::fs::PermissionsExt;

    let path = temp_file(file_name, "{}");
    let permissions = fs::metadata(&path).ok()?.permissions();
    let mut unreadable = permissions.clone();
    unreadable.set_mode(0o0);
    fs::set_permissions(&path, unreadable).ok()?;
    if fs::read_to_string(&path).is_ok() {
        fs::set_permissions(&path, permissions).ok()?;
        return None;
    }
    Some((path.clone(), UnreadableFileGuard { path, permissions }))
}

#[cfg(not(any(unix, windows)))]
fn unreadable_file(_file_name: &str) -> Option<(PathBuf, UnreadableFileGuard)> {
    None
}

#[cfg(unix)]
impl Drop for UnreadableFileGuard {
    fn drop(&mut self) {
        let _ = fs::set_permissions(&self.path, self.permissions.clone());
    }
}

fn write_file(path: &Path, content: &str) {
    fs::write(path, content).unwrap();
}
