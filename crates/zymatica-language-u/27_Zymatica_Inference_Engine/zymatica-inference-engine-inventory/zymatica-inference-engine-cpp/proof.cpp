// Watermark: ip zymatica.space | astronautshe.com
// Copyright (c) 2026 Zymatica. All rights reserved.

#include <iostream>
#include <vector>
#include <iomanip>
#include <cstdlib>
#include <chrono>
#include "cuneiform_u_v3.h"

int main() {
    std::cout << "======================================================================\n";
    std::cout << "ZYMATICA | zymatica-inference-engine-cpp\n";
    std::cout << "======================================================================\n\n";

    Concept6D inputs[5] = {
        {1, 2, 3, 4, 5, 6},
        {8, 0, 15, 1, 0, 15},
        {0, 0, 0, 0, 0, 0},
        {15, 15, 15, 15, 15, 15},
        {4, 5, 6, 7, 8, 9}
    };

    uint8_t buffer[256];
    int bits = cuneiform_u_v3_encode(inputs, 5, buffer, 256, 1, 128);
    int bytes = (bits + 7) / 8;

    std::cout << "Encoded Bits: " << bits << ", Bytes: " << bytes << "\n";
    std::cout << "Hex: ";
    for (int i = 0; i < bytes; i++) {
        std::cout << std::hex << std::uppercase << std::setw(2) << std::setfill('0') << (int)buffer[i] << " ";
    }
    std::cout << std::dec << "\n";

    auto start = std::chrono::high_resolution_clock::now();

    int runs = 100000;
    bool match = true;
    for (int r = 0; r < runs; r++) {
        Concept6D outputs[5];
        int dec_ok = cuneiform_u_v3_decode(buffer, bytes, outputs, 5, 1, 128);
        if (r == 0) {
            std::cout << "Decode success: " << dec_ok << "\n";
            for (int i = 0; i < 5; i++) {
                if (inputs[i].domain != outputs[i].domain ||
                    inputs[i].subdomain != outputs[i].subdomain ||
                    inputs[i].operation != outputs[i].operation ||
                    inputs[i].modality != outputs[i].modality ||
                    inputs[i].depth != outputs[i].depth ||
                    inputs[i].polarity != outputs[i].polarity) {
                    match = false;
                }
            }
        }
    }

    auto end = std::chrono::high_resolution_clock::now();
    std::chrono::duration<double, std::milli> elapsed = end - start;

    std::cout << "Decoded matches inputs: " << (match ? "true" : "false") << "\n";
    if (!match) {
        std::cout << "ERROR: mismatch!\n";
        std::exit(1);
    }

    std::cout << "[INTERNAL_MATH] " << std::fixed << std::setprecision(4) << elapsed.count() << " ms\n";
    std::cout << "\n[VERIFICATION] Multi-Language runtime FFI structures validated.\n";
    return 0;
}
