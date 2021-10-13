#/bin/bash

# Generate Rust bindings to C API for each architecture
bindgen bindings.h -o bindings/x86_64.rs --no-layout-tests -- -I$PANDA_ROOT/panda/include -I$PANDA_ROOT/build -I$PANDA_ROOT/build/x86_64-softmmu -I$PANDA_ROOT/include -I/usr/include/glib-2.0 -I/usr/lib/x86_64-linux-gnu/glib-2.0/include -I$PANDA_ROOT/target/i386 -I$PANDA_ROOT/tcg/i386 -I$PANDA_ROOT/tcg -DNEED_CPU_H -I$PANDA_ROOT
bindgen bindings.h -o bindings/i386.rs --no-layout-tests -- -I$PANDA_ROOT/panda/include -I$PANDA_ROOT/build -I$PANDA_ROOT/build/i386-softmmu -I$PANDA_ROOT/include -I/usr/include/glib-2.0 -I/usr/lib/x86_64-linux-gnu/glib-2.0/include -I$PANDA_ROOT/target/i386 -I$PANDA_ROOT/tcg/i386 -I$PANDA_ROOT/tcg -DNEED_CPU_H -I$PANDA_ROOT
bindgen bindings.h -o bindings/arm.rs --no-layout-tests -- -I$PANDA_ROOT/panda/include -I$PANDA_ROOT/build -I$PANDA_ROOT/build/arm-softmmu -I$PANDA_ROOT/include -I/usr/include/glib-2.0 -I/usr/lib/x86_64-linux-gnu/glib-2.0/include -I$PANDA_ROOT/target/arm -I$PANDA_ROOT/tcg/arm -I$PANDA_ROOT/tcg -DNEED_CPU_H -I$PANDA_ROOT
bindgen bindings.h -o bindings/ppc.rs --no-layout-tests -- -I$PANDA_ROOT/panda/include -I$PANDA_ROOT/build -I$PANDA_ROOT/build/ppc-softmmu -I$PANDA_ROOT/include -I/usr/include/glib-2.0 -I/usr/lib/x86_64-linux-gnu/glib-2.0/include -I$PANDA_ROOT/target/ppc -I$PANDA_ROOT/tcg/ppc -I$PANDA_ROOT/tcg -DNEED_CPU_H -I$PANDA_ROOT
bindgen bindings.h -o bindings/mips.rs --no-layout-tests -- -I$PANDA_ROOT/panda/include -I$PANDA_ROOT/build -I$PANDA_ROOT/build/mips-softmmu -I$PANDA_ROOT/include -I/usr/include/glib-2.0 -I/usr/lib/x86_64-linux-gnu/glib-2.0/include -I$PANDA_ROOT/target/mips -I$PANDA_ROOT/tcg/mips -I$PANDA_ROOT/tcg -DNEED_CPU_H -I$PANDA_ROOT -Ifake_headers
bindgen bindings.h -o bindings/mipsel.rs --no-layout-tests -- -I$PANDA_ROOT/panda/include -I$PANDA_ROOT/build -I$PANDA_ROOT/build/mipsel-softmmu -I$PANDA_ROOT/include -I/usr/include/glib-2.0 -I/usr/lib/x86_64-linux-gnu/glib-2.0/include -I$PANDA_ROOT/target/mips -I$PANDA_ROOT/tcg/mips -I$PANDA_ROOT/tcg -DNEED_CPU_H -I$PANDA_ROOT -Ifake_headers
bindgen bindings.h -o bindings/aarch64.rs --no-layout-tests -- -I$PANDA_ROOT/panda/include -I$PANDA_ROOT/build -I$PANDA_ROOT/build/aarch64-softmmu -I$PANDA_ROOT/include -I/usr/include/glib-2.0 -I/usr/lib/x86_64-linux-gnu/glib-2.0/include -I$PANDA_ROOT/target/arm -I$PANDA_ROOT/tcg/aarch64 -I$PANDA_ROOT/tcg -DNEED_CPU_H -I$PANDA_ROOT
bindgen bindings.h -o bindings/mips64.rs --no-layout-tests -- -I$PANDA_ROOT/panda/include -I$PANDA_ROOT/build -I$PANDA_ROOT/build/mips64-softmmu -I$PANDA_ROOT/include -I/usr/include/glib-2.0 -I/usr/lib/x86_64-linux-gnu/glib-2.0/include -I$PANDA_ROOT/target/mips -I$PANDA_ROOT/tcg/mips -I$PANDA_ROOT/tcg -DNEED_CPU_H -I$PANDA_ROOT -Ifake_headers

# Remove double-declared constant
sed '/pub const IPPORT_RESERVED: .* = 1024;/d' bindings/x86_64.rs -i
sed '/pub const IPPORT_RESERVED: .* = 1024;/d' bindings/i386.rs -i
sed '/pub const IPPORT_RESERVED: .* = 1024;/d' bindings/arm.rs -i
sed '/pub const IPPORT_RESERVED: .* = 1024;/d' bindings/ppc.rs -i
sed '/pub const IPPORT_RESERVED: .* = 1024;/d' bindings/mips.rs -i
sed '/pub const IPPORT_RESERVED: .* = 1024;/d' bindings/mipsel.rs -i
sed '/pub const IPPORT_RESERVED: .* = 1024;/d' bindings/aarch64.rs -i
sed '/pub const IPPORT_RESERVED: .* = 1024;/d' bindings/mips64.rs -i
