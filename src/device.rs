use super::binding::*;
use std::ffi::CStr;

pub struct Device {
    context: *mut ftdi_context,
}

impl Device {
    fn get_error(&mut self) -> String {
        unsafe {
            CStr::from_ptr(ftdi_get_error_string(self.context))
                .to_string_lossy()
                .into_owned()
        }
    }

    pub fn new(vid: u32, pid: u32) -> Result<Self, String> {
        let context = unsafe { ftdi_new() };
        let mut dev = Device { context };

        let ret = unsafe { ftdi_usb_open(context, vid as _, pid as _) };
        if ret < 0 {
            let error = dev.get_error();
            return Err(error);
        }
        dev.setup_mpsse();
        dev.jtag_reset();
        Ok(dev)
    }

    fn setup_mpsse(&mut self) {
        unsafe {
            ftdi_usb_reset(self.context);
            ftdi_set_interface(self.context, ftdi_interface_INTERFACE_A);
            ftdi_set_bitmode(self.context, 0, ftdi_mpsse_mode_BITMODE_MPSSE as _);
        }

        let setup = [
            SET_BITS_LOW as u8,
            0x08,
            0x0b,
            SET_BITS_HIGH as _,
            0,
            0,
            TCK_DIVISOR as _,
            0x01,
            0x00,
            SEND_IMMEDIATE as _,
        ];

        self.ftdi_write(&setup).expect("setup mpsse");
    }

    fn jtag_reset(&mut self) {
        let idle = [
            (MPSSE_WRITE_TMS | MPSSE_LSB | MPSSE_BITMODE | MPSSE_WRITE_NEG) as u8,
            0x05,
            0x1F,
        ];
        self.ftdi_write(&idle).expect("reset jtag");
    }

    fn ftdi_write(&mut self, data: &[u8]) -> Result<usize, String> {
        let res = unsafe { ftdi_write_data(self.context, data.as_ptr(), data.len() as _) };
        if res < 0 {
            Err(self.get_error())
        } else {
            Ok(res as _)
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { ftdi_free(self.context) };
    }
}
