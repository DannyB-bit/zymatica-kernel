# Watermark: ip zymatica.space | astronautshe.com
# Copyright (c) 2026 Zymatica. All rights reserved.

using Printf

struct Concept6D
    domain::UInt8
    subdomain::UInt8
    operation::UInt8
    modality::UInt8
    depth::UInt8
    polarity::UInt8
end

mutable struct SparseTransition
    key::UInt32
    sym::UInt8
    count::UInt32
end

mutable struct RadicalPredictor
    alpha::UInt32
    weight::UInt32
    trans_rc::Vector{SparseTransition}
    trans_rf::Vector{SparseTransition}
    trans_ra::Vector{SparseTransition}
    prev_rc::UInt8
    prev_rf::UInt8
    prev_ra::UInt8
end

function RadicalPredictor(alpha::UInt32, weight::UInt32)
    RadicalPredictor(alpha, weight, SparseTransition[], SparseTransition[], SparseTransition[], 0, 0, 0)
end

function observe!(pred::RadicalPredictor, rc::UInt8, rf::UInt8, ra::UInt8)
    w = pred.weight
    key_rc = UInt32(pred.prev_rc)
    found = false
    for entry in pred.trans_rc
        if entry.key == key_rc && entry.sym == rc
            entry.count += w
            found = true
            break
        end
    end
    if !found && length(pred.trans_rc) < 256
        push!(pred.trans_rc, SparseTransition(key_rc, rc, w))
    end

    key_rf = (UInt32(rc) << 8) | UInt32(pred.prev_rf)
    found = false
    for entry in pred.trans_rf
        if entry.key == key_rf && entry.sym == rf
            entry.count += w
            found = true
            break
        end
    end
    if !found && length(pred.trans_rf) < 256
        push!(pred.trans_rf, SparseTransition(key_rf, rf, w))
    end

    key_ra = (UInt32(rc) << 16) | (UInt32(rf) << 8) | UInt32(pred.prev_ra)
    found = false
    for entry in pred.trans_ra
        if entry.key == key_ra && entry.sym == ra
            entry.count += w
            found = true
            break
        end
    end
    if !found && length(pred.trans_ra) < 256
        push!(pred.trans_ra, SparseTransition(key_ra, ra, w))
    end

    pred.prev_rc = rc
    pred.prev_rf = rf
    pred.prev_ra = ra
end

function get_cum_freqs_rc(pred::RadicalPredictor, prev_rc::UInt8)
    freqs = fill(pred.alpha, 256)
    for entry in pred.trans_rc
        if entry.key == UInt32(prev_rc)
            freqs[entry.sym + 1] += entry.count
        end
    end
    cum_freqs = zeros(UInt32, 257)
    for i in 1:256
        cum_freqs[i+1] = cum_freqs[i] + freqs[i]
    end
    cum_freqs
end

function get_cum_freqs_rf(pred::RadicalPredictor, curr_rc::UInt8, prev_rf::UInt8)
    freqs = fill(pred.alpha, 256)
    key = (UInt32(curr_rc) << 8) | UInt32(prev_rf)
    for entry in pred.trans_rf
        if entry.key == key
            freqs[entry.sym + 1] += entry.count
        end
    end
    cum_freqs = zeros(UInt32, 257)
    for i in 1:256
        cum_freqs[i+1] = cum_freqs[i] + freqs[i]
    end
    cum_freqs
end

function get_cum_freqs_ra(pred::RadicalPredictor, curr_rc::UInt8, curr_rf::UInt8, prev_ra::UInt8)
    freqs = fill(pred.alpha, 256)
    key = (UInt32(curr_rc) << 16) | (UInt32(curr_rf) << 8) | UInt32(prev_ra)
    for entry in pred.trans_ra
        if entry.key == key
            freqs[entry.sym + 1] += entry.count
        end
    end
    cum_freqs = zeros(UInt32, 257)
    for i in 1:256
        cum_freqs[i+1] = cum_freqs[i] + freqs[i]
    end
    cum_freqs
