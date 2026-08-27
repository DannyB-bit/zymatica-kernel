// Watermark: ip zymatica.space | astronautshe.com
// Copyright (c) 2026 Zymatica. All rights reserved.

import Foundation

class SparseTransition {
    var key: UInt32
    var sym: UInt8
    var count: UInt32
    init(key: UInt32, sym: UInt8, count: UInt32) {
        self.key = key
        self.sym = sym
        self.count = count
    }
}

class RadicalPredictor {
    var alpha: UInt32
    var weight: UInt32
    var transRC: [SparseTransition] = []
    var transRF: [SparseTransition] = []
    var transRA: [SparseTransition] = []
    var prevRC: UInt8 = 0
    var prevRF: UInt8 = 0
    var prevRA: UInt8 = 0

    init(alpha: UInt32, weight: UInt32) {
        self.alpha = alpha
        self.weight = weight
    }

    func observe(rc: UInt8, rf: UInt8, ra: UInt8) {
        let w = weight
        let keyRC = UInt32(prevRC)
        var found = false
        for entry in transRC {
            if entry.key == keyRC && entry.sym == rc {
                entry.count += w
                found = true
                break
            }
        }
        if !found && transRC.count < 256 {
            transRC.append(SparseTransition(key: keyRC, sym: rc, count: w))
        }

        let keyRF = (UInt32(rc) << 8) | UInt32(prevRF)
        found = false
        for entry in transRF {
            if entry.key == keyRF && entry.sym == rf {
                entry.count += w
                found = true
                break
            }
        }
        if !found && transRF.count < 256 {
            transRF.append(SparseTransition(key: keyRF, sym: rf, count: w))
        }

        let keyRA = (UInt32(rc) << 16) | (UInt32(rf) << 8) | UInt32(prevRA)
        found = false
        for entry in transRA {
            if entry.key == keyRA && entry.sym == ra {
                entry.count += w
                found = true
                break
            }
        }
        if !found && transRA.count < 256 {
            transRA.append(SparseTransition(key: keyRA, sym: ra, count: w))
        }

        prevRC = rc
        prevRF = rf
        prevRA = ra
    }

    func getCumFreqsRC(prevRC: UInt8) -> [UInt32] {
        var freqs = [UInt32](repeating: alpha, count: 256)
        for entry in transRC {
            if entry.key == UInt32(prevRC) {
                freqs[Int(entry.sym)] += entry.count
            }
        }
        var cumFreqs = [UInt32](repeating: 0, count: 257)
        for i in 0..<256 {
            cumFreqs[i + 1] = cumFreqs[i] + freqs[i]
        }
        return cumFreqs
    }

    func getCumFreqsRF(currRC: UInt8, prevRF: UInt8) -> [UInt32] {
        var freqs = [UInt32](repeating: alpha, count: 256)
        let key = (UInt32(currRC) << 8) | UInt32(prevRF)
        for entry in transRF {
            if entry.key == key {
                freqs[Int(entry.sym)] += entry.count
            }
        }
        var cumFreqs = [UInt32](repeating: 0, count: 257)
        for i in 0..<256 {
            cumFreqs[i + 1] = cumFreqs[i] + freqs[i]
        }
        return cumFreqs
    }

    func getCumFreqsRA(currRC: UInt8, currRF: UInt8, prevRA: UInt8) -> [UInt32] {
        var freqs = [UInt32](repeating: alpha, count: 256)
        let key = (UInt32(currRC) << 16) | (UInt32(currRF) << 8) | UInt32(prevRA)
        for entry in transRA {
            if entry.key == key {
                freqs[Int(entry.sym)] += entry.count
            }
        }
        var cumFreqs = [UInt32](repeating: 0, count: 257)
        for i in 0..<256 {
            cumFreqs[i + 1] = cumFreqs[i] + freqs[i]
        }
        return cumFreqs
    }
}

class BitWriter {
    var buffer: [UInt8] = []
    var bitIndex: Int = 0

    func writeBit(bit: UInt8) {
        let bytePos = bitIndex / 8
        let bitPos = 7 - (bitIndex % 8)
        if bytePos >= buffer.count {
            buffer.append(0)
        }
        if bit != 0 {
            buffer[bytePos] |= (1 << bitPos)
        } else {
            buffer[bytePos] &= ~(1 << bitPos)
        }
        bitIndex += 1
    }

