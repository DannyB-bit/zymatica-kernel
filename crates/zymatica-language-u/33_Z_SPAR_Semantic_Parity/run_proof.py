#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Class 33: Z-SPAR (Zymatica Semantic Parity and Repair Protocol)
Standalone Finite-Field GF(16) RS(12,8) Cross-Model Semantic Verification
Author: Danny Bouldiez | Codebase by Devs One
"""

class GF16Py:
    EXP = [1, 2, 4, 8, 3, 6, 12, 11, 5, 10, 7, 14, 15, 13, 9, 1, 2, 4, 8, 3, 6, 12, 11, 5, 10, 7, 14, 15, 13, 9, 1, 2]
    LOG = [0, 0, 1, 4, 2, 8, 5, 10, 3, 14, 9, 7, 6, 13, 11, 12]

    @classmethod
    def add(cls, a, b):
        return (a ^ b) & 0x0F

    @classmethod
    def mul(cls, a, b):
        a, b = a & 0x0F, b & 0x0F
        if a == 0 or b == 0:
            return 0
        return cls.EXP[(cls.LOG[a] + cls.LOG[b]) % 15]

    @classmethod
    def div(cls, a, b):
        a, b = a & 0x0F, b & 0x0F
        if b == 0:
            raise ZeroDivisionError("GF(16) div by zero")
        if a == 0:
            return 0
        return cls.EXP[(cls.LOG[a] - cls.LOG[b] + 15) % 15]

    @classmethod
    def power(cls, a, exp):
        a = a & 0x0F
        if a == 0:
            return 0
        return cls.EXP[(cls.LOG[a] * exp) % 15]


def encode_z_spar_8d(state_8d):
    """Encodes 8 semantic coordinates into 4 parity symbols over GF(16)."""
    p = [0, 0, 0, 0]
    for j in range(4):
        root = GF16Py.EXP[j + 1]
        sum_val = 0
        for i, val in enumerate(state_8d):
            w = GF16Py.power(root, i + 1)
            sum_val = GF16Py.add(sum_val, GF16Py.mul(val, w))
        p[j] = sum_val
    return p


def verify_and_repair_z_spar(reconstructed_8d, expected_parity):
    """Computes semantic syndrome and automatically repairs up to 2 drifted semantic axes."""
    syndromes = [0, 0, 0, 0]
    all_zero = True
    for j in range(4):
        root = GF16Py.EXP[j + 1]
        sum_val = 0
        for i, val in enumerate(reconstructed_8d):
            w = GF16Py.power(root, i + 1)
            sum_val = GF16Py.add(sum_val, GF16Py.mul(val, w))
        s = GF16Py.add(expected_parity[j], sum_val)
        syndromes[j] = s
        if s != 0:
            all_zero = False

    if all_zero:
        return "EXACT_MATCH", list(reconstructed_8d)

    # 1-error correction
    for target_axis in range(8):
        candidate_err = None
        consistent = True
        for j in range(4):
            root = GF16Py.EXP[j + 1]
            w = GF16Py.power(root, target_axis + 1)
            try:
                err = GF16Py.div(syndromes[j], w)
                if candidate_err is not None and candidate_err != err:
                    consistent = False
                    break
                candidate_err = err
            except ZeroDivisionError:
                consistent = False
                break
        if consistent and candidate_err:
            corrected = list(reconstructed_8d)
            corrected[target_axis] = GF16Py.add(corrected[target_axis], candidate_err)
            return "REPAIRED_1_AXIS", corrected

    # 2-error correction
    for i1 in range(8):
        for i2 in range(i1 + 1, 8):
            r0, r1 = GF16Py.EXP[1], GF16Py.EXP[2]
            a11 = GF16Py.power(r0, i1 + 1)
            a12 = GF16Py.power(r0, i2 + 1)
            a21 = GF16Py.power(r1, i1 + 1)
            a22 = GF16Py.power(r1, i2 + 1)
            det = GF16Py.add(GF16Py.mul(a11, a22), GF16Py.mul(a12, a21))
            if det == 0:
                continue
            num1 = GF16Py.add(GF16Py.mul(a22, syndromes[0]), GF16Py.mul(a12, syndromes[1]))
            num2 = GF16Py.add(GF16Py.mul(a11, syndromes[1]), GF16Py.mul(a21, syndromes[0]))
            try:
                e1 = GF16Py.div(num1, det)
                e2 = GF16Py.div(num2, det)
                r2, r3 = GF16Py.EXP[3], GF16Py.EXP[4]
                chk_s2 = GF16Py.add(GF16Py.mul(GF16Py.power(r2, i1 + 1), e1), GF16Py.mul(GF16Py.power(r2, i2 + 1), e2))
                chk_s3 = GF16Py.add(GF16Py.mul(GF16Py.power(r3, i1 + 1), e1), GF16Py.mul(GF16Py.power(r3, i2 + 1), e2))
                if chk_s2 == syndromes[2] and chk_s3 == syndromes[3]:
                    corrected = list(reconstructed_8d)
                    corrected[i1] = GF16Py.add(corrected[i1], e1)
                    corrected[i2] = GF16Py.add(corrected[i2], e2)
                    return "REPAIRED_2_AXIS", corrected
            except Exception:
                continue

    return "UNCORRECTABLE_DIVERGENCE", list(reconstructed_8d)


def main():
    print("=" * 80)
    print(" [+] ZYMATICA CLASS 33: Z-SPAR SEMANTIC PARITY AND REPAIR ENGINE")
    print("     Cross-Model Finite-Field GF(16) RS(12,8) Semantic Error Correction")
    print("=" * 80)

    # Node A (e.g. SmolLM2) sends industrial command:
    # "Close valve 7 immediately because pressure is critical"
    # Semantic Coordinates: [D=1, SD=4, OP=8 (CLOSE), M=15 (MANDATORY), S=10, P=1, T=2, E=14]
    true_intent = [1, 4, 8, 15, 10, 1, 2, 14]
    parity = encode_z_spar_8d(true_intent)
    print(f" [TX] Originating Semantic State (Node A): {true_intent}")
    print(f" [TX] Transmitted GF(16) Semantic Parity:   {parity} (4 nibbles / 2 bytes)")

    # Node B (e.g. Qwen3.5) reconstructs meaning with model drift on OP (8 -> 3: REDUCE)
    drifted_intent = [1, 4, 3, 15, 10, 1, 2, 14]
    print(f" [RX] Reconstructed State (Node B Drift):   {drifted_intent} [OP: CLOSE -> REDUCE]")

    # Run Z-SPAR Decoder
    status, repaired_intent = verify_and_repair_z_spar(drifted_intent, parity)
    print(f" [Z-SPAR] Syndrome Evaluation:               {status}")
    print(f" [Z-SPAR] Repaired Semantic State:          {repaired_intent}")

    assert repaired_intent == true_intent, "Repaired intent must match true intent exactly"
    print(" [+] Automatic single-axis repair verified without natural language retransmission!")

    # Test Dual-Axis Drift: OP (8->2) and Modality (15->5)
    drifted_2axis = [1, 4, 2, 5, 10, 1, 2, 14]
    status2, repaired_2axis = verify_and_repair_z_spar(drifted_2axis, parity)
    print(f"\n [RX] Dual-Axis Model Drift:                 {drifted_2axis}")
    print(f" [Z-SPAR] Syndrome Evaluation:               {status2}")
    print(f" [Z-SPAR] Repaired Semantic State:          {repaired_2axis}")
    assert repaired_2axis == true_intent, "Dual-axis repair must restore exact intent"

    print("\n[PASS] CLASS 33 VERIFICATION: Z-SPAR REED-SOLOMON GF(16) SEMANTIC ECC PROVEN!")
    print("=" * 80)


if __name__ == "__main__":
    main()
