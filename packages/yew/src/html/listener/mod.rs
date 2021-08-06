#[macro_use]
mod macros;
mod events;

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    Element, FileList, HtmlInputElement as InputElement, HtmlTextAreaElement as TextAreaElement,
    InputEvent,
};

pub use events::*;

/// A type representing data from `oninput` event.
#[derive(Debug)]
pub struct InputData {
    /// Inserted characters. Contains value from
    /// [InputEvent](https://developer.mozilla.org/en-US/docs/Web/API/InputEvent/data).
    pub value: String,
    /// The InputEvent received.
    pub event: InputEvent,
}

fn oninput_handler(this: &Element, event: InputEvent) -> InputData {
    // Normally only InputElement or TextAreaElement can have an oninput event listener. In
    // practice though any element with `contenteditable=true` may generate such events,
    // therefore here we fall back to just returning the text content of the node.
    // See https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/input_event.
    let (v1, v2) = (
        this.dyn_ref().map(|input: &InputElement| input.value()),
        this.dyn_ref().map(|input: &TextAreaElement| input.value()),
    );
    let v3 = this.text_content();
    let value = v1.or(v2).or(v3)
        .expect("only an InputElement or TextAreaElement or an element with contenteditable=true can have an oninput event listener");
    InputData { value, event }
}

/// A wrapper type around a [`Event`](web_sys::Event) to provide helper functions for common actions
/// on a `change` event.
#[derive(Debug)]
pub struct ChangeEvent(web_sys::Event);

impl ChangeEvent {
    /// Gets the [`FileList`] from the element this `change` event was dispatched from.
    ///
    /// The target element must have a `files` property which is an instanceof [`FileList`],
    /// otherwise this function will return [`None`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use yew::{ChangeEvent, web_sys::FileList};
    /// let on_input_change = |e: ChangeEvent| {
    ///     let file_list: FileList = e.get_file_list()
    ///         .expect("onchange event to be dispatched from file input");
    ///     // continue using the file_list
    /// };
    /// ```
    pub fn get_file_list(&self) -> Option<FileList> {
        let target = self.target()?;
        let files = js_sys::Reflect::get(&target, &JsValue::from("files"))
            .expect("EventTarget should be an object");
        files
            .is_instance_of::<FileList>()
            .then(|| FileList::from(files))
    }

    /// Gets the DOMString of the `value` property from the element this `change` event was
    /// dispatched from.
    pub fn get_value(&self) -> Option<String> {
        let target = self.target()?;
        js_sys::Reflect::get(&target, &JsValue::from_str("value"))
            .expect("EventTarget should be an object")
            .as_string()
    }
}

// Allow users to get to the raw web_sys::Event
impl std::ops::Deref for ChangeEvent {
    type Target = web_sys::Event;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
