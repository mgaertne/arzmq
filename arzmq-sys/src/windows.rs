pub use std::os::windows::io::RawSocket as RawFd;

pub mod errno {
    // Use constants as defined in the windows header errno.h
    // libzmq should be compiled with VS2010 SDK headers or newer
    pub const EACCES: i32 = 13;
    pub const EADDRINUSE: i32 = 100;
    pub const EADDRNOTAVAIL: i32 = 101;
    pub const EAGAIN: i32 = 11;
    pub const EBUSY: i32 = 16;
    pub const ECONNREFUSED: i32 = 107;
    pub const EFAULT: i32 = 14;
    pub const EINTR: i32 = 4;
    pub const EHOSTUNREACH: i32 = 110;
    pub const ENETRESET: i32 = 117;
    pub const EINPROGRESS: i32 = 112;
    pub const EINVAL: i32 = 22;
    pub const EMFILE: i32 = 24;
    pub const EMSGSIZE: i32 = 115;
    pub const EAFNOSUPPORT: i32 = 102;
    pub const ENETUNREACH: i32 = 118;
    pub const ECONNABORTED: i32 = 106;
    pub const ECONNRESET: i32 = 108;
    pub const ENAMETOOLONG: i32 = 38;
    pub const ENETDOWN: i32 = 116;
    pub const ENOBUFS: i32 = 119;
    pub const ENODEV: i32 = 19;
    pub const ENOENT: i32 = 2;
    pub const ENOMEM: i32 = 12;
    pub const ENOTCONN: i32 = 126;
    pub const ETIMEDOUT: i32 = 138;
    pub const ENOTSOCK: i32 = 128;
    pub const ENOTSUP: i32 = 129;
    pub const EPROTO: i32 = 134;
    pub const EPROTONOSUPPORT: i32 = 135;
}
