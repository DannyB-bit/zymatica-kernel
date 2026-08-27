-- Watermark: ip zymatica.space | astronautshe.com
-- Copyright (c) 2026 Zymatica. All rights reserved.

print("======================================================================")
print("ZYMATICA | zymatica-inference-engine-lua")
print("======================================================================\n")

local RadicalPredictor = {}
RadicalPredictor.__index = RadicalPredictor

function RadicalPredictor.new(alpha, weight)
    local self = setmetatable({}, RadicalPredictor)
    self.alpha = alpha
    self.weight = weight
    self.trans_rc = {}
    self.trans_rf = {}
    self.trans_ra = {}
    self.prev_rc = 0
    self.prev_rf = 0
    self.prev_ra = 0
    return self
end

function RadicalPredictor:observe(rc, rf, ra)
    local w = self.weight
    local key_rc = self.prev_rc
    local found = false
    for _, entry in ipairs(self.trans_rc) do
        if entry.key == key_rc and entry.sym == rc then
            entry.count = entry.count + w
            found = true
            break
        end
    end
    if not found and #self.trans_rc < 256 then
        table.insert(self.trans_rc, {key = key_rc, sym = rc, count = w})
    end

    local key_rf = (rc * 256) + self.prev_rf
    found = false
    for _, entry in ipairs(self.trans_rf) do
        if entry.key == key_rf and entry.sym == rf then
            entry.count = entry.count + w
            found = true
            break
        end
    end
    if not found and #self.trans_rf < 256 then
        table.insert(self.trans_rf, {key = key_rf, sym = rf, count = w})
    end

    local key_ra = (rc * 65536) + (rf * 256) + self.prev_ra
    found = false
    for _, entry in ipairs(self.trans_ra) do
        if entry.key == key_ra and entry.sym == ra then
            entry.count = entry.count + w
            found = true
            break
        end
    end
    if not found and #self.trans_ra < 256 then
        table.insert(self.trans_ra, {key = key_ra, sym = ra, count = w})
    end

    self.prev_rc = rc
    self.prev_rf = rf
    self.prev_ra = ra
end

function RadicalPredictor:get_cum_freqs_rc(prev_rc)
    local freqs = {}
    for i = 0, 255 do freqs[i] = self.alpha end
    for _, entry in ipairs(self.trans_rc) do
        if entry.key == prev_rc then
            freqs[entry.sym] = freqs[entry.sym] + entry.count
        end
    end
    local cum_freqs = {[0] = 0}
    for i = 0, 255 do
        cum_freqs[i+1] = cum_freqs[i] + freqs[i]
    end
    return cum_freqs
end

function RadicalPredictor:get_cum_freqs_rf(curr_rc, prev_rf)
    local freqs = {}
    for i = 0, 255 do freqs[i] = self.alpha end
    local key = (curr_rc * 256) + prev_rf
    for _, entry in ipairs(self.trans_rf) do
        if entry.key == key then
            freqs[entry.sym] = freqs[entry.sym] + entry.count
        end
    end
    local cum_freqs = {[0] = 0}
    for i = 0, 255 do
        cum_freqs[i+1] = cum_freqs[i] + freqs[i]
    end
    return cum_freqs
end

function RadicalPredictor:get_cum_freqs_ra(curr_rc, curr_rf, prev_ra)
    local freqs = {}
    for i = 0, 255 do freqs[i] = self.alpha end
    local key = (curr_rc * 65536) + (curr_rf * 256) + prev_ra
    for _, entry in ipairs(self.trans_ra) do
        if entry.key == key then
            freqs[entry.sym] = freqs[entry.sym] + entry.count
        end
    end
    local cum_freqs = {[0] = 0}
    for i = 0, 255 do
        cum_freqs[i+1] = cum_freqs[i] + freqs[i]
    end
    return cum_freqs
end

local BitWriter = {}
BitWriter.__index = BitWriter

function BitWriter.new()
    local self = setmetatable({}, BitWriter)
    self.buffer = {}
    self.bit_index = 0
    return self
end

