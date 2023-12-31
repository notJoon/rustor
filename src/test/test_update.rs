#[cfg(test)]
mod actor_update_tests {
    use crate::model::{actor::ActorPool, state::ActorState};

    #[test]
    fn test_update_actor_state() {
        let mut actors = ActorPool::new();
        let id = actors.create_actor();

        assert_eq!(id, 0);

        let state = actors.get_actor_state(id).unwrap();
        assert_eq!(state, ActorState::Active);

        actors.update_actor_state(id).unwrap();

        let state = actors.get_actor_state(id).unwrap();
        assert_eq!(state, ActorState::Inactive);

        actors.update_actor_state(id).unwrap();

        let state = actors.get_actor_state(id).unwrap();
        assert_eq!(state, ActorState::Active);
    }
}
