# panda-rs

Rust bindings for [PANDA](https://github.com/panda-re/panda) for use in either plugin mode or libpanda mode.

## Getting Started

Some resources for helping introduce you to PANDA and panda-rs:

* [The release blog post](https://panda-re.mit.edu/blog/panda-rs/)
* [The documentation](https://docs.rs/panda-re)
* [Our collection of plugins](https://github.com/panda-re/panda-rs-plugins)
* [The panda-rs example(s)](https://github.com/panda-re/panda-rs/tree/master/panda-rs/examples)

For running an example, simply clone the repo then:

```
cd panda-rs
python3 record_example_replay.py
cargo run --example showcase --features=libpanda
```

## Features

* Programmatically drive QEMU - integrate with QEMU more easily
* Record/Replay - Record parts of execution for quickly replaying and analyzing
* Snapshots - Utilize QEMU's snapshot feature to resume execution from a predetermined state
* Attribute-based hooking - Just tag a function with the events it should run during
* Share code between plugins and libpanda use
    * Plugin support - create a PANDA plugin that can be used by other plugins or from the command line
    * libpanda - drive an instance of PANDA from your own analysis/script/application.
* Multithread plugins with ease. Not burdened by Python's GIL and C/C++'s lack of thread safety or eurgonomic threading, write concurrent plugins fearlessly.
* Take advantage of the Rust ecosystem—plugins are just crates, so everything from concurrency primatives to backend frameworks are a Cargo.toml line away

## Brief Example

Below we have a simple plugin (that can be built as a cdylib):

```rust
use panda::prelude::*;

#[panda::on_sys_write_enter]
fn sys_write_test(cpu: &mut CPUState, pc: u64, fd: u64, buf: u64, count: u64) {
    // Output the contents of every sys_write as a string
    println!(
        "sys_write buf = \"{}\"",
        String::from_utf8_lossy(&cpu.mem_read(buf, count as usize))
    );
}

#[panda::init]
fn init(_: &mut PluginHandle) {
    println!("Plugin initialized!");
}
```

however, if we want to make this into an executable that calls into libpanda  we can add a main:

```rust
fn main() {
    Panda::new()
        .generic("x86_64") // use a generic x86_64 linux QCOW (a VM image)
        .replay("my_application_replay") // load a replay of the name "my_application_replay"
        .run();
}
```

and enable the `libpanda` feature of panda-rs.

## Executing Examples

Sample snippets in the `panda-rs/examples` directory can be run by name, e.g. `showcase.rs` is executed with:

```
cd panda-rs
cargo run --example showcase --features=libpanda
```

Note the addition of the 'libpanda' feature. This is because Rust examples are standalone executables, not shared libraries (like PANDA plugins) and thus must consume PANDA as a library.
