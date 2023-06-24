use rayon::{ThreadPool, ThreadPoolBuilder};
use std::{
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Condvar, Mutex, MutexGuard, RwLock,
    },
};

use super::{errors::ActorError, message::Message, state::ActorState};

/// Global actor ID counter
static ACTOR_ID: AtomicUsize = AtomicUsize::new(0);

/// Actor pool containing all actors and a thread pool
#[derive(Debug)]
pub struct ActorPool {
    pub actor_list: Mutex<HashMap<usize, Arc<Actor>>>,
    thread_pool: ThreadPool,
}

impl ActorPool {
    /// Create a new actor pool with a thread pool
    pub fn new() -> Self {
        Default::default()
    }

    pub fn create_actor(&self) -> usize {
        let actor = Actor::new();
        let id = actor.id;

        {
            let actor_clone = Arc::clone(&actor);
            self.thread_pool.spawn(move || {
                actor_clone.run();
            });
        }

        let mut actor_list = self.actor_list.lock().unwrap();
        actor_list.insert(id, actor);

        id
    }

    pub fn remove_actor(&self, actor_id: usize) -> Result<(), ActorError> {
        let mut actor_list = self.actor_list();

        // if let Some(actor) = actor_list.remove(&actor_id) {
        //     let subs = actor.subs.read().unwrap();

        //     // Remove the actor from the subscriber list of other actors
        //     for (_, sub) in subs.iter() {
        //         sub.update_subscription(actor_id);
        //     }

        //     return Ok(());
        // }

        if let Some(i) = actor_list.iter().position(|actor| actor.1.id == actor_id) {
            let actor = actor_list.remove(&i).unwrap();
            actor.send_message(Message::Exit).unwrap();

            return Ok(());
        }

        Err(ActorError::TargetActorNotFound(actor_id.to_string()))
    }

    pub fn get_actor_state(&self, actor_id: usize) -> Result<ActorState, ActorError> {
        let actor = self.get_actor_info(actor_id)?;

        actor.get_state()
    }

    pub fn update_actor_state(&mut self, actor_id: usize) -> Result<(), ActorError> {
        let actor = self.get_actor_info(actor_id).unwrap();
        let mut state = actor.state.write().unwrap();

        match state.clone() {
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

        for subscriber_actor_id in subscriber_actor_ids {
            let subscriber_actor = self.get_actor_info(subscriber_actor_id).unwrap();
            target_actor.add_subscriber(subscriber_actor).unwrap();
        }

        Ok(target_actor)
    }

    pub fn get_actor_info(&self, actor_id: usize) -> Result<Arc<Actor>, ActorError> {
        let actor_list = self.actor_list();
        let actor = actor_list
            .get(&actor_id)
            .ok_or(ActorError::TargetActorNotFound(actor_id.to_string()))?;

        Ok(actor.to_owned())
    }

    fn actor_list(&self) -> MutexGuard<HashMap<usize, Arc<Actor>>> {
        self.actor_list.lock().unwrap()
    }

    pub fn message_loop(&self, actor_id: usize, message: Message) -> Result<(), ActorError> {
        let actor = self.get_actor_info(actor_id)?;

        actor.send_message(message)?;

        let processing = actor.processing.lock().unwrap();
        processing.1.notify_all();

        Ok(())
    }
}

impl Default for ActorPool {
    fn default() -> Self {
        let thread_pool = ThreadPoolBuilder::new()
            .num_threads(10)
            .build()
            .expect("Failed to create thread pool");

        ActorPool {
            actor_list: Mutex::new(HashMap::new()),
            thread_pool,
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
    pub processing: Mutex<(bool, Condvar)>,
}

impl Actor {
    /// Create a new actor with unique ID
    fn new() -> Arc<Self> {
        const INC: usize = 1;
        let id = ACTOR_ID.fetch_add(INC, Ordering::SeqCst);

        let actor = Actor {
            id,
            state: RwLock::new(ActorState::Active),
            value: RwLock::new(0),
            subs: RwLock::new(HashMap::new()),
            mailbox: Mutex::new(VecDeque::new()),
            processing: Mutex::new((false, Condvar::new())),
        };

        Arc::new(actor)
    }

    /// Runs the actor, processing messages from its mailbox
    fn run(&self) {
        loop {
            let message = {
                let mut mailbox = self.mailbox.lock().unwrap();

                // Wait for a message to be added to the mailbox
                while mailbox.is_empty() {
                    // Wait release the lock on the mailbox and blocks until a message is added
                    let processing = self.processing.lock().unwrap();
                    mailbox = processing.1.wait(mailbox).unwrap();
                }

                // Pop the next message off the front of the mailbox
                mailbox.pop_front().unwrap()
            };

            if let Err(e) = self.handle_message(message) {
                println!("Error handling message: {e:?}");
            }

            let processing = self.processing.lock().unwrap();
            processing.1.notify_all();
        }
    }

    /// Handles a message by matching its type and calling the appropriate handler
    pub fn handle_message(&self, message: Message) -> Result<(), ActorError> {
        let result = match message {
            Message::Increment(n) => self.increment(n),
            Message::Decrement(n) => self.decrement(n),
            Message::Exit => {
                self.exit();
                Ok(())
            }
            _ => Err(ActorError::InvalidMessage(message.to_string())),
        };

        let processing = self.processing.lock().unwrap();
        processing.1.notify_all();

        result
    }

    /// Add a message to the actor's mailbox
    pub fn send_message(&self, message: Message) -> Result<(), ActorError> {
        let mut mailbox = self.mailbox.lock().unwrap();
        mailbox.push_back(message);

        let processing = self.processing.lock().unwrap();
        processing.1.notify_one();

        Ok(())
    }

    /// Process the first message in the actor's mailbox
    pub fn process_message(&self) -> Result<(), ActorError> {
        let mut mailbox = self.mailbox.lock().unwrap();

        if let Some(message) = mailbox.pop_front() {
            let result = self.handle_message(message);

            let mut message_processed = self.processing.lock().unwrap();
            message_processed.0 = true;
            message_processed.1.notify_all();

            return result;
        }

        Ok(())
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_state(&self) -> Result<ActorState, ActorError> {
        match self.state.read() {
            Ok(state) => Ok(*state),
            Err(e) => Err(ActorError::LockError(e.to_string())),
        }
    }

    pub fn get_value(&self) -> Result<i32, ActorError> {
        match self.value.read() {
            Ok(value) => Ok(*value),
            Err(e) => Err(ActorError::LockError(e.to_string())),
        }
    }

    pub fn set_value(&self, value: i32) -> Result<(), ActorError> {
        let mut value_lock = self.value.write().unwrap();
        *value_lock = value;

        Ok(())
    }

    pub fn get_subscribers(&self) -> Vec<usize> {
        let subs = self.subs.read().unwrap();
        subs.keys().cloned().collect()
    }

    pub fn add_subscriber(&self, actor: Arc<Actor>) -> Result<Option<Arc<Actor>>, ActorError> {
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

    fn increment(&self, n: i32) -> Result<(), ActorError> {
        let mut value = self.get_value()?;
        value += n;

        self.set_value(value)
    }

    fn decrement(&self, n: i32) -> Result<(), ActorError> {
        let mut value = self.get_value()?;
        value -= n;

        self.set_value(value)
    }

    pub fn exit(&self) {
        let mut state = self.state.write().unwrap();
        *state = ActorState::Inactive;
    }
}
