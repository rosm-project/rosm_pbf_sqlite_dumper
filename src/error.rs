use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct DumperError {
    msg: String,
    source: Box<dyn Error>,
}

impl fmt::Display for DumperError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, src: {}", self.msg, self.source)
    }
}

impl Error for DumperError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl DumperError {
    pub fn new(err: Box<dyn Error>, msg: String) -> Self {
        DumperError { msg, source: err }
    }
}
