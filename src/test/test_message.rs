#[cfg(test)]
mod message_handling_test {
    use crate::model::actor::ActorPool;
    use crate::model::message::Message;

    #[test]
    fn test_message_passing() {
        let pool = ActorPool::new();

        let actor_id = pool.update_actor_list();
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
}