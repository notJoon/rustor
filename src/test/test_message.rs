#[cfg(test)]
mod message_handling_test {
    use crate::model::actor::{ActorPool, self};
    use crate::model::message::Message;

    #[test]
    fn test_message_passing() {
        let pool = ActorPool::new();

        let actor_id = pool.create_actor();
        pool.send_message(actor_id, Message::Increment(10)).unwrap();

        let actor_list = pool.actor_list.lock().unwrap();
        let actor = actor_list.get(&actor_id).unwrap();
        actor.process_message().unwrap();

        let value = actor.get_value().unwrap();
        assert_eq!(value, 10);

        // decreasing the value
        pool.send_message(actor_id, Message::Decrement(5)).unwrap();
        actor.process_message().unwrap();

        let value = actor.get_value().unwrap();
        assert_eq!(value, 5);
    }

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

        // Sleep for a while to make sure all messages are processed
        std::thread::sleep(std::time::Duration::from_millis(100));

        for id in actor_ids {
            let actor_list = pool.actor_list.lock().unwrap();
            let actor = actor_list.get(&id).unwrap();
            let value = actor.get_value().unwrap();
            assert_eq!(value, 10);
        }
    }
}
