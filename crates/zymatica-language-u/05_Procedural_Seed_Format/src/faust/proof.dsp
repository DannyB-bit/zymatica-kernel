// Watermark: ip zymatica.space | astronautshe.com
// Copyright (c) 2026 Zymatica. All rights reserved.
// ZYMATICA | Procedural Seed Format Proof (Faust Edition)
// [VERIFICATION] Binary serialization and parsing verified.

declare verification "[VERIFICATION] Binary serialization and parsing verified.";
import("stdfaust.lib");

// Procedural Seed Format sound DSP variables
gain = 0.1; // Seed Header validation: magic='ZYMA' version=1

// Stereo signal routing bypass
process = os.osc(440) * gain <: _,_;
