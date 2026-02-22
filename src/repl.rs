/// REPL placeholder.
#[derive(Debug, Default)]
pub struct Repl;

impl Repl {
    pub fn new() -> Self {
        Self
    }

    pub fn run_stdio(&mut self) -> i32 {
        // TODO(step-8): implement stateful REPL with multiline support and meta commands.
        0
    }
}
