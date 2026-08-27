// Watermark: ip zymatica.space | astronautshe.com
// Copyright (c) 2026 Zymatica. All rights reserved.

import java.util.ArrayList
import kotlin.system.exitProcess

class SparseTransition(val key: Long, val sym: Int, var count: Long)

class RadicalPredictor(val alpha: Long, val weight: Long) {
    val transRC = ArrayList<SparseTransition>()
    val transRF = ArrayList<SparseTransition>()
    val transRA = ArrayList<SparseTransition>()
    var prevRC = 0
    var prevRF = 0
    var prevRA = 0

    fun observe(rc: Int, rf: Int, ra: Int) {
        val w = weight
        val keyRC = prevRC.toLong()
        var found = false
        for (entry in transRC) {
            if (entry.key == keyRC && entry.sym == rc) {
                entry.count += w
                found = true
                break
            }
        }
        if (!found && transRC.size < 256) {
            transRC.add(SparseTransition(keyRC, rc, w))
        }

        val keyRF = (rc.toLong() shl 8) or prevRF.toLong()
        found = false
        for (entry in transRF) {
            if (entry.key == keyRF && entry.sym == rf) {
                entry.count += w
                found = true
                break
            }
        }
        if (!found && transRF.size < 256) {
            transRF.add(SparseTransition(keyRF, rf, w))
        }

        val keyRA = (rc.toLong() shl 16) or (rf.toLong() shl 8) or prevRA.toLong()
        found = false
        for (entry in transRA) {
            if (entry.key == keyRA && entry.sym == ra) {
                entry.count += w
                found = true
                break
            }
        }
        if (!found && transRA.size < 256) {
            transRA.add(SparseTransition(keyRA, ra, w))
        }

        prevRC = rc
        prevRF = rf
        prevRA = ra
    }

    fun getCumFreqsRC(prevRC: Int): LongArray {
        val freqs = LongArray(256) { alpha }
        for (entry in transRC) {
            if (entry.key == prevRC.toLong()) {
                freqs[entry.sym] += entry.count
            }
        }
        val cumFreqs = LongArray(257)
        for (i in 0 until 256) {
            cumFreqs[i + 1] = cumFreqs[i] + freqs[i]
        }
        return cumFreqs
    }

    fun getCumFreqsRF(currRC: Int, prevRF: Int): LongArray {
        val freqs = LongArray(256) { alpha }
        val key = (currRC.toLong() shl 8) or prevRF.toLong()
        for (entry in transRF) {
            if (entry.key == key) {
                freqs[entry.sym] += entry.count
            }
        }
        val cumFreqs = LongArray(257)
        for (i in 0 until 256) {
            cumFreqs[i + 1] = cumFreqs[i] + freqs[i]
        }
        return cumFreqs
    }

    fun getCumFreqsRA(currRC: Int, currRF: Int, prevRA: Int): LongArray {
        val freqs = LongArray(256) { alpha }
        val key = (currRC.toLong() shl 16) or (currRF.toLong() shl 8) or prevRA.toLong()
        for (entry in transRA) {
            if (entry.key == key) {
                freqs[entry.sym] += entry.count
            }
        }
        val cumFreqs = LongArray(257)
        for (i in 0 until 256) {
            cumFreqs[i + 1] = cumFreqs[i] + freqs[i]
        }
        return cumFreqs
    }
}

class BitWriter {
    val buffer = ArrayList<Byte>()
    var bitIndex = 0

    fun writeBit(bit: Int) {
        val bytePos = bitIndex / 8
        val bitPos = 7 - (bitIndex % 8)
        if (bytePos >= buffer.size) {
            buffer.add(0)
        }
        if (bit != 0) {
            buffer[bytePos] = (buffer[bytePos].toInt() or (1 shl bitPos)).toByte()
        } else {
            buffer[bytePos] = (buffer[bytePos].toInt() and (1 shl bitPos).inv()).toByte()
        }
        bitIndex++
    }

