#include <ftdi.h>
#include <stdio.h>
#include <stdlib.h>

int main() {
  struct ftdi_context *ftdi;
  ftdi = ftdi_new();
  struct ftdi_version_info version = ftdi_get_library_version();
  int ret;

  printf("version %s\n", version.version_str);
  ret = ftdi_usb_open(ftdi, 0x0403, 0x6014);
  if (ret < 0) {
    printf("error: %s\n", ftdi_get_error_string(ftdi));
    return 1;
  }
  ftdi_usb_reset(ftdi);
  ftdi_set_interface(ftdi, INTERFACE_A);
  ftdi_set_bitmode(ftdi, 0, BITMODE_MPSSE);

  unsigned char setup[256] = {
      SET_BITS_LOW, 0x08,          0x0b, TCK_DIVISOR, 0x01,
      0x00,         SET_BITS_HIGH, 0,    0,           SEND_IMMEDIATE};
  if (ftdi_write_data(ftdi, setup, 10) != 10) {
    printf("error: %s\n", ftdi_get_error_string(ftdi));
    return 1;
  }

  unsigned char idle[256] = {MPSSE_WRITE_TMS | MPSSE_LSB | MPSSE_BITMODE |
                                 MPSSE_WRITE_NEG,
                             0x05, 0x1F};
  if (ftdi_write_data(ftdi, idle, 3) != 3) {
    printf("error: %s\n", ftdi_get_error_string(ftdi));
    return 1;
  }

  unsigned char shift_dr[256] = {MPSSE_WRITE_TMS | MPSSE_LSB | MPSSE_BITMODE |
                                     MPSSE_WRITE_NEG,
                                 0x03, 0x02};
  if (ftdi_write_data(ftdi, shift_dr, 3) != 3) {
    printf("error: %s\n", ftdi_get_error_string(ftdi));
    return 1;
  }

  unsigned char read_dr[256] = {MPSSE_DO_READ | MPSSE_DO_WRITE | MPSSE_LSB |
                                    MPSSE_WRITE_NEG,
                                0x03,
                                0x00,
                                0x00,
                                0x00,
                                0x00,
                                0x00};
  if (ftdi_write_data(ftdi, read_dr, 7) != 7) {
    printf("error: %s\n", ftdi_get_error_string(ftdi));
    return 1;
  }

  unsigned char buf[256] = {0};
  int len = 4;
  int offset = 0;
  while (len > offset) {
    int read = ftdi_read_data(ftdi, &buf[offset], len - offset);
    offset += read;
  }
  printf("id: %02x%02x%02x%02x\n", buf[3], buf[2], buf[1], buf[0]);

  return 0;
}
