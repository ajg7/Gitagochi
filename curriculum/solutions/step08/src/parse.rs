//! Parsing log lines into commits.

use std::fmt;

/// The `i`-th `|`-separated field of `line`, or "" if missing.
pub fn field(line: &str, i: usize) -> &str {
    line.split('|').nth(i).unwrap_or("")
}

#[derive(Debug, Clone, PartialEq)]
pub struct Commit {
    pub hash: String,
    pub author: String,
    pub time: i64,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Feature,
    Fix,
    Docs,
    Other,
}

impl Commit {
    /// The kind of change, from the summary's prefix.
    pub fn kind(&self) -> Kind {
        match self.summary.as_str() {
            s if s.starts_with("feat") => Kind::Feature,
            s if s.starts_with("fix") => Kind::Fix,
            s if s.starts_with("docs") => Kind::Docs,
            _ => Kind::Other,
        }
    }
}

/// Parse `hash|author|time|summary`; None if a field is missing or bad.
pub fn parse_commit(line: &str) -> Option<Commit> {
    let mut parts = line.splitn(4, '|');
    let (Some(hash), Some(author), Some(time), Some(summary)) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return None;
    };
    Some(Commit {
        hash: hash.to_string(),
        author: author.to_string(),
        time: time.parse().ok()?,
        summary: summary.to_string(),
    })
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    MissingField(usize),
    BadTime(usize),
}

/// Parse line number `n` (1-based) of a log.
pub fn parse_line(line: &str, n: usize) -> Result<Commit, ParseError> {
    let mut parts = line.splitn(4, '|');
    let (Some(hash), Some(author), Some(time), Some(summary)) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(ParseError::MissingField(n));
    };
    let time = time.parse::<i64>().map_err(|_| ParseError::BadTime(n))?;
    Ok(Commit {
        hash: hash.to_string(),
        author: author.to_string(),
        time,
        summary: summary.to_string(),
    })
}

/// Parse every non-blank line, stopping at the first error.
pub fn parse_log(log: &str) -> Result<Vec<Commit>, ParseError> {
    let mut out = Vec::new();
    for (i, line) in log.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        out.push(parse_line(line, i + 1)?);
    }
    Ok(out)
}

impl fmt::Display for Commit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}: {}", self.hash, self.author, self.summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields() {
        assert_eq!(field("a|b", 1), "b");
        assert_eq!(parse_commit("a|b|1|c").map(|c| c.time), Some(1));
    }
}
