use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

use wasm_bindgen::JsValue;

#[derive(Debug, Clone)]
pub struct SuspenseException;

impl Error for SuspenseException {}

impl Display for SuspenseException {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "It's not a true mistake, but part of Suspense's job. If you catch the error, keep throwing it out")
    }
}

pub static SUSPENSE_EXCEPTION: JsValue = JsValue::from_str("It's not a true mistake, but part of Suspense's job. If you catch the error, keep throwing it out");

pub fn track_used_thenable(usable: &JsValue) -> Result<JsValue, SuspenseException> {
    Err(&SUSPENSE_EXCEPTION)
}
