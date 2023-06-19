use std::{
    collections::{HashMap, VecDeque},
    sync::{atomic::AtomicUsize, Arc, Mutex, MutexGuard},
};

use super::{errors::ActorError, message::Message, state::ActorState};

static ACTOR_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub struct ActorSystem {
    pub actor_list: Mutex<HashMap<usize, Arc<Actor>>>,
}

impl ActorSystem {
    pub fn new() -> Self {
        ActorSystem {
            actor_list: Mutex::new(HashMap::new()),
        }
    }

    pub fn add_actor(&self) -> usize {
        let actor = Actor::new();
        let id = actor.id;

        let mut actor_list = self.actor_list();
        actor_list.insert(id, Arc::new(actor));

        id
    }

    pub fn remove_actor(&self, actor_id: usize) -> Result<(), ActorError> {
        let mut actor_list = self.actor_list();

        match actor_list.contains_key(&actor_id) {
            true => {
                let actor = actor_list.remove(&actor_id).unwrap();
                let subs = actor.subs.lock().unwrap();

                // remove an actor from the subscription list of all other actors
                for (_, sub) in subs.iter() {
                    sub.update_subscription(actor_id);
                }

                Ok(())
            }
            false => Err(ActorError::TargetActorNotFound(actor_id.to_string())),
        }
    }

    pub fn get_actor_state(&self, actor_id: usize) -> Result<ActorState, ActorError> {
        let actor = self.get_actor_info(actor_id)?;

        actor.get_state()
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
            target_actor.add_subscriber(subscriber_actor);
        }

        Ok(target_actor)
    }

    fn get_actor_info(&self, actor_id: usize) -> Result<Arc<Actor>, ActorError> {
        let actor_list = self.actor_list();
        let actor = actor_list
            .get(&actor_id)
            .ok_or(ActorError::TargetActorNotFound(actor_id.to_string()))?;

        Ok(actor.clone())
    }

    fn actor_list(&self) -> MutexGuard<HashMap<usize, Arc<Actor>>> {
        self.actor_list.lock().unwrap()
    }
}

#[derive(Debug)]
pub struct Actor {
    pub id: usize,
    pub state: Mutex<ActorState>,
    pub value: Mutex<i32>,
    pub subs: Mutex<HashMap<usize, Arc<Actor>>>,
    pub mailbox: Mutex<VecDeque<Message>>,
}

impl Actor {
    fn new() -> Self {
        let id = ACTOR_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        Actor {
            id,
            state: Mutex::new(ActorState::Active),
            value: Mutex::new(0),
            subs: Mutex::new(HashMap::new()),
            mailbox: Mutex::new(VecDeque::new()),
        }
    }

    fn get_id(&self) -> usize {
        self.id
    }

    fn get_state(&self) -> Result<ActorState, ActorError> {
        match self.state.lock() {
            Ok(state) => Ok(*state),
            Err(e) => Err(ActorError::LockError(e.to_string())),
        }
    }

    fn get_value(&self) -> Result<i32, ActorError> {
        match self.value.lock() {
            Ok(value) => Ok(*value),
            Err(e) => Err(ActorError::LockError(e.to_string())),
        }
    }

    fn subs(&self) -> MutexGuard<HashMap<usize, Arc<Actor>>> {
        self.subs.lock().unwrap()
    }

    fn get_subscribers(&self) -> Vec<usize> {
        let subs = self.subs();

        subs.keys().map(|k| *k).collect()
    }

    fn add_subscriber(&self, actor: Arc<Actor>) {
        let mut subs = self.subs();
        subs.insert(actor.get_id(), actor);
    }

    fn remove_subscriber(&self, actor_id: usize) {
        let mut subs = self.subs();
        subs.remove(&actor_id);
    }

    fn update_subscription(&self, actor_id: usize) {
        if self.subs().contains_key(&actor_id) {
            self.remove_subscriber(actor_id);
        }
    }
}
