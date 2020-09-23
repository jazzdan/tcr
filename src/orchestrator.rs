use std::io::{self};
use std::io::{Error, ErrorKind};

#[mockall::automock]
pub trait Runner {
    fn run(&self) -> io::Result<std::process::Output>;
}

pub struct Orchestrator<'a> {
    build: &'a dyn Runner,
    test: &'a dyn Runner,
    commit: &'a dyn Runner,
    revert: &'a dyn Runner,
}

fn cmd_failed(
    output: std::result::Result<std::process::Output, std::io::Error>,
) -> Option<std::io::Error> {
    match output {
        Ok(res) => {
            if !res.status.success() {
                return Some(Error::new(
                    ErrorKind::Other,
                    "cmd returned non-zero exit code",
                ));
            }
            return None;
        }
        Err(e) => {
            return Some(e);
        }
    }
}

impl Orchestrator<'_> {
    pub fn new<'a>(
        build: &'a dyn Runner,
        test: &'a dyn Runner,
        commit: &'a dyn Runner,
        revert: &'a dyn Runner,
    ) -> Orchestrator<'a> {
        return Orchestrator {
            build: build,
            test: test,
            commit: commit,
            revert: revert,
        };
    }
    // TODO(dmiller): in the future this should take a notify event, or a list of changed paths or something
    pub fn handle_event(&self) -> std::result::Result<(), std::io::Error> {
        let build = self.build.run();
        match cmd_failed(build) {
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
        match cmd_failed(test) {
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

    fn run_revert(&self) -> io::Result<std::process::Output> {
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

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
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
        let build = fail();

        let test = not_called();
        let commit = not_called();
        let revert = called();

        let orc = Orchestrator {
            build: &build,
            test: &test,
            commit: &commit,
            revert: &revert,
        };

        orc.handle_event().expect("This shouldn't error");
    }

    #[test]
    fn test_orchestrator_build_succeeds_test_fails() {
        let build = succeed();
        let test = fail();

        let commit = not_called();
        let revert = called();

        let orc = Orchestrator {
            build: &build,
            test: &test,
            commit: &commit,
            revert: &revert,
        };

        orc.handle_event().expect("This shouldn't error");
    }
}
