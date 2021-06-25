//! POSIX-style `*at` functions.

#[cfg(any(target_os = "macos", target_os = "ios"))]
use crate::fs::CloneFlags;
use crate::{imp, io, path};
#[cfg(not(any(
    target_os = "redox",
    target_os = "wasi",
    target_os = "macos",
    target_os = "ios"
)))]
use imp::fs::Dev;
use imp::{
    fs::{Access, AtFlags, Mode, OFlags, Stat},
    time::Timespec,
};
use io_lifetimes::{AsFd, BorrowedFd, OwnedFd};
use std::ffi::{CStr, OsString};
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;
#[cfg(target_os = "wasi")]
use std::os::wasi::ffi::OsStringExt;

/// `openat(dirfd, path, oflags, mode)`
#[inline]
pub fn openat<P: path::Arg, Fd: AsFd>(
    dirfd: &Fd,
    path: P,
    oflags: OFlags,
    mode: Mode,
) -> io::Result<OwnedFd> {
    let dirfd = dirfd.as_fd();
    path.into_with_c_str(|path| imp::syscalls::openat(dirfd, &path, oflags, mode))
}

/// `readlinkat(fd, path)`
///
/// If `reuse` is non-empty, reuse its buffer to store the result if possible.
#[inline]
pub fn readlinkat<P: path::Arg, Fd: AsFd>(
    dirfd: &Fd,
    path: P,
    reuse: OsString,
) -> io::Result<OsString> {
    let dirfd = dirfd.as_fd();
    path.into_with_c_str(|path| _readlinkat(dirfd, &path, reuse))
}

fn _readlinkat(dirfd: BorrowedFd<'_>, path: &CStr, reuse: OsString) -> io::Result<OsString> {
    // This code would benefit from having a better way to read into
    // uninitialized memory, but that requires `unsafe`.
    let mut buffer = reuse.into_vec();
    buffer.clear();
    buffer.resize(256, 0u8);

    loop {
        let nread = imp::syscalls::readlinkat(dirfd, path, &mut buffer)?;

        let nread = nread as usize;
        assert!(nread <= buffer.len());
        if nread < buffer.len() {
            buffer.resize(nread, 0u8);
            return Ok(OsString::from_vec(buffer));
        }
        buffer.resize(buffer.len() * 2, 0u8);
    }
}

/// `mkdirat(fd, path, mode)`
#[inline]
pub fn mkdirat<P: path::Arg, Fd: AsFd>(dirfd: &Fd, path: P, mode: Mode) -> io::Result<()> {
    let dirfd = dirfd.as_fd();
    path.into_with_c_str(|path| imp::syscalls::mkdirat(dirfd, &path, mode))
}

/// `linkat(old_dirfd, old_path, new_dirfd, new_path, flags)`
#[inline]
pub fn linkat<P: path::Arg, Q: path::Arg, PFd: AsFd, QFd: AsFd>(
    old_dirfd: &PFd,
    old_path: P,
    new_dirfd: &QFd,
    new_path: Q,
    flags: AtFlags,
) -> io::Result<()> {
    let old_dirfd = old_dirfd.as_fd();
    let new_dirfd = new_dirfd.as_fd();
    old_path.into_with_c_str(|old_path| {
        new_path.into_with_c_str(|new_path| {
            imp::syscalls::linkat(old_dirfd, &old_path, new_dirfd, &new_path, flags)
        })
    })
}

/// `unlinkat(fd, path, flags)`
#[inline]
pub fn unlinkat<P: path::Arg, Fd: AsFd>(dirfd: &Fd, path: P, flags: AtFlags) -> io::Result<()> {
    let dirfd = dirfd.as_fd();
    path.into_with_c_str(|path| imp::syscalls::unlinkat(dirfd, &path, flags))
}

/// `renameat(old_dirfd, old_path, new_dirfd, new_path)`
#[inline]
pub fn renameat<P: path::Arg, Q: path::Arg, PFd: AsFd, QFd: AsFd>(
    old_dirfd: &PFd,
    old_path: P,
    new_dirfd: &QFd,
    new_path: Q,
) -> io::Result<()> {
    let old_dirfd = old_dirfd.as_fd();
    let new_dirfd = new_dirfd.as_fd();
    old_path.into_with_c_str(|old_path| {
        new_path.into_with_c_str(|new_path| {
            imp::syscalls::renameat(old_dirfd, &old_path, new_dirfd, &new_path)
        })
    })
}

