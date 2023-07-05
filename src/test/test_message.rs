#[cfg(test)]
mod message_handling_test {
    use std::thread;

    use crate::model::actor::ActorPool;
    use crate::model::message::Message;
    use crate::model::state::ActorState;

    #[test]
    fn test_handling_multiple_messages() {
        let actor = ActorPool::new();
        let id = actor.create_actor();

        let value = actor.get_actor_value(id).unwrap();
        assert_eq!(value, 0);

        let increment_values = vec![10, 20, 55, 130];
        for value in increment_values {
            let message = Message::Increment(value);
            actor.message_loop(id, message).unwrap();
        }

        thread::sleep(std::time::Duration::from_millis(100));

        let value = actor.get_actor_value(id).unwrap();
        assert_eq!(value, 215);

        let decrement_values = vec![10, 20, 55, 130];
        for value in decrement_values {
            let message = Message::Decrement(value);
            actor.message_loop(id, message).unwrap();
        }

        thread::sleep(std::time::Duration::from_millis(100));

        let value = actor.get_actor_value(id).unwrap();
        assert_eq!(value, 0);
    }

    #[test]
    fn test_message_propagation() {
        let pool = ActorPool::new();

        let actor1 = pool.create_actor();
        let actor2 = pool.create_actor();
        let actor3 = pool.create_actor();

        let actor1_value = pool.get_actor_value(actor1).unwrap();
        let actor2_value = pool.get_actor_value(actor2).unwrap();
        let actor3_value = pool.get_actor_value(actor3).unwrap();

        assert_eq!(actor1_value, 0);
        assert_eq!(actor2_value, 0);
        assert_eq!(actor3_value, 0);

        // Add actor2 and actor3 to actor1's subscription list
        pool.subscribe(actor1, vec![actor2, actor3]).unwrap();

        // Send message to actor1
        let message = Message::Increment(10);
        pool.message_loop(actor1, message).unwrap();

        thread::sleep(std::time::Duration::from_millis(100));

        let new_actor1_value = pool.get_actor_value(actor1).unwrap();
        let new_actor2_value = pool.get_actor_value(actor2).unwrap();
        let new_actor3_value = pool.get_actor_value(actor3).unwrap();

        assert_eq!(new_actor1_value, 10);
        assert_eq!(new_actor2_value, 10);
        assert_eq!(new_actor3_value, 10);
    }

    #[test]
    fn test_send_message_to_inactive_actor() {
        let pool = ActorPool::new();
        let actor = pool.create_actor();

        let actor_state = pool.update_actor_state(actor).unwrap();

        assert_eq!(actor_state, ActorState::Inactive);

        // Send message to inactive actor
        let messages = vec![
            Message::Increment(10),
            Message::Increment(20),
            Message::Increment(30),
        ];

        for message in messages {
            pool.message_loop(actor, message).unwrap();
        }

        thread::sleep(std::time::Duration::from_millis(100));

        assert_eq!(pool.get_actor_value(actor).unwrap(), 0);

        // Change the actor's state to active
        let actor_state = pool.update_actor_state(actor).unwrap();
        assert_eq!(actor_state, ActorState::Active);

        // Read messages from the actor's mailbox and update the actor's value
        assert_eq!(pool.get_actor_value(actor).unwrap(), 60);
    }
}