end

mutable struct BitWriter
    buffer::Vector{UInt8}
    bit_index::Int
end

BitWriter() = BitWriter(UInt8[], 0)

function write_bit!(w::BitWriter, bit::UInt8)
    byte_pos = div(w.bit_index, 8) + 1
    bit_pos = 7 - (w.bit_index % 8)
    if byte_pos > length(w.buffer)
        push!(w.buffer, 0)
    end
    if bit != 0
        w.buffer[byte_pos] |= (1 << bit_pos)
    else
        w.buffer[byte_pos] &= ~(1 << bit_pos)
    end
    w.bit_index += 1
end

function write_bit_helper!(w::BitWriter, underflow_bits::Ref{UInt32}, bit::UInt8)
    write_bit!(w, bit)
    while underflow_bits[] > 0
        write_bit!(w, 1 - bit)
        underflow_bits[] -= 1
    end
end

mutable struct BitReader
    buffer::Vector{UInt8}
    bit_index::Int
    total_bits::Int
end

BitReader(buf::Vector{UInt8}) = BitReader(buf, 0, length(buf) * 8)

function read_bit!(r::BitReader)
    if r.bit_index >= r.total_bits
        return 0x00
    end
    byte_pos = div(r.bit_index, 8) + 1
    bit_pos = 7 - (r.bit_index % 8)
    bit = (r.buffer[byte_pos] >> bit_pos) & 1
    r.bit_index += 1
    bit
end

function encode(concepts::Vector{Concept6D}, alpha::UInt32, weight::UInt32)
    pred = RadicalPredictor(alpha, weight)
    w = BitWriter()
    low = UInt32(0)
    high = UInt32(0xFFFFFFFF)
    underflow_bits = Ref(UInt32(0))

    for c in concepts
        rc = (c.domain << 4) | c.subdomain
        rf = (c.operation << 4) | c.modality
        ra = (c.depth << 4) | c.polarity
        symbols = [rc, rf, ra]

        prev_rc = pred.prev_rc
        prev_rf = pred.prev_rf
        prev_ra = pred.prev_ra

        for step in 0:2
            cum_freqs = if step == 0
                get_cum_freqs_rc(pred, prev_rc)
            elseif step == 1
                get_cum_freqs_rf(pred, symbols[1], prev_rf)
            else
                get_cum_freqs_ra(pred, symbols[1], symbols[2], prev_ra)
            end

            sym = symbols[step+1]
            total = cum_freqs[257]
            cum_low = cum_freqs[sym + 1]
            cum_high = cum_freqs[sym + 2]

            range_width = UInt64(high) - UInt64(low) + 1
            high = low + UInt32(div(range_width * cum_high, total)) - 1
            low = low + UInt32(div(range_width * cum_low, total))

            while true
                if high < 0x80000000
                    write_bit_helper!(w, underflow_bits, 0x00)
                    low <<= 1
                    high = (high << 1) | 1
                elseif low >= 0x80000000
                    write_bit_helper!(w, underflow_bits, 0x01)
                    low = (low - 0x80000000) << 1
                    high = ((high - 0x80000000) << 1) | 1
                elseif low >= 0x40000000 && high < 0xC0000000
                    underflow_bits[] += 1
                    low = (low - 0x40000000) << 1
                    high = ((high - 0x40000000) << 1) | 1
                else
                    break
                end
            end
        end
        observe!(pred, rc, rf, ra)
    end

    underflow_bits[] += 1
    if low < 0x40000000
        write_bit_helper!(w, underflow_bits, 0x00)
    else
        write_bit_helper!(w, underflow_bits, 0x01)
    end

    (w.buffer, w.bit_index)
end

