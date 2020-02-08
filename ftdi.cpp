#include <ftdi.h>
#include <memory.h>
#include <stdio.h>
#include <stdlib.h>

// References:
// FTDI AN108
// https://www.ftdichip.com/Support/Documents/AppNotes/AN_108_Command_Processor_for_MPSSE_and_MCU_Host_Bus_Emulation_Modes.pdf
// MPSSE Example
// http://developer.intra2net.com/mailarchive/html/libftdi/2010/msg00372.html

bool dr_shift(ftdi_context *ftdi, size_t length_bits, uint8_t out[],
              uint8_t in[]) {
  uint8_t shift_dr[256] = {MPSSE_WRITE_TMS | MPSSE_LSB | MPSSE_BITMODE |
                               MPSSE_WRITE_NEG,
                           // length in bits -1
                           0x03,
                           // data
                           // 0100: From Run-Test/Idle to Shift-DR
                           0x02};
  if (ftdi_write_data(ftdi, shift_dr, 3) != 3) {
    printf("error: %s\n", ftdi_get_error_string(ftdi));
    return false;
  }

  size_t length_bytes = length_bits / 8;
  if (length_bytes > 0) {
    uint8_t read_dr_byte[256] = {
        MPSSE_DO_READ | MPSSE_DO_WRITE | MPSSE_LSB | MPSSE_WRITE_NEG,
        // length in bytes -1 lo
        (length_bytes - 1) & 0xff,
        // length in bytes -1 hi
        (length_bytes - 1) >> 8,
        // data
    };
    memcpy(&read_dr_byte[3], out, length_bytes);
    if (ftdi_write_data(ftdi, read_dr_byte, 3 + length_bytes) !=
        3 + length_bytes) {
      printf("error: %s\n", ftdi_get_error_string(ftdi));
      return false;
    }
  }

  if (length_bits % 8) {
    uint8_t read_dr_bit[256] = {
        MPSSE_DO_READ | MPSSE_DO_WRITE | MPSSE_LSB | MPSSE_WRITE_NEG | MPSSE_BITMODE,
        // length in bits -1
        (length_bits % 8) - 1,
        // data
    };
    read_dr_bit[2] = out[length_bits / 8];
    if (ftdi_write_data(ftdi, read_dr_bit, 3) !=
        3) {
      printf("error: %s\n", ftdi_get_error_string(ftdi));
      return false;
    }
  }

  int len = (length_bits + 7) / 8;
  int offset = 0;
  while (len > offset) {
    int read = ftdi_read_data(ftdi, &in[offset], len - offset);
    offset += read;
  }

  uint8_t idle[256] = {MPSSE_WRITE_TMS | MPSSE_LSB | MPSSE_BITMODE |
                           MPSSE_WRITE_NEG,
                       // length in bits -1
                       0x02,
                       // data
                       // 110: From Shift-DR to Run-Test/Idle
                       0x03};
  if (ftdi_write_data(ftdi, idle, 3) != 3) {
    printf("error: %s\n", ftdi_get_error_string(ftdi));
    return false;
  }

  return true;
}

int main() {
  struct ftdi_context *ftdi;
  ftdi = ftdi_new();
  struct ftdi_version_info version = ftdi_get_library_version();
  int ret;

  printf("libftdi version %s\n", version.version_str);
  ret = ftdi_usb_open(ftdi, 0x0403, 0x6014);
  if (ret < 0) {
    printf("error: %s\n", ftdi_get_error_string(ftdi));
    return 1;
  }
  ftdi_usb_reset(ftdi);
  ftdi_set_interface(ftdi, INTERFACE_A);
  ftdi_set_bitmode(ftdi, 0, BITMODE_MPSSE);

  uint8_t setup[256] = {SET_BITS_LOW,  0x08, 0x0b, TCK_DIVISOR,   0x01, 0x00,
                        SET_BITS_HIGH, 0,    0,    SEND_IMMEDIATE};
  if (ftdi_write_data(ftdi, setup, 10) != 10) {
    printf("error: %s\n", ftdi_get_error_string(ftdi));
    return 1;
  }

  // Clock Data to TMS pin (no read)
  uint8_t idle[256] = {MPSSE_WRITE_TMS | MPSSE_LSB | MPSSE_BITMODE |
                           MPSSE_WRITE_NEG,
                       // length in bits -1
                       0x05,
                       // data
                       // 111110: Goto Test-Logic-Reset, then Run-Test/Idle
                       0x1F};
  if (ftdi_write_data(ftdi, idle, 3) != 3) {
    printf("error: %s\n", ftdi_get_error_string(ftdi));
    return 1;
  }

  uint8_t out[256] = {0};
  uint8_t buf[256] = {0};
  dr_shift(ftdi, 32, out, buf);
  uint32_t id = (uint32_t)buf[3] << 24 | (uint32_t)buf[2] << 16 |
                (uint32_t)buf[1] << 8 | (uint32_t)buf[0];
  printf("id: %08X\n", id);
  return 0;
}