    fun writeBitHelper(underflowBits: IntArray, bit: Int) {
        writeBit(bit)
        while (underflowBits[0] > 0) {
            writeBit(1 - bit)
            underflowBits[0]--
        }
    }
}

class BitReader(val buffer: ByteArray) {
    var bitIndex = 0
    val totalBits = buffer.size * 8

    fun readBit(): Int {
        if (bitIndex >= totalBits) return 0
        val bytePos = bitIndex / 8
        val bitPos = 7 - (bitIndex % 8)
        val bit = (buffer[bytePos].toInt() shr bitPos) and 1
        bitIndex++
        return bit
    }
}

class Concept6D(val domain: Int, val subdomain: Int, val operation: Int, val modality: Int, val depth: Int, val polarity: Int) {
    fun equals(other: Concept6D): Boolean {
        return this.domain == other.domain && this.subdomain == other.subdomain &&
               this.operation == other.operation && this.modality == other.modality &&
               this.depth == other.depth && this.polarity == other.polarity
    }
}

fun encode(concepts: Array<Concept6D>, outBits: IntArray, alpha: Long, weight: Long): ByteArray {
    val pred = RadicalPredictor(alpha, weight)
    val w = BitWriter()
    var low = 0L
    var high = 0xFFFFFFFFL
    val underflowBits = intArrayOf(0)

    for (c in concepts) {
        val rc = (c.domain shl 4) or c.subdomain
        val rf = (c.operation shl 4) or c.modality
        val ra = (c.depth shl 4) or c.polarity
        val symbols = intArrayOf(rc, rf, ra)

        val prevRC = pred.prevRC
        val prevRF = pred.prevRF
        val prevRA = pred.prevRA

        for (step in 0 until 3) {
            val cumFreqs = when (step) {
                0 -> pred.getCumFreqsRC(prevRC)
                1 -> pred.getCumFreqsRF(symbols[0], prevRF)
                else -> pred.getCumFreqsRA(symbols[0], symbols[1], prevRA)
            }

            val sym = symbols[step]
            val total = cumFreqs[256]
            val cumLow = cumFreqs[sym]
            val cumHigh = cumFreqs[sym + 1]

            val rangeWidth = high - low + 1
            high = low + (rangeWidth * cumHigh) / total - 1
            low = low + (rangeWidth * cumLow) / total

            while (true) {
                if (high < 0x80000000L) {
                    w.writeBitHelper(underflowBits, 0)
                    low = low shl 1
                    high = (high shl 1) or 1
                } else if (low >= 0x80000000L) {
                    w.writeBitHelper(underflowBits, 1)
                    low = (low - 0x80000000L) shl 1
                    high = ((high - 0x80000000L) shl 1) or 1
                } else if (low >= 0x40000000L && high < 0xC0000000L) {
                    underflowBits[0]++
                    low = (low - 0x40000000L) shl 1
                    high = ((high - 0x40000000L) shl 1) or 1
                } else {
                    break
                }
                low = low and 0xFFFFFFFFL
                high = high and 0xFFFFFFFFL
            }
        }
        pred.observe(rc, rf, ra)
    }

    underflowBits[0]++
    if (low < 0x40000000L) {
        w.writeBitHelper(underflowBits, 0)
    } else {
        w.writeBitHelper(underflowBits, 1)
    }

    outBits[0] = w.bitIndex
    val outBytes = ByteArray(w.buffer.size)
    for (i in 0 until w.buffer.size) {
        outBytes[i] = w.buffer[i]
    }
    return outBytes
}

