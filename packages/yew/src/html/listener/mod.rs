#[macro_use]
mod macros;
mod events;

use wasm_bindgen::JsCast;
use web_sys::{
    Element, FileList, HtmlInputElement as InputElement, HtmlSelectElement as SelectElement,
    HtmlTextAreaElement as TextAreaElement, InputEvent,
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
    #[inline]
    fn get_typed_target<T: JsCast>(&self) -> Option<T> {
        self.target()?.dyn_into::<T>().ok()
    }

    /// Gets the [`FileList`] from the element this `change` event was dispatched from.
    ///
    /// The target element must be an input element with the `file` type to get the [`FileList`],
    /// otherwise this function will return [`None`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use yew::{ChangeEvent, web_sys::FileList};
    /// let onchange = |e: ChangeEvent| {
    ///     let file_list: FileList = e.get_file_list()
    ///         .expect("onchange event to be dispatched from file input");
    ///     // continue using the file_list
    /// };
    /// ```
    pub fn get_file_list(&self) -> Option<FileList> {
        let input = self.get_typed_target::<InputElement>()?;
        if input.type_().eq_ignore_ascii_case("file") {
            input.files()
        } else {
            None
        }
    }

    /// Gets the [`API value`](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#a-form-control's-value)
    /// from the element this `change` event was dispatched from.
    ///
    /// The `value` can be returned, when the target element is one of the following:
    /// - input
    /// - textarea
    /// - select - based on the selectedness of its option elements
    ///
    pub fn get_value(&self) -> Option<String> {
        let target = self.get_typed_target::<Element>()?;
        match target.node_name().as_ref() {
            "INPUT" => Some(target.unchecked_into::<InputElement>().value()),
            "TEXTAREA" => Some(target.unchecked_into::<TextAreaElement>().value()),
            "SELECT" => Some(target.unchecked_into::<SelectElement>().value()),
            _ => None,
        }
    }
}

// Allow users to get to the raw web_sys::Event
impl std::ops::Deref for ChangeEvent {
    type Target = web_sys::Event;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
