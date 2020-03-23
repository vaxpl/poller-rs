use libc::{input_event, timeval};
use poller::{Events, Poller};
use std::fs::File;
use std::io::Read;
use std::os::unix::io::AsRawFd;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct InputEvent {
    inner: input_event,
}

impl std::fmt::Debug for InputEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputEvent")
            .field("time", &TimeVal::from(self.inner.time))
            .field("type", &self.inner.type_)
            .field("code", &self.inner.code)
            .field("value", &self.inner.value)
            .finish()
    }
}
impl std::fmt::Display for InputEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "InputEvent {{ time: {}, type: {}, code: {}, value: {} }}",
            TimeVal::from(self.inner.time),
            self.inner.type_,
            self.inner.code,
            self.inner.value
        )
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct TimeVal {
    inner: timeval,
}

impl From<timeval> for TimeVal {
    fn from(val: timeval) -> Self {
        Self { inner: val }
    }
}

impl Into<timeval> for TimeVal {
    fn into(self) -> timeval {
        self.inner
    }
}

impl Into<(u32, u32)> for TimeVal {
    fn into(self) -> (u32, u32) {
        (self.inner.tv_sec as u32, self.inner.tv_usec as u32)
    }
}

impl Into<u64> for TimeVal {
    fn into(self) -> u64 {
        self.inner.tv_sec as u64 * 1_000_000u64 + self.inner.tv_usec as u64
    }
}

impl std::fmt::Debug for TimeVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TimeVal")
            .field("sec", &self.inner.tv_sec)
            .field("usec", &self.inner.tv_usec)
            .finish()
    }
}

impl std::fmt::Display for TimeVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{number:>0width$}",
            self.inner.tv_sec,
            number = self.inner.tv_usec,
            width = 6
        )
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut evdev = File::open("/dev/input/event5")?; // Mouse evdev
    let mut poller = Poller::new();
    poller.add(evdev.as_raw_fd(), Events::new().with_read())?;
    const N: usize = std::mem::size_of::<input_event>();
    let mut buf: [u8; N] = [0; N];
    loop {
        let events = poller.pull_events(1000)?;
        for (_fd, _events) in events.iter() {
            evdev.read_exact(&mut buf)?;
            let a = unsafe { std::mem::transmute::<&[u8; N], &InputEvent>(&buf) };
            println!("{}", a);
        }
    }
}
