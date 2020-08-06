#/bin/bash
bindgen bindings.h -o bindings.rs --no-layout-tests -- -I/home/jordan/dev/panda/panda/include -I/home/jordan/dev/panda/build -I/home/jordan/dev/panda/build/x86_64-softmmu -I/home/jordan/dev/panda/include -I/usr/include/glib-2.0 -I/usr/lib/x86_64-linux-gnu/glib-2.0/include -I/home/jordan/dev/panda/target/i386 -I/home/jordan/dev/panda/tcg/i386 -I/home/jordan/dev/panda/tcg -DNEED_CPU_H -I/home/jordan/dev/panda

#TODO: make bindgen script use panda env variables and whatnot
