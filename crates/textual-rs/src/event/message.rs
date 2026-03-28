//! Typed message trait and dynamic dispatch support for widget communication.

use std::any::Any;

/// Marker trait for typed messages in textual-rs.
/// All messages must implement Any (automatically satisfied by 'static types).
/// Messages are plain structs posted to the message queue and dispatched via bubbling.
pub trait Message: Any + 'static {
    /// Whether this message should bubble up the widget tree.
    /// Default: true. Override to false for messages that should only
    /// be handled by the originating widget's direct parent.
    fn bubbles() -> bool
    where
        Self: Sized,
    {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;

    struct MyMsg {
        value: i32,
    }

    impl Message for MyMsg {}

    #[test]
    fn message_impl_can_be_downcast_via_any() {
        let msg = MyMsg { value: 42 };
        // Upcast to &dyn Any then downcast back to &MyMsg
        let any_ref: &dyn Any = &msg;
        let downcast = any_ref.downcast_ref::<MyMsg>();
        assert!(downcast.is_some());
        assert_eq!(downcast.unwrap().value, 42);
    }

    #[test]
    fn message_boxed_as_any_downcasts_correctly() {
        let msg: Box<dyn Any> = Box::new(MyMsg { value: 99 });
        let downcast = msg.downcast_ref::<MyMsg>();
        assert!(downcast.is_some());
        assert_eq!(downcast.unwrap().value, 99);
    }

    #[test]
    fn message_bubbles_default_true() {
        assert!(MyMsg::bubbles());
    }
}
