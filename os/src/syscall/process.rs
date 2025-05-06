//! Process management syscalls
use crate::{mm::{translated_const_pointer, translated_mutable_pointer, MapPermission, PageTable, StepByOne, VirtAddr}, task::{change_program_brk, current_user_token, exit_current_and_run_next, get_current_memory_set, get_syscall_cnt, suspend_current_and_run_next}};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = crate::timer::get_time_us();
    let ts_user_pointer = translated_mutable_pointer::<TimeVal>(current_user_token(), _ts).unwrap();
    unsafe {
        *ts_user_pointer = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
// 如果 trace_request 为 0，则 id 应被视作 *const u8 ，表示读取当前任务 id 地址处一个字节的无符号整数值。此时应忽略 data 参数。返回值为 id 地址处的值。
//如果 trace_request 为 1，则 id 应被视作 *const u8 ，表示写入 data （作为 u8，即只考虑最低位的一个字节）到该用户程序 id 地址处。返回值应为0。
//如果 trace_request 为 2，表示查询当前任务调用编号为 id 的系统调用的次数，返回值为这个调用次数。本次调用也计入统计 。
//否则，忽略其他参数，返回值为 -1。
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    match _trace_request {
        0 => {
            match translated_const_pointer::<u8>(current_user_token(), _id as *const u8) {
                Ok(user_ptr) => {
                    unsafe {
                        return *user_ptr as isize;
                    }
                }
                Err(_) => {
                    error!("sys_trace: translated_const_pointer failed");
                    return -1;
                }
            }
        }
        1 => {
            match translated_mutable_pointer::<u8>(current_user_token(), _id as *mut u8) {
                Ok(user_ptr) => {
                    unsafe {
                        *user_ptr = _data as u8;
                    }
                    return 0;
                }
                Err(_) => {
                    error!("sys_trace: translated_mutable_pointer failed");
                    return -1;
                }
            }
        }
        2 => {
            return get_syscall_cnt(_id);
        }
        _ => {
            error!("Trace request {} not implemented", _trace_request);
        }
    }
    -1
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _prot: usize) -> isize {
    trace!("kernel: sys_mmap");
    if _prot & !0x7 !=0 {
        error!("sys_mmap: invalid prot");
        return -1;
    }
    if _prot & 0x7 ==0 {
        error!("sys_mmap: prot should be readable, writable or executable");
        return -1;
    }
    let start = VirtAddr::from(_start);
    if !start.aligned() {
        error!("sys_mmap: start address is not aligned");
        return -1;
    }
    let end = VirtAddr::from(_start + _len);
    
    let page_table = PageTable::from_token(current_user_token());
    let mut permission = MapPermission::empty();
    if _prot & 0x1 != 0 {
        permission |= MapPermission::R;
    }
    if _prot & 0x2 != 0 {
        permission |= MapPermission::W;
    }
    if _prot & 0x4 != 0 {
        permission |= MapPermission::X;
    }
    permission |= MapPermission::U;
    

    let mut start_temp = start;
    // check if address already mapped
    while start_temp<end {
        trace!("mmap: start:{:?}, end:{:?}",start,end);
        let mut vpn = start_temp.floor();
        if let Some(ppn) = page_table.translate(vpn) {
            if ppn.is_valid() {
                error!("sys_mmap: address already mapped");
                return -1;
            } 
            // error!("sys_mmap: address already mapped");
            // return -1;
        }
        vpn.step();
        let mut end_temp: VirtAddr = vpn.into();
        end_temp = end_temp.min(end);
        start_temp = end_temp;
    }
    let memory_set = get_current_memory_set();
    unsafe{
        (*memory_set).insert_framed_area(start, end, permission);
    }
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap");
    let start = VirtAddr::from(_start);
    if !start.aligned() {
        error!("sys_mmap: start address is not aligned");
        return -1;
    }
    let end = VirtAddr::from(_start + _len);
    
    let page_table = PageTable::from_token(current_user_token());

    let mut start_temp = start;
    // check if address already mapped
    while start_temp<end {
        let mut vpn = start_temp.floor();
        if let Some(ppn) = page_table.translate(vpn) {
            if !ppn.is_valid() {
                error!("sys_mmap: address not mapped");
                return -1;
            } 
            // error!("sys_mmap: address already mapped");
            // return -1;
        }
        vpn.step();
        let mut end_temp: VirtAddr = vpn.into();
        end_temp = end_temp.min(end);
        start_temp = end_temp;
    }
    let memory_set = get_current_memory_set();
    unsafe{
        (*memory_set).remove_framed_area(start, end);
    }
    0
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
