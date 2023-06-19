use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum ActorError {
    TargetActorNotFound(String),
    TargetActorIsOffline(String),
    InvalidMessage(String),
    InvalidOperation(String),
    NotInSubscriberList(String, String),
    DividedByZero,
}

impl fmt::Display for ActorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ActorError::TargetActorNotFound(ref pid) => write!(f, "Target actor not found: {pid}"),
            ActorError::TargetActorIsOffline(ref pid) => write!(f, "Target actor is offline: {pid}"),
            ActorError::InvalidMessage(ref msg) => write!(f, "Invalid message: {msg}"),
            ActorError::InvalidOperation(ref op) => write!(f, "Invalid operation: {op}"),
            ActorError::NotInSubscriberList(ref pid1, ref pid2) => write!(f, "Actor {pid1} is not in the subscriber list of {pid2}"),
            ActorError::DividedByZero => write!(f, "Divided by zero"),
        }
    }
}

impl Error for ActorError {
    fn description(&self) -> &str {
        match *self {
            ActorError::TargetActorNotFound(_) => "Target actor not found",
            ActorError::TargetActorIsOffline(_) => "Target actor is offline",
            ActorError::InvalidMessage(_) => "Invalid message",
            ActorError::InvalidOperation(_) => "Invalid operation",
            ActorError::NotInSubscriberList(_, _) => "Actor is not in the subscriber list",
            ActorError::DividedByZero => "Divided by zero",
        }
    }
}