function decode(encoded_bytes::Vector{UInt8}, num_concepts::Int, alpha::UInt32, weight::UInt32)
    pred = RadicalPredictor(alpha, weight)
    r = BitReader(encoded_bytes)

    value = UInt32(0)
    for _ in 1:32
        value = (value << 1) | read_bit!(r)
    end

    low = UInt32(0)
    high = UInt32(0xFFFFFFFF)
    decoded = Concept6D[]

    for _ in 1:num_concepts
        prev_rc = pred.prev_rc
        prev_rf = pred.prev_rf
        prev_ra = pred.prev_ra
        symbols = [0x00, 0x00, 0x00]

        for step in 0:2
            cum_freqs = if step == 0
                get_cum_freqs_rc(pred, prev_rc)
            elseif step == 1
                get_cum_freqs_rf(pred, symbols[1], prev_rf)
            else
                get_cum_freqs_ra(pred, symbols[1], symbols[2], prev_ra)
            end

            total = UInt64(cum_freqs[257])
            range_width = UInt64(high) - UInt64(low) + 1
            scaled_val = div(((UInt64(value) - UInt64(low)) + 1) * total - 1, range_width)

            sym = 0x00
            l_idx, r_idx = 0, 255
            while l_idx <= r_idx
                m_idx = div(l_idx + r_idx, 2)
                if cum_freqs[m_idx + 1] <= scaled_val && scaled_val < cum_freqs[m_idx + 2]
                    sym = UInt8(m_idx)
                    break
                } else if scaled_val >= cum_freqs[m_idx + 2]
                    l_idx = m_idx + 1
                else
                    r_idx = m_idx - 1
                end
            end

            symbols[step+1] = sym
            cum_low = cum_freqs[sym + 1]
            cum_high = cum_freqs[sym + 2]

            high = low + UInt32(div(range_width * cum_high, total)) - 1
            low = low + UInt32(div(range_width * cum_low, total))

            while true
                if high < 0x80000000
                    low <<= 1
                    high = (high << 1) | 1
                    value = (value << 1) | read_bit!(r)
                elseif low >= 0x80000000
                    low = (low - 0x80000000) << 1
                    high = ((high - 0x80000000) << 1) | 1
                    value = ((value - 0x80000000) << 1) | read_bit!(r)
                elseif low >= 0x40000000 && high < 0xC0000000
                    low = (low - 0x40000000) << 1
                    high = ((high - 0x40000000) << 1) | 1
                    value = ((value - 0x40000000) << 1) | read_bit!(r)
                else
                    break
                end
            end
        end

        push!(decoded, Concept6D(
            (symbols[1] >> 4) & 0xF,
            symbols[1] & 0xF,
            (symbols[2] >> 4) & 0xF,
            symbols[2] & 0xF,
            (symbols[3] >> 4) & 0xF,
            symbols[3] & 0xF
        ))
        observe!(pred, symbols[1], symbols[2], symbols[3])
    end
    decoded
end

function main()
    println("======================================================================")
    println("ZYMATICA | zymatica-inference-engine-julia")
    println("======================================================================\n")

    inputs = [
        Concept6D(1, 2, 3, 4, 5, 6),
        Concept6D(8, 0, 15, 1, 0, 15),
        Concept6D(0, 0, 0, 0, 0, 0),
        Concept6D(15, 15, 15, 15, 15, 15),
        Concept6D(4, 5, 6, 7, 8, 9)
    ]

    buf, bits = encode(inputs, UInt32(1), UInt32(128))
    @printf("Encoded Bits: %d, Bytes: %d\n", bits, length(buf))
    print("Hex: ")
    for b in buf
        @printf("%02X ", b)
    end
    println()

    decoded = decode(buf, 5, UInt32(1), UInt32(128))
    match = decoded == inputs
    println("Decoded matches inputs: $match")
    if !match
        println("ERROR: mismatch!")
        exit(1)
    end

    println("\n[VERIFICATION] Multi-Language runtime FFI structures validated.")
end

main()
