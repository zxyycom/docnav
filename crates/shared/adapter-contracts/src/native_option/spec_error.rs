#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdapterOptionSpecError {
    InvalidDeclarationPath { identity: String, path: Vec<String> },
    InvalidFieldDeclaration { identity: String, reason: String },
}

impl std::fmt::Display for AdapterOptionSpecError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDeclarationPath { identity, path } => write!(
                formatter,
                "adapter option {identity} declaration path must be options.<key>, got {}",
                display_path(path)
            ),
            Self::InvalidFieldDeclaration { identity, reason } => write!(
                formatter,
                "adapter option {identity} field declaration is invalid: {reason}"
            ),
        }
    }
}

impl std::error::Error for AdapterOptionSpecError {}

fn display_path(path: &[String]) -> String {
    if path.is_empty() {
        "<missing>".to_owned()
    } else {
        path.join(".")
    }
}
