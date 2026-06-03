mod datafile;
mod display;
mod rank;

use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

const TOP_N: usize = 5;

fn main() -> ExitCode {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => {
            eprintln!("zztop: could not resolve $HOME");
            return ExitCode::from(2);
        }
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let entries = match datafile::load() {
        Ok(e) => e,
        Err(datafile::LoadError::FileNotFound | datafile::LoadError::Empty) => {
            eprintln!("zztop: no zsh-z data found");
            return ExitCode::from(1);
        }
        Err(datafile::LoadError::MissingHome) => {
            eprintln!("zztop: could not resolve $HOME");
            return ExitCode::from(2);
        }
        Err(datafile::LoadError::Io(e)) => {
            eprintln!("zztop: failed to read zsh-z data: {e}");
            return ExitCode::from(2);
        }
    };

    let top = rank::top_n(&entries, now, TOP_N);
    if top.is_empty() {
        eprintln!("zztop: no existing paths in zsh-z data");
        return ExitCode::from(1);
    }

    match display::pick(&top, &home) {
        Ok(Some(path)) => {
            println!("{}", path.display());
            ExitCode::from(0)
        }
        Ok(None) => ExitCode::from(1),
        Err(e) => {
            eprintln!("zztop: {e}");
            ExitCode::from(2)
        }
    }
}