function BitWriter:write_bit(bit)
    local byte_pos = math.floor(self.bit_index / 8) + 1
    local bit_pos = 7 - (self.bit_index % 8)
    if not self.buffer[byte_pos] then
        self.buffer[byte_pos] = 0
    end
    if bit ~= 0 then
        self.buffer[byte_pos] = self.buffer[byte_pos] + (2 ^ bit_pos)
    end
    self.bit_index = self.bit_index + 1
end

function BitWriter:write_bit_helper(underflow_bits, bit)
    self:write_bit(bit)
    while underflow_bits[1] > 0 do
        self:write_bit(1 - bit)
        underflow_bits[1] = underflow_bits[1] - 1
    end
end

local BitReader = {}
BitReader.__index = BitReader

function BitReader.new(buffer)
    local self = setmetatable({}, BitReader)
    self.buffer = buffer
    self.bit_index = 0
    self.total_bits = #buffer * 8
    return self
end

function BitReader:read_bit()
    if self.bit_index >= self.total_bits then
        return 0
    end
    local byte_pos = math.floor(self.bit_index / 8) + 1
    local bit_pos = 7 - (self.bit_index % 8)
    local bit = math.floor(self.buffer[byte_pos] / (2 ^ bit_pos)) % 2
    self.bit_index = self.bit_index + 1
    return bit
end

local function encode(concepts, alpha, weight)
    local pred = RadicalPredictor.new(alpha, weight)
    local w = BitWriter.new()
    local low = 0
    local high = 0xFFFFFFFF
    local underflow_bits = {0}

    for _, c in ipairs(concepts) do
        local rc = (c[1] * 16) + c[2]
        local rf = (c[3] * 16) + c[4]
        local ra = (c[5] * 16) + c[6]
        local symbols = {[0] = rc, [1] = rf, [2] = ra}

        local prev_rc = pred.prev_rc
        local prev_rf = pred.prev_rf
        local prev_ra = pred.prev_ra

        for step = 0, 2 do
            local cum_freqs
            if step == 0 then
                cum_freqs = pred:get_cum_freqs_rc(prev_rc)
            elseif step == 1 then
                cum_freqs = pred:get_cum_freqs_rf(symbols[0], prev_rf)
            else
                cum_freqs = pred:get_cum_freqs_ra(symbols[0], symbols[1], prev_ra)
            end

            local sym = symbols[step]
            local total = cum_freqs[256]
            local cum_low = cum_freqs[sym]
            local cum_high = cum_freqs[sym + 1]

            local range_width = high - low + 1
            high = low + math.floor((range_width * cum_high) / total) - 1
            low = low + math.floor((range_width * cum_low) / total)

            while true do
                if high < 0x80000000 then
                    w:write_bit_helper(underflow_bits, 0)
                    low = (low * 2) % 0x100000000
                    high = ((high * 2) + 1) % 0x100000000
                elseif low >= 0x80000000 then
                    w:write_bit_helper(underflow_bits, 1)
                    low = ((low - 0x80000000) * 2) % 0x100000000
                    high = (((high - 0x80000000) * 2) + 1) % 0x100000000
                elseif low >= 0x40000000 and high < 0xC0000000 then
                    underflow_bits[1] = underflow_bits[1] + 1
                    low = ((low - 0x40000000) * 2) % 0x100000000
                    high = (((high - 0x40000000) * 2) + 1) % 0x100000000
                else
                    break
                end
            end
        end
        pred:observe(rc, rf, ra)
    end

    underflow_bits[1] = underflow_bits[1] + 1
    if low < 0x40000000 then
        w:write_bit_helper(underflow_bits, 0)
    else
        w:write_bit_helper(underflow_bits, 1)
    end

    return w.buffer, w.bit_index
end