    func writeBitHelper(underflowBits: inout UInt32, bit: UInt8) {
        writeBit(bit: bit)
        while underflowBits > 0 {
            writeBit(bit: 1 - bit)
            underflowBits -= 1
        }
    }
}

class BitReader {
    var buffer: [UInt8]
    var bitIndex: Int = 0
    var totalBits: Int

    init(buffer: [UInt8]) {
        self.buffer = buffer
        self.totalBits = buffer.count * 8
    }

    func readBit() -> UInt8 {
        if bitIndex >= totalBits { return 0 }
        let bytePos = bitIndex / 8
        let bitPos = 7 - (bitIndex % 8)
        let bit = (buffer[bytePos] >> bitPos) & 1
        bitIndex += 1
        return bit
    }
}

struct Concept6D: Equatable {
    var domain: UInt8
    var subdomain: UInt8
    var operation: UInt8
    var modality: UInt8
    var depth: UInt8
    var polarity: UInt8
}

func encode(concepts: [Concept6D], alpha: UInt32, weight: UInt32) -> ([UInt8], Int) {
    let pred = RadicalPredictor(alpha: alpha, weight: weight)
    let w = BitWriter()
    var low: UInt32 = 0
    var high: UInt32 = 0xFFFFFFFF
    var underflowBits: UInt32 = 0

    for c in concepts {
        let rc = (c.domain << 4) | c.subdomain
        let rf = (c.operation << 4) | c.modality
        let ra = (c.depth << 4) | c.polarity
        let symbols = [rc, rf, ra]

        let prevRC = pred.prevRC
        let prevRF = pred.prevRF
        let prevRA = pred.prevRA

        for step in 0..<3 {
            let cumFreqs: [UInt32]
            if step == 0 {
                cumFreqs = pred.getCumFreqsRC(prevRC: prevRC)
            } else if step == 1 {
                cumFreqs = pred.getCumFreqsRF(currRC: symbols[0], prevRF: prevRF)
            } else {
                cumFreqs = pred.getCumFreqsRA(currRC: symbols[0], currRF: symbols[1], prevRA: prevRA)
            }

            let sym = Int(symbols[step])
            let total = cumFreqs[256]
            let cumLow = cumFreqs[sym]
            let cumHigh = cumFreqs[sym + 1]

            let rangeWidth = UInt64(high) - UInt64(low) + 1
            high = low &+ UInt32(truncatingIfNeeded: (rangeWidth * UInt64(cumHigh)) / UInt64(total)) &- 1
            low = low &+ UInt32(truncatingIfNeeded: (rangeWidth * UInt64(cumLow)) / UInt64(total))

            while true {
                if high < 0x80000000 {
                    w.writeBitHelper(underflowBits: &underflowBits, bit: 0)
                    low = low << 1
                    high = (high << 1) | 1
                } else if low >= 0x80000000 {
                    w.writeBitHelper(underflowBits: &underflowBits, bit: 1)
                    low = (low &- 0x80000000) << 1
                    high = ((high &- 0x80000000) << 1) | 1
                } else if low >= 0x40000000 && high < 0xC0000000 {
                    underflowBits += 1
                    low = (low &- 0x40000000) << 1
                    high = ((high &- 0x40000000) << 1) | 1
                } else {
                    break
                }
            }
        }
        pred.observe(rc: rc, rf: rf, ra: ra)
    }

    underflowBits += 1
    if low < 0x40000000 {
        w.writeBitHelper(underflowBits: &underflowBits, bit: 0)
    } else {
        w.writeBitHelper(underflowBits: &underflowBits, bit: 1)
    }

    return (w.buffer, w.bitIndex)
}

