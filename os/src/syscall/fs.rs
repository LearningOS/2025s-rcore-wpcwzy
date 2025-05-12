//! File and filesystem-related syscalls
use crate::drivers::BLOCK_DEVICE;
use crate::fs::{open_file, OpenFlags, Stat, OSInode};
use crate::mm::{translated_byte_buffer, translated_mutable_pointer, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token};
use alloc::sync::Arc;
use easy_fs::{EasyFileSystem, Inode};
use lazy_static::*;

lazy_static! {
    pub static ref ROOT_INODE: Arc<Inode> = {
        let efs = EasyFileSystem::open(BLOCK_DEVICE.clone());
        Arc::new(EasyFileSystem::root_inode(&efs))
    };
}

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_write", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_read", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        trace!("kernel: sys_read .. file.read");
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    trace!("kernel:pid[{}] sys_open", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    trace!("kernel:pid[{}] sys_close", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

/// YOUR JOB: Implement fstat.
pub fn sys_fstat(fd: usize, st: *mut Stat) -> isize {
    trace!(
        "kernel:pid[{}] sys_fstat",
        current_task().unwrap().pid.0
    );
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    let file_arc = inner.fd_table[fd].as_ref().expect("fd_table entry should be Some after check");
    // Dereference Arc to get &dyn File, then call as_any(), instructed by gemini pro 2.5
    if let Some(os_inode) = (**file_arc).as_any().downcast_ref::<OSInode>() {
        trace!("kernel:pid[{}] sys_fstat - successfully downcasted fd {} to OSInode", current_task().unwrap().pid.0, fd);
        let stat = os_inode.stat();
        drop(inner); // Drop the inner lock before translating the pointer to avoid deadlock
        match translated_mutable_pointer(current_user_token(), st) {
            Ok(st) => {
                trace!("kernel:pid[{}] sys_fstat - successfully translated mutable pointer", current_task().unwrap().pid.0);
                unsafe { *st = stat };
            }
            Err(_) => {
                trace!("kernel:pid[{}] sys_fstat - failed to translate mutable pointer", current_task().unwrap().pid.0);
                return -1;
            }
        }
        return 0;
    } else {
        trace!("kernel:pid[{}] sys_fstat - failed to downcast fd {} to OSInode", current_task().unwrap().pid.0, fd);
        return -1;
    }
}

/// YOUR JOB: Implement linkat.
pub fn sys_linkat(_old_name: *const u8, new_name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_linkat NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let token = current_user_token();
    let new_path = translated_str(token, new_name);
    let old_path = translated_str(token, _old_name);
    if new_path == old_path {
        error!("sys_linkat:pid[{}] sys_linkat - new_path is same as old_path", current_task().unwrap().pid.0);
        return -1;
    }
    if let Some(_) = ROOT_INODE.link(old_path.as_str(), new_path.as_str()) {
        trace!("sys_linkat:pid[{}] sys_linkat - successfully linked {} to {}", current_task().unwrap().pid.0, old_path, new_path);
        return 0;
    } else {
        error!("sys_linkat:pid[{}] sys_linkat - failed to link {} to {}", current_task().unwrap().pid.0, old_path, new_path);
        return -1;
    }
}

/// YOUR JOB: Implement unlinkat.
pub fn sys_unlinkat(name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_unlinkat NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let path = translated_str(current_user_token(), name);
    let ret = ROOT_INODE.unlink(path.as_str());
    ret
}
