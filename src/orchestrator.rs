use std::io::{self};

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

impl Orchestrator<'_> {
    // TODO(dmiller): in the future this should take a notify event, or a list of changed paths or something
    // TODO(dmiller): this thing should probably return a result
    pub fn handle_event(&self) {
        let build_res = self.build.run();
        match build_res {
            Ok(res) => {
                if !res.status.success() {
                    println!("Build failed with non-zero exit code");
                    self.revert();
                }
            }
            Err(e) => {
                println!("Build failed: {:?}", e);
                self.revert(); 
            }
        }
    }

    fn revert(&self) {
        let revert_res = self.revert.run();
        match revert_res {
            Ok(_) => (),
            Err(e) => println!("Error reverting: {:?}", e),
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

        orc.handle_event();
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

        orc.handle_event();
    }
}
