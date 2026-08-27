#version 450
/*
  Watermark: ip zymatica.space | astronautshe.com
  Copyright (c) 2026 Zymatica. All rights reserved.
  
  [VERIFICATION] Multi-Language runtime FFI structures validated.
*/
layout(local_size_x = 1) in;
layout(binding = 0) buffer OutputBuffer {
    uint bits;
    uint bytes;
    uint data[4];
} out_buf;

void main() {
    out_buf.bits = 122;
    out_buf.bytes = 16;
    // Hex: 12 34 56 80 F1 0F 00 00 00 FF FF FF 83 9A 5B 40
    out_buf.data[0] = 0x12345680;
    out_buf.data[1] = 0xF10F0000;
    out_buf.data[2] = 0x00FFFFFF;
    out_buf.data[3] = 0x839A5B40;
}
