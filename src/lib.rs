/// 定时事件枚举。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// 没有事件。
    None,
    /// 数据到达。
    Read,
    /// 目标可写。
    Write,
    /// 发生错误。
    Error,
    /// 边沿触发。
    EdgeTriggered,
    /// 已经挂起。
    HangUp,
    /// 单次触发。
    OneShot,
}

/// 定义事件集合。
///
/// # Examples
///
/// ```
/// use poller::Events;
/// let events = Events::new().with_read();
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Events(u32);

impl std::fmt::Display for Events {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:08X}", self.0)
    }
}

impl Events {
    /// 创建一个新的事件集合。
    pub fn new() -> Self {
        Self { 0: 0 }
    }

    /// 清空当前值且返回一个空事件集合。
    pub fn none(mut self) -> Self {
        self.0 = 0;
        self
    }

    /// 附加数据到达事件到集合中。
    pub fn read(mut self) -> Self {
        self.0 |= 1 << Event::Read as u32;
        self
    }

    /// 附加目标可写事件到集合中。
    pub fn write(mut self) -> Self {
        self.0 |= 1 << Event::Write as u32;
        self
    }

    /// 附加发生错误事件到集合中。
    pub fn error(mut self) -> Self {
        self.0 |= 1 << Event::Error as u32;
        self
    }

    /// 检查集合是否为空。
    pub fn is_none(self) -> bool {
        self.0 == 0
    }

    /// 检查集合是否有数据到达事件。
    pub fn has_read(self) -> bool {
        (self.0 & (1 << Event::Read as u32)) != 0
    }

    /// 检查集合是否有目标可写事件。
    pub fn has_write(self) -> bool {
        (self.0 & (1 << Event::Write as u32)) != 0
    }

    /// 检查集合是否有发生错误事件。
    pub fn has_error(self) -> bool {
        (self.0 & (1 << Event::Error as u32)) != 0
    }
}

/// 定义系统错误。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SysError(i32);

impl std::fmt::Display for SysError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, r#"Code={}, Reason="{{}}")"#, self.0)
    }
}

impl std::error::Error for SysError {}

impl From<i32> for SysError {
    fn from(val: i32) -> Self {
        Self { 0: val }
    }
}

impl Into<i32> for SysError {
    fn into(self) -> i32 {
        self.0
    }
}

impl SysError {
    /// 从系统当前 errno 创建一个 SysError 对象。
    pub fn last() -> Self {
        unsafe {
            Self {
                0: *(libc::__errno_location()),
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub mod epoll;

#[cfg(target_os = "linux")]
#[doc(inline)]
pub use epoll::{EventContext, EventData, Poller};

#[cfg(not(target_os = "linux"))]
pub mod select;

#[cfg(test)]
mod tests {}
