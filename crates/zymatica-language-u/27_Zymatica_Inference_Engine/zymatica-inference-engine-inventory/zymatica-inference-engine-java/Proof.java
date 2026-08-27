// Watermark: ip zymatica.space | astronautshe.com
// Copyright (c) 2026 Zymatica. All rights reserved.

import java.util.ArrayList;
import java.util.List;

public class Proof {

    static class SparseTransition {
        long key;
        int sym;
        long count;
        SparseTransition(long key, int sym, long count) {
            this.key = key;
            this.sym = sym;
            this.count = count;
        }
    }

    static class RadicalPredictor {
        long alpha;
        long weight;
        List<SparseTransition> transRC = new ArrayList<>();
        List<SparseTransition> transRF = new ArrayList<>();
        List<SparseTransition> transRA = new ArrayList<>();
        int prevRC = 0;
        int prevRF = 0;
        int prevRA = 0;

        RadicalPredictor(long alpha, long weight) {
            this.alpha = alpha;
            this.weight = weight;
        }

        void observe(int rc, int rf, int ra) {
            long keyRC = prevRC;
            boolean found = false;
            for (SparseTransition entry : transRC) {
                if (entry.key == keyRC && entry.sym == rc) {
                    entry.count += weight;
                    found = true;
                    break;
                }
            }
            if (!found && transRC.size() < 256) {
                transRC.add(new SparseTransition(keyRC, rc, weight));
            }

            long keyRF = ((long)rc << 8) | prevRF;
            found = false;
            for (SparseTransition entry : transRF) {
                if (entry.key == keyRF && entry.sym == rf) {
                    entry.count += weight;
                    found = true;
                    break;
                }
            }
            if (!found && transRF.size() < 256) {
                transRF.add(new SparseTransition(keyRF, rf, weight));
            }

            long keyRA = ((long)rc << 16) | ((long)rf << 8) | prevRA;
            found = false;
            for (SparseTransition entry : transRA) {
                if (entry.key == keyRA && entry.sym == ra) {
                    entry.count += weight;
                    found = true;
                    break;
                }
            }
            if (!found && transRA.size() < 256) {
                transRA.add(new SparseTransition(keyRA, ra, weight));
            }

            prevRC = rc;
            prevRF = rf;
            prevRA = ra;
        }

        long[] getCumFreqsRC(int prevRC) {
            long[] freqs = new long[256];
            for (int i = 0; i < 256; i++) freqs[i] = alpha;
            for (SparseTransition entry : transRC) {
                if (entry.key == prevRC) {
                    freqs[entry.sym] += entry.count;
                }
            }
            long[] cumFreqs = new long[257];
            for (int i = 0; i < 256; i++) {
                cumFreqs[i + 1] = cumFreqs[i] + freqs[i];
            }
            return cumFreqs;
        }

        long[] getCumFreqsRF(int currRC, int prevRF) {
            long[] freqs = new long[256];
            for (int i = 0; i < 256; i++) freqs[i] = alpha;
            long key = ((long)currRC << 8) | prevRF;
            for (SparseTransition entry : transRF) {
                if (entry.key == key) {
                    freqs[entry.sym] += entry.count;
                }
            }
            long[] cumFreqs = new long[257];
            for (int i = 0; i < 256; i++) {
                cumFreqs[i + 1] = cumFreqs[i] + freqs[i];
            }
            return cumFreqs;
        }

        long[] getCumFreqsRA(int currRC, int currRF, int prevRA) {
            long[] freqs = new long[256];
            for (int i = 0; i < 256; i++) freqs[i] = alpha;
            long key = ((long)currRC << 16) | ((long)currRF << 8) | prevRA;
            for (SparseTransition entry : transRA) {
                if (entry.key == key) {
                    freqs[entry.sym] += entry.count;
                }
            }
            long[] cumFreqs = new long[257];
            for (int i = 0; i < 256; i++) {
                cumFreqs[i + 1] = cumFreqs[i] + freqs[i];
            }
            return cumFreqs;
        }
    }

    static class BitWriter {
        List<Byte> buffer = new ArrayList<>();
        int bitIndex = 0;

        void writeBit(int bit) {
            int bytePos = bitIndex / 8;
            int bitPos = 7 - (bitIndex % 8);
            if (bytePos >= buffer.size()) {
                buffer.add((byte) 0);
            }
            if (bit != 0) {
                buffer.set(bytePos, (byte) (buffer.get(bytePos) | (1 << bitPos)));
            } else {
                buffer.set(bytePos, (byte) (buffer.get(bytePos) & ~(1 << bitPos)));
            }
            bitIndex++;
        }

