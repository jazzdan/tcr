pub trait Orchestrator {
    fn handle_event(&self, event: notify::Event);
}

pub struct TestOrchestrator {}

impl Orchestrator for TestOrchestrator {
    fn handle_event(&self, event: notify::Event) {
        println!("{:?}", event);
    }
}
