#[cfg(test)]
mod actor_tests {
    use crate::model::{actor::ActorSystem, state::ActorState};

    #[test]
    fn test_create_new_actor_and_get_its_information() {
        let actors = ActorSystem::new();
        let id = actors.update_actor_list();

        assert_eq!(id, 0);

        let state = actors.get_actor_state(id).unwrap();
        assert_eq!(state, ActorState::Active);
    }
}
