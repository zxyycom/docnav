use docnav_diagnostics::ProtocolDiagnosticCode;
use serde::{Deserialize, Deserializer, Serializer};

pub fn serialize<S>(code: &ProtocolDiagnosticCode, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(code.protocol_code())
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<ProtocolDiagnosticCode, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    ProtocolDiagnosticCode::from_protocol_code(&value)
        .ok_or_else(|| serde::de::Error::custom(format!("unknown protocol error code {value:?}")))
}
