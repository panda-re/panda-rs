#/bin/bash
bindgen bindings.h -o bindings.rs --no-layout-tests -- -I$PANDA_ROOT/panda/include -I$PANDA_ROOT/build -I$PANDA_ROOT/build/x86_64-softmmu -I$PANDA_ROOT/include -I/usr/include/glib-2.0 -I/usr/lib/x86_64-linux-gnu/glib-2.0/include -I$PANDA_ROOT/target/i386 -I$PANDA_ROOT/tcg/i386 -I$PANDA_ROOT/tcg -DNEED_CPU_H -I$PANDA_ROOT

#TODO: make bindgen script use panda env variables and whatnot