/// `symlinkat(old_dirfd, old_path, new_dirfd, new_path)`
#[inline]
pub fn symlinkat<P: path::Arg, Q: path::Arg, Fd: AsFd>(
    old_path: P,
    new_dirfd: &Fd,
    new_path: Q,
) -> io::Result<()> {
    let new_dirfd = new_dirfd.as_fd();
    old_path.into_with_c_str(|old_path| {
        new_path
            .into_with_c_str(|new_path| imp::syscalls::symlinkat(&old_path, new_dirfd, &new_path))
    })
}

/// `fstatat(dirfd, path, flags)`
#[inline]
#[doc(alias = "fstatat")]
pub fn statat<P: path::Arg, Fd: AsFd>(dirfd: &Fd, path: P, flags: AtFlags) -> io::Result<Stat> {
    let dirfd = dirfd.as_fd();
    path.into_with_c_str(|path| imp::syscalls::statat(dirfd, &path, flags))
}

/// `faccessat(dirfd, path, access, flags)`
#[inline]
#[doc(alias = "faccessat")]
pub fn accessat<P: path::Arg, Fd: AsFd>(
    dirfd: &Fd,
    path: P,
    access: Access,
    flags: AtFlags,
) -> io::Result<()> {
    let dirfd = dirfd.as_fd();
    path.into_with_c_str(|path| imp::syscalls::accessat(dirfd, &path, access, flags))
}

/// `utimensat(dirfd, path, times, flags)`
#[inline]
pub fn utimensat<P: path::Arg, Fd: AsFd>(
    dirfd: &Fd,
    path: P,
    times: &[Timespec; 2],
    flags: AtFlags,
) -> io::Result<()> {
    let dirfd = dirfd.as_fd();
    path.into_with_c_str(|path| imp::syscalls::utimensat(dirfd, &path, times, flags))
}

/// `fchmodat(dirfd, path, mode, 0)`
///
/// The flags argument is fixed to 0, so `AT_SYMLINK_NOFOLLOW` is not
/// supported. <details>
/// Platform support for this flag varies widely.
/// </details>
///
/// Note that this implementation does not support `O_PATH` file descriptors,
/// even on platforms where the host libc emulates it.
#[cfg(not(target_os = "wasi"))]
#[inline]
#[doc(alias = "fchmodat")]
pub fn chmodat<P: path::Arg, Fd: AsFd>(dirfd: &Fd, path: P, mode: Mode) -> io::Result<()> {
    let dirfd = dirfd.as_fd();
    path.into_with_c_str(|path| imp::syscalls::chmodat(dirfd, &path, mode))
}

/// `fclonefileat(src, dst_dir, dst, flags)`
#[cfg(any(target_os = "macos", target_os = "ios"))]
#[inline]
pub fn fclonefileat<Fd: AsFd, DstFd: AsFd, P: path::Arg>(
    src: &Fd,
    dst_dir: &DstFd,
    dst: P,
    flags: CloneFlags,
) -> io::Result<()> {
    let srcfd = src.as_fd();
    let dst_dirfd = dst_dir.as_fd();
    dst.into_with_c_str(|dst| {
        imp::syscalls::fclonefileat(srcfd.as_fd(), dst_dirfd.as_fd(), &dst, flags)
    })
}

/// `mknodat(dirfd, path, mode, dev)`
#[cfg(not(any(
    target_os = "redox",
    target_os = "wasi",
    target_os = "macos",
    target_os = "ios"
)))]
#[inline]
pub fn mknodat<P: path::Arg, Fd: AsFd>(
    dirfd: &Fd,
    path: P,
    mode: Mode,
    dev: Dev,
) -> io::Result<()> {
    let dirfd = dirfd.as_fd();
    path.into_with_c_str(|path| imp::syscalls::mknodat(dirfd, &path, mode, dev))
}
