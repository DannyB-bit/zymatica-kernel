import argparse

# ==============================================================================
# COPY OF THE ACTUAL RANGE CODER CODEBASE (test_semantic_vocab_range_coder.py)
# ==============================================================================

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
    def __init__(self, data):
        self.data = data
        self.byte_index = 0
        self.bit_index = 0

    def read_bit(self):
        if self.byte_index >= len(self.data):
            return 0
        bit = (self.data[self.byte_index] >> (7 - self.bit_index)) & 1
        self.bit_index += 1
        if self.bit_index == 8:
            self.bit_index = 0
            self.byte_index += 1
        return bit

def range_encode_radicals(radicals, alpha=1, weight=128):
    pred = PythonRadicalPredictor(alpha, weight)
    w = BitWriter()
    low = 0
    high = 0xFFFFFFFF
    underflow_bits = [0]

    for rc, rf, ra in radicals:
        symbols = [rc, rf, ra]
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
            high = low + (range_width * cum_high) // total - 1
            low = low + (range_width * cum_low) // total

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
        pred.observe(rc, rf, ra)

    underflow_bits[0] += 1
    if low < 0x40000000:
        w.write_bit_helper(underflow_bits, 0)
    else:
        w.write_bit_helper(underflow_bits, 1)
    return w.flush()

def range_decode_radicals(encoded_bytes, num_concepts, alpha=1, weight=128):
    pred = PythonRadicalPredictor(alpha, weight)
    r = BitReader(encoded_bytes)
    value = 0
    for _ in range(32):
        value = (value << 1) | r.read_bit()

    low = 0
    high = 0xFFFFFFFF
    decoded_radicals = []

    for c in range(num_concepts):
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
            scaled_val = (((value - low) + 1) * total - 1) // range_width

            # Binary search
            sym = 0
            l = 0
            rr = 255
            while l <= rr:
                mid = (l + rr) // 2
                if cum_freqs[mid] <= scaled_val < cum_freqs[mid + 1]:
                    sym = mid
                    break
                elif scaled_val >= cum_freqs[mid + 1]:
                    l = mid + 1
                else:
                    rr = mid - 1

            symbols[step] = sym
            cum_low = cum_freqs[sym]
            cum_high = cum_freqs[sym + 1]

            high = low + (range_width * cum_high) // total - 1
            low = low + (range_width * cum_low) // total

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
        decoded_radicals.append((symbols[0], symbols[1], symbols[2]))
        pred.observe(symbols[0], symbols[1], symbols[2])
    return decoded_radicals

# ==============================================================================

def run_proof():
    print("======================================================================")
    print("ZYMATICA | LLD-AC Range Coder: Actual Codebase Implementation Proof")
    print("======================================================================\n")

    # Sample sequence of radicals: (R_C, R_F, R_A)
    # Replicates typical repetitive/structured state packets
    input_radicals = [
        (0x12, 0x01, 0x80),
        (0x12, 0x01, 0x80),
        (0x11, 0x00, 0xA0),
        (0x11, 0x00, 0xA0),
        (0x11, 0x00, 0xA0),
        (0x21, 0x01, 0xA0),
        (0x22, 0x02, 0xF0),
        (0x22, 0x02, 0xF0)
    ]

    print("[1] Original Radical Sequence (3 Bytes per concept):")
    for idx, rad in enumerate(input_radicals):
        print(f"  Concept {idx+1}: RC=0x{rad[0]:02X}, RF=0x{rad[1]:02X}, RA=0x{rad[2]:02X}")
    
    uncompressed_bytes = len(input_radicals) * 3
    print(f"  -> Total Uncompressed Size: {uncompressed_bytes} bytes")

    print("\n[2] Executing Range Encoder...")
    compressed_bytes = range_encode_radicals(input_radicals, alpha=1, weight=128)
    compressed_len = len(compressed_bytes)
    print(f"  -> Compressed Size: {compressed_len} bytes")
    print(f"  -> Binary Stream (Hex): {compressed_bytes.hex().upper()}")

    print("\n[3] Executing Lossless Decoder...")
    decoded_radicals = range_decode_radicals(compressed_bytes, len(input_radicals), alpha=1, weight=128)
    
    # Validation check
    assert input_radicals == decoded_radicals, "Validation failed! Decoded sequence does not match original."
    print("  -> Lossless verification passed. Decoded sequence is identical.")

    compression_ratio = uncompressed_bytes / compressed_len
    savings = (1 - (compressed_len / uncompressed_bytes)) * 100
    print("\n[4] Summary Metrics:")
    print(f"  - Uncompressed:      {uncompressed_bytes} bytes")
    print(f"  - Compressed:        {compressed_len} bytes")
    print(f"  - Space Savings:     {savings:.2f}%")
    print(f"  - Compression Ratio: {compression_ratio:.2f}x")

    print("\n[VERIFICATION] LLD-AC range coder verified from actual codebase.")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Zymatica LLD-AC Range Coder Proof")
    parser.add_argument("--test", action="store_true", help="Run test mode")
    args = parser.parse_args()
    run_proof()
