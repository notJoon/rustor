use std::{collections::{HashMap, VecDeque}, sync::{Arc, atomic::AtomicUsize, Mutex}};

use super::{state::ActorState, message::Message};

/* TODO:

    1. 액터의 생성과 제거 [v]
    2. 정보 가져오기 (value, state, subscribers) [v]
    3. 메시지 전달
    4. 액터 정지
    5. 구독 시스템 구현

 */

static ACTOR_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub struct Actor {
    pub id: usize,
    pub state: ActorState,
    pub value: i32,
    pub subs: Mutex<HashMap<usize, Arc<Actor>>>,
    pub mailbox: Mutex<VecDeque<Message>>,
}

impl Actor {
    pub fn new() -> Self {
        let id = ACTOR_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Actor {
            id,
            state: ActorState::Active,
            value: 0,
            subs: Mutex::new(HashMap::new()),
            mailbox: Mutex::new(VecDeque::new()),
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_state(&self) -> ActorState {
        self.state
    }

    pub fn get_value(&self) -> i32 {
        self.value
    }

    pub fn get_subscribers(&self) -> Vec<usize> {
        let subs = self.subs.lock().unwrap();

        subs.keys().map(|k| *k).collect()
    }

    pub fn add_subscriber(&self, actor: Arc<Actor>) {
        let mut subs = self.subs.lock().unwrap();
        subs.insert(actor.get_id(), actor);
    }

    pub fn remove_subscriber(&self, actor_id: usize) {
        let mut subs = self.subs.lock().unwrap();
        subs.remove(&actor_id);
    }

    pub fn send_message(&self, message: Message) {
        unimplemented!("send_message")
    }

    pub fn get_message(&self) -> Option<Message> {
        unimplemented!("get_message")
    }
}