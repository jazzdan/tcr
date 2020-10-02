use gitignore;

pub struct Checker<'a> {
    gitignore: Option<gitignore::File<'a>>,
}

impl Checker<'_> {
    pub fn new<'a>(root: std::path::PathBuf, gitignore: Option<gitignore::File<'a>>) -> Checker<'a> {
        return Checker{
            gitignore: gitignore,
        };
    }
     

    pub fn is_ignored(&mut self, path: std::path::PathBuf) -> bool {
        match &self.gitignore {
            Some(gi) => {
                match gi.is_excluded(&path) {
                    Ok(m) => {
                        if m {
                            return true;
                        }
                    },
                    Err(_) => {},
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
    use tempdir;

    #[test]
    fn test_ignore_no_gitignore() {
        let tmp_dir = tempdir::TempDir::new("example").unwrap();
        let mut checker = Checker::new(tmp_dir.path().to_path_buf(), None);

        let path = std::path::Path::new("foo");
        assert_eq!(checker.is_ignored(path.to_path_buf()), false);
    }
}
