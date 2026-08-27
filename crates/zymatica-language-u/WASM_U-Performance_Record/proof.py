# -*- coding: utf-8 -*-
# Watermark: ip zymatica.space | astronautshe.com
# Parity Verification Engine

import os
import sys
import json
import random
import subprocess
import hashlib

class PythonRadicalPredictor:
    def __init__(self, alpha=1, weight=128):
        self.alpha = alpha
        self.weight = weight
        self.trans_rc = {}
        self.trans_rf = {}
        self.trans_ra = {}
        self.prev_rc = 0
        self.prev_rf = 0
        self.prev_ra = 0

    def observe(self, rc, rf, ra):
        key_rc = self.prev_rc
        if key_rc not in self.trans_rc:
            self.trans_rc[key_rc] = {}
        self.trans_rc[key_rc][rc] = self.trans_rc[key_rc].get(rc, 0) + self.weight

        key_rf = (rc << 8) | self.prev_rf
        if key_rf not in self.trans_rf:
            self.trans_rf[key_rf] = {}
        self.trans_rf[key_rf][rf] = self.trans_rf[key_rf].get(rf, 0) + self.weight

        key_ra = (rc << 16) | (rf << 8) | self.prev_ra
        if key_ra not in self.trans_ra:
            self.trans_ra[key_ra] = {}
        self.trans_ra[key_ra][ra] = self.trans_ra[key_ra].get(ra, 0) + self.weight

        self.prev_rc = rc
        self.prev_rf = rf
        self.prev_ra = ra

    def get_cum_freqs_rc(self, prev_rc):
        freqs = [self.alpha] * 256
        if prev_rc in self.trans_rc:
            for sym, count in self.trans_rc[prev_rc].items():
                freqs[sym] += count
        cum_freqs = [0] * 257
        for i in range(256):
            cum_freqs[i+1] = cum_freqs[i] + freqs[i]
        return cum_freqs

    def get_cum_freqs_rf(self, curr_rc, prev_rf):
        freqs = [self.alpha] * 256
        key = (curr_rc << 8) | prev_rf
        if key in self.trans_rf:
            for sym, count in self.trans_rf[key].items():
                freqs[sym] += count
        cum_freqs = [0] * 257
        for i in range(256):
            cum_freqs[i+1] = cum_freqs[i] + freqs[i]
        return cum_freqs

    def get_cum_freqs_ra(self, curr_rc, curr_rf, prev_ra):
        freqs = [self.alpha] * 256
        key = (curr_rc << 16) | (curr_rf << 8) | prev_ra
        if key in self.trans_ra:
            for sym, count in self.trans_ra[key].items():
                freqs[sym] += count
        cum_freqs = [0] * 257
        for i in range(256):
            cum_freqs[i+1] = cum_freqs[i] + freqs[i]
        return cum_freqs

class BitWriter:
    def __init__(self):
        self.buffer = []
        self.current_byte = 0
        self.bit_count = 0

    def write_bit(self, bit):
        self.current_byte = (self.current_byte << 1) | (bit & 1)
        self.bit_count += 1
        if self.bit_count % 8 == 0:
            self.buffer.append(self.current_byte)
            self.current_byte = 0

    def write_bit_helper(self, underflow_bits, bit):
        self.write_bit(bit)
        for _ in range(underflow_bits[0]):
            self.write_bit(1 - bit)
        underflow_bits[0] = 0

    def flush(self):
        if self.bit_count % 8 != 0:
            padding_bits = 8 - (self.bit_count % 8)
            self.current_byte <<= padding_bits
            self.buffer.append(self.current_byte)
            self.current_byte = 0
            self.bit_count += padding_bits
        return bytes(self.buffer)

class BitReader:
    def __init__(self, buffer):
        self.buffer = buffer
        self.bit_index = 0
        self.total_bits = len(buffer) * 8

    def read_bit(self):
        if self.bit_index >= self.total_bits:
            return 0
        byte_pos = self.bit_index // 8
        bit_pos = 7 - (self.bit_index % 8)
        bit = (self.buffer[byte_pos] >> bit_pos) & 1
        self.bit_index += 1
        return bit

