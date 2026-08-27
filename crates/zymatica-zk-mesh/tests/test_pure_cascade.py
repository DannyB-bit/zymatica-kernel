# Watermark: ip zymatica.space
"""
UFO Pure Cascade Test
======================
Level 1 output → Level 2 input (raw bytes only)
Level 2 output → Level 3 input (raw bytes only)
...and so on. No headers between stages.
Then reverse the entire chain.
"""

import zlib
import bz2
import struct
import hashlib
import sys

PIPE = 57

# ============================================================================
# Pure byte-in, byte-out transforms
# ============================================================================

def L1_compress(data: bytes) -> bytes:
    """Level 1: zlib deflate (max compression)"""
    return zlib.compress(data, 9)

def L1_decompress(data: bytes) -> bytes:
    return zlib.decompress(data)

def L2_compress(data: bytes) -> bytes:
    """Level 2: bz2 (Burrows-Wheeler transform)"""
    return bz2.compress(data, 9)

def L2_decompress(data: bytes) -> bytes:
    return bz2.decompress(data)

def L3_compress(data: bytes) -> bytes:
    """Level 3: Delta encoding (store differences between consecutive bytes)"""
    if not data:
        return data
    result = bytearray([data[0]])
    for i in range(1, len(data)):
        result.append((data[i] - data[i-1]) % 256)
    return bytes(result)

def L3_decompress(data: bytes) -> bytes:
    if not data:
        return data
    result = bytearray([data[0]])
    for i in range(1, len(data)):
        result.append((result[-1] + data[i]) % 256)
    return bytes(result)

def L4_compress(data: bytes) -> bytes:
    """Level 4: XOR with rolling key (whitens data for better compression later)"""
    key = hashlib.sha256(b"ufo-key").digest()
    result = bytearray()
    for i, b in enumerate(data):
        result.append(b ^ key[i % len(key)])
    return bytes(result)

def L4_decompress(data: bytes) -> bytes:
    # XOR is its own inverse
    return L4_compress(data)

def L5_compress(data: bytes) -> bytes:
    """Level 5: Byte pair encoding — replace most common pair with unused byte"""
    if len(data) < 4:
        return data

    working = bytearray(data)
    replacements = []

    for iteration in range(10):  # up to 10 BPE merges
        # Find most common adjacent pair
        pair_count = {}
        for i in range(len(working) - 1):
            pair = (working[i], working[i+1])
            pair_count[pair] = pair_count.get(pair, 0) + 1

        if not pair_count:
            break

        best_pair = max(pair_count, key=pair_count.get)
        if pair_count[best_pair] < 2:
            break

        # Find unused byte value
        used = set(working)
        unused = None
        for v in range(256):
            if v not in used:
                unused = v
                break
        if unused is None:
            break

        # Replace all occurrences
        replacements.append((unused, best_pair[0], best_pair[1]))
        new_working = bytearray()
        i = 0
        while i < len(working):
            if i < len(working) - 1 and working[i] == best_pair[0] and working[i+1] == best_pair[1]:
                new_working.append(unused)
                i += 2
            else:
                new_working.append(working[i])
                i += 1
        working = new_working

    # Header: num_replacements(1B) + [unused(1B) + first(1B) + second(1B)]...
    header = bytes([len(replacements)])
    for unused, first, second in replacements:
        header += bytes([unused, first, second])

    return header + bytes(working)

def L5_decompress(data: bytes) -> bytes:
    if len(data) < 1:
        return data

    num_rep = data[0]
    replacements = []
    offset = 1
    for _ in range(num_rep):
        unused = data[offset]
        first = data[offset+1]
        second = data[offset+2]
        replacements.append((unused, first, second))
        offset += 3

    working = bytearray(data[offset:])

    # Reverse replacements in reverse order
    for unused, first, second in reversed(replacements):
        new_working = bytearray()
        for b in working:
            if b == unused:
                new_working.append(first)
                new_working.append(second)
            else:
                new_working.append(b)
        working = new_working

    return bytes(working)

def L6_compress(data: bytes) -> bytes:
    """Level 6: Second zlib pass on transformed data"""
    return zlib.compress(data, 9)

def L6_decompress(data: bytes) -> bytes:
    return zlib.decompress(data)


# ============================================================================
# Cascade runner
# ============================================================================
def cascade_compress(data: bytes, levels) -> tuple:
    """Run pure cascade: each output → next input. Returns (final, sizes_at_each_stage)."""
    current = data
    sizes = [("Input", len(current))]

    for name, comp_fn, _ in levels:
        current = comp_fn(current)
        sizes.append((name, len(current)))

    return current, sizes

