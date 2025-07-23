use alloc::ffi::CString;
use core::{
    ffi::{CStr, c_long, c_void},
    fmt::Formatter,
    hint::cold_path,
    ops::Deref,
    ptr, slice,
    str::FromStr,
};

use derive_more::{Debug as DebugDeriveMore, Display as DisplayDeriveMore};
use num_traits::PrimInt;
use parking_lot::FairMutex;

use crate::{ZmqError, ZmqResult, socket::PollEvents, zmq_sys_crate};

const MAX_OPTION_STR_LEN: usize = i32::MAX as usize;

#[derive(DisplayDeriveMore, DebugDeriveMore)]
#[debug("RawContext {{ ... }}")]
#[display("RawContext")]
pub(crate) struct RawContext {
    context: FairMutex<*mut c_void>,
}

impl RawContext {
    pub(crate) fn new() -> ZmqResult<Self> {
        match unsafe { zmq_sys_crate::zmq_ctx_new() } {
            null_ptr if null_ptr.is_null() => {
                cold_path();
                match unsafe { zmq_sys_crate::zmq_errno() } {
                    errno @ zmq_sys_crate::errno::EMFILE => Err(ZmqError::from(errno)),
                    _ => unreachable!(),
                }
            }
            context => Ok(RawContext {
                context: FairMutex::new(context),
            }),
        }
    }

