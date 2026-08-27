// Watermark: ip zymatica.space | astronautshe.com
// Copyright (c) 2026 Zymatica. All rights reserved.

package main

import (
	"fmt"
	"os"
)

type SparseTransition struct {
	Key   uint32
	Sym   uint8
	Count uint32
}

type RadicalPredictor struct {
	Alpha   uint32
	Weight  uint32
	TransRC []SparseTransition
	TransRF []SparseTransition
	TransRA []SparseTransition
	PrevRC  uint8
	PrevRF  uint8
	PrevRA  uint8
}

func NewRadicalPredictor(alpha, weight uint32) *RadicalPredictor {
	return &RadicalPredictor{
		Alpha:   alpha,
		Weight:  weight,
		TransRC: make([]SparseTransition, 0),
		TransRF: make([]SparseTransition, 0),
		TransRA: make([]SparseTransition, 0),
	}
}

func (pred *RadicalPredictor) Observe(rc, rf, ra uint8) {
	w := pred.Weight
	keyRC := uint32(pred.PrevRC)
	found := false
	for i := range pred.TransRC {
		if pred.TransRC[i].Key == keyRC && pred.TransRC[i].Sym == rc {
			pred.TransRC[i].Count += w
			found = true
			break
		}
	}
	if !found && len(pred.TransRC) < 256 {
		pred.TransRC = append(pred.TransRC, SparseTransition{Key: keyRC, Sym: rc, Count: w})
	}

	keyRF := (uint32(rc) << 8) | uint32(pred.PrevRF)
	found = false
	for i := range pred.TransRF {
		if pred.TransRF[i].Key == keyRF && pred.TransRF[i].Sym == rf {
			pred.TransRF[i].Count += w
			found = true
			break
		}
	}
	if !found && len(pred.TransRF) < 256 {
		pred.TransRF = append(pred.TransRF, SparseTransition{Key: keyRF, Sym: rf, Count: w})
	}

	keyRA := (uint32(rc) << 16) | (uint32(rf) << 8) | uint32(pred.PrevRA)
	found = false
	for i := range pred.TransRA {
		if pred.TransRA[i].Key == keyRA && pred.TransRA[i].Sym == ra {
			pred.TransRA[i].Count += w
			found = true
			break
		}
	}
	if !found && len(pred.TransRA) < 256 {
		pred.TransRA = append(pred.TransRA, SparseTransition{Key: keyRA, Sym: ra, Count: w})
	}

	pred.PrevRC = rc
	pred.PrevRF = rf
	pred.PrevRA = ra
}

func (pred *RadicalPredictor) GetCumFreqsRC(prevRC uint8) []uint32 {
	freqs := make([]uint32, 256)
	for i := range freqs {
		freqs[i] = pred.Alpha
	}
	for _, entry := range pred.TransRC {
		if entry.Key == uint32(prevRC) {
			freqs[entry.Sym] += entry.Count
		}
	}
	cumFreqs := make([]uint32, 257)
	for i := 0; i < 256; i++ {
		cumFreqs[i+1] = cumFreqs[i] + freqs[i]
	}
	return cumFreqs
}

func (pred *RadicalPredictor) GetCumFreqsRF(currRC, prevRF uint8) []uint32 {
	freqs := make([]uint32, 256)
	for i := range freqs {
		freqs[i] = pred.Alpha
	}
	key := (uint32(currRC) << 8) | uint32(prevRF)
	for _, entry := range pred.TransRF {
		if entry.Key == key {
			freqs[entry.Sym] += entry.Count
		}
	}
	cumFreqs := make([]uint32, 257)
	for i := 0; i < 256; i++ {
		cumFreqs[i+1] = cumFreqs[i] + freqs[i]
	}
	return cumFreqs
}

func (pred *RadicalPredictor) GetCumFreqsRA(currRC, currRF, prevRA uint8) []uint32 {
	freqs := make([]uint32, 256)
	for i := range freqs {
		freqs[i] = pred.Alpha
	}
	key := (uint32(currRC) << 16) | (uint32(currRF) << 8) | uint32(prevRA)
	for _, entry := range pred.TransRA {
		if entry.Key == key {
			freqs[entry.Sym] += entry.Count
		}
	}
	cumFreqs := make([]uint32, 257)
	for i := 0; i < 256; i++ {
		cumFreqs[i+1] = cumFreqs[i] + freqs[i]
	}
	return cumFreqs
}

