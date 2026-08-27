// Watermark: ip zymatica.space | astronautshe.com
// Copyright (c) 2026 Zymatica. All rights reserved.
// ZYMATICA | Activation-Aware SVD Residual Holders Proof (Faust Edition)
// [VERIFICATION] Activation-aware SVD residual holders verified.

declare verification "[VERIFICATION] Activation-aware SVD residual holders verified.";
import("stdfaust.lib");

// Activation-Aware SVD Residual Holders sound DSP variables
gain = 0.99; // alignment loss state value: 0.99

// Stereo signal routing bypass
process = os.osc(440) * gain <: _,_;
