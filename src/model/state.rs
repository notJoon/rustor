use std::fmt;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ActorState {
    Active,
    Inactive,
}

impl fmt::Display for ActorState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ActorState::Active => write!(f, "Active"),
            ActorState::Inactive => write!(f, "Inactive"),
        }
    }
}
