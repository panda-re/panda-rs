#/bin/bash
cargo build && \
    cp target/debug/libpanda_rs_example.so $PANDA_PATH/x86_64-softmmu/panda/plugins/panda_panda_rs_example.so && \
    $PANDA_PATH/x86_64-softmmu/panda-system-x86_64 -os "linux-64-ubuntu:4.15.0-72-generic-noaslr-nokaslr" -replay test -panda panda_rs_example -m 1G
