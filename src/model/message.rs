use std::fmt;

#[derive(Debug)]
pub enum Message {
    Increment(i32),
    Decrement(i32),
    Multiply(i32),
    Divide(i32),
    Exit,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Message::Increment(n) => write!(f, "Increment({})", n),
            Message::Decrement(n) => write!(f, "Decrement({})", n),
            Message::Multiply(n) => write!(f, "Multiply({})", n),
            Message::Divide(n) => write!(f, "Divide({})", n),
            Message::Exit => write!(f, "Exit"),
        }
    }
}
