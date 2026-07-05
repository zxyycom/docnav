use docnav_typed_fields::{
    FieldBoundKind, FieldConstraints, FieldNumericRange, FieldRange, SchemaMetadataView, ValueKind,
};

pub(crate) fn expected_value_description(metadata: &SchemaMetadataView) -> String {
    match metadata.value_kind {
        ValueKind::String => "a string".to_owned(),
        ValueKind::Integer => integer_description(&metadata.constraints),
        ValueKind::Number => number_description(&metadata.constraints),
        ValueKind::Boolean => "a boolean".to_owned(),
        ValueKind::Array => "an array".to_owned(),
        ValueKind::Object => "an object".to_owned(),
        ValueKind::Json => "a JSON value".to_owned(),
    }
}

pub(crate) fn value_kind_name(kind: ValueKind) -> &'static str {
    match kind {
        ValueKind::String => "string",
        ValueKind::Integer => "integer",
        ValueKind::Number => "number",
        ValueKind::Boolean => "boolean",
        ValueKind::Array => "array",
        ValueKind::Object => "object",
        ValueKind::Json => "json",
    }
}

fn integer_description(constraints: &FieldConstraints) -> String {
    match constraints.numeric_range {
        FieldNumericRange::Integer(range) => {
            range_description("integer", range).unwrap_or_else(|| "an integer".to_owned())
        }
        _ => "an integer".to_owned(),
    }
}

fn number_description(constraints: &FieldConstraints) -> String {
    match constraints.numeric_range {
        FieldNumericRange::Number(range) => {
            range_description("number", range).unwrap_or_else(|| "a number".to_owned())
        }
        _ => "a number".to_owned(),
    }
}

fn range_description<T>(kind: &str, range: FieldRange<T>) -> Option<String>
where
    T: std::fmt::Display + Copy,
{
    match (range.minimum, range.maximum) {
        (Some(min), Some(max))
            if min.kind == FieldBoundKind::Closed && max.kind == FieldBoundKind::Closed =>
        {
            Some(format!("{kind} in range {}..{}", min.value, max.value))
        }
        (Some(min), None) if min.kind == FieldBoundKind::Closed => {
            Some(format!("{kind} >= {}", min.value))
        }
        (None, Some(max)) if max.kind == FieldBoundKind::Closed => {
            Some(format!("{kind} <= {}", max.value))
        }
        _ => None,
    }
}