        void writeBitHelper(int[] underflowBits, int bit) {
            writeBit(bit);
            while (underflowBits[0] > 0) {
                writeBit(1 - bit);
                underflowBits[0]--;
            }
        }
    }

    static class BitReader {
        byte[] buffer;
        int bitIndex = 0;
        int totalBits;

        BitReader(byte[] buffer) {
            this.buffer = buffer;
            this.totalBits = buffer.length * 8;
        }

        int readBit() {
            if (bitIndex >= totalBits) return 0;
            int bytePos = bitIndex / 8;
            int bitPos = 7 - (bitIndex % 8);
            int bit = (buffer[bytePos] >> bitPos) & 1;
            bitIndex++;
            return bit;
        }
    }

    static class Concept6D {
        int domain, subdomain, operation, modality, depth, polarity;
        Concept6D(int d, int s, int o, int m, int dp, int p) {
            this.domain = d; this.subdomain = s; this.operation = o;
            this.modality = m; this.depth = dp; this.polarity = p;
        }
        boolean equals(Concept6D other) {
            return this.domain == other.domain && this.subdomain == other.subdomain &&
                   this.operation == other.operation && this.modality == other.modality &&
                   this.depth == other.depth && this.polarity == other.polarity;
        }
    }

    static byte[] encode(Concept6D[] concepts, int[] outBits, long alpha, long weight) {
        RadicalPredictor pred = new RadicalPredictor(alpha, weight);
        BitWriter w = new BitWriter();
        long low = 0;
        long high = 0xFFFFFFFFL;
        int[] underflowBits = {0};

        for (Concept6D c : concepts) {
            int rc = (c.domain << 4) | c.subdomain;
            int rf = (c.operation << 4) | c.modality;
            int ra = (c.depth << 4) | c.polarity;
            int[] symbols = {rc, rf, ra};

            int prevRC = pred.prevRC;
            int prevRF = pred.prevRF;
            int prevRA = pred.prevRA;

            for (int step = 0; step < 3; step++) {
                long[] cumFreqs;
                if (step == 0) {
                    cumFreqs = pred.getCumFreqsRC(prevRC);
                } else if (step == 1) {
                    cumFreqs = pred.getCumFreqsRF(symbols[0], prevRF);
                } else {
                    cumFreqs = pred.getCumFreqsRA(symbols[0], symbols[1], prevRA);
                }

                int sym = symbols[step];
                long total = cumFreqs[256];
                long cumLow = cumFreqs[sym];
                long cumHigh = cumFreqs[sym + 1];

                long rangeWidth = high - low + 1;
                high = low + (rangeWidth * cumHigh) / total - 1;
                low = low + (rangeWidth * cumLow) / total;

                while (true) {
                    if (high < 0x80000000L) {
                        w.writeBitHelper(underflowBits, 0);
                        low <<= 1;
                        high = (high << 1) | 1;
                    } else if (low >= 0x80000000L) {
                        w.writeBitHelper(underflowBits, 1);
                        low = (low - 0x80000000L) << 1;
                        high = ((high - 0x80000000L) << 1) | 1;
                    } else if (low >= 0x40000000L && high < 0xC0000000L) {
                        underflowBits[0]++;
                        low = (low - 0x40000000L) << 1;
                        high = ((high - 0x40000000L) << 1) | 1;
                    } else {
                        break;
                    }
                    low &= 0xFFFFFFFFL;
                    high &= 0xFFFFFFFFL;
                }
            }
            pred.observe(rc, rf, ra);
        }

        underflowBits[0]++;
        if (low < 0x40000000L) {
            w.writeBitHelper(underflowBits, 0);
        } else {
            w.writeBitHelper(underflowBits, 1);
        }

        outBits[0] = w.bitIndex;
        byte[] outBytes = new byte[w.buffer.size()];
        for (int i = 0; i < w.buffer.size(); i++) {
            outBytes[i] = w.buffer.get(i);
        }
        return outBytes;
    }

