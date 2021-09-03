use crate::functional::use_hook;
use std::{borrow::Borrow, cell::RefCell, rc::Rc};

struct UseEffect<Destructor> {
    destructor: Option<Box<Destructor>>,
}

/// This hook is used for hooking into the component's lifecycle.
///
/// # Example
/// ```rust
/// # use yew::prelude::*;
/// # use std::rc::Rc;
/// #
/// #[function_component(UseEffect)]
/// fn effect() -> Html {
///     let counter = use_state(|| 0);
///
///     let counter_one = counter.clone();
///     use_effect(move || {
///         // Make a call to DOM API after component is rendered
///         yew::utils::document().set_title(&format!("You clicked {} times", *counter_one));
///
///         // Perform the cleanup
///         || yew::utils::document().set_title(&format!("You clicked 0 times"))
///     });
///
///     let onclick = {
///         let counter = counter.clone();
///         Callback::from(move |_| counter.set(*counter + 1))
///     };
///
///     html! {
///         <button {onclick}>{ format!("Increment to {}", *counter) }</button>
///     }
/// }
/// ```
pub fn use_effect<Destructor>(callback: impl FnOnce() -> Destructor + 'static)
where
    Destructor: FnOnce() + 'static,
{
    let callback = Box::new(callback);
    use_hook(
        move || {
            let effect: UseEffect<Destructor> = UseEffect { destructor: None };
            RefCell::new(effect)
        },
        |_, updater| {
            // Run on every render
            updater.post_render(move |state: &RefCell<UseEffect<Destructor>>| {
                if let Some(de) = state.borrow_mut().destructor.take() {
                    de();
                }
                let new_destructor = callback();
                state
                    .borrow_mut()
                    .destructor
                    .replace(Box::new(new_destructor));
                false
            });
        },
        |hook| {
            if let Some(destructor) = hook.borrow_mut().destructor.take() {
                destructor()
            }
        },
    )
}

struct UseEffectDeps<Destructor, Dependents> {
    destructor: Option<Box<Destructor>>,
    deps: Rc<Dependents>,
}

/// This hook is similar to [`use_effect`] but it accepts dependencies.
///
/// Whenever the dependencies are changed, the effect callback is called again.
/// To detect changes, dependencies must implement `PartialEq`.
/// Note that the destructor also runs when dependencies change.
pub fn use_effect_with_deps<Callback, Destructor, Dependents>(callback: Callback, deps: Dependents)
where
    Callback: FnOnce(&Dependents) -> Destructor + 'static,
    Destructor: FnOnce() + 'static,
    Dependents: PartialEq + 'static,
{
    let deps = Rc::new(deps);
    let deps_c = deps.clone();

    use_hook(
        move || {
            let destructor: Option<Box<Destructor>> = None;
            RefCell::new(UseEffectDeps {
                destructor,
                deps: deps_c,
            })
        },
        move |_, updater| {
            updater.post_render(
                move |state: &RefCell<UseEffectDeps<Destructor, Dependents>>| {
                    if state.borrow().deps != deps {
                        if let Some(de) = state.borrow_mut().destructor.take() {
                            de();
                        }
                        let new_destructor = callback(deps.borrow());
                        state.borrow_mut().deps = deps;
                        state
                            .borrow_mut()
                            .destructor
                            .replace(Box::new(new_destructor));
                    } else if state.borrow().destructor.is_none() {
                        let new_destructor = Box::new(callback(&state.borrow().deps));
                        state.borrow_mut().destructor.replace(new_destructor);
                    }
                    false
                },
            );
        },
        |hook| {
            if let Some(destructor) = hook.borrow_mut().destructor.take() {
                destructor()
            }
        },
    );
}