    fn set(&self, option: i32, value: i32) -> ZmqResult<()> {
        let context = self.context.lock();
        if unsafe { zmq_sys_crate::zmq_ctx_set(*context, option, value) } == -1 {
            cold_path();
            match unsafe { zmq_sys_crate::zmq_errno() } {
                errno @ (zmq_sys_crate::errno::EINVAL | zmq_sys_crate::errno::EFAULT) => {
                    return Err(ZmqError::from(errno));
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    fn set_ext(&self, option: i32, value: &str) -> ZmqResult<()> {
        let c_value = CString::from_str(value)?;

        let context = self.context.lock();
        if unsafe {
            zmq_sys_crate::zmq_ctx_set_ext(
                *context,
                option,
                c_value.as_ptr() as *const c_void,
                value.len(),
            )
        } == -1
        {
            cold_path();
            match unsafe { zmq_sys_crate::zmq_errno() } {
                errno @ (zmq_sys_crate::errno::EINVAL | zmq_sys_crate::errno::EFAULT) => {
                    return Err(ZmqError::from(errno));
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub(crate) fn set_ctxopt_string(&self, option: i32, value: &str) -> ZmqResult<()> {
        self.set_ext(option, value)
    }

    pub(crate) fn set_ctxopt_bool(&self, option: i32, value: bool) -> ZmqResult<()> {
        let value = if value { 1 } else { 0 };
        self.set(option, value)
    }

    pub(crate) fn set_ctxopt_int<V>(&self, option: i32, value: V) -> ZmqResult<()>
    where
        V: PrimInt + Into<i32>,
    {
        self.set(option, value.into())
    }

    fn get(&self, option: i32) -> ZmqResult<i32> {
        let context = self.context.lock();
        match unsafe { zmq_sys_crate::zmq_ctx_get(*context, option) } {
            -1 => match unsafe { zmq_sys_crate::zmq_errno() } {
                errno @ (zmq_sys_crate::errno::EINVAL | zmq_sys_crate::errno::EFAULT) => {
                    Err(ZmqError::from(errno))
                }
                _ => unreachable!(),
            },
            value => Ok(value),
        }
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub(crate) fn get_ext(&self, option: i32) -> ZmqResult<String> {
        let mut buffer: [u8; MAX_OPTION_STR_LEN] = [0; MAX_OPTION_STR_LEN];

        let context = self.context.lock();
        if unsafe {
            zmq_sys_crate::zmq_ctx_get_ext(
                *context,
                option,
                buffer.as_mut_ptr() as *mut c_void,
                &mut buffer.len(),
            )
        } == -1
        {
            cold_path();
            match unsafe { zmq_sys_crate::zmq_errno() } {
                errno @ (zmq_sys_crate::errno::EINVAL | zmq_sys_crate::errno::EFAULT) => {
                    return Err(ZmqError::from(errno));
                }
                _ => unreachable!(),
            }
        }
        CStr::from_bytes_until_nul(&buffer)?
            .to_owned()
            .into_string()
            .map_err(ZmqError::from)
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub(crate) fn get_ctxopt_string(&self, option: i32) -> ZmqResult<String> {
        self.get_ext(option)
    }

    pub(crate) fn get_ctxpt_bool(&self, option: i32) -> ZmqResult<bool> {
        self.get(option).map(|value| value != 0)
    }

    pub(crate) fn get_ctxopt_int<V>(&self, option: i32) -> ZmqResult<V>
    where
        V: PrimInt + From<i32>,
    {
        self.get(option).map(|value| value.into())
    }

    pub(crate) fn shutdown(&self) -> ZmqResult<()> {
        let context = self.context.lock();
        match unsafe { zmq_sys_crate::zmq_ctx_shutdown(*context) } {
            -1 => {
                cold_path();
                match unsafe { zmq_sys_crate::zmq_errno() } {
                    errno @ zmq_sys_crate::errno::EFAULT => Err(ZmqError::from(errno)),
                    _ => unreachable!(),
                }
            }
            _ => Ok(()),
        }
    }

    pub(crate) fn terminate(&self) {
        let context = self.context.lock();
        while unsafe { zmq_sys_crate::zmq_ctx_term(*context) } != 0 {
            cold_path();
            match unsafe { zmq_sys_crate::zmq_errno() } {
                zmq_sys_crate::errno::EINTR => (),
                zmq_sys_crate::errno::ETERM => break,
                _ => unreachable!(),
            }
        }
    }
}

impl Drop for RawContext {
    fn drop(&mut self) {
        self.terminate()
    }
}

pub(crate) struct RawSocket {
    pub(crate) socket: FairMutex<*mut c_void>,
}

impl RawSocket {
    pub(crate) fn from_ctx(context: &RawContext, socket_type: i32) -> ZmqResult<Self> {
        let context_guard = context.context.lock();
        let socket_ptr = unsafe { zmq_sys_crate::zmq_socket(*context_guard, socket_type) };
        if socket_ptr.is_null() {
            cold_path();
            match unsafe { zmq_sys_crate::zmq_errno() } {
                errno @ (zmq_sys_crate::errno::EINVAL
                | zmq_sys_crate::errno::EFAULT
                | zmq_sys_crate::errno::EMFILE
                | zmq_sys_crate::errno::ETERM) => return Err(ZmqError::from(errno)),
                _ => unreachable!(),
            }
        }

        drop(context_guard);
        Ok(Self {
            socket: FairMutex::new(socket_ptr),
        })
    }

    pub(crate) fn connect(&self, endpoint: &str) -> ZmqResult<()> {
        let c_endpoint = CString::from_str(endpoint)?;

        let socket_guard = self.socket.lock();
        if unsafe { zmq_sys_crate::zmq_connect(*socket_guard, c_endpoint.as_ptr()) } == -1 {
            cold_path();
            match unsafe { zmq_sys_crate::zmq_errno() } {
                errno @ (zmq_sys_crate::errno::EINVAL
                | zmq_sys_crate::errno::EPROTONOSUPPORT
                | zmq_sys_crate::errno::ENOCOMPATPROTO
                | zmq_sys_crate::errno::ETERM
                | zmq_sys_crate::errno::ENOTSOCK
                | zmq_sys_crate::errno::EMTHREAD) => {
                    return Err(ZmqError::from(errno));
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    pub(crate) fn disconnect(&self, endpoint: &str) -> ZmqResult<()> {
        let c_endpoint = CString::from_str(endpoint)?;

        let socket_guard = self.socket.lock();
        if unsafe { zmq_sys_crate::zmq_disconnect(*socket_guard, c_endpoint.as_ptr()) } == -1 {
            cold_path();
            match unsafe { zmq_sys_crate::zmq_errno() } {
                errno @ (zmq_sys_crate::errno::EINVAL
                | zmq_sys_crate::errno::ETERM
                | zmq_sys_crate::errno::ENOTSOCK
                | zmq_sys_crate::errno::ENOENT) => return Err(ZmqError::from(errno)),
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    pub(crate) fn bind(&self, endpoint: &str) -> ZmqResult<()> {
        let c_endpoint = CString::from_str(endpoint)?;

        let socket_guard = self.socket.lock();
        if unsafe { zmq_sys_crate::zmq_bind(*socket_guard, c_endpoint.as_ptr()) } == -1 {
            cold_path();
            match unsafe { zmq_sys_crate::zmq_errno() } {
                errno @ (zmq_sys_crate::errno::EINVAL
                | zmq_sys_crate::errno::EPROTONOSUPPORT
                | zmq_sys_crate::errno::ENOCOMPATPROTO
                | zmq_sys_crate::errno::EADDRINUSE
                | zmq_sys_crate::errno::EADDRNOTAVAIL
                | zmq_sys_crate::errno::ENODEV
                | zmq_sys_crate::errno::ETERM
                | zmq_sys_crate::errno::ENOTSOCK
                | zmq_sys_crate::errno::EMTHREAD) => {
                    return Err(ZmqError::from(errno));
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    pub(crate) fn unbind(&self, endpoint: &str) -> ZmqResult<()> {
        let c_endpoint = CString::from_str(endpoint)?;

        let socket_guard = self.socket.lock();
        if unsafe { zmq_sys_crate::zmq_unbind(*socket_guard, c_endpoint.as_ptr()) } == -1 {
            cold_path();
            match unsafe { zmq_sys_crate::zmq_errno() } {
                errno @ (zmq_sys_crate::errno::EINVAL
                | zmq_sys_crate::errno::ETERM
                | zmq_sys_crate::errno::ENOTSOCK
                | zmq_sys_crate::errno::ENOENT) => return Err(ZmqError::from(errno)),
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    fn get_sockopt(&self, option: i32, value_ptr: *mut c_void, size: &mut usize) -> ZmqResult<()> {
        let socket = self.socket.lock();
        if unsafe { zmq_sys_crate::zmq_getsockopt(*socket, option, value_ptr, size) } == -1 {
            cold_path();
            match unsafe { zmq_sys_crate::zmq_errno() } {
                errno @ (zmq_sys_crate::errno::EINVAL
                | zmq_sys_crate::errno::ETERM
                | zmq_sys_crate::errno::ENOTSOCK
                | zmq_sys_crate::errno::EINTR) => {
                    return Err(ZmqError::from(errno));
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    pub(crate) fn get_sockopt_bytes(&self, option: i32) -> ZmqResult<Vec<u8>> {
        let mut buffer = vec![0; MAX_OPTION_STR_LEN];

        self.get_sockopt(
            option,
            buffer.as_mut_ptr() as *mut c_void,
            &mut buffer.len(),
        )?;

        buffer.truncate(buffer.len());
        Ok(buffer)
    }

    pub(crate) fn get_sockopt_string(&self, option: i32) -> ZmqResult<String> {
        let value = self.get_sockopt_bytes(option)?;

        if value.is_empty() {
            return Ok(String::new());
        }

        CStr::from_bytes_until_nul(&value)?
            .to_owned()
            .into_string()
            .map_err(ZmqError::from)
    }

    pub(crate) fn get_sockopt_int<V>(&self, option: i32) -> ZmqResult<V>
    where
        V: PrimInt + Default,
    {
        let mut value = V::default();
        let mut size = size_of::<V>();
        let value_ptr = &mut value as *mut V as *mut c_void;

        self.get_sockopt(option, value_ptr, &mut size)?;

        Ok(value)
    }

    pub(crate) fn get_sockopt_bool(&self, option: i32) -> ZmqResult<bool> {
        let mut value = 0;
        let mut size = size_of::<i32>();
        let value_ptr = &mut value as *mut i32 as *mut c_void;

        self.get_sockopt(option, value_ptr, &mut size)?;

        Ok(value > 0)
    }

    fn set_sockopt(&self, option: i32, value_ptr: *const c_void, size: usize) -> ZmqResult<()> {
        let socket = self.socket.lock();
        if unsafe { zmq_sys_crate::zmq_setsockopt(*socket, option, value_ptr, size) } == -1 {
            cold_path();
            match unsafe { zmq_sys_crate::zmq_errno() } {
                errno @ (zmq_sys_crate::errno::EINVAL
                | zmq_sys_crate::errno::ETERM
                | zmq_sys_crate::errno::ENOTSOCK
                | zmq_sys_crate::errno::EINTR) => {
                    return Err(ZmqError::from(errno));
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    pub(crate) fn set_sockopt_bytes(&self, option: i32, value: &[u8]) -> ZmqResult<()> {
        self.set_sockopt(option, value.as_ptr() as *const c_void, value.len())
    }

    pub(crate) fn set_sockopt_string(&self, option: i32, value: &str) -> ZmqResult<()> {
        let c_value = CString::from_str(value)?;
        self.set_sockopt_bytes(option, c_value.as_bytes())
    }

    pub(crate) fn set_sockopt_bool(&self, option: i32, value: bool) -> ZmqResult<()> {
        let value = if value { 1 } else { 0 };
        self.set_sockopt(
            option,
            &value as *const i32 as *const c_void,
            size_of::<i32>(),
        )
    }

    pub(crate) fn set_sockopt_int<V>(&self, option: i32, value: V) -> ZmqResult<()>
    where
        V: PrimInt,
    {
        self.set_sockopt(option, &value as *const V as *const c_void, size_of::<V>())
    }

    pub(crate) fn monitor(&self, endpoint: &str, event: i32) -> ZmqResult<()> {
        let c_endpoint = CString::from_str(endpoint)?;

        let socket_guard = self.socket.lock();
        if unsafe { zmq_sys_crate::zmq_socket_monitor(*socket_guard, c_endpoint.as_ptr(), event) }
            == -1
        {
            cold_path();
            match unsafe { zmq_sys_crate::zmq_errno() } {
                errno @ (zmq_sys_crate::errno::ETERM
                | zmq_sys_crate::errno::EPROTONOSUPPORT
                | zmq_sys_crate::errno::EINVAL) => {
                    return Err(ZmqError::from(errno));
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    pub(crate) fn send(&self, zmq_msg: &mut RawMessage, flags: i32) -> ZmqResult<()> {
        let socket_guard = self.socket.lock();

        if unsafe { zmq_sys_crate::zmq_msg_send(&mut zmq_msg.message, *socket_guard, flags) } == -1
        {
            match unsafe { zmq_sys_crate::zmq_errno() } {
                errno @ (zmq_sys_crate::errno::EAGAIN
                | zmq_sys_crate::errno::ENOTSUP
                | zmq_sys_crate::errno::EINVAL
                | zmq_sys_crate::errno::EFSM
                | zmq_sys_crate::errno::ETERM
                | zmq_sys_crate::errno::ENOTSOCK
                | zmq_sys_crate::errno::EINTR
                | zmq_sys_crate::errno::EHOSTUNREACH) => {
                    return Err(ZmqError::from(errno));
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    pub(crate) fn recv(&self, flags: i32) -> ZmqResult<RawMessage> {
        let mut msg = RawMessage::new();
        {
            let socket_guard = self.socket.lock();

            if unsafe { zmq_sys_crate::zmq_msg_recv(&mut msg.message, *socket_guard, flags) } == -1
            {
                match unsafe { zmq_sys_crate::zmq_errno() } {
                    errno @ (zmq_sys_crate::errno::EAGAIN
                    | zmq_sys_crate::errno::ENOTSUP
                    | zmq_sys_crate::errno::EFSM
                    | zmq_sys_crate::errno::ETERM
                    | zmq_sys_crate::errno::ENOTSOCK
                    | zmq_sys_crate::errno::EINTR
                    | zmq_sys_crate::errno::EFAULT) => {
                        return Err(ZmqError::from(errno));
                    }
                    _ => unreachable!(),
                }
            }
        }

        Ok(msg)
    }

    pub(crate) fn poll(&self, events: PollEvents, timeout_ms: i64) -> ZmqResult<i32> {
        let poll_item = RawPollItem::from_socket(self, events);

        let mut poll_item_guard = poll_item.item.lock();
        match unsafe {
            zmq_sys_crate::zmq_poll(
                &mut *poll_item_guard as *mut zmq_sys_crate::zmq_pollitem_t,
                1,
                timeout_ms as c_long,
            )
        } {
            -1 => match unsafe { zmq_sys_crate::zmq_errno() } {
                errno @ (zmq_sys_crate::errno::ETERM
                | zmq_sys_crate::errno::EFAULT
                | zmq_sys_crate::errno::EINTR) => Err(ZmqError::from(errno)),
                _ => unreachable!(),
            },
            num_events => Ok(num_events),
        }
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub(crate) fn connect_peer(&self, endpoint: &str) -> ZmqResult<u32> {
        let c_endpoint = CString::from_str(endpoint)?;

        let socket_guard = self.socket.lock();
        let routing_id =
            unsafe { zmq_sys_crate::zmq_connect_peer(*socket_guard, c_endpoint.as_ptr()) };
        if routing_id == 0 {
            cold_path();
            match unsafe { zmq_sys_crate::zmq_errno() } {
                errno @ (zmq_sys_crate::errno::EINVAL
                | zmq_sys_crate::errno::EPROTONOSUPPORT
                | zmq_sys_crate::errno::ENOCOMPATPROTO
                | zmq_sys_crate::errno::ETERM
                | zmq_sys_crate::errno::ENOTSOCK
                | zmq_sys_crate::errno::EMTHREAD
                | zmq_sys_crate::errno::ENOTSUP
                | zmq_sys_crate::errno::EFAULT) => {
                    return Err(ZmqError::from(errno));
                }
                _ => unreachable!(),
            }
        }

        Ok(routing_id)
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub(crate) fn join(&self, group: &str) -> ZmqResult<()> {
        let c_group = CString::from_str(group)?;

        let socket_guard = self.socket.lock();
        if unsafe { zmq_sys_crate::zmq_join(*socket_guard, c_group.as_ptr()) } == -1 {
            cold_path();
            match unsafe { zmq_sys_crate::zmq_errno() } {
                errno @ (zmq_sys_crate::errno::ETERM
                | zmq_sys_crate::errno::EINTR
                | zmq_sys_crate::errno::ENOTSOCK
                | zmq_sys_crate::errno::EMTHREAD) => return Err(ZmqError::from(errno)),
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub(crate) fn leave(&self, group: &str) -> ZmqResult<()> {
        let c_group = CString::from_str(group)?;

        let socket_guard = self.socket.lock();
        if unsafe { zmq_sys_crate::zmq_leave(*socket_guard, c_group.as_ptr()) } == -1 {
            cold_path();
            match unsafe { zmq_sys_crate::zmq_errno() } {
                errno @ (zmq_sys_crate::errno::ETERM
                | zmq_sys_crate::errno::EINTR
                | zmq_sys_crate::errno::ENOTSOCK
                | zmq_sys_crate::errno::EMTHREAD) => return Err(ZmqError::from(errno)),
                _ => unreachable!(),
            }
        }

        Ok(())
    }
}

impl Drop for RawSocket {
    fn drop(&mut self) {
        let socket_guard = self.socket.lock();
        if unsafe { zmq_sys_crate::zmq_close(*socket_guard) } == -1 {
            match unsafe { zmq_sys_crate::zmq_errno() } {
                zmq_sys_crate::errno::ENOTSOCK => (),
                _ => unreachable!(),
            }
        }
    }
}

pub(crate) struct RawMessage {
    message: zmq_sys_crate::zmq_msg_t,
}

impl RawMessage {
    fn alloc<F>(func: F) -> RawMessage
    where
        F: FnOnce(&mut zmq_sys_crate::zmq_msg_t) -> i32,
    {
        let mut message = zmq_sys_crate::zmq_msg_t::default();
        if func(&mut message) == -1 {
            panic!("failed to allocate message");
        }

        Self { message }
    }

    pub(crate) fn new() -> Self {
        Self::alloc(|msg| unsafe { zmq_sys_crate::zmq_msg_init(msg) })
    }

    fn with_size_uninit(len: usize) -> RawMessage {
        Self::alloc(|msg| unsafe { zmq_sys_crate::zmq_msg_init_size(msg, len) })
    }

    pub(crate) fn with_size(size: usize) -> Self {
        let mut msg = Self::with_size_uninit(size);

        let msg_len = msg.len();
        unsafe {
            let data = zmq_sys_crate::zmq_msg_data(&mut msg.message);
            let data_ptr = slice::from_raw_parts_mut(data as *mut u8, msg_len);
            ptr::write_bytes(data_ptr.as_mut_ptr(), 0, size);
        }
        msg
    }

    pub(crate) fn len(&self) -> usize {
        unsafe { zmq_sys_crate::zmq_msg_size(&self.message) }
    }

    pub(crate) fn get_more(&self) -> bool {
        (unsafe { zmq_sys_crate::zmq_msg_more(&self.message) }) != 0
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub(crate) fn set_routing_id(&mut self, value: u32) -> ZmqResult<()> {
        if unsafe { zmq_sys_crate::zmq_msg_set_routing_id(&mut self.message, value) } == -1 {
            cold_path();
            match unsafe { zmq_sys_crate::zmq_errno() } {
                event @ zmq_sys_crate::errno::EINVAL => return Err(ZmqError::from(event)),
                _ => unreachable!(),
            }
        }
        Ok(())
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub(crate) fn routing_id(&self) -> Option<u32> {
        match unsafe { zmq_sys_crate::zmq_msg_routing_id(&self.message as *const _ as *mut _) } {
            0 => None,
            value => Some(value),
        }
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub(crate) fn set_group(&mut self, value: &str) -> ZmqResult<()> {
        let c_value = CString::from_str(value)?;

        if unsafe { zmq_sys_crate::zmq_msg_set_group(&mut self.message, c_value.as_ptr()) } == -1 {
            cold_path();
            let errno = unsafe { zmq_sys_crate::zmq_errno() };
            return Err(ZmqError::from(errno));
        }
        Ok(())
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub(crate) fn group(&self) -> Option<String> {
        let msg_group =
            unsafe { zmq_sys_crate::zmq_msg_group(&self.message as *const _ as *mut _) };

        if msg_group.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(msg_group) }
            .to_owned()
            .into_string()
            .ok()
    }
}

impl Default for RawMessage {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for RawMessage {
    fn clone(&self) -> Self {
        let mut copy = Self::with_size(self.len());

        if unsafe {
            zmq_sys_crate::zmq_msg_copy(&mut copy.message, &self.message as *const _ as *mut _)
        } != 0
        {
            panic!("unable to clone message");
        }

        #[cfg(feature = "draft-api")]
        if let Some(routing_id) = self.routing_id() {
            copy.set_routing_id(routing_id).unwrap();
        }
        #[cfg(feature = "draft-api")]
        if let Some(group) = self.group() {
            copy.set_group(&group).unwrap();
        }

        copy
    }
}

impl Drop for RawMessage {
    fn drop(&mut self) {
        if unsafe { zmq_sys_crate::zmq_msg_close(&mut self.message) } == -1 {
            cold_path();
            match unsafe { zmq_sys_crate::zmq_errno() } {
                zmq_sys_crate::errno::EFAULT => (),
                _ => unreachable!(),
            }
        }
    }
}

impl Deref for RawMessage {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        let msg_len = self.len();

        let ptr = &self.message as *const _ as *mut _;

        unsafe {
            let data = zmq_sys_crate::zmq_msg_data(ptr);
            if data.is_null() {
                return &[];
            }
            slice::from_raw_parts(data as *mut u8, msg_len)
        }
    }
}

impl AsRef<[u8]> for RawMessage {
    fn as_ref(&self) -> &[u8] {
        self.deref()
    }
}

impl core::fmt::Display for RawMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match str::from_utf8(self.as_ref()) {
            Ok(msg_str) => write!(f, "{msg_str}"),
            _ => write!(f, "{:X?}", self.as_ref()),
        }
    }
}

impl core::fmt::Debug for RawMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match str::from_utf8(self.as_ref()) {
            Ok(msg_str)
                if !msg_str.is_empty()
                    && self.as_ref().iter().all(|char| !(1u8..9u8).contains(char)) =>
            {
                write!(f, "\"{msg_str}\"")?
            }
            _ => write!(f, "{:X?}", self.as_ref())?,
        }

        #[cfg(feature = "draft-api")]
        if let Some(routing_id) = self.routing_id() {
            write!(f, " (routing_id: {routing_id})")?;
        }

        #[cfg(feature = "draft-api")]
        if let Some(group) = self.group()
            && !group.is_empty()
        {
            write!(f, " (group: {group})")?;
        }

        Ok(())
    }
}

impl From<Vec<u8>> for RawMessage {
    fn from(value: Vec<u8>) -> Self {
        if value.is_empty() {
            return Self::new();
        }

        RawMessage::from(value.into_boxed_slice())
    }
}

#[cfg(not(feature = "draft-api"))]
unsafe extern "C" fn drop_zmq_msg_t(data: *mut c_void, hint: *mut c_void) {
    let _ = unsafe { Box::from_raw(slice::from_raw_parts_mut(data as *mut u8, hint as usize)) };
}

impl From<Box<[u8]>> for RawMessage {
    #[cfg(not(feature = "draft-api"))]
    fn from(value: Box<[u8]>) -> Self {
        if value.is_empty() {
            return Self::new();
        }

        let size = value.len();
        let data = Box::into_raw(value);

        let mut message = zmq_sys_crate::zmq_msg_t::default();
        unsafe {
            zmq_sys_crate::zmq_msg_init_data(
                &mut message,
                data as *mut c_void,
                size,
                Some(drop_zmq_msg_t),
                size as *mut c_void,
            )
        };
        Self { message }
    }

    #[cfg(feature = "draft-api")]
    fn from(value: Box<[u8]>) -> Self {
        Self::from(value.as_ref())
    }
}

impl<'a> From<&'a [u8]> for RawMessage {
    #[cfg(not(feature = "draft-api"))]
    fn from(value: &'a [u8]) -> Self {
        if value.is_empty() {
            return Self::new();
        }

        let mut message = Self::with_size(value.len());

        let msg_len = message.len();
        unsafe {
            let data = zmq_sys_crate::zmq_msg_data(&mut message.message);
            let data_ptr = slice::from_raw_parts_mut(data as *mut u8, msg_len);
            ptr::copy_nonoverlapping(value.as_ptr(), data_ptr.as_mut_ptr(), value.len());
        }

        message
    }

    #[cfg(feature = "draft-api")]
    fn from(value: &'a [u8]) -> Self {
        if value.is_empty() {
            return Self::new();
        }

        let size = value.len();
        let mut message = zmq_sys_crate::zmq_msg_t::default();
        unsafe {
            zmq_sys_crate::zmq_msg_init_buffer(&mut message, value.as_ptr() as *const c_void, size)
        };

        Self { message }
    }
}

impl<'a> From<&'a str> for RawMessage {
    fn from(value: &'a str) -> Self {
        RawMessage::from(value.as_bytes())
    }
}

impl<'a> From<&'a String> for RawMessage {
    fn from(value: &'a String) -> Self {
        RawMessage::from(value.as_bytes())
    }
}

#[derive(DebugDeriveMore)]
#[debug("RawPollItem {{ ... }}")]
pub(crate) struct RawPollItem {
    pub(crate) item: FairMutex<zmq_sys_crate::zmq_pollitem_t>,
}

impl RawPollItem {
    pub(crate) fn from_socket(socket: &RawSocket, events: PollEvents) -> Self {
        let socket_guard = socket.socket.lock();
        let poll_item = zmq_sys_crate::zmq_pollitem_t {
            socket: *socket_guard,
            fd: 0,
            events: events.bits(),
            revents: 0,
        };

        Self {
            item: FairMutex::new(poll_item),
        }
    }
}
