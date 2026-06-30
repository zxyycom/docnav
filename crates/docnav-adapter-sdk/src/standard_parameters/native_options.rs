use docnav_protocol::{Operation, Options};

use crate::{AdapterError, NativeOptionSpec};

pub(super) fn validated_native_options(
    operation: Operation,
    raw_options: Option<Options>,
    specs: &[NativeOptionSpec],
) -> Result<Option<Options>, AdapterError> {
    let Some(raw_options) = raw_options else {
        return Ok(None);
    };
    let mut options = Options::new();
    for (key, value) in raw_options {
        let Some(spec) = specs.iter().find(|spec| spec.option_key == key) else {
            return Err(native_option_error(
                &key,
                format!("unknown native option {key:?}"),
            ));
        };
        if !spec.supports(operation) {
            return Err(native_option_error(
                &key,
                format!(
                    "native option {key:?} is not supported by {}",
                    operation.as_str()
                ),
            ));
        }
        let parsed = spec
            .parse_value(&value)
            .map_err(|reason| native_option_error(&key, reason))?;
        options.insert(key, parsed);
    }
    Ok((!options.is_empty()).then_some(options))
}

fn native_option_error(key: &str, reason: String) -> AdapterError {
    let field = format!("arguments.options.{key}");
    super::invalid_request_error(&field, &reason)
}
