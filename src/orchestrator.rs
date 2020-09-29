use std::io::{self};
use std::io::{Error, ErrorKind};

pub struct FileChangeEvent {
    pub paths: std::vec::Vec<std::path::PathBuf>,
}

#[mockall::automock]
pub trait Runner {
    fn run(&mut self) -> io::Result<std::process::Output>;
}

pub struct Orchestrator<'a> {
    root: std::path::PathBuf,
    build: &'a mut dyn Runner,
    test: &'a mut dyn Runner,
    commit: &'a mut dyn Runner,
    revert: &'a mut dyn Runner,
}

fn handle_output(
    output: std::result::Result<std::process::Output, std::io::Error>,
) -> Option<std::io::Error> {
    match output {
        Ok(res) => {
            println!("{:?}", std::str::from_utf8(&res.stdout));
            println!("{:?}", std::str::from_utf8(&res.stderr));

            if !res.status.success() {
                return Some(Error::new(
                    ErrorKind::Other,
                    "cmd returned non-zero exit code",
                ));
            }
            // TODO(dmiller): print output
            return None;
        }
        Err(e) => {
            return Some(e);
        }
    }
}

impl Orchestrator<'_> {
    pub fn new<'a>(
        build: &'a mut dyn Runner,
        test: &'a mut dyn Runner,
        commit: &'a mut dyn Runner,
        revert: &'a mut dyn Runner,
    ) -> Orchestrator<'a> {
        let root = std::env::current_dir().unwrap();
        return Orchestrator {
            root: root,
            build: build,
            test: test,
            commit: commit,
            revert: revert,
        };
    }
    pub fn handle_event(
        &mut self,
        event: FileChangeEvent,
    ) -> std::result::Result<(), std::io::Error> {
        if event
            .paths
            .iter()
            .all(|p| p.starts_with(self.root.join(".git")))
        {
            return Ok(());
        }
        let build = self.build.run();
        match handle_output(build) {
            Some(err) => {
                println!("Build failed: {:?}", err);
                let res = self.run_revert();
                if res.is_err() {
                    let err = res.err();
                    return Err(err.unwrap());
                }
                return Ok(());
            }
            None => {}
        }
        let test = self.test.run();
        match handle_output(test) {
            Some(err) => {
                println!("Test failed: {:?}", err);
                let res = self.run_revert();
                if res.is_err() {
                    let err = res.err();
                    return Err(err.unwrap());
                }
                return Ok(());
            }
            None => {}
        }

        let commit = self.commit.run();
        match commit {
            Ok(_res) => {}
            Err(e) => {
                return Err(e);
            }
        }
        return Ok(());
    }

    fn run_revert(&mut self) -> io::Result<std::process::Output> {
        let revert_res = self.revert.run();
        match revert_res {
            Ok(out) => Ok(out),
            Err(e) => {
                println!("Error reverting: {:?}", e);
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> std::path::PathBuf {
        return std::path::PathBuf::from(r"/home/stuff");
    }

    fn ok_event() -> FileChangeEvent {
        return FileChangeEvent {
            paths: vec![std::path::PathBuf::from(r"/home/stuff/hi")],
        };
    }

    #[test]
    fn test_mock_runner() {
        let mut mock = MockRunner::default();
        mock.expect_run()
            .returning(|| std::process::Command::new("true").output());
        mock.run().expect("not to fail");
    }

    fn fail() -> MockRunner {
        let mut fail = MockRunner::default();
        fail.expect_run()
            .times(1)
            .returning(|| std::process::Command::new("false").output());

        return fail;
    }

    fn not_called() -> MockRunner {
        let mut fail = MockRunner::default();
        fail.expect_run().never();

        return fail;
    }

    fn succeed() -> MockRunner {
        let mut success = MockRunner::default();
        success
            .expect_run()
            .times(1)
            .returning(|| std::process::Command::new("true").output());

        return success;
    }

    fn called() -> MockRunner {
        return succeed();
    }

    #[test]
    fn test_orchestrator_build_fails() {
        let mut build = fail();

        let mut test = not_called();
        let mut commit = not_called();
        let mut revert = called();

        let mut orc = Orchestrator {
            root: root(),
            build: &mut build,
            test: &mut test,
            commit: &mut commit,
            revert: &mut revert,
        };

        orc.handle_event(ok_event()).expect("This shouldn't error");
    }

    #[test]
    fn test_orchestrator_build_succeeds_test_fails() {
        let mut build = succeed();
        let mut test = fail();

        let mut commit = not_called();
        let mut revert = called();

        let mut orc = Orchestrator {
            root: root(),
            build: &mut build,
            test: &mut test,
            commit: &mut commit,
            revert: &mut revert,
        };

        orc.handle_event(ok_event()).expect("This shouldn't error");
    }

    #[test]
    fn ignore_git_directory() {
        let mut build = not_called();
        let mut test = not_called();
        let mut commit = not_called();
        let mut revert = not_called();

        let mut orc = Orchestrator {
            root: root(),
            build: &mut build,
            test: &mut test,
            commit: &mut commit,
            revert: &mut revert,
        };

        let event = FileChangeEvent {
            paths: vec![std::path::PathBuf::from(r"/home/stuff/.git")],
        };

        orc.handle_event(event).expect("This shouldn't error");
    }
}
