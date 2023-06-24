#[cfg(test)]
mod message_handling_test {
    use std::thread;

    use crate::model::actor::ActorPool;
    use crate::model::message::Message;

    #[test]
    fn test_message_passing_update_actor_value() {
        let actor = ActorPool::new();
        let id = actor.create_actor();

        let value = actor.get_actor_value(id).unwrap();
        assert_eq!(value, 0);

        println!("id: {}, value: {}", id, value);

        let message = Message::Increment(10);
        actor.message_loop(id, message).unwrap();

        thread::sleep(std::time::Duration::from_millis(100));

        let value = actor.get_actor_value(id).unwrap();
        assert_eq!(value, 10);

        println!("id: {}, value: {}", id, value);
    }

    #[test]
    fn test_send_message_to_actors() {
        let pool = ActorPool::new();

        let actor_ids: Vec<usize> = (0..10).map(|_| pool.create_actor()).collect();

        for actor_id in actor_ids.iter() {
            if actor_id % 2 == 0 {
                pool.message_loop(*actor_id, Message::Increment(10))
                    .unwrap();
            }
        }

        for actor_id in actor_ids.iter() {
            let value = pool.get_actor_value(*actor_id).unwrap();

            println!("id: {}, value: {}", actor_id, value);
            
            // if actor_id % 2 == 0 {
            //     assert_eq!(value, 10);
            // }
            // assert_eq!(value, 0);
        }
    }
}
