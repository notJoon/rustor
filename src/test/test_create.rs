#[cfg(test)]
mod actor_create_tests {
    use crate::model::{actor::ActorPool, state::ActorState};

    #[test]
    fn test_create_multiple_actors_get_its_values() {
        for i in 0..10 {
            let actors = ActorPool::new();
            let id = actors.update_actor_list();

            assert_eq!(id, i);

            let value = actors.get_actor_value(id).unwrap();
            assert_eq!(value, 0);

            let state = actors.get_actor_state(id).unwrap();
            assert_eq!(state, ActorState::Active);

            println!("id: {}, value: {}, state: {:?}", id, value, state);
        }
    }
}
