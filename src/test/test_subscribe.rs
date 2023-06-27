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

    #[test]
    fn test_detect_cycle_subscribe() {
        let pool = ActorPool::new();

        let a1 = pool.create_actor();
        let a2 = pool.create_actor();
        let a3 = pool.create_actor();

        // Create Cycle : a1 -> a2 -> a3 -> a1
        pool.subscribe(a1, vec![a2]).unwrap();
        pool.subscribe(a2, vec![a3]).unwrap();
        pool.subscribe(a3, vec![a1]).unwrap();

        let has_cycle = pool.detect_cycle(a1).unwrap();

        assert_eq!(has_cycle, true)
    }

    #[test]
    fn test_has_no_cycle() {
        let pool = ActorPool::new();

        let a1 = pool.create_actor();
        let a2 = pool.create_actor();
        let a3 = pool.create_actor();

        pool.subscribe(a1, vec![a2]).unwrap();
        pool.subscribe(a2, vec![a3]).unwrap();

        let has_cycle = pool.detect_cycle(a1).unwrap();

        assert_eq!(has_cycle, false)
    }
}
