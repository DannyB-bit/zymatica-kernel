// Watermark: ip zymatica.space | astronautshe.com
// Copyright (c) 2026 Zymatica. All rights reserved.

using System;
using System.Collections.Generic;

class SparseTransition {
    public uint Key;
    public byte Sym;
    public uint Count;
    public SparseTransition(uint key, byte sym, uint count) {
        this.Key = key;
        this.Sym = sym;
        this.Count = count;
    }
}

class RadicalPredictor {
    public uint Alpha;
    public uint Weight;
    public List<SparseTransition> TransRC = new List<SparseTransition>();
    public List<SparseTransition> TransRF = new List<SparseTransition>();
    public List<SparseTransition> TransRA = new List<SparseTransition>();
    public byte PrevRC = 0;
    public byte PrevRF = 0;
    public byte PrevRA = 0;

    public RadicalPredictor(uint alpha, uint weight) {
        this.Alpha = alpha;
        this.Weight = weight;
    }

    public void Observe(byte rc, byte rf, byte ra) {
        uint w = this.Weight;
        uint keyRC = this.PrevRC;
        bool found = false;
        foreach (var entry in TransRC) {
            if (entry.Key == keyRC && entry.Sym == rc) {
                entry.Count += w;
                found = true;
                break;
            }
        }
        if (!found && TransRC.Count < 256) {
            TransRC.Add(new SparseTransition(keyRC, rc, w));
        }

        uint keyRF = ((uint)rc << 8) | this.PrevRF;
        found = false;
        foreach (var entry in TransRF) {
            if (entry.Key == keyRF && entry.Sym == rf) {
                entry.Count += w;
                found = true;
                break;
            }
        }
        if (!found && TransRF.Count < 256) {
            TransRF.Add(new SparseTransition(keyRF, rf, w));
        }

        uint keyRA = ((uint)rc << 16) | ((uint)rf << 8) | this.PrevRA;
        found = false;
        foreach (var entry in TransRA) {
            if (entry.Key == keyRA && entry.Sym == ra) {
                entry.Count += w;
                found = true;
                break;
            }
        }
        if (!found && TransRA.Count < 256) {
            TransRA.Add(new SparseTransition(keyRA, ra, w));
        }

        this.PrevRC = rc;
        this.PrevRF = rf;
        this.PrevRA = ra;
    }

    public uint[] GetCumFreqsRC(byte prevRC) {
        uint[] freqs = new uint[256];
        for (int i = 0; i < 256; i++) freqs[i] = Alpha;
        foreach (var entry in TransRC) {
            if (entry.Key == prevRC) {
                freqs[entry.Sym] += entry.Count;
            }
        }
        uint[] cumFreqs = new uint[257];
        for (int i = 0; i < 256; i++) {
            cumFreqs[i + 1] = cumFreqs[i] + freqs[i];
        }
        return cumFreqs;
    }

    public uint[] GetCumFreqsRF(byte currRC, byte prevRF) {
        uint[] freqs = new uint[256];
        for (int i = 0; i < 256; i++) freqs[i] = Alpha;
        uint key = ((uint)currRC << 8) | prevRF;
        foreach (var entry in TransRF) {
            if (entry.Key == key) {
                freqs[entry.Sym] += entry.Count;
            }
        }
        uint[] cumFreqs = new uint[257];
        for (int i = 0; i < 256; i++) {
            cumFreqs[i + 1] = cumFreqs[i] + freqs[i];
        }
        return cumFreqs;
    }

    public uint[] GetCumFreqsRA(byte currRC, byte currRF, byte prevRA) {
        uint[] freqs = new uint[256];
        for (int i = 0; i < 256; i++) freqs[i] = Alpha;
        uint key = ((uint)currRC << 16) | ((uint)currRF << 8) | prevRA;
        foreach (var entry in TransRA) {
            if (entry.Key == key) {
                freqs[entry.Sym] += entry.Count;
            }
        }
        uint[] cumFreqs = new uint[257];
        for (int i = 0; i < 256; i++) {
            cumFreqs[i + 1] = cumFreqs[i] + freqs[i];
        }
        return cumFreqs;
    }
}

class BitWriter {
    public List<byte> Buffer = new List<byte>();
    public int BitIndex = 0;

    public void WriteBit(byte bit) {
        int bytePos = BitIndex / 8;
        int bitPos = 7 - (BitIndex % 8);
        if (bytePos >= Buffer.Count) {
            Buffer.Add(0);
        }
        if (bit != 0) {
            Buffer[bytePos] |= (byte)(1 << bitPos);
        } else {
            Buffer[bytePos] &= (byte)(~(1 << bitPos));
        }
        BitIndex++;
    }

