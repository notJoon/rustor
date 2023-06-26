use std::{
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Condvar, Mutex, RwLock,
    },
};

use super::{errors::ActorError, message::Message, state::ActorState};

static ACTOR_ID: AtomicUsize = AtomicUsize::new(0);
static MAILBOX_CAPACITY: usize = 10;
static INITIAL_VALUE: i32 = 0;

/// Actor pool containing all actors and a thread pool
#[derive(Debug)]
pub struct ActorPool {
    pub actor_list: Mutex<HashMap<usize, Arc<Actor>>>,
}

impl ActorPool {
    pub fn new() -> Self {
        Default::default()
    }

    /// Create a new actor and add it to the actor list
    pub fn create_actor(&self) -> usize {
        let actor = Actor::new();
        let id = actor.id;
        let actor_clone = Arc::clone(&actor);

        // each `Actor` runs on an independent thread when it is created, 
        // and when it receives a message, it consumes and processes the message in its own mailbox (via `Actor::execute_messages`).
        std::thread::spawn(move || {
            actor_clone.execute_messages();
        });

        let mut actor_list = self.actor_list.lock().unwrap();
        actor_list.insert(id, actor);

        id
    }

    pub fn get_actor_state(&self, actor_id: usize) -> Result<ActorState, ActorError> {
        let actor = self.get_actor_info(actor_id)?;

        actor.get_state()
    }

    // TODO: Move this to a Actor method
    pub fn update_actor_state(&mut self, actor_id: usize) -> Result<(), ActorError> {
        let actor = self.get_actor_info(actor_id)?;
        let mut state = actor.state.write().unwrap();

        // Change the current state to the opposite state
        match state.to_owned() {
            ActorState::Active => *state = ActorState::Inactive,
            ActorState::Inactive => *state = ActorState::Active,
        }

        Ok(())
    }

    pub fn get_actor_value(&self, actor_id: usize) -> Result<i32, ActorError> {
        let actor = self.get_actor_info(actor_id)?;

        actor.get_value()
    }

    pub fn get_actor_subscribers(&self, actor_id: usize) -> Result<Vec<usize>, ActorError> {
        let actor = self.get_actor_info(actor_id)?;

        Ok(actor.get_subscribers())
    }

    pub fn subscribe(
        &self,
        target_actor_id: usize,
        subscriber_actor_ids: Vec<usize>,
    ) -> Result<Arc<Actor>, ActorError> {
        let target_actor = self.get_actor_info(target_actor_id).unwrap();

        // Add subscribers to the target actor
        for subscriber_actor_id in subscriber_actor_ids {
            let subscriber_actor = self.get_actor_info(subscriber_actor_id).unwrap();
            target_actor.add_subscriber(subscriber_actor).unwrap();
        }

        Ok(target_actor)
    }

    pub fn get_actor_info(&self, actor_id: usize) -> Result<Arc<Actor>, ActorError> {
        let actor_list = self.actor_list.lock().unwrap();
        let actor = actor_list
            .get(&actor_id)
            .ok_or(ActorError::TargetActorNotFound(actor_id.to_string()))?;

        Ok(actor.to_owned())
    }

    /// `ActorPool::message_loop` method is used to send a message to a specific actor.
    pub fn message_loop(&self, actor_id: usize, message: Message) -> Result<(), ActorError> {
        let actor = self.get_actor_info(actor_id)?;

        actor.send_message(message)
    }
}

impl Default for ActorPool {
    fn default() -> Self {
        ActorPool {
            actor_list: Mutex::new(HashMap::new()),
        }
    }
}

#[derive(Debug)]
pub struct Actor {
    pub id: usize,
    pub state: RwLock<ActorState>,
    pub value: RwLock<i32>,
    pub subs: RwLock<HashMap<usize, Arc<Actor>>>,
    pub mailbox: Mutex<VecDeque<Message>>,
    pub condvar: Condvar,
}

impl Actor {
    /// Create a new actor with unique ID
    fn new() -> Arc<Self> {
        const INC: usize = 1;
        let id = ACTOR_ID.fetch_add(INC, Ordering::SeqCst);

        let actor = Actor {
            id,
            state: RwLock::new(ActorState::Active),
            value: RwLock::new(INITIAL_VALUE),
            subs: RwLock::new(HashMap::new()),
            mailbox: Mutex::new(VecDeque::with_capacity(MAILBOX_CAPACITY)),
            condvar: Condvar::new(),
        };

        Arc::new(actor)
    }