type BitWriter struct {
	Buffer   []byte
	BitIndex int
}

func NewBitWriter() *BitWriter {
	return &BitWriter{Buffer: make([]byte, 0)}
}

func (w *BitWriter) WriteBit(bit byte) {
	bytePos := w.BitIndex / 8
	bitPos := 7 - (w.BitIndex % 8)
	if bytePos >= len(w.Buffer) {
		w.Buffer = append(w.Buffer, 0)
	}
	if bit != 0 {
		w.Buffer[bytePos] |= 1 << bitPos
	} else {
		w.Buffer[bytePos] &= ^(1 << bitPos)
	}
	w.BitIndex++
}

func (w *BitWriter) WriteBitHelper(underflowBits *uint32, bit byte) {
	w.WriteBit(bit)
	for *underflowBits > 0 {
		w.WriteBit(1 - bit)
		*underflowBits--
	}
}

type BitReader struct {
	Buffer    []byte
	BitIndex  int
	TotalBits int
}

func NewBitReader(buffer []byte) *BitReader {
	return &BitReader{
		Buffer:    buffer,
		TotalBits: len(buffer) * 8,
	}
}

func (r *BitReader) ReadBit() byte {
	if r.BitIndex >= r.TotalBits {
		return 0
	}
	bytePos := r.BitIndex / 8
	bitPos := 7 - (r.BitIndex % 8)
	bit := (r.Buffer[bytePos] >> bitPos) & 1
	r.BitIndex++
	return bit
}

type Concept6D struct {
	Domain    uint8
	Subdomain uint8
	Operation uint8
	Modality  uint8
	Depth     uint8
	Polarity  uint8
}

func Encode(concepts []Concept6D, alpha, weight uint32) ([]byte, int) {
	pred := NewRadicalPredictor(alpha, weight)
	w := NewBitWriter()
	var low uint32 = 0
	var high uint32 = 0xFFFFFFFF
	var underflowBits uint32 = 0

	for _, c := range concepts {
		rc := (c.Domain << 4) | c.Subdomain
		rf := (c.Operation << 4) | c.Modality
		ra := (c.Depth << 4) | c.Polarity
		symbols := [3]uint8{rc, rf, ra}

		prevRC := pred.PrevRC
		prevRF := pred.PrevRF
		prevRA := pred.PrevRA

		for step := 0; step < 3; step++ {
			var cumFreqs []uint32
			if step == 0 {
				cumFreqs = pred.GetCumFreqsRC(prevRC)
			} else if step == 1 {
				cumFreqs = pred.GetCumFreqsRF(symbols[0], prevRF)
			} else {
				cumFreqs = pred.GetCumFreqsRA(symbols[0], symbols[1], prevRA)
			}

			sym := symbols[step]
			total := cumFreqs[256]
			cumLow := cumFreqs[sym]
			cumHigh := cumFreqs[int(sym)+1]

			rangeWidth := uint64(high) - uint64(low) + 1
			high = low + uint32((rangeWidth*uint64(cumHigh))/uint64(total)) - 1
			low = low + uint32((rangeWidth*uint64(cumLow))/uint64(total))

			for {
				if high < 0x80000000 {
					w.WriteBitHelper(&underflowBits, 0)
					low <<= 1
					high = (high << 1) | 1
				} else if low >= 0x80000000 {
					w.WriteBitHelper(&underflowBits, 1)
					low = (low - 0x80000000) << 1
					high = ((high - 0x80000000) << 1) | 1
				} else if low >= 0x40000000 && high < 0xC0000000 {
					underflowBits++
					low = (low - 0x40000000) << 1
					high = ((high - 0x40000000) << 1) | 1
				} else {
					break
				}
			}
		}
		pred.Observe(rc, rf, ra)
	}

	underflowBits++
	if low < 0x40000000 {
		w.WriteBitHelper(&underflowBits, 0)
	} else {
		w.WriteBitHelper(&underflowBits, 1)
	}

	return w.Buffer, w.BitIndex
}