def python_encode(concepts, alpha=1, weight=128):
    pred = PythonRadicalPredictor(alpha, weight)
    w = BitWriter()
    low = 0
    high = 0xFFFFFFFF
    underflow_bits = [0]
    trace_info = []

    for c_idx, c in enumerate(concepts):
        rc = (c['domain'] << 4) | c['subdomain']
        rf = (c['operation'] << 4) | c['modality']
        ra = (c['depth'] << 4) | c['polarity']
        symbols = [rc, rf, ra]
        types = ["RC", "RF", "RA"]

        prev_rc = pred.prev_rc
        prev_rf = pred.prev_rf
        prev_ra = pred.prev_ra

        for step in range(3):
            if step == 0:
                cum_freqs = pred.get_cum_freqs_rc(prev_rc)
            elif step == 1:
                cum_freqs = pred.get_cum_freqs_rf(symbols[0], prev_rf)
            else:
                cum_freqs = pred.get_cum_freqs_ra(symbols[0], symbols[1], prev_ra)

            sym = symbols[step]
            total = cum_freqs[256]
            cum_low = cum_freqs[sym]
            cum_high = cum_freqs[sym + 1]

            range_width = high - low + 1
            
            high_before = high
            low_before = low
            
            high = (low + (range_width * cum_high) // total - 1) & 0xFFFFFFFF
            low = (low + (range_width * cum_low) // total) & 0xFFFFFFFF

            bits_written = []
            temp_underflow = [underflow_bits[0]]
            
            # Simulate bit writing helper to capture trace outputs
            def write_bit_simulate(bit):
                bits_written.append(str(bit))
            def write_bit_helper_simulate(u_bits, bit):
                write_bit_simulate(bit)
                for _ in range(u_bits[0]):
                    write_bit_simulate(1 - bit)
                u_bits[0] = 0
            
            while True:
                if high_before < 0x80000000:
                    write_bit_helper_simulate(temp_underflow, 0)
                    low_before = (low_before << 1) & 0xFFFFFFFF
                    high_before = ((high_before << 1) | 1) & 0xFFFFFFFF
                elif low_before >= 0x80000000:
                    write_bit_helper_simulate(temp_underflow, 1)
                    low_before = ((low_before - 0x80000000) << 1) & 0xFFFFFFFF
                    high_before = (((high_before - 0x80000000) << 1) | 1) & 0xFFFFFFFF
                elif low_before >= 0x40000000 and high_before < 0xC0000000:
                    temp_underflow[0] += 1
                    low_before = ((low_before - 0x40000000) << 1) & 0xFFFFFFFF
                    high_before = (((high_before - 0x40000000) << 1) | 1) & 0xFFFFFFFF
                else:
                    break

            # Now write the real bits
            while True:
                if high < 0x80000000:
                    w.write_bit_helper(underflow_bits, 0)
                    low = (low << 1) & 0xFFFFFFFF
                    high = ((high << 1) | 1) & 0xFFFFFFFF
                elif low >= 0x80000000:
                    w.write_bit_helper(underflow_bits, 1)
                    low = ((low - 0x80000000) << 1) & 0xFFFFFFFF
                    high = (((high - 0x80000000) << 1) | 1) & 0xFFFFFFFF
                elif low >= 0x40000000 and high < 0xC0000000:
                    underflow_bits[0] += 1
                    low = ((low - 0x40000000) << 1) & 0xFFFFFFFF
                    high = (((high - 0x40000000) << 1) | 1) & 0xFFFFFFFF
                else:
                    break

            trace_info.append({
                "concept_idx": c_idx,
                "step": step,
                "symbol_type": types[step],
                "symbol_value": sym,
                "low_before": f"0x{low_before:08x}",
                "high_before": f"0x{high_before:08x}",
                "cum_low": cum_low,
                "cum_high": cum_high,
                "total": total,
                "bits_written": "".join(bits_written)
            })
            
        pred.observe(rc, rf, ra)

    underflow_bits[0] += 1
    if low < 0x40000000:
        w.write_bit_helper(underflow_bits, 0)
    else:
        w.write_bit_helper(underflow_bits, 1)
    return w.flush(), w.bit_count, trace_info


def python_decode(encoded_bytes, num_concepts, alpha=1, weight=128):
    pred = PythonRadicalPredictor(alpha, weight)
    r = BitReader(encoded_bytes)

    value = 0
    for _ in range(32):
        value = (value << 1) | r.read_bit()

    low = 0
    high = 0xFFFFFFFF
    decoded = []

    for _ in range(num_concepts):
        prev_rc = pred.prev_rc
        prev_rf = pred.prev_rf
        prev_ra = pred.prev_ra
        symbols = [0, 0, 0]

        for step in range(3):
            if step == 0:
                cum_freqs = pred.get_cum_freqs_rc(prev_rc)
            elif step == 1:
                cum_freqs = pred.get_cum_freqs_rf(symbols[0], prev_rf)
            else:
                cum_freqs = pred.get_cum_freqs_ra(symbols[0], symbols[1], prev_ra)

            total = cum_freqs[256]
            range_width = high - low + 1
            scaled_val = ((value - low + 1) * total - 1) // range_width

            sym = 0
            l = 0
            rr = 255
            while l <= rr:
                mid = (l + rr) // 2
                if cum_freqs[mid] <= scaled_val < cum_freqs[mid+1]:
                    sym = mid
                    break
                elif scaled_val >= cum_freqs[mid+1]:
                    l = mid + 1
                else:
                    rr = mid - 1

            symbols[step] = sym
            cum_low = cum_freqs[sym]
            cum_high = cum_freqs[sym+1]

            high = (low + (range_width * cum_high) // total - 1) & 0xFFFFFFFF
            low = (low + (range_width * cum_low) // total) & 0xFFFFFFFF

            while True:
                if high < 0x80000000:
                    low = (low << 1) & 0xFFFFFFFF
                    high = ((high << 1) | 1) & 0xFFFFFFFF
                    value = ((value << 1) | r.read_bit()) & 0xFFFFFFFF
                elif low >= 0x80000000:
                    low = ((low - 0x80000000) << 1) & 0xFFFFFFFF
                    high = (((high - 0x80000000) << 1) | 1) & 0xFFFFFFFF
                    value = (((value - 0x80000000) << 1) | r.read_bit()) & 0xFFFFFFFF
                elif low >= 0x40000000 and high < 0xC0000000:
                    low = ((low - 0x40000000) << 1) & 0xFFFFFFFF
                    high = (((high - 0x40000000) << 1) | 1) & 0xFFFFFFFF
                    value = (((value - 0x40000000) << 1) | r.read_bit()) & 0xFFFFFFFF
                else:
                    break

        rc, rf, ra = symbols
        decoded.append({
            'domain': rc >> 4,
            'subdomain': rc & 0x0F,
            'operation': rf >> 4,
            'modality': rf & 0x0F,
            'depth': ra >> 4,
            'polarity': ra & 0x0F
        })
        pred.observe(rc, rf, ra)

    return decoded

def generate_fuzz_data(count=100):
    concepts = []
    for _ in range(count):
        concepts.append({
            'domain': random.randint(0, 15),
            'subdomain': random.randint(0, 15),
            'operation': random.randint(0, 15),
            'modality': random.randint(0, 15),
            'depth': random.randint(0, 15),
            'polarity': random.randint(0, 15)
        })
    return concepts

def run_parity_test():
    print("=" * 80)
    print("  [+] Starting Fuzz Parity Test Engine...")
    print("=" * 80)
    
    # 1. Generate 100 random coordinate structures
    test_concepts = generate_fuzz_data(100)
    print(f"  - Generated {len(test_concepts)} random 6D coordinates.")
    
    # Write them to a JSON file for the Node.js / WASM script to read
    with open('test_input.json', 'w') as f:
        json.dump(test_concepts, f)
        
    # 2. Run Python range encoding
    py_bytes, py_bits, trace_data = python_encode(test_concepts)
    
    # Save the trace data to parity_trace.json
    with open('parity_trace.json', 'w') as f:
        json.dump(trace_data, f, indent=2)
    print("  [+] Step-by-step state trace outputted to parity_trace.json")
    
    py_decoded = python_decode(py_bytes, len(test_concepts))
    
    # Check Python self-parity
    for idx, (orig, dec) in enumerate(zip(test_concepts, py_decoded)):
        if orig != dec:
            print(f"  [-] ERROR: Python self-parity failed at element {idx}!")
            return False
    print("  [+] Python self-parity checks passed successfully.")
    
    # Save python compressed payload
    with open('payload_py.bin', 'wb') as f:
        f.write(py_bytes)
        
    # 3. Compile Zig code to WASM if not already done
    print("  - Building Zig WASM target...")
    try:
        subprocess.run([
            "zig", "build-exe", "proof.zig", 
            "-target", "wasm32-freestanding", 
            "-O", "ReleaseFast", 
            "--name", "proof_wasm", 
            "--export=wasm_encode", "--export=wasm_get_encoded_bits", 
            "--export=wasm_decode", "--export=run_verification"
        ], check=True)
        print("  [+] Compiled proof_wasm.wasm successfully!")
    except Exception as e:
        print(f"  [-] Failed to compile proof.zig: {e}")
        print("  [-] Make sure Zig is installed and available in PATH.")
        return False
        
    # 4. Invoke Node.js cross-runtime verification tool
    print("  - Running Node.js/WASM encoding task...")
    try:
        subprocess.run(["node", "run_wasm.js"], check=True)
    except Exception as e:
        print(f"  [-] Node.js/WASM execution execution error: {e}")
        return False
        
    # 5. Assert byte parity between Python and WASM
    if not os.path.exists('payload_wasm.bin'):
        print("  [-] ERROR: Node.js did not produce payload_wasm.bin!")
        return False
        
    with open('payload_wasm.bin', 'rb') as f:
        wasm_bytes = f.read()
        
    print(f"  - Python compressed size: {len(py_bytes)} bytes ({py_bits} bits)")
    print(f"  - WASM compressed size:   {len(wasm_bytes)} bytes")
    
    # Assert exact byte match
    if py_bytes != wasm_bytes:
        print("  [-] ERROR: Bit-Parity Mismatch between Python and WebAssembly!")
        print(f"    - Python MD5: {hashlib.md5(py_bytes).hexdigest()}")
        print(f"    - WASM MD5:   {hashlib.md5(wasm_bytes).hexdigest()}")
        return False
        
    print("  [+] SUCCESS: Isomorphic Bit-Parity Verified! Python and WASM produced byte-for-byte identical output.")
    
    # 6. Check Decoded Parity from WASM output
    with open('test_output_wasm.json', 'r') as f:
        wasm_decoded = json.load(f)
        
    for idx, (orig, dec) in enumerate(zip(test_concepts, wasm_decoded)):
        if orig != dec:
            print(f"  [-] ERROR: Decoded value from WASM mismatches original at index {idx}!")
            return False
            
    print("  [+] SUCCESS: Reconstructed coordinates from WASM match input identically.")
    return True

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == '--fuzz':
        run_parity_test()
    else:
        run_parity_test()
