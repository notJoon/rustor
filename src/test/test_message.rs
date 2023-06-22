#[cfg(test)]
mod message_handling_test {
    use crate::model::actor::ActorPool;
    use crate::model::message::Message;

    #[test]
    fn test_message_passing_with_thread_pool() {
        let pool = ActorPool::new();

        let mut actor_ids = Vec::new();

        println!("start push");
        for _ in 0..10 {
            actor_ids.push(pool.create_actor());
        }
        println!("finish push");

        println!("start send");
        for id in actor_ids.clone() {
            pool.send_message(id, Message::Increment(10)).unwrap();
        }
        println!("finish send");

        println!("start wait");
        for id in actor_ids {
            let actor_list = pool.actor_list.lock().unwrap();
            let actor = actor_list.get(&id).unwrap();

            println!("wait for actor {}", id);
            // We now wait until the message has been processed by the Actor
            let mut message_processed = actor.message_processed.lock().unwrap();
            while !*message_processed {
                message_processed = actor.condvar.wait(message_processed).unwrap();
            }
            println!("actor {} finished", id);

            println!("get value for actor {}", id);
            let value = actor.get_value().unwrap();
            assert_eq!(value, 10);
        }
        println!("finish wait");
        println!("all actors finished");
    }
}