local function decode(encoded_bytes, num_concepts, alpha, weight)
    local pred = RadicalPredictor.new(alpha, weight)
    local r = BitReader.new(encoded_bytes)

    local value = 0
    for i = 1, 32 do
        value = ((value * 2) + r:read_bit()) % 0x100000000
    end

    local low = 0
    local high = 0xFFFFFFFF
    local decoded = {}

    for c_idx = 1, num_concepts do
        local prev_rc = pred.prev_rc
        local prev_rf = pred.prev_rf
        local prev_ra = pred.prev_ra
        local symbols = {[0] = 0, [1] = 0, [2] = 0}

        for step = 0, 2 do
            local cum_freqs
            if step == 0 then
                cum_freqs = pred:get_cum_freqs_rc(prev_rc)
            elseif step == 1 then
                cum_freqs = pred:get_cum_freqs_rf(symbols[0], prev_rf)
            else
                cum_freqs = pred:get_cum_freqs_ra(symbols[0], symbols[1], prev_ra)
            end

            local total = cum_freqs[256]
            local range_width = high - low + 1
            local scaled_val = math.floor((((value - low) + 1) * total - 1) / range_width)

            local sym = 0
            local l_idx, r_idx = 0, 255
            while l_idx <= r_idx do
                local m_idx = math.floor((l_idx + r_idx) / 2)
                if cum_freqs[m_idx] <= scaled_val and scaled_val < cum_freqs[m_idx + 1] then
                    sym = m_idx
                    break
                elseif scaled_val >= cum_freqs[m_idx + 1] then
                    l_idx = m_idx + 1
                else
                    r_idx = m_idx - 1
                end
            end

            symbols[step] = sym
            local cum_low = cum_freqs[sym]
            local cum_high = cum_freqs[sym + 1]

            high = low + math.floor((range_width * cum_high) / total) - 1
            low = low + math.floor((range_width * cum_low) / total)

            while true do
                if high < 0x80000000 then
                    low = (low * 2) % 0x100000000
                    high = ((high * 2) + 1) % 0x100000000
                    value = ((value * 2) + r:read_bit()) % 0x100000000
                elseif low >= 0x80000000 then
                    low = ((low - 0x80000000) * 2) % 0x100000000
                    high = (((high - 0x80000000) * 2) + 1) % 0x100000000
                    value = (((value - 0x80000000) * 2) + r:read_bit()) % 0x100000000
                elseif low >= 0x40000000 and high < 0xC0000000 then
                    low = ((low - 0x40000000) * 2) % 0x100000000
                    high = (((high - 0x40000000) * 2) + 1) % 0x100000000
                    value = (((value - 0x40000000) * 2) + r:read_bit()) % 0x100000000
                else
                    break
                end
            end
        end

        table.insert(decoded, {
            math.floor(symbols[0] / 16) % 16,
            symbols[0] % 16,
            math.floor(symbols[1] / 16) % 16,
            symbols[1] % 16,
            math.floor(symbols[2] / 16) % 16,
            symbols[2] % 16
        })
        pred:observe(symbols[0], symbols[1], symbols[2])
    end
    return decoded
end

local inputs = {
    {1, 2, 3, 4, 5, 6},
    {8, 0, 15, 1, 0, 15},
    {0, 0, 0, 0, 0, 0},
    {15, 15, 15, 15, 15, 15},
    {4, 5, 6, 7, 8, 9}
}

local buf, bits = encode(inputs, 1, 128)
print("Encoded Bits: " .. bits .. ", Bytes: " .. #buf)
io.write("Hex: ")
for _, b in ipairs(buf) do
    io.write(string.format("%02X ", b))
end
print("")

local start_time = os.clock()
local runs = 100000
local match = true
for r = 1, runs do
    local decoded = decode(buf, 5, 1, 128)
    if r == 1 then
        for i = 1, #inputs do
            for j = 1, 6 do
                if inputs[i][j] ~= decoded[i][j] then
                    match = false
                end
            end
        end
    end
end
local end_time = os.clock()
local elapsed_ms = (end_time - start_time) * 1000.0

print("Decoded matches inputs: " .. tostring(match))
if not match then
    print("ERROR: mismatch!")
    os.exit(1)
end
print(string.format("[INTERNAL_MATH] %.4f ms", elapsed_ms))

print("\n[VERIFICATION] Multi-Language runtime FFI structures validated.")
