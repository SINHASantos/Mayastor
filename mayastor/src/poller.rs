use crate::bdev::nexus::Error;
use spdk_sys::{spdk_poller, spdk_poller_register, spdk_poller_unregister};
use std::os::raw::c_void;

#[derive(Debug)]
pub struct PollTask {
    /// pointer to the allocated poller
    pub poller: *mut spdk_poller,
}

impl Default for PollTask {
    fn default() -> Self {
        Self {
            poller: std::ptr::null_mut(),
        }
    }
}

pub trait SetPoller {
    fn set_inner_poller(&mut self, p: *mut spdk_poller);
}

/// signature of the poll function, this is similar as the function generated by
/// bindgen but does not require the user of this interface to deal with the
/// Option
pub type PollFunction = extern "C" fn(*mut c_void) -> i32;

pub fn register_poller<T: SetPoller>(
    poll_fn: PollFunction,
    ctx: Box<T>,
    interval: u64,
) -> Result<(), Error> {
    // first try to see if we can create the poller to begin with
    let ptr = Box::into_raw(ctx);

    let poller =
        unsafe { spdk_poller_register(Some(poll_fn), ptr as *mut _, interval) };

    // get hold of the pointer again such that the data is dropped
    // if there is an error or set the poller pointer within T otherwise
    let mut ctx = unsafe { Box::from_raw(ptr) };

    if poller.is_null() {
        return Err(Error::Internal("failed to create poller".into()));
    } else {
        ctx.set_inner_poller(poller);
        std::mem::forget(ctx);
    }

    Ok(())
}

impl Drop for PollTask {
    fn drop(&mut self) {
        if !self.poller.is_null() {
            trace!("deregister poller {:?}", self.poller);
            unsafe {
                spdk_poller_unregister(&mut self.poller);
                self.poller = std::ptr::null_mut();
            }
        }
    }
}