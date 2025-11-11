use dioxus::prelude::*;
use std::{cell::RefCell, rc::Rc};

pub fn use_previous<T>(current: T) -> Option<T>
where
    T: Clone + PartialEq + 'static,
{
    let state_ref = use_hook(|| Rc::new(RefCell::new(None::<T>)));
    let previous_value = state_ref.borrow().clone();

    use_effect(use_reactive!(|(current,)| {
        *state_ref.borrow_mut() = Some(current);
    }));

    previous_value
}
