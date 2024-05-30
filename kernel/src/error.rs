#[derive(Debug, Clone, Copy, PartialEq)]
enum Code {
    Success,
    Full,
    Empty,
    LastOfCode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Error {
    code: Code,
}

#[allow(dead_code)]
impl Error {
    const fn new(code: Code) -> Self {
        Error { code }
    }

    fn name(&self) -> &str {
        match self.code {
            Code::Success => "Success",
            Code::Full => "Full",
            Code::Empty => "Empty",
            _ => "Unknown",
        }
    }

    fn code(&self) -> Code {
        self.code
    }
}

impl Error {
    pub const SUCCESS: Error = Error::new(Code::Success);
    pub const FULL: Error = Error::new(Code::Full);
    pub const EMPTY: Error = Error::new(Code::Empty);
    pub const LAST_OF_CODE: Error = Error::new(Code::LastOfCode);
}

