// NOTE(dmiller): I'm confused why I have to do this
#[path = "runner.rs"] mod runner;

pub struct Orchestrator<'a> {
    build: &'a dyn runner::Runner,
}

impl Orchestrator<'_> {
    pub fn handle_event(&self, event: notify::Event) {
        println!("{:?}", event);
    }
}