    /// Add a message to the actor's mailbox
    pub fn send_message(&self, message: Message) -> Result<(), ActorError> {
        if self.state.read().unwrap().to_owned() == ActorState::Inactive {
            return Err(ActorError::TargetActorIsOffline(self.id.to_string()));
        }

        // Check if the mailbox is full
        let mut mailbox = self.mailbox.lock().unwrap();

        if mailbox.len() >= mailbox.capacity() {
            return Err(ActorError::MailboxOverflow(self.id.to_string()));
        }

        mailbox.push_back(message.clone());

        self.propagate_message(message)?;

        // Send a notification via `Condvar` whenever a message is added to the `mailbox`.
        // Each time a message is added, the `execute_messages` (created via `ActorPool::create_actor`) will be woken up and process the message.
        self.condvar.notify_all();

        Ok(())
    }

    fn propagate_message(&self, message: Message) -> Result<(), ActorError> {
        let subs = self.subs.read().unwrap();

        for (_, actor) in subs.iter() {
            actor.send_message(message.clone())?;
        }

        Ok(())
    }

    /// The `execute_messages` method performs an infinite loop,
    /// and when the `mailbox` is empty, `Condvar`(in here, `self.condvar`) waits for another message.
    ///
    /// When a message is added to the mailbox, `Condvar` is notified and processes the message.
    /// This allows each actor to continuously process messages in their own thread.
    fn execute_messages(&self) {
        loop {
            let mut mailbox = self.mailbox.lock().unwrap();

            // When the mailbox is empty, the `condvar` will wait for a notification
            if mailbox.is_empty() {
                mailbox = self.condvar.wait(mailbox).unwrap();
            }

            // Consume messages in the mailbox
            while let Some(msg) = mailbox.pop_front() {
                self.handle_message(msg).unwrap();
            }
        }
    }

    /// Handles a message by matching its type and calling the appropriate handler
    pub fn handle_message(&self, message: Message) -> Result<(), ActorError> {
        let result = match message {
            Message::Increment(n) => self.increment(n),
            Message::Decrement(n) => self.decrement(n),
            _ => Err(ActorError::InvalidMessage(message.to_string())),
        };

        self.condvar.notify_all();

        result
    }

    fn get_id(&self) -> usize {
        self.id
    }

    fn get_state(&self) -> Result<ActorState, ActorError> {
        match self.state.read() {
            Ok(state) => Ok(*state),
            Err(e) => Err(ActorError::LockError(e.to_string())),
        }
    }

    fn get_value(&self) -> Result<i32, ActorError> {
        match self.value.read() {
            Ok(value) => Ok(*value),
            Err(e) => Err(ActorError::LockError(e.to_string())),
        }
    }

    fn set_value(&self, value: i32) -> Result<(), ActorError> {
        let mut value_lock = self.value.write().unwrap();
        *value_lock = value;

        Ok(())
    }

    pub fn get_subscribers(&self) -> Vec<usize> {
        let subs = self.subs.read().unwrap();
        subs.keys().cloned().collect()
    }

    fn add_subscriber(&self, actor: Arc<Actor>) -> Result<Option<Arc<Actor>>, ActorError> {
        let mut subs = self.subs.write().unwrap();

        if subs.contains_key(&actor.get_id()) {
            return Err(ActorError::ActorAlreadyExists(actor.get_id().to_string()));
        }

        Ok(subs.insert(actor.get_id(), actor))
    }

    fn remove_subscriber(&self, actor_id: usize) -> Result<(), ActorError> {
        let mut subs = self.subs.write().unwrap();

        if subs.contains_key(&actor_id) {
            subs.remove(&actor_id);
        }

        Err(ActorError::TargetActorNotFound(actor_id.to_string()))
    }

    fn update_subscription(&self, actor_id: usize) -> Option<Arc<Actor>> {
        let mut subs = self.subs.write().unwrap();
        subs.remove(&actor_id)
    }

    // Message handlers

    fn update_value<F>(&self, modifier: F) -> Result<(), ActorError>
    where
        F: FnOnce(i32) -> Result<i32, ActorError>,
    {
        let mut value = self.get_value()?;
        value = modifier(value)?;

        self.set_value(value)
    }

    fn increment(&self, n: i32) -> Result<(), ActorError> {
        self.update_value(|value| Ok(value + n))
    }

    fn decrement(&self, n: i32) -> Result<(), ActorError> {
        self.update_value(|value| Ok(value - n))
    }
}
