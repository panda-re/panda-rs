define_hooks2_callbacks!{
    fn(add_callback_on_process_start) on_process_start(
        cpu: &mut CPUState,
        procname: *const c_char,
        asid: target_ulong,
        pid: target_pid_t,
    );

    fn(add_callback_on_process_end) on_process_end(
        cpu: &mut CPUState,
        procname: *const c_char,
        asid: target_ulong,
        pid: target_pid_t,
    );

    fn(add_callback_on_thread_start) on_thread_start(
        cpu: &mut CPUState,
        procname: *const c_char,
        asid: target_ulong,
        pid: target_pid_t,
        tid: target_pid_t,
    );

    fn(add_callback_on_thread_end) on_thread_end(
        cpu: &mut CPUState,
        procname: *const c_char,
        asid: target_ulong,
        pid: target_pid_t,
        tid: target_pid_t,
    );

    fn(add_callback_on_mmap_updated) on_mmap_updated(
        cpu: &mut CPUState,
        libname: *const c_char,
        base: target_ulong,
        size: target_ulong,
    );
}
