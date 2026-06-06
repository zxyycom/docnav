use super::document::Heading;

pub const FULL_DOCUMENT_REF: &str = "doc:full";

pub(super) fn heading_ref(heading: &Heading) -> String {
    if heading.path_occurrence == 1 {
        format!("L{}:{}", heading.line, heading.path)
    } else {
        format!(
            "L{}#{}:{}",
            heading.line, heading.path_occurrence, heading.path
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ParsedRef {
    Heading {
        line: usize,
        path: String,
        occurrence: usize,
    },
}

impl ParsedRef {
    pub(super) fn parse(ref_id: &str) -> Option<Self> {
        let rest = ref_id.strip_prefix('L')?;
        let (prefix, path) = rest.split_once(':')?;
        let (line, occurrence) = match prefix.split_once('#') {
            Some((line, occurrence)) => (line, occurrence.parse::<usize>().ok()?),
            None => (prefix, 1),
        };
        let line = line.parse::<usize>().ok()?;

        if line == 0 || path.is_empty() || occurrence == 0 {
            return None;
        }

        Some(Self::Heading {
            line,
            path: path.to_owned(),
            occurrence,
        })
    }

    pub(super) fn matches(&self, heading: &Heading) -> bool {
        match self {
            Self::Heading {
                line,
                path,
                occurrence,
            } => {
                heading.line == *line
                    && heading.path == *path
                    && heading.path_occurrence == *occurrence
            }
        }
    }
}
