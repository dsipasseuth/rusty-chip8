#[derive(Debug)]
pub enum EmulationError {
    UnknownOpcode(u16),
    UnknownInput,
    Quit,
}
