use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdapterDefinitionError {
    UnsupportedCapabilityCombination {
        id: String,
        capability: &'static str,
        reason: &'static str,
    },
}

impl fmt::Display for AdapterDefinitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedCapabilityCombination {
                id,
                capability,
                reason,
            } => write!(
                formatter,
                "adapter definition {id} has unsupported {capability} capability combination: {reason}"
            ),
        }
    }
}

impl std::error::Error for AdapterDefinitionError {}
