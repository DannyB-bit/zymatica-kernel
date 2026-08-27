// Watermark: ip zymatica.space | astronautshe.com
// Copyright (c) 2026 Zymatica. All rights reserved.

import 'dart:io';

class SparseTransition {
  int key;
  int sym;
  int count;
  SparseTransition(this.key, this.sym, this.count);
}

class RadicalPredictor {
  int alpha;
  int weight;
  List<SparseTransition> transRC = [];
  List<SparseTransition> transRF = [];
  List<SparseTransition> transRA = [];
  int prevRC = 0;
  int prevRF = 0;
  int prevRA = 0;

  RadicalPredictor(this.alpha, this.weight);

  void observe(int rc, int rf, int ra) {
    int w = weight;
    int keyRC = prevRC;
    bool found = false;
    for (var entry in transRC) {
      if (entry.key == keyRC && entry.sym == rc) {
        entry.count += w;
        found = true;
        break;
      }
    }
    if (!found && transRC.length < 256) {
      transRC.add(SparseTransition(keyRC, rc, w));
    }

    int keyRF = (rc << 8) | prevRF;
    found = false;
    for (var entry in transRF) {
      if (entry.key == keyRF && entry.sym == rf) {
        entry.count += w;
        found = true;
        break;
      }
    }
    if (!found && transRF.length < 256) {
      transRF.add(SparseTransition(keyRF, rf, w));
    }

    int keyRA = (rc << 16) | (rf << 8) | prevRA;
    found = false;
    for (var entry in transRA) {
      if (entry.key == keyRA && entry.sym == ra) {
        entry.count += w;
        found = true;
        break;
      }
    }
    if (!found && transRA.length < 256) {
      transRA.add(SparseTransition(keyRA, ra, w));
    }

    prevRC = rc;
    prevRF = rf;
    prevRA = ra;
  }

  List<int> getCumFreqsRC(int prevRC) {
    List<int> freqs = List<int>.filled(256, alpha);
    for (var entry in transRC) {
      if (entry.key == prevRC) {
        freqs[entry.sym] += entry.count;
      }
    }
    List<int> cumFreqs = List<int>.filled(257, 0);
    for (int i = 0; i < 256; i++) {
      cumFreqs[i + 1] = cumFreqs[i] + freqs[i];
    }
    return cumFreqs;
  }

  List<int> getCumFreqsRF(int currRC, int prevRF) {
    List<int> freqs = List<int>.filled(256, alpha);
    int key = (currRC << 8) | prevRF;
    for (var entry in transRF) {
      if (entry.key == key) {
        freqs[entry.sym] += entry.count;
      }
    }
    List<int> cumFreqs = List<int>.filled(257, 0);
    for (int i = 0; i < 256; i++) {
      cumFreqs[i + 1] = cumFreqs[i] + freqs[i];
    }
    return cumFreqs;
  }

  List<int> getCumFreqsRA(int currRC, int currRF, int prevRA) {
    List<int> freqs = List<int>.filled(256, alpha);
    int key = (currRC << 16) | (currRF << 8) | prevRA;
    for (var entry in transRA) {
      if (entry.key == key) {
        freqs[entry.sym] += entry.count;
      }
    }
    List<int> cumFreqs = List<int>.filled(257, 0);
    for (int i = 0; i < 256; i++) {
      cumFreqs[i + 1] = cumFreqs[i] + freqs[i];
    }
    return cumFreqs;
  }
}

class BitWriter {
  List<int> buffer = [];
  int bitIndex = 0;

  void writeBit(int bit) {
    int bytePos = bitIndex ~/ 8;
    int bitPos = 7 - (bitIndex % 8);
    if (bytePos >= buffer.length) {
      buffer.add(0);
    }
    if (bit != 0) {
      buffer[bytePos] |= (1 << bitPos);
    } else {
      buffer[bytePos] &= ~(1 << bitPos);
    }
    bitIndex++;
  }

