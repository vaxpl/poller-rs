//! Linux 增强型 I/O 事件通知。
//!
use crate::{Events, SysError};
use libc::{close, epoll_create1, epoll_ctl, epoll_wait};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

impl From<u32> for Events {
    fn from(val: u32) -> Self {
        let mut events = Events::new();
        if (val & libc::EPOLLIN as u32) == libc::EPOLLIN as u32 {
            events = events.read();
        }
        if (val & libc::EPOLLOUT as u32) == libc::EPOLLOUT as u32 {
            events = events.write();
        }
        if (val & libc::EPOLLERR as u32) == libc::EPOLLERR as u32 {
            events = events.error();
        }
        events
    }
}

impl Into<u32> for Events {
    fn into(self) -> u32 {
        let mut events = 0u32;
        if self.has_read() {
            events |= libc::EPOLLIN as u32;
        }
        if self.has_write() {
            events |= libc::EPOLLOUT as u32;
        }
        if self.has_error() {
            events |= libc::EPOLLERR as u32;
        }
        events
    }
}

/// 定义事件关联上下文。
pub type EventContext = Arc<dyn Any + Send + Sync>;

/// 定义事件数据。
///
/// # Fields
/// * `0` - 触发的文件描述符。
/// * `1` - 触发的事件集合。
/// * `2` - 触发的事件对应上下文。
pub type EventData<'a> = (i32, Events, Option<&'a EventContext>);

/// 定义文件 I/O 事件通知器。
///
/// 每个实例可以管理多个 `fd` 的 I/O 事件。
#[derive(Debug)]
pub struct Poller {
    epoll_fd: i32,
    watches: HashMap<i32, (Events, Option<EventContext>)>,
}

impl Default for Poller {
    fn default() -> Self {
        Self {
            epoll_fd: -1,
            watches: HashMap::new(),
        }
    }
}

impl Drop for Poller {
    fn drop(&mut self) {
        if self.epoll_fd > 0 {
            unsafe {
                close(self.epoll_fd);
            };
            self.epoll_fd = -1;
        }
    }
}

impl Poller {
    /// 创建一个新的 I/O 事件通知器。
    pub fn new() -> Result<Self, SysError> {
        let epoll_fd = unsafe { epoll_create1(0) };
        if epoll_fd < 0 {
            Err(SysError::last())
        } else {
            Ok(Self {
                epoll_fd,
                watches: HashMap::new(),
            })
        }
    }

    /// 添加一个文件描述符到监视列表中。
    ///
    /// **注意：** 此函数不会把 `fd` 的所有权转移到 `Poller` 内，请确保在 `Poller` 活动期内 `fd` 都是可用的。
    pub fn add(
        &mut self,
        fd: i32,
        events: Events,
        ctx: Option<EventContext>,
    ) -> Result<(), SysError> {
        unsafe {
            let mut ev = libc::epoll_event {
                events: events.into(),
                u64: fd as u64,
            };
            let err = epoll_ctl(self.epoll_fd, libc::EPOLL_CTL_ADD, fd, &mut ev);
            if err < 0 {
                return Err(SysError::last());
            }
            self.watches.insert(fd, (events, ctx));
            Ok(())
        }
    }

    /// 将一个文件描述符从监视列表中移除。
    pub fn remove(&mut self, fd: i32) -> Result<(), SysError> {
        if !self.watches.contains_key(&fd) {
            return Err(SysError::from(libc::ENOENT));
        }
        let err =
            unsafe { epoll_ctl(self.epoll_fd, libc::EPOLL_CTL_DEL, fd, std::ptr::null_mut()) };
        if err < 0 {
            Err(SysError::last())
        } else {
            self.watches.remove(&fd).unwrap();
            Ok(())
        }
    }

    /// 拉取所有被监测到的 I/O 事件。
    ///
    /// # Examples
    ///
    /// ```
    /// use poller::{Events, Poller};
    /// let mut poller = Poller::new().unwrap();
    /// poller.add(1, Events::new().write(), None).unwrap();
    /// for (fd, events, ctx) in poller.pull_events(1000).unwrap().iter() {
    ///     println!("Fd={}, Events={}, Context={:?}", fd, events, ctx);
    /// }
    /// ```
    pub fn pull_events(&self, timeout_ms: i32) -> Result<Vec<EventData>, SysError> {
        unsafe {
            let mut ev: Vec<libc::epoll_event> = Vec::with_capacity(self.watches.len());
            let nfds = epoll_wait(
                self.epoll_fd,
                ev.as_mut_ptr(),
                self.watches.len() as i32,
                timeout_ms,
            );
            if nfds < 0 {
                return Err(SysError::last());
            }
            ev.set_len(nfds as usize);
            Ok(ev
                .into_iter()
                .map(|x| {
                    if let Some(v) = self.watches.get(&(x.u64 as i32)) {
                        (x.u64 as i32, Events::from(x.events), v.1.as_ref())
                    } else {
                        (x.u64 as i32, Events::from(x.events), None)
                    }
                })
                .collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poller() {
        unsafe {
            let cstr = std::ffi::CString::new("/proc/uptime").unwrap();
            let fd = libc::open(cstr.as_ptr(), libc::O_RDONLY);
            let mut poller = Poller::new().unwrap();
            assert_eq!(poller.add(fd, Events::new().read(), None).is_ok(), true);
            for _ in 0..1000 {
                assert_eq!(poller.pull_events(1000).unwrap().len(), 1);
            }
            assert_eq!(poller.remove(fd).is_ok(), true);
            for _ in 0..1000 {
                assert_eq!(poller.add(fd, Events::new().read(), None).is_ok(), true);
                assert_eq!(poller.remove(fd).is_ok(), true);
            }
            libc::close(fd);
        }
    }
}