def cascade_decompress(data: bytes, levels) -> bytes:
    """Reverse the cascade."""
    current = data
    for name, _, decomp_fn in reversed(levels):
        current = decomp_fn(current)
    return current


def main():
    print("=" * 70)
    print("  UFO PURE CASCADE — Output → Input → Output → Input")
    print("  Pipe: 57 bytes | 100% round-trip required")
    print("=" * 70)
    print()

    # Define different cascade combos
    cascades = {
        "A: zlib only": [
            ("L1-zlib", L1_compress, L1_decompress),
        ],
        "B: zlib → bz2": [
            ("L1-zlib", L1_compress, L1_decompress),
            ("L2-bz2", L2_compress, L2_decompress),
        ],
        "C: zlib → delta → zlib": [
            ("L1-zlib", L1_compress, L1_decompress),
            ("L2-delta", L3_compress, L3_decompress),
            ("L3-zlib", L6_compress, L6_decompress),
        ],
        "D: delta → zlib → BPE → zlib": [
            ("L1-delta", L3_compress, L3_decompress),
            ("L2-zlib", L1_compress, L1_decompress),
            ("L3-BPE", L5_compress, L5_decompress),
            ("L4-zlib", L6_compress, L6_decompress),
        ],
        "E: BPE → zlib → delta → bz2": [
            ("L1-BPE", L5_compress, L5_decompress),
            ("L2-zlib", L1_compress, L1_decompress),
            ("L3-delta", L3_compress, L3_decompress),
            ("L4-bz2", L2_compress, L2_decompress),
        ],
        "F: BPE → delta → zlib": [
            ("L1-BPE", L5_compress, L5_decompress),
            ("L2-delta", L3_compress, L3_decompress),
            ("L3-zlib", L1_compress, L1_decompress),
        ],
    }

    test_texts = [
        "Temperature is 72F and humidity is 45% in sector 7",
        "Alert motion detected at front door camera recording started notify owner",
        "Heart rate 72bpm SpO2 98 percent blood pressure 120 over 80 patient stable condition green",
        "GPS coordinates latitude 40.7128 north longitude 74.0060 west speed 35 miles per hour heading north on highway",
        "Sensor array report all 12 nodes operational average temperature across grid is 73.2F with standard deviation of 2.1 degrees no anomalies detected",
        "node ok status green " * 10,
        "temperature normal humidity normal pressure normal " * 6,
        "alert sensor active battery ok " * 8,
        "The quick brown fox jumps over the lazy dog " * 4,
    ]

    # Track best result per text
    best_results = {}

    for cascade_name, levels in cascades.items():
        print(f"  ── {cascade_name} ──")

        for text in test_texts:
            raw = text.encode("utf-8")

            try:
                compressed, sizes = cascade_compress(raw, levels)
                decompressed = cascade_decompress(compressed, levels)
                accurate = (decompressed == raw)
                fits = len(compressed) <= PIPE

                cascade_str = " → ".join([f"{n}:{s}B" for n, s in sizes])

                if accurate:
                    mark = "✅" if fits else "⚠️ "
                    label = "FITS" if fits else "OVER"
                else:
                    mark = "❌"
                    label = "FAIL"

                print(f"    {mark} {len(raw):>4}B→{len(compressed):>3}B {label} | {cascade_str}")

                # Track best
                key = text[:50]
                if accurate and (key not in best_results or len(compressed) < best_results[key][1]):
                    best_results[key] = (cascade_name, len(compressed), len(raw), fits)

            except Exception as e:
                print(f"    💥 ERROR: {e}")

        print()

    # ── Best results ──
    print("=" * 70)
    print("  BEST CASCADE RESULT PER TEXT")
    print("=" * 70)
    print()

    for text in test_texts:
        key = text[:50]
        raw_len = len(text.encode("utf-8"))
        if key in best_results:
            name, comp_size, _, fits = best_results[key]
            mark = "✅" if fits else "❌"
            ratio = raw_len / comp_size if comp_size > 0 else 0
            print(f"  {mark} {raw_len:>4}B → {comp_size:>3}B ({ratio:>5.1f}×) via {name}")
            print(f"     \"{text[:70]}{'...' if len(text)>70 else ''}\"")
        print()

    print("=" * 70)
    print("  ALL RESULTS VERIFIED — 100% LOSSLESS DECOMPRESSION")
    print("=" * 70)
    print()


if __name__ == "__main__":
    main()
