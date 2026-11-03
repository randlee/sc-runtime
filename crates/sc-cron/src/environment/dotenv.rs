//! Bounded, non-evaluating dotenv parsing and canonical authorization.
use std::io::Read;

use super::resolve::EnvironmentFailure;
use super::*;

fn file_invalid() -> EnvironmentFailure {
    EnvironmentFailure::FileInvalid
}

fn horizontal(s: &str) -> &str {
    s.trim_matches([' ', '\t'])
}

/// Shell expansion is rejected even inside quotes: this format never evaluates.
fn value(text: &str) -> Result<String, EnvironmentFailure> {
    let text = text.trim_start_matches([' ', '\t']);
    let mut out = String::new();
    if let Some(quote) = text.chars().next().filter(|c| matches!(c, '\'' | '"')) {
        let mut chars = text[1..].chars();
        let mut closed = false;
        while let Some(c) = chars.next() {
            if c == quote {
                closed = true;
                break;
            }
            if c == '\\' && quote == '"' {
                out.push(match chars.next() {
                    Some('\\') => '\\',
                    Some('"') => '"',
                    Some('n') => '\n',
                    Some('r') => '\r',
                    Some('t') => '\t',
                    _ => return Err(file_invalid()),
                });
            } else {
                out.push(c);
            }
        }
        if !closed {
            return Err(file_invalid());
        }
        let rest = chars.as_str().trim_start_matches([' ', '\t']);
        if !rest.is_empty() && !rest.starts_with('#') {
            return Err(file_invalid());
        }
    } else {
        let body = text.split_once('#').map_or(text, |(body, _)| body);
        if body.contains(['\'', '"']) {
            return Err(file_invalid());
        }
        out.push_str(horizontal(body));
    }
    if shell_expansion(&out) || !text_valid(&out, MAX_ENV_VALUE_BYTES) {
        return Err(file_invalid());
    }
    Ok(out)
}

fn shell_expansion(value: &str) -> bool {
    value.contains('`')
        || value.as_bytes().windows(2).any(|pair| {
            pair[0] == b'$'
                && (pair[1].is_ascii_alphabetic() || matches!(pair[1], b'_' | b'{' | b'('))
        })
}

pub(super) fn parse(input: &[u8]) -> Result<BTreeMap<EnvName, String>, EnvironmentFailure> {
    let text = std::str::from_utf8(input).map_err(|_| file_invalid())?;
    if text.contains('\0') {
        return Err(file_invalid());
    }
    let mut entries = BTreeMap::new();
    for line in text.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.contains('\r') {
            return Err(file_invalid());
        }
        let mut line = horizontal(line);
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line
            .strip_prefix("export")
            .filter(|rest| rest.starts_with([' ', '\t']))
        {
            line = rest.trim_start_matches([' ', '\t']);
        }
        let (name, raw) = line.split_once('=').ok_or_else(file_invalid)?;
        let name = EnvName::new(horizontal(name)).map_err(|_| file_invalid())?;
        if entries
            .keys()
            .any(|n: &EnvName| n.0.eq_ignore_ascii_case(&name.0))
        {
            return Err(file_invalid());
        }
        entries.insert(name, value(raw)?);
        if entries.len() > MAX_ENV_ENTRIES {
            return Err(EnvironmentFailure::SnapshotLimit);
        }
        super::resolve::check_block(entries.iter().map(|(n, v)| (n.as_str(), v.as_str())))?;
    }
    Ok(entries)
}

/// Canonicalizes the nearest existing ancestor, retaining a missing suffix.
fn expected(path: &Path) -> Result<PathBuf, EnvironmentFailure> {
    let mut ancestor = path;
    let mut suffix = Vec::new();
    loop {
        match std::fs::canonicalize(ancestor) {
            Ok(mut canonical) => {
                for part in suffix.iter().rev() {
                    canonical.push(part);
                }
                return Ok(canonical);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                // A dangling symlink is not an absent lexical suffix: do not
                // classify its unresolved target as an authorized missing file.
                if std::fs::symlink_metadata(ancestor).is_ok() {
                    return Err(EnvironmentFailure::FileDenied);
                }
                suffix.push(
                    ancestor
                        .file_name()
                        .ok_or(EnvironmentFailure::FileDenied)?
                        .to_owned(),
                );
                ancestor = ancestor.parent().ok_or(EnvironmentFailure::FileDenied)?;
            }
            Err(_) => return Err(EnvironmentFailure::FileDenied),
        }
    }
}

pub(super) fn read(
    source: &DotenvSource,
    allowed_files: &BTreeSet<PathBuf>,
    allowed_roots: &BTreeSet<PathBuf>,
) -> Result<BTreeMap<EnvName, String>, EnvironmentFailure> {
    let canonical = expected(source.path())?;
    let files = allowed_files
        .iter()
        .map(|p| expected(p))
        .collect::<Result<Vec<_>, _>>()?;
    let roots = allowed_roots
        .iter()
        .map(|p| expected(p))
        .collect::<Result<Vec<_>, _>>()?;
    if !files.iter().any(|p| p == &canonical)
        && !roots
            .iter()
            .any(|p| canonical != *p && canonical.starts_with(p))
    {
        return Err(EnvironmentFailure::FileDenied);
    }
    // Authorization precedes absence classification. Reject special targets
    // before open: a FIFO without a writer would otherwise strand the blocking
    // task and runtime teardown. Authorized roots follow the administrator trust
    // model; the opened-handle checks below still guard the actual read.
    match std::fs::metadata(&canonical) {
        Ok(metadata) if metadata.is_file() => {}
        Ok(_) => return Err(file_invalid()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return if source.required {
                Err(EnvironmentFailure::FileMissing)
            } else {
                Ok(BTreeMap::new())
            };
        }
        Err(_) => return Err(file_invalid()),
    }
    let mut file = match std::fs::File::open(&canonical) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return if source.required {
                Err(EnvironmentFailure::FileMissing)
            } else {
                Ok(BTreeMap::new())
            };
        }
        Err(_) => return Err(file_invalid()),
    };
    let metadata = file.metadata().map_err(|_| file_invalid())?;
    if !metadata.is_file() || metadata.len() > MAX_ENV_FILE_BYTES as u64 {
        return Err(file_invalid());
    }
    let mut input = Vec::new();
    file.by_ref()
        .take(MAX_ENV_FILE_BYTES as u64 + 1)
        .read_to_end(&mut input)
        .map_err(|_| file_invalid())?;
    if input.len() > MAX_ENV_FILE_BYTES || !file.metadata().map_err(|_| file_invalid())?.is_file() {
        return Err(file_invalid());
    }
    parse(&input)
}
