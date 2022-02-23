use panda::mem::{map_memory, physical_memory_write, PAGE_SIZE};
use panda::prelude::*;
use panda::{GuestPtr, GuestType};

#[derive(GuestType, Debug, PartialEq)]
struct Test {
    x: u32,
    y: u32,
}

const ADDRESS: target_ptr_t = 0x1000;

#[panda::after_machine_init]
fn setup(_: &mut CPUState) {
    // Map 2MB memory for this emulation
    map_memory("mymem", 2 * 1024 * PAGE_SIZE, ADDRESS).unwrap();

    // Write code into memory
    physical_memory_write(ADDRESS, b"\x34\x12\x00\x00\x20\x00\x00\x00");

    // read memory back using GuestPtr
    let ptr: GuestPtr<u32> = ADDRESS.into();

    assert_eq!(*ptr, 0x1234);
    assert_eq!(*ptr.offset(1), 0x20);
    println!("u32 ptr read success!");

    let mut ptr = ptr.cast::<Test>();
    assert_eq!(*ptr, Test { x: 0x1234, y: 0x20 });
    println!("Struct read success!");

    ptr.write(|test| {
        test.x = 0x2345;
        test.y = 0x21;
    });

    assert_eq!(*ptr, Test { x: 0x2345, y: 0x21 });
    println!("Write to GuestPtr cache success");

    ptr.clear_cache();

    assert_eq!(*ptr, Test { x: 0x2345, y: 0x21 });
    println!("Write to memory success");
}

fn main() {
    Panda::new().arch(panda::Arch::x86_64).configurable().run();
}
