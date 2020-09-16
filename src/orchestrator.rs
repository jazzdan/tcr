// NOTE(dmiller): I'm confused why I have to do this
#[path = "runner.rs"] mod runner;

pub trait Orchestrator: Sized {
    fn handle_event(&self, event: notify::Event);
}

pub struct TestOrchestrator {
    // build: dyn runner::Runner,
}

impl Orchestrator for TestOrchestrator {
    fn handle_event(&self, event: notify::Event) {
        println!("{:?}", event);
    }
}
