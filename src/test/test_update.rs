#[cfg(test)]
mod actor_update_tests {
    use crate::model::{actor::ActorPool, state::ActorState};

    #[test]
    fn test_update_actor_state() {
        let mut actors = ActorPool::new();
        let id = actors.update_actor_list();

        assert_eq!(id, 0);

        let state = actors.get_actor_state(id).unwrap();
        assert_eq!(state, ActorState::Active);

        println!("id: {}, state: {:?}", id, state);

        actors.update_actor_state(id).unwrap();

        let state = actors.get_actor_state(id).unwrap();
        assert_eq!(state, ActorState::Inactive);

        println!("id: {}, state: {:?}", id, state);
    }
}