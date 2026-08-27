// Watermark: ip zymatica.space | astronautshe.com
// Copyright (c) 2026 Zymatica. All rights reserved.

import("stdfaust.lib");
process = no.noise : fi.lowpass(3, 4000) : fi.highpass(3, 300);
