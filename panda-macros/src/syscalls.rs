define_syscalls_callbacks!{
    (on_sys_write_enter, add_callback_on_sys_write_enter, (fd: target_ulong, buf: target_ptr_t, count: target_ulong))
}
