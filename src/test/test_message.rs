#[cfg(test)]
mod message_handling_test {
    use std::thread;
    use std::time::Duration;

    use crate::model::actor::ActorPool;
    use crate::model::message::Message;

    #[test]
    fn test_message_passing_with_thread_pool() {
        let pool = ActorPool::new();

        let mut actor_ids = Vec::new();

        for _ in 0..10 {
            actor_ids.push(pool.create_actor());
        }

        for id in actor_ids.clone() {
            pool.send_message(id, Message::Increment(10)).unwrap();
        }

        thread::sleep(Duration::from_secs(1));

        for id in actor_ids {
            let actor_list = pool.actor_list.lock().unwrap();
            let actor = actor_list.get(&id).unwrap();

            let value = actor.get_value().unwrap();
            assert_eq!(value, 10);
        }
    }
}