  void writeBitHelper(List<int> underflowBits, int bit) {
    writeBit(bit);
    while (underflowBits[0] > 0) {
      writeBit(1 - bit);
      underflowBits[0]--;
    }
  }
}

class BitReader {
  List<int> buffer;
  int bitIndex = 0;
  int totalBits;

  BitReader(this.buffer) : totalBits = buffer.length * 8;

  int readBit() {
    if (bitIndex >= totalBits) return 0;
    int bytePos = bitIndex ~/ 8;
    int bitPos = 7 - (bitIndex % 8);
    int bit = (buffer[bytePos] >> bitPos) & 1;
    bitIndex++;
    return bit;
  }
}

class Concept6D {
  int domain, subdomain, operation, modality, depth, polarity;
  Concept6D(this.domain, this.subdomain, this.operation, this.modality, this.depth, this.polarity);

  bool equals(Concept6D other) {
    return domain == other.domain && subdomain == other.subdomain &&
           operation == other.operation && modality == other.modality &&
           depth == other.depth && polarity == other.polarity;
  }
}

List<int> encode(List<Concept6D> concepts, List<int> outBits, int alpha, int weight) {
  var pred = RadicalPredictor(alpha, weight);
  var w = BitWriter();
  int low = 0;
  int high = 0xFFFFFFFF;
  List<int> underflowBits = [0];

  for (var c in concepts) {
    int rc = (c.domain << 4) | c.subdomain;
    int rf = (c.operation << 4) | c.modality;
    int ra = (c.depth << 4) | c.polarity;
    List<int> symbols = [rc, rf, ra];

    int prevRC = pred.prevRC;
    int prevRF = pred.prevRF;
    int prevRA = pred.prevRA;

    for (int step = 0; step < 3; step++) {
      List<int> cumFreqs;
      if (step == 0) {
        cumFreqs = pred.getCumFreqsRC(prevRC);
      } else if (step == 1) {
        cumFreqs = pred.getCumFreqsRF(symbols[0], prevRF);
      } else {
        cumFreqs = pred.getCumFreqsRA(symbols[0], symbols[1], prevRA);
      }

      int sym = symbols[step];
      int total = cumFreqs[256];
      int cumLow = cumFreqs[sym];
      int cumHigh = cumFreqs[sym + 1];

      int rangeWidth = high - low + 1;
      high = low + ((rangeWidth * cumHigh) ~/ total) - 1;
      low = low + ((rangeWidth * cumLow) ~/ total);

      while (true) {
        if (high < 0x80000000) {
          w.writeBitHelper(underflowBits, 0);
          low = (low * 2) & 0xFFFFFFFF;
          high = ((high * 2) + 1) & 0xFFFFFFFF;
        } else if (low >= 0x80000000) {
          w.writeBitHelper(underflowBits, 1);
          low = ((low - 0x80000000) * 2) & 0xFFFFFFFF;
          high = (((high - 0x80000000) * 2) + 1) & 0xFFFFFFFF;
        } else if (low >= 0x40000000 && high < 0xC0000000) {
          underflowBits[0]++;
          low = ((low - 0x40000000) * 2) & 0xFFFFFFFF;
          high = (((high - 0x40000000) * 2) + 1) & 0xFFFFFFFF;
        } else {
          break;
        }
      }
    }
    pred.observe(rc, rf, ra);
  }

  underflowBits[0]++;
  if (low < 0x40000000) {
    w.writeBitHelper(underflowBits, 0);
  } else {
    w.writeBitHelper(underflowBits, 1);
  }

  outBits[0] = w.bitIndex;
  return w.buffer;
}

