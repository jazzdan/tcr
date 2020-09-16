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
    pub fn handle_event(&self, event: notify::Event) {
        println!("{:?}", event);
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
    fn mytest() {
        let mut mock = MockRunner::default();
        mock.expect_run().returning(|| std::process::Command::new("true").output());
        mock.run().expect("not to fail");
    }
}
