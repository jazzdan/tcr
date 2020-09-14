mod runner {
    pub trait Runner {
        fn run(&self) -> Result<std::process::Output>
    }
}