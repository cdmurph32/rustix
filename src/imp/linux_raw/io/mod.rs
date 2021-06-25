mod poll_fd;
mod types;

pub use poll_fd::{PollFd, PollFlags};
pub use types::{
    DupFlags, MapFlags, PipeFlags, ProtFlags, ReadWriteFlags, Tcflag, Termios, UserFaultFdFlags,
    Winsize, ICANON, PIPE_BUF,
};

use std::os::raw::{c_int, c_uint};

pub(crate) const AT_FDCWD: c_int = linux_raw_sys::general::AT_FDCWD;
pub(crate) const STDIN_FILENO: c_uint = linux_raw_sys::general::STDIN_FILENO;
pub(crate) const STDOUT_FILENO: c_uint = linux_raw_sys::general::STDOUT_FILENO;
pub(crate) const STDERR_FILENO: c_uint = linux_raw_sys::general::STDERR_FILENO;
