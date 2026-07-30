use crate::document::{JsonDocument, JsonNode, JsonValue};

pub(crate) const ROOT_REF: &str = "json:#";
const UPPERCASE_HEX: &[u8; 16] = b"0123456789ABCDEF";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RefError {
    Invalid { reason: &'static str },
    NotFound,
}

pub(crate) fn canonical_ref(tokens: &[&str]) -> String {
    canonical_ref_from_tokens(tokens.iter().copied())
}

impl JsonDocument {
    pub(crate) fn resolve_ref(&self, ref_id: &str) -> Result<&JsonNode, RefError> {
        let parsed = ParsedRef::parse(ref_id)?;
        let mut node = &self.root;

        for token in &parsed.tokens {
            node = match &node.value {
                JsonValue::Object(members) => members
                    .iter()
                    .find(|member| member.name == *token)
                    .map(|member| &member.value)
                    .ok_or(RefError::NotFound)?,
                JsonValue::Array(elements) => {
                    if !is_canonical_array_index(token) {
                        return Err(invalid("expected a canonical nonnegative array index"));
                    }
                    let index = token.parse::<usize>().map_err(|_| RefError::NotFound)?;
                    elements.get(index).ok_or(RefError::NotFound)?
                }
                JsonValue::String(_)
                | JsonValue::Number(_)
                | JsonValue::Boolean(_)
                | JsonValue::Null => return Err(RefError::NotFound),
            };
        }

        Ok(node)
    }
}

#[derive(Debug, Eq, PartialEq)]
struct ParsedRef {
    tokens: Vec<String>,
}

impl ParsedRef {
    fn parse(ref_id: &str) -> Result<Self, RefError> {
        let fragment = ref_id
            .strip_prefix(ROOT_REF)
            .ok_or_else(|| invalid("expected ref to start with json:#"))?;
        if !fragment.is_ascii() {
            return Err(invalid("expected an ASCII-safe URI fragment"));
        }

        let pointer = percent_decode(fragment)?;
        if pointer.is_empty() {
            return Ok(Self { tokens: Vec::new() });
        }
        let path = pointer
            .strip_prefix('/')
            .ok_or_else(|| invalid("expected a non-root fragment to start with /"))?;
        let tokens = path
            .split('/')
            .map(decode_pointer_token)
            .collect::<Result<Vec<_>, _>>()?;

        let canonical = canonical_ref_from_tokens(tokens.iter().map(std::string::String::as_str));
        if canonical != ref_id {
            return Err(invalid("expected canonical JSON ref spelling"));
        }

        Ok(Self { tokens })
    }
}

fn canonical_ref_from_tokens<'token>(tokens: impl IntoIterator<Item = &'token str>) -> String {
    let mut ref_id = ROOT_REF.to_owned();
    for token in tokens {
        ref_id.push('/');
        push_encoded_token(&mut ref_id, token);
    }
    ref_id
}

fn push_encoded_token(ref_id: &mut String, token: &str) {
    for &byte in token.as_bytes() {
        match byte {
            b'~' => ref_id.push_str("~0"),
            b'/' => ref_id.push_str("~1"),
            byte if is_fragment_byte_allowed(byte) => ref_id.push(char::from(byte)),
            byte => {
                ref_id.push('%');
                ref_id.push(char::from(UPPERCASE_HEX[usize::from(byte >> 4)]));
                ref_id.push(char::from(UPPERCASE_HEX[usize::from(byte & 0x0f)]));
            }
        }
    }
}

fn is_fragment_byte_allowed(byte: u8) -> bool {
    matches!(
        byte,
        b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'.'
            | b'_'
            | b'~'
            | b'!'
            | b'$'
            | b'&'
            | b'\''
            | b'('
            | b')'
            | b'*'
            | b'+'
            | b','
            | b';'
            | b'='
            | b':'
            | b'@'
            | b'/'
            | b'?'
    )
}

fn percent_decode(fragment: &str) -> Result<String, RefError> {
    let encoded = fragment.as_bytes();
    let mut decoded = Vec::with_capacity(encoded.len());
    let mut cursor = 0;

    while let Some(&byte) = encoded.get(cursor) {
        if byte != b'%' {
            decoded.push(byte);
            cursor += 1;
            continue;
        }

        let high = encoded
            .get(cursor + 1)
            .copied()
            .and_then(uppercase_hex_value)
            .ok_or_else(|| invalid("expected a complete uppercase percent escape"))?;
        let low = encoded
            .get(cursor + 2)
            .copied()
            .and_then(uppercase_hex_value)
            .ok_or_else(|| invalid("expected a complete uppercase percent escape"))?;
        decoded.push((high << 4) | low);
        cursor += 3;
    }

    String::from_utf8(decoded).map_err(|_| invalid("percent-decoded fragment is not valid UTF-8"))
}

fn uppercase_hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn decode_pointer_token(encoded: &str) -> Result<String, RefError> {
    let mut decoded = String::with_capacity(encoded.len());
    let mut characters = encoded.chars();

    while let Some(character) = characters.next() {
        if character != '~' {
            decoded.push(character);
            continue;
        }

        match characters.next() {
            Some('0') => decoded.push('~'),
            Some('1') => decoded.push('/'),
            _ => return Err(invalid("expected ~0 or ~1 JSON Pointer escape")),
        }
    }

    Ok(decoded)
}

fn is_canonical_array_index(token: &str) -> bool {
    if token == "0" {
        return true;
    }

    let bytes = token.as_bytes();
    matches!(bytes.first(), Some(b'1'..=b'9')) && bytes[1..].iter().all(u8::is_ascii_digit)
}

const fn invalid(reason: &'static str) -> RefError {
    RefError::Invalid { reason }
}

#[cfg(test)]
mod tests;
