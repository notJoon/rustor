#[cfg(test)]
mod actor_subscribe_system_test {
    use crate::model::actor::ActorPool;

    #[test]
    fn test_get_default_actor_subscribe_list() {
        let actors = ActorPool::new();
        let id = actors.create_actor();

        let subscribers = actors.get_actor_subscribers(id).unwrap();
        assert_eq!(subscribers.len(), 0);

        println!("id: {}, subscribers: {:?}", id, subscribers);
    }

    fn pools(n: usize) -> ActorPool {
        let actors = ActorPool::new();

        for _ in 0..n {
            actors.create_actor();
        }

        actors
    }

    #[test]
    fn test_add_subscriber_into_root() {
        let actors = pools(10);

        let target_actor_id = 0;
        let subscriber_actor_ids = vec![1, 2, 3, 4, 5];

        let target_actor = actors
            .subscribe(target_actor_id, subscriber_actor_ids)
            .unwrap();

        let subscribers = target_actor.get_subscribers();
        assert_eq!(subscribers.len(), 5);

        let target_actor_id = 8;
        let subscriber_actor_ids = vec![5, 6, 9];

        let target_actor = actors
            .subscribe(target_actor_id, subscriber_actor_ids)
            .unwrap();

        let subscribers = target_actor.get_subscribers();
        assert_eq!(subscribers.len(), 3);

        for i in 0..10 {
            let actor = actors.get_actor_info(i).unwrap();
            let subscribers = actor.get_subscribers();

            println!("id: {}, subscribers: {:?}", i, subscribers);
        }
    }
}
