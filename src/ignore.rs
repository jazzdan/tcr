use gitignore;

pub struct Checker<'a> {
    root: std::path::PathBuf,
    // TODO maybe replace this with the gitignore module in the ignore crate
    gitignore: Option<gitignore::File<'a>>,
}

// TODO
fn is_editor_file(path: std::path::PathBuf) -> bool {
    return false;
}

// TODO this isn't working for the current gitignore file. Files in ./target are triggering changes
impl Checker<'_> {
    pub fn new<'a>(
        root: std::path::PathBuf,
        gitignore: Option<gitignore::File<'a>>,
    ) -> Checker<'a> {
        return Checker { root, gitignore };
    }

    pub fn is_ignored(&mut self, path: std::path::PathBuf) -> bool {
        if path.starts_with(self.root.join(".git")) {
            return true;
        }
        match &self.gitignore {
            Some(gi) => {
                // NOTE: this behaves strangely when files don't exist
                match gi.is_excluded(&path) {
                    Ok(m) => {
                        if m {
                            return true;
                        }
                    }
                    Err(_) => {}
                }
            }
            None => {}
        }
        return false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;
    use tempdir;

    #[test]
    fn test_ignore_no_gitignore() {
        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        let mut checker = Checker::new(tmp_dir.path().to_path_buf(), None);

        let path = tmp_dir.path().join("foo");
        assert_eq!(checker.is_ignored(path.to_path_buf()), false);
    }

    #[test]
    fn test_gitignore_no_match() {
        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        let gi_path = &tmp_dir.path().join(".gitignore");

        let mut file = File::create(gi_path).unwrap();
        file.write_all(b"bar").unwrap();

        let gi = gitignore::File::new(gi_path).unwrap();

        let mut checker = Checker::new(tmp_dir.path().to_path_buf(), Some(gi));

        let path = tmp_dir.path().join("foo");

        let mut file = File::create(path.to_owned()).unwrap();
        file.write_all(b"foo").unwrap();

        assert_eq!(checker.is_ignored(path.to_path_buf()), false);
    }

    #[test]
    fn test_gitignore_match() {
        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        let gi_path = &tmp_dir.path().join(".gitignore");

        let mut file = File::create(gi_path).unwrap();
        file.write_all(b"bar").unwrap();

        let gi = gitignore::File::new(gi_path).unwrap();

        let mut checker = Checker::new(tmp_dir.path().to_path_buf(), Some(gi));

        let path = tmp_dir.path().join("bar");

        let mut file = File::create(path.to_owned()).unwrap();
        file.write_all(b"bar").unwrap();

        assert_eq!(checker.is_ignored(path.to_path_buf()), true);
    }

    #[test]
    fn test_ignores_git_dir() {
        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        let gi_path = &tmp_dir.path().join(".gitignore");
        let git_dir_path = &tmp_dir.path().join(".git");

        std::fs::create_dir(git_dir_path).unwrap();
        let mut file = File::create(gi_path).unwrap();
        file.write_all(b"bar").unwrap();

        let gi = gitignore::File::new(gi_path).unwrap();

        let mut checker = Checker::new(tmp_dir.path().to_path_buf(), Some(gi));

        let path = git_dir_path.join("some_file");
        let mut file = File::create(path.to_owned()).unwrap();
        file.write_all(b"foo").unwrap();

        assert_eq!(checker.is_ignored(path.to_path_buf()), true);
    }
}
