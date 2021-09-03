mod use_context;
mod use_effect;
mod use_reducer;
mod use_ref;
mod use_state;

pub use use_context::*;
pub use use_effect::*;
pub use use_reducer::*;
pub use use_ref::*;
pub use use_state::*;

use crate::functional::{HookUpdater, CURRENT_HOOK};
use std::rc::Rc;

/// Low level building block of creating hooks.
///
/// It is used to created the pre-defined primitive hooks.
/// Generally, it isn't needed to create hooks and should be avoided as most custom hooks can be
/// created by combining other hooks as described in [Yew Docs].
///
/// The `initializer` callback is called once to create the initial state of the hook.
/// `runner` callback handles the logic of the hook. It is called when the hook function is called.
/// `destructor`, as the name implies, is called to cleanup the leftovers of the hook.
///
/// See the pre-defined hooks for examples of how to use this function.
///
/// [Yew Docs]: https://yew.rs/next/concepts/function-components/custom-hooks
pub fn use_hook<InternalHook: 'static, Output, Tear: FnOnce(&InternalHook) + 'static>(
    initializer: impl FnOnce() -> InternalHook,
    runner: impl FnOnce(Rc<InternalHook>, HookUpdater) -> Output,
    destructor: Tear,
) -> Output {
    // Extract current hook
    let updater = CURRENT_HOOK.with(|hook_state| {
        // Determine which hook position we're at and increment for the next hook
        let hook_pos = hook_state.counter;
        hook_state.counter += 1;

        // Initialize hook if this is the first call
        if hook_pos >= hook_state.hooks.len() {
            let initial_state = Rc::new(initializer());
            hook_state.hooks.push(initial_state.clone());
            hook_state.destroy_listeners.push(Box::new(move || {
                destructor(&initial_state);
            }));
        }

        let hook = hook_state
            .hooks
            .get(hook_pos)
            .expect("Not the same number of hooks. Hooks must not be called conditionally")
            .clone();

        HookUpdater {
            hook,
            process_message: hook_state.process_message.clone(),
        }
    });

    // In order to convert from a `dyn Any` back to `InternalHook` we can move the Rc to a ptr type
    // and use std::ptr::cast then reconstruct the Rc with the correct type.
    let hook = updater.hook.clone();
    assert!(
        hook.is::<InternalHook>(),
        "Incompatible hook type. Hooks must always be called in the same order"
    );
    // SAFETY:
    // `into_raw`/`from_raw` does not change the strong count and a clone of the `Rc` is in updater
    // and `hook_state` so the memory cannot be deallocated between the into and from calls.
    //
    // The ptr is originally from Rc which satisfies one of the requirements of `from_raw`.
    // The second requirement is that `U` has the same size and memory alignment as `T`.
    // The dyn ptr is a wide pointer but when casting we take just the data ptr which we know
    // to be the initial state (due to the assert check) that was created in this function so
    // the ptr can be cast and converted back into the initial type Rc<RefCell<InternalHook>>.
    let hook = Rc::into_raw(hook).cast();
    let hook = unsafe { Rc::from_raw(hook) };

    // Execute the actual hook closure we were given. Let it mutate the hook state and let
    // it create a callback that takes the mutable hook state.
    runner(hook, updater)
}
