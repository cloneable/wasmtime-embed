use wit_bindgen::rt::string::String;

wit_bindgen::generate!("my-component");
use crate::example::wasm_embedding::my_host;

struct MyComponentImpl;

impl exports::demo::Demo for MyComponentImpl {
    fn exchange_strings(s: String) -> String {
        my_host::log(&s);
        s
    }
}

export_my_component!(MyComponentImpl);