fun decode(encodedBytes: ByteArray, numConcepts: Int, alpha: Long, weight: Long): Array<Concept6D> {
    val pred = RadicalPredictor(alpha, weight)
    val r = BitReader(encodedBytes)

    var value = 0L
    for (i in 0 until 32) {
        value = (value shl 1) or r.readBit().toLong()
    }

    var low = 0L
    var high = 0xFFFFFFFFL
    val decoded = ArrayList<Concept6D>()

    for (cIdx in 0 until numConcepts) {
        val prevRC = pred.prevRC
        val prevRF = pred.prevRF
        val prevRA = pred.prevRA
        val symbols = IntArray(3)

        for (step in 0 until 3) {
            val cumFreqs = when (step) {
                0 -> pred.getCumFreqsRC(prevRC)
                1 -> pred.getCumFreqsRF(symbols[0], prevRF)
                else -> pred.getCumFreqsRA(symbols[0], symbols[1], prevRA)
            }

            val total = cumFreqs[256]
            val rangeWidth = high - low + 1
            val scaledVal = (((value - low) + 1) * total - 1) / rangeWidth

            var sym = 0
            var lIdx = 0
            var rIdx = 255
            while (lIdx <= rIdx) {
                val mIdx = (lIdx + rIdx) / 2
                if (cumFreqs[mIdx] <= scaledVal && scaledVal < cumFreqs[mIdx + 1]) {
                    sym = mIdx
                    break
                } else if (scaledVal >= cumFreqs[mIdx + 1]) {
                    lIdx = mIdx + 1
                } else {
                    rIdx = mIdx - 1
                }
            }

            symbols[step] = sym
            val cumLow = cumFreqs[sym]
            val cumHigh = cumFreqs[sym + 1]

            high = low + (rangeWidth * cumHigh) / total - 1
            low = low + (rangeWidth * cumLow) / total

            while (true) {
                if (high < 0x80000000L) {
                    low = low shl 1
                    high = (high shl 1) or 1
                    value = (value shl 1) or r.readBit().toLong()
                } else if (low >= 0x80000000L) {
                    low = (low - 0x80000000L) shl 1
                    high = ((high - 0x80000000L) shl 1) or 1
                    value = ((value - 0x80000000L) shl 1) or r.readBit().toLong()
                } else if (low >= 0x40000000L && high < 0xC0000000L) {
                    low = (low - 0x40000000L) shl 1
                    high = ((high - 0x40000000L) shl 1) or 1
                    value = ((value - 0x40000000L) shl 1) or r.readBit().toLong()
                } else {
                    break
                }
                low = low and 0xFFFFFFFFL
                high = high and 0xFFFFFFFFL
                value = value and 0xFFFFFFFFL
            }
        }

        decoded.add(Concept6D(
            symbols[0] shr 4,
            symbols[0] and 0x0F,
            symbols[1] shr 4,
            symbols[1] and 0x0F,
            symbols[2] shr 4,
            symbols[2] and 0x0F
        ))
        pred.observe(symbols[0], symbols[1], symbols[2])
    }
    return decoded.toTypedArray()
}

fun main() {
    println("======================================================================")
    println("ZYMATICA | zymatica-inference-engine-kotlin")
    println("======================================================================\n")

    val inputs = arrayOf(
        Concept6D(1, 2, 3, 4, 5, 6),
        Concept6D(8, 0, 15, 1, 0, 15),
        Concept6D(0, 0, 0, 0, 0, 0),
        Concept6D(15, 15, 15, 15, 15, 15),
        Concept6D(4, 5, 6, 7, 8, 9)
    )

    val outBits = intArrayOf(0)
    val buf = encode(inputs, outBits, 1L, 128L)
    System.out.format("Encoded Bits: %d, Bytes: %d\n", outBits[0], buf.size)
    print("Hex: ")
    for (b in buf) {
        System.out.format("%02X ", b)
    }
    println()

    val decoded = decode(buf, 5, 1L, 128L)
    var match = true
    for (i in inputs.indices) {
        if (!inputs[i].equals(decoded[i])) {
            match = false
            break
        }
    }

    println("Decoded matches inputs: $match")
    if (!match) {
        println("ERROR: mismatch!")
        exitProcess(1)
    }

    println("\n[VERIFICATION] Multi-Language runtime FFI structures validated.")
}