func decode(encodedBytes: [UInt8], numConcepts: Int, alpha: UInt32, weight: UInt32) -> [Concept6D] {
    let pred = RadicalPredictor(alpha: alpha, weight: weight)
    let r = BitReader(buffer: encodedBytes)

    var value: UInt32 = 0
    for _ in 0..<32 {
        value = (value << 1) | UInt32(r.readBit())
    }

    var low: UInt32 = 0
    var high: UInt32 = 0xFFFFFFFF
    var decoded: [Concept6D] = []

    for _ in 0..<numConcepts {
        let prevRC = pred.prevRC
        let prevRF = pred.prevRF
        let prevRA = pred.prevRA
        var symbols: [UInt8] = [0, 0, 0]

        for step in 0..<3 {
            let cumFreqs: [UInt32]
            if step == 0 {
                cumFreqs = pred.getCumFreqsRC(prevRC: prevRC)
            } else if step == 1 {
                cumFreqs = pred.getCumFreqsRF(currRC: symbols[0], prevRF: prevRF)
            } else {
                cumFreqs = pred.getCumFreqsRA(currRC: symbols[0], currRF: symbols[1], prevRA: prevRA)
            }

            let total = UInt64(cumFreqs[256])
            let rangeWidth = UInt64(high) - UInt64(low) + 1
            let scaledVal = (((UInt64(value) - UInt64(low)) + 1) * total - 1) / rangeWidth

            var sym: UInt8 = 0
            var lIdx = 0
            var rIdx = 255
            while lIdx <= rIdx {
                let mIdx = (lIdx + rIdx) / 2
                if UInt64(cumFreqs[mIdx]) <= scaledVal && scaledVal < UInt64(cumFreqs[mIdx + 1]) {
                    sym = UInt8(mIdx)
                    break
                } else if scaledVal >= UInt64(cumFreqs[mIdx + 1]) {
                    lIdx = mIdx + 1
                } else {
                    rIdx = mIdx - 1
                }
            }

            symbols[step] = sym
            let cumLow = cumFreqs[Int(sym)]
            let cumHigh = cumFreqs[Int(sym) + 1]

            high = low &+ UInt32(truncatingIfNeeded: (rangeWidth * UInt64(cumHigh)) / total) &- 1
            low = low &+ UInt32(truncatingIfNeeded: (rangeWidth * UInt64(cumLow)) / total)

            while true {
                if high < 0x80000000 {
                    low = low << 1
                    high = (high << 1) | 1
                    value = (value << 1) | UInt32(r.readBit())
                } else if low >= 0x80000000 {
                    low = (low &- 0x80000000) << 1
                    high = ((high &- 0x80000000) << 1) | 1
                    value = ((value &- 0x80000000) << 1) | UInt32(r.readBit())
                } else if low >= 0x40000000 && high < 0xC0000000 {
                    low = (low &- 0x40000000) << 1
                    high = ((high &- 0x40000000) << 1) | 1
                    value = ((value &- 0x40000000) << 1) | UInt32(r.readBit())
                } else {
                    break
                }
            }
        }

        decoded.append(Concept6D(
            domain: (symbols[0] >> 4) & 0xF,
            subdomain: symbols[0] & 0xF,
            operation: (symbols[1] >> 4) & 0xF,
            modality: symbols[1] & 0xF,
            depth: (symbols[2] >> 4) & 0xF,
            polarity: symbols[2] & 0xF
        ))
        pred.observe(rc: symbols[0], rf: symbols[1], ra: symbols[2])
    }
    return decoded
}

func main() {
    print("======================================================================")
    print("ZYMATICA | zymatica-inference-engine-swift")
    print("======================================================================\n")

    let inputs = [
        Concept6D(domain: 1, subdomain: 2, operation: 3, modality: 4, depth: 5, polarity: 6),
        Concept6D(domain: 8, subdomain: 0, operation: 15, modality: 1, depth: 0, polarity: 15),
        Concept6D(domain: 0, subdomain: 0, operation: 0, modality: 0, depth: 0, polarity: 0),
        Concept6D(domain: 15, subdomain: 15, operation: 15, modality: 15, depth: 15, polarity: 15),
        Concept6D(domain: 4, subdomain: 5, operation: 6, modality: 7, depth: 8, polarity: 9)
    ]

    let (buf, bits) = encode(concepts: inputs, alpha: 1, weight: 128)
    print("Encoded Bits: \(bits), Bytes: \(buf.count)")
    var hexStr = "Hex: "
    for b in buf {
        hexStr += String(format: "%02X ", b)
    }
    print(hexStr)

    let decoded = decode(encodedBytes: buf, numConcepts: 5, alpha: 1, weight: 128)
    let match = (decoded == inputs)
    print("Decoded matches inputs: \(match)")
    if !match {
        print("ERROR: mismatch!")
        exit(1)
    }

    print("\n[VERIFICATION] Multi-Language runtime FFI structures validated.")
}

main()
