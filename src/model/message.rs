use std::fmt;

#[derive(Debug, Clone)]
pub enum Message {
    Increment(i32),
    Decrement(i32),
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Message::Increment(n) => write!(f, "Increment({})", n),
            Message::Decrement(n) => write!(f, "Decrement({})", n),
        }
    }
}
