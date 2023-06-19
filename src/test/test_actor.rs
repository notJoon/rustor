#[cfg(test)]
mod basic_actor_test {
    use crate::model::{actor::Actor, state::ActorState};

    #[test]
    fn test_create_new_actor() {
        let actor = Actor::new();

        assert_eq!(actor.get_id(), 0);
        assert_eq!(actor.get_state(), ActorState::Active);
    }
}

#[cfg(test)]
mod test_subscriber {
    use std::sync::Arc;

    use crate::model::actor::Actor;

    #[test]
    fn test_add_and_remove_subscriber() {
        let a1 = Actor::new();
        let a2 = Arc::new(Actor::new());
        let a3 = Arc::new(Actor::new());
        
        a1.add_subscriber(a2.clone());
        a1.add_subscriber(a3.clone());

        let subs = a1.get_subscribers();
        assert_eq!(subs.contains(&a2.get_id()), true);
        assert_eq!(subs.contains(&a3.get_id()), true);
        assert_eq!(subs.len(), 2);

        a1.remove_subscriber(a2.get_id());

        let subs = a1.get_subscribers();
        assert_eq!(subs.contains(&a2.get_id()), false);
        assert_eq!(subs.contains(&a3.get_id()), true);
        assert_eq!(subs.len(), 1);

        a1.remove_subscriber(a3.get_id());

        let subs = a1.get_subscribers();
        assert_eq!(subs.contains(&a3.get_id()), false);
        assert_eq!(subs.len(), 0);
    }

    #[test]
    fn test_empty_subscribers() {
        let a1 = Actor::new();

        let subs = a1.get_subscribers();
        assert_eq!(subs.len(), 0);
    }
}