use crate::datafile::Entry;
use dialoguer::{theme::ColorfulTheme, Select};
use std::path::{Path, PathBuf};

pub fn display_path(path: &Path, home: &Path) -> String {
    if let Ok(rest) = path.strip_prefix(home) {
        let rest_str = rest.to_string_lossy();
        if rest_str.is_empty() {
            "~".to_string()
        } else {
            format!("~/{}", rest_str)
        }
    } else {
        path.to_string_lossy().into_owned()
    }
}

pub fn pick(entries: &[&Entry], home: &Path) -> dialoguer::Result<Option<PathBuf>> {
    let items: Vec<String> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| format!("{}  {}", i + 1, display_path(&e.path, home)))
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact_opt()?;

    Ok(selection.map(|idx| entries[idx].path.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitutes_tilde_for_home_prefix() {
        let home = Path::new("/Users/x");
        let path = Path::new("/Users/x/Workspace/foo");
        assert_eq!(display_path(path, home), "~/Workspace/foo");
    }

    #[test]
    fn shows_path_verbatim_when_outside_home() {
        let home = Path::new("/Users/x");
        let path = Path::new("/etc/hosts");
        assert_eq!(display_path(path, home), "/etc/hosts");
    }

    #[test]
    fn home_itself_displays_as_tilde() {
        let home = Path::new("/Users/x");
        let path = Path::new("/Users/x");
        assert_eq!(display_path(path, home), "~");
    }
}