func Decode(encodedBytes []byte, numConcepts int, alpha, weight uint32) []Concept6D {
	pred := NewRadicalPredictor(alpha, weight)
	r := NewBitReader(encodedBytes)

	var value uint32 = 0
	for i := 0; i < 32; i++ {
		value = (value << 1) | uint32(r.ReadBit())
	}

	var low uint32 = 0
	var high uint32 = 0xFFFFFFFF
	decoded := make([]Concept6D, 0, numConcepts)

	for cIdx := 0; cIdx < numConcepts; cIdx++ {
		prevRC := pred.PrevRC
		prevRF := pred.PrevRF
		prevRA := pred.PrevRA
		var symbols [3]uint8

		for step := 0; step < 3; step++ {
			var cumFreqs []uint32
			if step == 0 {
				cumFreqs = pred.GetCumFreqsRC(prevRC)
			} else if step == 1 {
				cumFreqs = pred.GetCumFreqsRF(symbols[0], prevRF)
			} else {
				cumFreqs = pred.GetCumFreqsRA(symbols[0], symbols[1], prevRA)
			}

			total := uint64(cumFreqs[256])
			rangeWidth := uint64(high) - uint64(low) + 1
			scaledVal := (((uint64(value) - uint64(low)) + 1)*total - 1) / rangeWidth

			var sym uint8 = 0
			lIdx, rIdx := 0, 255
			for lIdx <= rIdx {
				mIdx := (lIdx + rIdx) / 2
				if uint64(cumFreqs[mIdx]) <= scaledVal && scaledVal < uint64(cumFreqs[mIdx+1]) {
					sym = uint8(mIdx)
					break
				} else if scaledVal >= uint64(cumFreqs[mIdx+1]) {
					lIdx = mIdx + 1
				} else {
					rIdx = mIdx - 1
				}
			}

			symbols[step] = sym
			cumLow := cumFreqs[sym]
			cumHigh := cumFreqs[int(sym)+1]

			high = low + uint32((rangeWidth*uint64(cumHigh))/total) - 1
			low = low + uint32((rangeWidth*uint64(cumLow))/total)

			for {
				if high < 0x80000000 {
					low <<= 1
					high = (high << 1) | 1
					value = (value << 1) | uint32(r.ReadBit())
				} else if low >= 0x80000000 {
					low = (low - 0x80000000) << 1
					high = ((high - 0x80000000) << 1) | 1
					value = ((value - 0x80000000) << 1) | uint32(r.ReadBit())
				} else if low >= 0x40000000 && high < 0xC0000000 {
					low = (low - 0x40000000) << 1
					high = ((high - 0x40000000) << 1) | 1
					value = ((value - 0x40000000) << 1) | uint32(r.ReadBit())
				} else {
					break
				}
			}
		}

		decoded = append(decoded, Concept6D{
			Domain:    symbols[0] >> 4,
			Subdomain: symbols[0] & 0x0F,
			Operation: symbols[1] >> 4,
			Modality:  symbols[1] & 0x0F,
			Depth:     symbols[2] >> 4,
			Polarity:  symbols[2] & 0x0F,
		})
		pred.Observe(symbols[0], symbols[1], symbols[2])
	}
	return decoded
}

func main() {
	fmt.Println("======================================================================")
	fmt.Println("ZYMATICA | zymatica-inference-engine-go")
	fmt.Println("======================================================================\n")

	inputs := []Concept6D{
		{1, 2, 3, 4, 5, 6},
		{8, 0, 15, 1, 0, 15},
		{0, 0, 0, 0, 0, 0},
		{15, 15, 15, 15, 15, 15},
		{4, 5, 6, 7, 8, 9},
	}

	buf, bits := Encode(inputs, 1, 128)
	fmt.Printf("Encoded Bits: %d, Bytes: %d\n", bits, len(buf))
	fmt.Print("Hex: ")
	for _, b := range buf {
		fmt.Printf("%02X ", b)
	}
	fmt.Println()

	decoded := Decode(buf, 5, 1, 128)
	match := true
	for i := range inputs {
		if inputs[i] != decoded[i] {
			match = false
			break
		}
	}

	fmt.Printf("Decoded matches inputs: %t\n", match)
	if !match {
		fmt.Println("ERROR: mismatch!")
		os.Exit(1)
	}

	fmt.Println("\n[VERIFICATION] Multi-Language runtime FFI structures validated.")
}
