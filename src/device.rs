use super::binding::*;
use std::ffi::CStr;

pub struct Device {
    context: *mut ftdi_context,
}

unsafe fn get_error(context: *mut ftdi_context) -> String {
    CStr::from_ptr(ftdi_get_error_string(context))
        .to_string_lossy()
        .into_owned()
}

impl Device {
    pub fn new(vid: u32, pid: u32) -> Result<Self, String> {
        let context = unsafe { ftdi_new() };
        let ret = unsafe { ftdi_usb_open(context, vid as _, pid as _) };
        if ret < 0 {
            let error = unsafe { get_error(context) };
            return Err(error);
        }
        Ok(Device { context })
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            ftdi_free(self.context)
        };
    }
}
