use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub path: PathBuf,
    pub rank: f64,
    pub time: u64,
}

#[derive(Debug)]
pub enum LoadError {
    MissingHome,
    FileNotFound,
    Empty,
    Io(std::io::Error),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::MissingHome => write!(f, "could not resolve $HOME"),
            LoadError::FileNotFound => write!(f, "no zsh-z data found"),
            LoadError::Empty => write!(f, "no zsh-z data found"),
            LoadError::Io(e) => write!(f, "failed to read zsh-z data: {e}"),
        }
    }
}

pub fn load() -> Result<Vec<Entry>, LoadError> {
    let path = resolve_datafile_path()?;
    load_from(&path)
}

fn resolve_datafile_path() -> Result<PathBuf, LoadError> {
    if let Some(custom) = std::env::var_os("_Z_DATA") {
        if !custom.is_empty() {
            return Ok(PathBuf::from(custom));
        }
    }
    let home = dirs::home_dir().ok_or(LoadError::MissingHome)?;
    Ok(home.join(".z"))
}

fn load_from(path: &std::path::Path) -> Result<Vec<Entry>, LoadError> {
    let contents = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(LoadError::FileNotFound);
        }
        Err(e) => return Err(LoadError::Io(e)),
    };
    if contents.trim().is_empty() {
        return Err(LoadError::Empty);
    }
    Ok(contents.lines().filter_map(parse_line).collect())
}

fn parse_line(line: &str) -> Option<Entry> {
    // Format: path|rank|time. Path may contain `|`, so peel from the right.
    let (rest, time_str) = line.rsplit_once('|')?;
    let (path_str, rank_str) = rest.rsplit_once('|')?;
    if path_str.is_empty() {
        return None;
    }
    let rank: f64 = rank_str.parse().ok()?;
    let time: u64 = time_str.parse().ok()?;
    Some(Entry {
        path: PathBuf::from(path_str),
        rank,
        time,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parses_typical_line() {
        let entry = parse_line("/Users/me/Workspace/foo|10.5|1700000000").unwrap();
        assert_eq!(entry.path, PathBuf::from("/Users/me/Workspace/foo"));
        assert_eq!(entry.rank, 10.5);
        assert_eq!(entry.time, 1_700_000_000);
    }

    #[test]
    fn handles_pipe_in_path() {
        let entry = parse_line("/weird|dir/foo|1.0|1700000000").unwrap();
        assert_eq!(entry.path, PathBuf::from("/weird|dir/foo"));
        assert_eq!(entry.rank, 1.0);
        assert_eq!(entry.time, 1_700_000_000);
    }

    #[test]
    fn rejects_malformed_lines() {
        assert!(parse_line("").is_none());
        assert!(parse_line("nopipes").is_none());
        assert!(parse_line("only|onepipe").is_none());
        assert!(parse_line("/path|notanumber|1700000000").is_none());
        assert!(parse_line("/path|1.0|notanumber").is_none());
        assert!(parse_line("|1.0|1700000000").is_none()); // empty path
    }

    #[test]
    fn load_from_mixed_fixture_skips_invalid() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("z");
        let mut f = std::fs::File::create(&p).unwrap();
        writeln!(f, "/a|1.0|100").unwrap();
        writeln!(f, "garbage line").unwrap();
        writeln!(f, "/b|2.5|200").unwrap();
        writeln!(f).unwrap();
        let entries = load_from(&p).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].path, PathBuf::from("/a"));
        assert_eq!(entries[1].path, PathBuf::from("/b"));
    }

    #[test]
    fn load_from_returns_empty_for_zero_byte_file() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("z");
        std::fs::File::create(&p).unwrap();
        match load_from(&p) {
            Err(LoadError::Empty) => {}
            other => panic!("expected Empty, got {:?}", other),
        }
    }

    #[test]
    fn load_from_returns_file_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("does-not-exist");
        match load_from(&p) {
            Err(LoadError::FileNotFound) => {}
            other => panic!("expected FileNotFound, got {:?}", other),
        }
    }

    #[test]
    fn honors_z_data_env_var() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("custom-z");
        let mut f = std::fs::File::create(&p).unwrap();
        writeln!(f, "/x|1.0|100").unwrap();
        // Note: env vars are process-global; this test must not run in parallel
        // with another that touches _Z_DATA. Cargo test runs tests in a thread
        // pool, so we resolve via the helper and assert the resolved path here
        // rather than mutating env state across tests.
        unsafe {
            std::env::set_var("_Z_DATA", &p);
        }
        let resolved = resolve_datafile_path().unwrap();
        unsafe {
            std::env::remove_var("_Z_DATA");
        }
        assert_eq!(resolved, p);
    }
}