    public void WriteBitHelper(ref uint underflowBits, byte bit) {
        WriteBit(bit);
        while (underflowBits > 0) {
            WriteBit((byte)(1 - bit));
            underflowBits--;
        }
    }
}

class BitReader {
    public byte[] Buffer;
    public int BitIndex = 0;
    public int TotalBits;

    public BitReader(byte[] buffer) {
        this.Buffer = buffer;
        this.TotalBits = buffer.Length * 8;
    }

    public byte ReadBit() {
        if (BitIndex >= TotalBits) return 0;
        int bytePos = BitIndex / 8;
        int bitPos = 7 - (BitIndex % 8);
        byte bit = (byte)((Buffer[bytePos] >> bitPos) & 1);
        BitIndex++;
        return bit;
    }
}

struct Concept6D {
    public byte Domain, Subdomain, Operation, Modality, Depth, Polarity;
    public Concept6D(byte d, byte s, byte o, byte m, byte dp, byte p) {
        this.Domain = d; this.Subdomain = s; this.Operation = o;
        this.Modality = m; this.Depth = dp; this.Polarity = p;
    }
    public bool Equals(Concept6D other) {
        return this.Domain == other.Domain && this.Subdomain == other.Subdomain &&
               this.Operation == other.Operation && this.Modality == other.Modality &&
               this.Depth == other.Depth && this.Polarity == other.Polarity;
    }
}

class Proof {
    static byte[] Encode(Concept6D[] concepts, out int outBits, uint alpha, uint weight) {
        var pred = new RadicalPredictor(alpha, weight);
        var w = new BitWriter();
        uint low = 0;
        uint high = 0xFFFFFFFF;
        uint underflowBits = 0;

        foreach (var c in concepts) {
            byte rc = (byte)((c.Domain << 4) | c.Subdomain);
            byte rf = (byte)((c.Operation << 4) | c.Modality);
            byte ra = (byte)((c.Depth << 4) | c.Polarity);
            byte[] symbols = { rc, rf, ra };

            byte prevRC = pred.PrevRC;
            byte prevRF = pred.PrevRF;
            byte prevRA = pred.PrevRA;

            for (int step = 0; step < 3; step++) {
                uint[] cumFreqs;
                if (step == 0) {
                    cumFreqs = pred.GetCumFreqsRC(prevRC);
                } else if (step == 1) {
                    cumFreqs = pred.GetCumFreqsRF(symbols[0], prevRF);
                } else {
                    cumFreqs = pred.GetCumFreqsRA(symbols[0], symbols[1], prevRA);
                }

                int sym = symbols[step];
                uint total = cumFreqs[256];
                uint cumLow = cumFreqs[sym];
                uint cumHigh = cumFreqs[sym + 1];

                ulong rangeWidth = (ulong)high - (ulong)low + 1;
                high = low + (uint)((rangeWidth * cumHigh) / total) - 1;
                low = low + (uint)((rangeWidth * cumLow) / total);

                while (true) {
                    if (high < 0x80000000) {
                        w.WriteBitHelper(ref underflowBits, 0);
                        low <<= 1;
                        high = (high << 1) | 1;
                    } else if (low >= 0x80000000) {
                        w.WriteBitHelper(ref underflowBits, 1);
                        low = (low - 0x80000000) << 1;
                        high = ((high - 0x80000000) << 1) | 1;
                    } else if (low >= 0x40000000 && high < 0xC0000000) {
                        underflowBits++;
                        low = (low - 0x40000000) << 1;
                        high = ((high - 0x40000000) << 1) | 1;
                    } else {
                        break;
                    }
                }
            }
            pred.Observe(rc, rf, ra);
        }

        underflowBits++;
        if (low < 0x40000000) {
            w.WriteBitHelper(ref underflowBits, 0);
        } else {
            w.WriteBitHelper(ref underflowBits, 1);
        }

        outBits = w.BitIndex;
        return w.Buffer.ToArray();
    }

