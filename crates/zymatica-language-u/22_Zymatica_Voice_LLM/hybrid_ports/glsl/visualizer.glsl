#version 430
// Watermark: ip zymatica.space | astronautshe.com
// Copyright (c) 2026 Zymatica. All rights reserved.

layout(local_size_x = 16, local_size_y = 16) in;
layout(rgba32f, binding = 0) uniform image2D imgOutput;

uniform float u_time;
uniform float u_amplitude; // Audio amplitude feed

void main() {
    ivec2 texelCoords = ivec2(gl_GlobalInvocationID.xy);
    float val = sin(float(texelCoords.x) * 0.05 + u_time) * u_amplitude;
    vec4 color = vec4(0.54, 0.36, 0.96, 1.0) * val;
    imageStore(imgOutput, texelCoords, color);
}
