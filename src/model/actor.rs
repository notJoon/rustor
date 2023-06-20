use rayon::{ThreadPool, ThreadPoolBuilder};
use std::{
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, MutexGuard, RwLock,
    },
    thread,
    time::Duration,
};

// TODO: 액터 사용 종료시 생성된 액터들의 메모리를 해제해야 함
// TODO: 메시지 전파 기능 구현

use super::{
    errors::ActorError,
    message::{self, Message},
    state::ActorState,
};

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
            self.thread_pool.install(move || {
                actor_clone.run();
            });
        }

        let mut actor_list = self.actor_list();
        actor_list.insert(id, actor);

        id
    }

    pub fn remove_actor(&self, actor_id: usize) -> Result<(), ActorError> {
        let mut actor_list = self.actor_list();

        if let Some(actor) = actor_list.remove(&actor_id) {
            let subs = actor.subs.read().unwrap();

            // Remove the actor from the subscriber list of other actors
            for (_, sub) in subs.iter() {
                sub.update_subscription(actor_id);
            }

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

    pub fn send_message(&self, actor_id: usize, message: Message) -> Result<(), ActorError> {
        let actor_list = self.actor_list.lock().unwrap();

        match actor_list.get(&actor_id) {
            Some(actor) => actor.send_message(message)?,
            None => return Err(ActorError::TargetActorNotFound(actor_id.to_string())),
        }

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
        };

        let actor_arc = Arc::new(actor);

        {
            let actor_clone = Arc::clone(&actor_arc);
            thread::spawn(move || {
                actor_clone.run();
            });
        }

        actor_arc
    }

    /// Runs the actor, processing messages from its mailbox
    fn run(&self) {
        loop {
            let message = {
                let mut mailbox = self.mailbox.lock().unwrap();
                mailbox.pop_front()
            };

            match message {
                Some(message) => self.handle_message(message).unwrap(),
                None => thread::sleep(Duration::from_millis(100)),
            }
        }
    }

    /// Handles a message by matching its type and calling the appropriate handler
    pub fn handle_message(&self, message: Message) -> Result<(), ActorError> {
        match message {
            Message::Increment(n) => self.increment(n),
            Message::Decrement(n) => self.decrement(n),
            _ => Err(ActorError::InvalidMessage(message.to_string())),
        }
    }

    /// Add a message to the actor's mailbox
    pub fn send_message(&self, message: Message) -> Result<(), ActorError> {
        let mut mailbox = self.mailbox.lock().unwrap();
        mailbox.push_back(message);

        Ok(())
    }

    /// Process the first message in the actor's mailbox
    pub fn process_message(&self) -> Result<(), ActorError> {
        let mut mailbox = self.mailbox.lock().unwrap();

        if let Some(message) = mailbox.pop_front() {
            self.handle_message(message)?;
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
        let mut value = self.value.write().unwrap();
        *value += n;

        Ok(())
    }

    fn decrement(&self, n: i32) -> Result<(), ActorError> {
        let mut value = self.value.write().unwrap();
        *value -= n;

        Ok(())
    }
}
