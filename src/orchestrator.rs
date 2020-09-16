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
    pub fn handle_event(&self) {
        let build_res = self.build.run();
        match build_res {
            Ok(_) => {
                println!("Build succeeded");
            }
            Err(e) => {
                println!("ERROR: {:?}", e);
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
        let mut fail = MockRunner::default();
        fail.expect_run()
            .times(1)
            .returning(|| std::process::Command::new("true").output());

        return fail;
    }

    #[test]
    fn test_orchestrator_build_fails() {
        let build = fail();

        let test = not_called();
        let commit = not_called();
        let revert = not_called();

        let orc = Orchestrator {
            build: &build,
            test: &test,
            commit: &commit,
            revert: &revert,
        };

        orc.handle_event();
    }
}