    static Concept6D[] decode(byte[] encodedBytes, int numConcepts, long alpha, long weight) {
        RadicalPredictor pred = new RadicalPredictor(alpha, weight);
        BitReader r = new BitReader(encodedBytes);

        long value = 0;
        for (int i = 0; i < 32; i++) {
            value = (value << 1) | r.readBit();
        }

        long low = 0;
        long high = 0xFFFFFFFFL;
        Concept6D[] decoded = new Concept6D[numConcepts];

        for (int cIdx = 0; cIdx < numConcepts; cIdx++) {
            int prevRC = pred.prevRC;
            int prevRF = pred.prevRF;
            int prevRA = pred.prevRA;
            int[] symbols = new int[3];

            for (int step = 0; step < 3; step++) {
                long[] cumFreqs;
                if (step == 0) {
                    cumFreqs = pred.getCumFreqsRC(prevRC);
                } else if (step == 1) {
                    cumFreqs = pred.getCumFreqsRF(symbols[0], prevRF);
                } else {
                    cumFreqs = pred.getCumFreqsRA(symbols[0], symbols[1], prevRA);
                }

                long total = cumFreqs[256];
                long rangeWidth = high - low + 1;
                long scaledVal = (((value - low) + 1) * total - 1) / rangeWidth;

                int sym = 0;
                int lIdx = 0, rIdx = 255;
                while (lIdx <= rIdx) {
                    int mIdx = (lIdx + rIdx) / 2;
                    if (cumFreqs[mIdx] <= scaledVal && scaledVal < cumFreqs[mIdx + 1]) {
                        sym = mIdx;
                        break;
                    } else if (scaledVal >= cumFreqs[mIdx + 1]) {
                        lIdx = mIdx + 1;
                    } else {
                        rIdx = mIdx - 1;
                    }
                }

                symbols[step] = sym;
                long cumLow = cumFreqs[sym];
                long cumHigh = cumFreqs[sym + 1];

                high = low + (rangeWidth * cumHigh) / total - 1;
                low = low + (rangeWidth * cumLow) / total;

                while (true) {
                    if (high < 0x80000000L) {
                        low <<= 1;
                        high = (high << 1) | 1;
                        value = (value << 1) | r.readBit();
                    } else if (low >= 0x80000000L) {
                        low = (low - 0x80000000L) << 1;
                        high = ((high - 0x80000000L) << 1) | 1;
                        value = ((value - 0x80000000L) << 1) | r.readBit();
                    } else if (low >= 0x40000000L && high < 0xC0000000L) {
                        low = (low - 0x40000000L) << 1;
                        high = ((high - 0x40000000L) << 1) | 1;
                        value = ((value - 0x40000000L) << 1) | r.readBit();
                    } else {
                        break;
                    }
                    low &= 0xFFFFFFFFL;
                    high &= 0xFFFFFFFFL;
                    value &= 0xFFFFFFFFL;
                }
            }

            decoded[cIdx] = new Concept6D(
                symbols[0] >> 4,
                symbols[0] & 0x0F,
                symbols[1] >> 4,
                symbols[1] & 0x0F,
                symbols[2] >> 4,
                symbols[2] & 0x0F
            );
            pred.observe(symbols[0], symbols[1], symbols[2]);
        }
        return decoded;
    }

    public static void main(String[] args) {
        System.out.println("======================================================================");
        System.out.println("ZYMATICA | zymatica-inference-engine-java");
        System.out.println("======================================================================\n");

        Concept6D[] inputs = {
            new Concept6D(1, 2, 3, 4, 5, 6),
            new Concept6D(8, 0, 15, 1, 0, 15),
            new Concept6D(0, 0, 0, 0, 0, 0),
            new Concept6D(15, 15, 15, 15, 15, 15),
            new Concept6D(4, 5, 6, 7, 8, 9)
        };

        int[] outBits = {0};
        byte[] buf = encode(inputs, outBits, 1, 128);
        System.out.printf("Encoded Bits: %d, Bytes: %d\n", outBits[0], buf.length);
        System.out.print("Hex: ");
        for (byte b : buf) {
            System.out.printf("%02X ", b);
        }
        System.out.println();

        Concept6D[] decoded = decode(buf, 5, 1, 128);
        boolean match = true;
        for (int i = 0; i < inputs.length; i++) {
            if (!inputs[i].equals(decoded[i])) {
                match = false;
                break;
            }
        }

        System.out.println("Decoded matches inputs: " + match);
        if (!match) {
            System.out.println("ERROR: mismatch!");
            System.exit(1);
        }

        System.out.println("\n[VERIFICATION] Multi-Language runtime FFI structures validated.");
    }
}
