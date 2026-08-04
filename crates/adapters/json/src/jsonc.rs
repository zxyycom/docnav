use crate::document::SourceRegion;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CommentKind {
    Line,
    Block,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CommentToken {
    pub(crate) kind: CommentKind,
    pub(crate) span: SourceRegion,
}

pub(crate) struct Scan {
    pub(crate) parse_view: String,
    pub(crate) comments: Vec<CommentToken>,
    pub(crate) comment_line_starts: Vec<usize>,
    pub(crate) commas: Vec<usize>,
    pub(crate) has_jsonc_syntax: bool,
    #[cfg(test)]
    pub(crate) scanned_bytes: usize,
}

#[derive(Debug)]
pub(crate) struct ScanError(pub(crate) &'static str);

pub(crate) fn scan(source: &str) -> Result<Scan, ScanError> {
    let bytes = source.as_bytes();
    let mut parse_view = bytes.to_vec();
    let mut comments = Vec::new();
    let mut comment_line_starts = Vec::new();
    let mut commas: Vec<usize> = Vec::new();
    let mut comma_preceded_by_value: Vec<bool> = Vec::new();
    let mut containers: Vec<u8> = Vec::new();
    let mut cursor = 0;
    let mut line_start = 0;
    let mut last_significant: Option<u8> = None;
    let mut has_jsonc_syntax = false;

    while cursor < bytes.len() {
        match bytes[cursor] {
            b'"' => {
                cursor = scan_string(bytes, cursor)?;
                last_significant = Some(b'"');
            }
            b'/' if bytes.get(cursor + 1) == Some(&b'/') => {
                let start = cursor;
                let comment_line_start = line_start;
                while cursor < bytes.len() && !matches!(bytes[cursor], b'\r' | b'\n') {
                    parse_view[cursor] = b' ';
                    cursor += 1;
                }
                comments.push(CommentToken {
                    kind: CommentKind::Line,
                    span: SourceRegion { start, end: cursor },
                });
                comment_line_starts.push(comment_line_start);
                has_jsonc_syntax = true;
            }
            b'/' if bytes.get(cursor + 1) == Some(&b'*') => {
                let start = cursor;
                let comment_line_start = line_start;
                parse_view[cursor] = b' ';
                parse_view[cursor + 1] = b' ';
                cursor += 2;
                let mut closed = false;
                while cursor < bytes.len() {
                    if bytes[cursor] == b'*' && bytes.get(cursor + 1) == Some(&b'/') {
                        parse_view[cursor] = b' ';
                        parse_view[cursor + 1] = b' ';
                        cursor += 2;
                        closed = true;
                        break;
                    }
                    match bytes[cursor] {
                        b'\r' if bytes.get(cursor + 1) == Some(&b'\n') => {
                            cursor += 2;
                            line_start = cursor;
                        }
                        b'\r' | b'\n' => {
                            cursor += 1;
                            line_start = cursor;
                        }
                        _ => {
                            parse_view[cursor] = b' ';
                            cursor += 1;
                        }
                    }
                }
                if !closed {
                    return Err(ScanError("unterminated JSONC block comment"));
                }
                comments.push(CommentToken {
                    kind: CommentKind::Block,
                    span: SourceRegion { start, end: cursor },
                });
                comment_line_starts.push(comment_line_start);
                has_jsonc_syntax = true;
            }
            b' ' | b'\t' => cursor += 1,
            b'\r' if bytes.get(cursor + 1) == Some(&b'\n') => {
                cursor += 2;
                line_start = cursor;
            }
            b'\r' | b'\n' => {
                cursor += 1;
                line_start = cursor;
            }
            opening @ (b'[' | b'{') => {
                containers.push(opening);
                last_significant = Some(opening);
                cursor += 1;
            }
            closing @ (b']' | b'}') => {
                let expected_opening = if closing == b']' { b'[' } else { b'{' };
                if containers.last() == Some(&expected_opening) {
                    containers.pop();
                    if last_significant == Some(b',') {
                        let &comma = commas
                            .last()
                            .ok_or(ScanError("missing trailing comma evidence"))?;
                        let preceded_by_value = *comma_preceded_by_value
                            .last()
                            .ok_or(ScanError("missing trailing comma predecessor evidence"))?;
                        if preceded_by_value {
                            parse_view[comma] = b' ';
                            has_jsonc_syntax = true;
                        }
                    }
                }
                last_significant = Some(closing);
                cursor += 1;
            }
            b',' => {
                commas.push(cursor);
                comma_preceded_by_value.push(!matches!(
                    last_significant,
                    None | Some(b'[' | b'{' | b',' | b':')
                ));
                last_significant = Some(b',');
                cursor += 1;
            }
            byte => {
                last_significant = Some(byte);
                cursor += 1;
            }
        }
    }

    Ok(Scan {
        parse_view: String::from_utf8(parse_view)
            .expect("replacing complete UTF-8 code-unit bytes with ASCII preserves UTF-8"),
        comments,
        comment_line_starts,
        commas,
        has_jsonc_syntax,
        #[cfg(test)]
        scanned_bytes: cursor,
    })
}

fn scan_string(bytes: &[u8], start: usize) -> Result<usize, ScanError> {
    let mut cursor = start + 1;
    while let Some(&byte) = bytes.get(cursor) {
        match byte {
            b'"' => return Ok(cursor + 1),
            b'\\' => {
                if bytes.get(cursor + 1).is_none() {
                    return Err(ScanError("unterminated JSON string escape"));
                }
                cursor += 2;
            }
            _ => cursor += 1,
        }
    }
    Err(ScanError("unterminated JSON string token"))
}