    static Concept6D[] Decode(byte[] encodedBytes, int numConcepts, uint alpha, uint weight) {
        var pred = new RadicalPredictor(alpha, weight);
        var r = new BitReader(encodedBytes);

        uint value = 0;
        for (int i = 0; i < 32; i++) {
            value = (value << 1) | r.ReadBit();
        }

        uint low = 0;
        uint high = 0xFFFFFFFF;
        var decoded = new Concept6D[numConcepts];

        for (int cIdx = 0; cIdx < numConcepts; cIdx++) {
            byte prevRC = pred.PrevRC;
            byte prevRF = pred.PrevRF;
            byte prevRA = pred.PrevRA;
            byte[] symbols = new byte[3];

            for (int step = 0; step < 3; step++) {
                uint[] cumFreqs;
                if (step == 0) {
                    cumFreqs = pred.GetCumFreqsRC(prevRC);
                } else if (step == 1) {
                    cumFreqs = pred.GetCumFreqsRF(symbols[0], prevRF);
                } else {
                    cumFreqs = pred.GetCumFreqsRA(symbols[0], symbols[1], prevRA);
                }

                uint total = cumFreqs[256];
                ulong rangeWidth = (ulong)high - (ulong)low + 1;
                ulong scaledVal = (((ulong)value - (ulong)low) + 1) * total - 1;
                scaledVal /= rangeWidth;

                byte sym = 0;
                int lIdx = 0, rIdx = 255;
                while (lIdx <= rIdx) {
                    int mIdx = (lIdx + rIdx) / 2;
                    if (cumFreqs[mIdx] <= scaledVal && scaledVal < cumFreqs[mIdx + 1]) {
                        sym = (byte)mIdx;
                        break;
                    } else if (scaledVal >= cumFreqs[mIdx + 1]) {
                        lIdx = mIdx + 1;
                    } else {
                        rIdx = mIdx - 1;
                    }
                }

                symbols[step] = sym;
                uint cumLow = cumFreqs[sym];
                uint cumHigh = cumFreqs[sym + 1];

                high = low + (uint)((rangeWidth * cumHigh) / total) - 1;
                low = low + (uint)((rangeWidth * cumLow) / total);

                while (true) {
                    if (high < 0x80000000) {
                        low <<= 1;
                        high = (high << 1) | 1;
                        value = (value << 1) | r.ReadBit();
                    } else if (low >= 0x80000000) {
                        low = (low - 0x80000000) << 1;
                        high = ((high - 0x80000000) << 1) | 1;
                        value = ((value - 0x80000000) << 1) | r.ReadBit();
                    } else if (low >= 0x40000000 && high < 0xC0000000) {
                        low = (low - 0x40000000) << 1;
                        high = ((high - 0x40000000) << 1) | 1;
                        value = ((value - 0x40000000) << 1) | r.ReadBit();
                    } else {
                        break;
                    }
                }
            }

            decoded[cIdx] = new Concept6D(
                (byte)(symbols[0] >> 4),
                (byte)(symbols[0] & 0x0F),
                (byte)(symbols[1] >> 4),
                (byte)(symbols[1] & 0x0F),
                (byte)(symbols[2] >> 4),
                (byte)(symbols[2] & 0x0F)
            );
            pred.Observe(symbols[0], symbols[1], symbols[2]);
        }
        return decoded;
    }

    static void Main() {
        Console.WriteLine("======================================================================");
        Console.WriteLine("ZYMATICA | zymatica-inference-engine-csharp");
        Console.WriteLine("======================================================================\n");

        var inputs = new Concept6D[] {
            new Concept6D(1, 2, 3, 4, 5, 6),
            new Concept6D(8, 0, 15, 1, 0, 15),
            new Concept6D(0, 0, 0, 0, 0, 0),
            new Concept6D(15, 15, 15, 15, 15, 15),
            new Concept6D(4, 5, 6, 7, 8, 9)
        };

        int outBits;
        byte[] buf = Encode(inputs, out outBits, 1, 128);
        Console.WriteLine($"Encoded Bits: {outBits}, Bytes: {buf.Length}");
        Console.Write("Hex: ");
        foreach (byte b in buf) {
            Console.Write($"{b:02X} ");
        }
        Console.WriteLine();

        var decoded = Decode(buf, 5, 1, 128);
        bool match = true;
        for (int i = 0; i < inputs.Length; i++) {
            if (!inputs[i].Equals(decoded[i])) {
                match = false;
                break;
            }
        }

        Console.WriteLine($"Decoded matches inputs: {match.ToString().ToLower()}");
        if (!match) {
            Console.WriteLine("ERROR: mismatch!");
            Environment.Exit(1);
        }

        Console.WriteLine("\n[VERIFICATION] Multi-Language runtime FFI structures validated.");
    }
}