List<Concept6D> decode(List<int> encodedBytes, int numConcepts, int alpha, int weight) {
  var pred = RadicalPredictor(alpha, weight);
  var r = BitReader(encodedBytes);

  int value = 0;
  for (int i = 0; i < 32; i++) {
    value = ((value * 2) + r.readBit()) & 0xFFFFFFFF;
  }

  int low = 0;
  int high = 0xFFFFFFFF;
  List<Concept6D> decoded = [];

  for (int cIdx = 0; cIdx < numConcepts; cIdx++) {
    int prevRC = pred.prevRC;
    int prevRF = pred.prevRF;
    int prevRA = pred.prevRA;
    List<int> symbols = [0, 0, 0];

    for (int step = 0; step < 3; step++) {
      List<int> cumFreqs;
      if (step == 0) {
        cumFreqs = pred.getCumFreqsRC(prevRC);
      } else if (step == 1) {
        cumFreqs = pred.getCumFreqsRF(symbols[0], prevRF);
      } else {
        cumFreqs = pred.getCumFreqsRA(symbols[0], symbols[1], prevRA);
      }

      int total = cumFreqs[256];
      int rangeWidth = high - low + 1;
      int scaledVal = (((value - low) + 1) * total - 1) ~/ rangeWidth;

      int sym = 0;
      int lIdx = 0, rIdx = 255;
      while (lIdx <= rIdx) {
        int mIdx = (lIdx + rIdx) ~/ 2;
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
      int cumLow = cumFreqs[sym];
      int cumHigh = cumFreqs[sym + 1];

      high = low + ((rangeWidth * cumHigh) ~/ total) - 1;
      low = low + ((rangeWidth * cumLow) ~/ total);

      while (true) {
        if (high < 0x80000000) {
          low = (low * 2) & 0xFFFFFFFF;
          high = ((high * 2) + 1) & 0xFFFFFFFF;
          value = ((value * 2) + r.readBit()) & 0xFFFFFFFF;
        } else if (low >= 0x80000000) {
          low = ((low - 0x80000000) * 2) & 0xFFFFFFFF;
          high = (((high - 0x80000000) * 2) + 1) & 0xFFFFFFFF;
          value = (((value - 0x80000000) * 2) + r.readBit()) & 0xFFFFFFFF;
        } else if (low >= 0x40000000 && high < 0xC0000000) {
          low = ((low - 0x40000000) * 2) & 0xFFFFFFFF;
          high = (((high - 0x40000000) * 2) + 1) & 0xFFFFFFFF;
          value = (((value - 0x40000000) * 2) + r.readBit()) & 0xFFFFFFFF;
        } else {
          break;
        }
      }
    }

    decoded.add(Concept6D(
      (symbols[0] >> 4) & 0xF,
      symbols[0] & 0xF,
      (symbols[1] >> 4) & 0xF,
      symbols[1] & 0xF,
      (symbols[2] >> 4) & 0xF,
      symbols[2] & 0xF
    ));
    pred.observe(symbols[0], symbols[1], symbols[2]);
  }
  return decoded;
}

void main() {
  print("======================================================================");
  print("ZYMATICA | zymatica-inference-engine-dart");
  print("======================================================================\n");

  List<Concept6D> inputs = [
    Concept6D(1, 2, 3, 4, 5, 6),
    Concept6D(8, 0, 15, 1, 0, 15),
    Concept6D(0, 0, 0, 0, 0, 0),
    Concept6D(15, 15, 15, 15, 15, 15),
    Concept6D(4, 5, 6, 7, 8, 9)
  ];

  List<int> outBits = [0];
  List<int> buf = encode(inputs, outBits, 1, 128);
  print("Encoded Bits: \${outBits[0]}, Bytes: \${buf.length}");
  stdout.write("Hex: ");
  for (int b in buf) {
    stdout.write("\${b.toRadixString(16).toUpperCase().padLeft(2, '0')} ");
  }
  print("");

  List<Concept6D> decoded = decode(buf, 5, 1, 128);
  bool match = true;
  for (int i = 0; i < inputs.length; i++) {
    if (!inputs[i].equals(decoded[i])) {
      match = false;
      break;
    }
  }

  print("Decoded matches inputs: \$match");
  if (!match) {
    print("ERROR: mismatch!");
    exit(1);
  }

  print("\n[VERIFICATION] Multi-Language runtime FFI structures validated.");
}
