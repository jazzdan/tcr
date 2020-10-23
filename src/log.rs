pub struct VerboseLogger {
    verbose: bool,
}

impl VerboseLogger {
    pub fn new(verbose: bool) -> VerboseLogger {
        return VerboseLogger { verbose };
    }
    pub fn log(&self, s: impl Into<String>) {
        if self.verbose {
            println!("{}", s.into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbose_logger() {
        let l = VerboseLogger { verbose: true };
        l.log("Hello world");
        l.log(String::from("foo"));
    }
}
