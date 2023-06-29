use wasmtime::*;

pub struct FfiString {
    ptr: u32,
    len: u32,
}

impl From<FfiString> for u64 {
    fn from(value: FfiString) -> Self {
        (value.ptr as u64) << 32 | (value.len as u64)
    }
}

impl From<u64> for FfiString {
    fn from(value: u64) -> Self {
        FfiString {
            ptr: (value >> 32) as u32,
            len: (value & 0xFFFFFFFFu64) as u32,
        }
    }
}

fn main() -> Result<()> {
    let Some(file) = std::env::args().nth(1) else {
        anyhow::bail!("no module file specified")
    };

    let config = Config::new();
    let engine = Engine::new(&config)?;

    let mut store = Store::new(&engine, ());
    let mut linker = Linker::new(&engine);

    linker.func_wrap(
        "my-host",
        "log",
        |mut caller: Caller<'_, ()>, ptr: u32, len: u32| {
            let mem = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => anyhow::bail!("failed to find host memory"),
            };
            let data = mem
                .data(&caller)
                .get(ptr as usize..)
                .and_then(|arr| arr.get(..len as usize));
            let msg = match data {
                Some(data) => match ::std::str::from_utf8(data) {
                    Ok(s) => s,
                    Err(_) => anyhow::bail!("invalid utf-8"),
                },
                None => anyhow::bail!("pointer/length out of bounds"),
            };
            println!("LOG: {msg}");
            Ok(())
        },
    )?;

    let module = Module::from_file(&engine, file)?;
    let instance = linker.instantiate(&mut store, &module)?;

    let alloc_string = instance.get_typed_func::<u32, u32>(&mut store, "alloc_string")?;

    let input = "Hi!";

    let ptr = alloc_string.call(&mut store, input.len() as u32)?;
    let mem = instance
        .get_memory(&mut store, "memory")
        .ok_or_else(|| anyhow::anyhow!("cannot get memory"))?;
    mem.write(&mut store, ptr as usize, input.as_bytes())?;

    let exchange_strings = instance.get_typed_func::<u64, u64>(&mut store, "exchange_strings")?;

    let output: FfiString = exchange_strings
        .call(
            &mut store,
            FfiString {
                ptr,
                len: input.len() as u32,
            }
            .into(),
        )?
        .into();

    let data = mem
        .data(&store)
        .get(output.ptr as usize..)
        .and_then(|arr| arr.get(..output.len as usize));
    let output = match data {
        Some(data) => match ::std::str::from_utf8(data) {
            Ok(s) => s,
            Err(_) => anyhow::bail!("invalid utf-8"),
        },
        None => anyhow::bail!("pointer/length out of bounds"),
    };

    println!("Output: {output}");

    Ok(())
}
