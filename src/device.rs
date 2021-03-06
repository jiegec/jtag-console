use super::binding::*;
use bitvec::prelude::*;
use std::ffi::CStr;

pub type JTAGResult<T> = Result<T, String>;

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

    pub fn new(vid: u32, pid: u32) -> JTAGResult<Self> {
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
            ftdi_set_latency_timer(self.context, 10);
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

    pub fn jtag_reset(&mut self) {
        let idle = [
            (MPSSE_WRITE_TMS | MPSSE_LSB | MPSSE_BITMODE | MPSSE_WRITE_NEG) as u8,
            0x05,
            0x1F,
        ];
        self.ftdi_write(&idle).expect("reset jtag");
    }

    fn ftdi_write(&mut self, data: &[u8]) -> JTAGResult<usize> {
        let res = unsafe { ftdi_write_data(self.context, data.as_ptr(), data.len() as _) };
        if res < 0 {
            Err(self.get_error())
        } else {
            Ok(res as _)
        }
    }

    pub fn shift_dr(&mut self, output: &BitVec<Local, u8>) -> JTAGResult<BitVec<Local, u8>> {
        self.shift_xr(true, output)
    }

    pub fn shift_ir(&mut self, output: &BitVec<Local, u8>) -> JTAGResult<BitVec<Local, u8>> {
        self.shift_xr(false, output)
    }

    fn shift_xr(&mut self, dr: bool, output: &BitVec<Local, u8>) -> JTAGResult<BitVec<Local, u8>> {
        let shift = [
            (MPSSE_WRITE_TMS | MPSSE_LSB | MPSSE_BITMODE | MPSSE_WRITE_NEG) as u8,
            0x03,
            if dr { 0x02 } else { 0x06 },
        ];
        self.ftdi_write(&shift)?;

        let mut temp = output.clone();
        let last_bit = temp.pop().unwrap();
        let mut input = temp.clone();

        let bits = temp.len();
        let bytes = bits / 8;

        let input_slice = input.as_mut_slice();
        let mut offset = 0;

        if bytes > 0 {
            let slice = &temp.as_slice()[0..bytes];
            for chunk in slice.chunks(1024) {
                let mut shift_bytes = vec![
                    (MPSSE_DO_READ | MPSSE_DO_WRITE | MPSSE_LSB | MPSSE_WRITE_NEG) as u8,
                    ((chunk.len() - 1) & 0xff) as _,
                    ((chunk.len() - 1) >> 8) as _,
                ];
                shift_bytes.extend(chunk);
                self.ftdi_write(&shift_bytes)?;

                let read_bytes = unsafe {
                    ftdi_read_data(
                        self.context,
                        input_slice.as_mut_ptr().add(offset),
                        (input_slice.len() - offset) as _,
                    )
                };
                if read_bytes < 0 {
                    return Err(self.get_error());
                }
                offset += read_bytes as usize;
            }
        }

        if bits % 8 != 0 {
            let mut shift_bits = vec![
                (MPSSE_DO_READ | MPSSE_DO_WRITE | MPSSE_LSB | MPSSE_WRITE_NEG | MPSSE_BITMODE)
                    as u8,
                ((bits % 8) - 1) as _,
            ];
            shift_bits.push(temp.as_slice()[bytes]);
            self.ftdi_write(&shift_bits)?;
        }

        // last bit is handled here
        let idle = [
            (MPSSE_WRITE_TMS | MPSSE_DO_READ | MPSSE_LSB | MPSSE_BITMODE | MPSSE_WRITE_NEG) as u8,
            0x02,
            0x03 | ((last_bit as u8) << 7),
        ];
        self.ftdi_write(&idle)?;

        while offset < input_slice.len() {
            let read_bytes = unsafe {
                ftdi_read_data(
                    self.context,
                    input_slice.as_mut_ptr().add(offset),
                    (input_slice.len() - offset) as _,
                )
            };
            if read_bytes < 0 {
                return Err(self.get_error());
            }
            offset += read_bytes as usize;
        }

        if bits % 8 != 0 {
            input_slice[input_slice.len() - 1] >>= 8 - (bits % 8);
        }

        let mut last_bit_read = [0];
        unsafe { ftdi_read_data(self.context, last_bit_read.as_mut_ptr(), 1) };

        input.push(last_bit_read[0] != 0);

        Ok(input)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { ftdi_free(self.context) };
    }
}
