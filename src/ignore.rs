use crate::orchestrator::FileChangeEvent;
use ignore::gitignore::Gitignore;

pub struct Checker {
    root: std::path::PathBuf,
    gitignore: Option<Gitignore>,
    emacs_re: regex::Regex,
}

impl Checker {
    pub fn new(root: std::path::PathBuf, gitignore: Option<Gitignore>) -> Checker {
        let emacs_re = regex::Regex::new(r".*/*.#.*").unwrap();
        return Checker {
            root,
            gitignore,
            emacs_re,
        };
    }

    // make this a FileChangeEvent
    pub fn is_ignored(&mut self, event: FileChangeEvent) -> bool {
        let paths = event.paths;
        if paths.iter().all(|p| p.starts_with(self.root.join(".git"))) {
            return true;
        }
        if paths.iter().all(|p| self.is_editor_file(&p)) {
            return true;
        }
        match &self.gitignore {
            Some(gi) => {
                let is_dir = event.is_dir;
                return paths
                    .iter()
                    .all(|p| gi.matched_path_or_any_parents(p, is_dir).is_ignore());
            }
            None => {}
        }
        return false;
    }

    // emacs **/.#*
    // vim "**/4913", "**/*~", "**/.*.swp", "**/.*.swx", "**/.*.swo", "**/.*.swn"
    fn is_editor_file(&mut self, path: &std::path::PathBuf) -> bool {
        match path.extension() {
            Some(e) => {
                let s = e.to_str().unwrap();
                if s.starts_with("sw") {
                    return true;
                }
            }
            None => {}
        }

        return match path.to_str() {
            Some(p) => self.emacs_re.is_match(p),
            None => false,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;
    use tempdir;

    fn event_for_path(path: std::path::PathBuf) -> FileChangeEvent {
        return FileChangeEvent {
            paths: vec![path],
            is_dir: false,
        };
    }

    fn event_for_dir(path: std::path::PathBuf) -> FileChangeEvent {
        return FileChangeEvent {
            paths: vec![path],
            is_dir: true,
        };
    }

    #[test]
    fn test_ignore_no_gitignore() {
        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        let mut checker = Checker::new(tmp_dir.path().to_path_buf(), None);

        let event = event_for_path(tmp_dir.path().join("foo"));
        assert_eq!(checker.is_ignored(event), false);
    }

    #[test]
    fn test_gitignore_no_match() {
        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        let gi_path = &tmp_dir.path().join(".gitignore");

        let mut file = File::create(gi_path).unwrap();
        file.write_all(b"bar").unwrap();

        let (gi, _) = Gitignore::new(gi_path);

        let mut checker = Checker::new(tmp_dir.path().to_path_buf(), Some(gi));

        let path = tmp_dir.path().join("foo");
        let event = event_for_path(tmp_dir.path().join("foo"));

        let mut file = File::create(path.to_owned()).unwrap();
        file.write_all(b"foo").unwrap();

        assert_eq!(checker.is_ignored(event), false);
    }

    #[test]
    fn test_gitignore_match() {
        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        let gi_path = &tmp_dir.path().join(".gitignore");

        let mut file = File::create(gi_path).unwrap();
        file.write_all(b"bar").unwrap();

        let (gi, _) = Gitignore::new(gi_path);

        let mut checker = Checker::new(tmp_dir.path().to_path_buf(), Some(gi));

        let path = tmp_dir.path().join("bar");
        let event = event_for_path(tmp_dir.path().join("bar"));

        let mut file = File::create(path.to_owned()).unwrap();
        file.write_all(b"bar").unwrap();

        assert_eq!(checker.is_ignored(event), true);
    }

    #[test]
    fn test_ignores_git_dir() {
        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        let gi_path = &tmp_dir.path().join(".gitignore");
        let git_dir_path = &tmp_dir.path().join(".git");

        std::fs::create_dir(git_dir_path).unwrap();
        let mut file = File::create(gi_path).unwrap();
        file.write_all(b"bar").unwrap();

        let (gi, _) = Gitignore::new(gi_path);

        let mut checker = Checker::new(tmp_dir.path().to_path_buf(), Some(gi));

        let path = git_dir_path.join("some_file");
        let event = event_for_path(git_dir_path.join("some_file"));
        let mut file = File::create(path.to_owned()).unwrap();
        file.write_all(b"foo").unwrap();

        assert_eq!(checker.is_ignored(event), true);
    }

    #[test]
    fn test_gitignore_match_file_doesnt_exist() {
        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        let gi_path = &tmp_dir.path().join(".gitignore");

        let mut file = File::create(gi_path).unwrap();
        file.write_all(b"bar").unwrap();

        let (gi, _) = Gitignore::new(gi_path);

        let mut checker = Checker::new(tmp_dir.path().to_path_buf(), Some(gi));

        let path = tmp_dir.path().join("bar");
        let event = event_for_path(path);
        assert_eq!(checker.is_ignored(event), true);
    }

    #[test]
    fn test_gitignore_match_emacs_tmp_file() {
        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        let mut checker = Checker::new(tmp_dir.path().to_path_buf(), None);

        let path = tmp_dir.path().join(".#blah");
        let event = event_for_path(path);
        assert_eq!(checker.is_ignored(event), true);
    }

    #[test]
    fn test_gitignore_match_vim_tmp_file() {
        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        let mut checker = Checker::new(tmp_dir.path().to_path_buf(), None);

        let path = tmp_dir.path().join(".something.swp");
        let event = event_for_path(path);
        assert_eq!(checker.is_ignored(event), true);
    }

    #[test]
    fn test_gitignore_match_target_directory_glob() {
        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        let gi_path = &tmp_dir.path().join(".gitignore");

        let mut file = File::create(gi_path).unwrap();
        file.write_all(b"target/**").unwrap();

        let (gi, err) = Gitignore::new(gi_path);
        match err {
            Some(_e) => println!("Failed to create gitignore"),
            None => {}
        }

        let mut checker = Checker::new(tmp_dir.path().to_path_buf(), Some(gi));

        let base_path = tmp_dir.path().join("target").join("debug");
        let path = base_path.join("tcr.d");
        let event = event_for_path(base_path.join("tcr.d"));

        std::fs::create_dir_all(base_path).unwrap();
        let mut file = File::create(path.to_owned()).unwrap();
        file.write_all(b"hello world").unwrap();

        assert_eq!(checker.is_ignored(event), true);
    }

    #[test]
    fn test_gitignore_match_target_directory() {
        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        let gi_path = &tmp_dir.path().join(".gitignore");

        let mut file = File::create(gi_path).unwrap();
        file.write_all(b"target").unwrap();

        let (gi, err) = Gitignore::new(gi_path);
        match err {
            Some(_e) => println!("Failed to create gitignore"),
            None => {}
        }

        let mut checker = Checker::new(tmp_dir.path().to_path_buf(), Some(gi));

        let base_path = tmp_dir.path().join("target").join("debug");
        let path = base_path.join("tcr.d");
        let event = event_for_path(base_path.join("tcr.d"));

        std::fs::create_dir_all(base_path).unwrap();
        let mut file = File::create(path.to_owned()).unwrap();
        file.write_all(b"hello world").unwrap();

        assert_eq!(checker.is_ignored(event), true);
    }
}
