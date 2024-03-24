#[derive(Debug)]
pub struct REACT_ELEMENT_TYPE;

#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

// pub enum ElementType {
//
// }
