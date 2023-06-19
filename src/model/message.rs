use std::fmt;

#[derive(Debug)]
pub enum Message {
    Add(i32),
    Sub(i32),
    Mul(i32),
    Div(i32),
    Get(i32),
    Set(i32),
    Stop(i32, i32),
    Subscribe(usize),
    Unsubscribe(usize),
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Message::Add(n) => write!(f, "Add({})", n),
            Message::Sub(n) => write!(f, "Sub({})", n),
            Message::Mul(n) => write!(f, "Mul({})", n),
            Message::Div(n) => write!(f, "Div({})", n),
            Message::Get(n) => write!(f, "Get({})", n),
            Message::Set(n) => write!(f, "Set({})", n),
            Message::Stop(n, epoch) => write!(f, "Stop {} for {}", n, epoch),
            Message::Subscribe(pid) => write!(f, "Subscribe({})", pid),
            Message::Unsubscribe(pid) => write!(f, "Unsubscribe({})", pid),
        }
    }
}
