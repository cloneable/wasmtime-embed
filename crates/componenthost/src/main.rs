use wasmtime::component::{Component, Linker};
use wasmtime::*;

wasmtime::component::bindgen!({
    world: "my-component",
    path: "../component/wit/test.wit",
});

struct MyHostImpl;

impl example::wasm_embedding::my_host::Host for MyHostImpl {
    fn log(&mut self, msg: String) -> wasmtime::Result<()> {
        println!("log: {msg}");
        Ok(())
    }
}

fn main() -> Result<()> {
    let Some(componentfile) = std::env::args().nth(1) else {
        anyhow::bail!("no component file specified")
    };

    let mut config = Config::new();
    config.wasm_component_model(true);

    let engine = Engine::new(&config)?;
    let mut linker = Linker::new(&engine);
    MyComponent::add_to_linker(&mut linker, |state: &mut MyHostImpl| state)?;
    let mut store = Store::new(&engine, MyHostImpl);

    let component = Component::from_file(&engine, componentfile)?;
    let (bindings, _instance) = MyComponent::instantiate(&mut store, &component, &linker)?;

    let output = bindings.demo().call_exchange_strings(&mut store, "Hi!")?;

    println!("Output: {output}");

    Ok(())
}
