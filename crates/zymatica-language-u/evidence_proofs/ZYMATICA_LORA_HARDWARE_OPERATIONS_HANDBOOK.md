# ZYMATICA: LoRa Hardware Operations & Integration Handbook
*IP Class 06/11 | Zymatica Proprietary Protocol Specification*

> *Watermark: ip zymatica.space | astronautshe.com*

---

This handbook serves as the definitive reference for configuring, troubleshooting, and verifying the physical transmission layers of the Sumerian: Language-U Semantic Communication Protocol on RAK wireless hardware. It unifies low-level kernel driver overrides, hardware abstraction library (HAL) source patches, and dynamic python-steered transceiver execution.

---

## 1. Physical Hardware Layout & GPIO Map
RAK wireless concentrators (specifically the RAK2287 module utilizing the Semtech SX1302 chip and dual SX1250 radios) must be wired to specific GPIO pins on the host edge node (e.g. Raspberry Pi 4). 

* **SPI Device Interface:** `/dev/spidev0.0`
* **Concentrator Reset (GPIO 17):** Driven low for active state, high for reset.
* **Concentrator Power Enable (GPIO 18):** Driven high to active VCC to the SX1302 and SX1250 modules.
* **SX1261 Reset (GPIO 22):** Driven high for default state.
* **AD5338R Reset (GPIO 13):** Driven high for default state.

### Startup Boot Sequence Constraint
The SPI concentrator requires a warm-up phase to stabilize internal registers. To prevent the chip version query from returning a fail code (`0x00`), a **mandatory 2-second delay** must be executed immediately after driving the reset pins low before the first SPI packet command is written.

---

## 2. Linux kernel & Concentrator Initialization
Before running the validation loop, the host GPIO state must be manually set up. Save the following shell script as `reset_lgw.sh` inside the Semtech HAL working directory (`~/sx1302_hal/libloragw/`):

```bash
#!/usr/bin/env bash
# Watermark: ip zymatica.space | astronautshe.com
set -u

SX1302_RESET_PIN=17
SX1302_POWER_EN_PIN=18
SX1261_RESET_PIN=22
AD5338R_RESET_PIN=13

sleep_short() {
  sleep 0.10
}

set_pin() {
  local pin="$1"
  local level="$2"
  
  # Try pinctrl CLI tool
  if command -v pinctrl >/dev/null 2>&1; then
    if [ "$level" = "high" ]; then
      pinctrl set "$pin" op dh
    else
      pinctrl set "$pin" op dl
    fi
    return 0
  fi
  
  # Try legacy raspi-gpio tool
  if command -v raspi-gpio >/dev/null 2>&1; then
    if [ "$level" = "high" ]; then
      raspi-gpio set "$pin" op dh
    else
      raspi-gpio set "$pin" op dl
    fi
    return 0
  fi
  
  # Fallback to sysfs direct interface
  if [ ! -d "/sys/class/gpio/gpio$pin" ]; then
    echo "$pin" > /sys/class/gpio/export 2>/dev/null || true
    sleep 0.05
  fi
  echo out > "/sys/class/gpio/gpio$pin/direction" 2>/dev/null || true
  if [ "$level" = "high" ]; then
    echo 1 > "/sys/class/gpio/gpio$pin/value"
  else
    echo 0 > "/sys/class/gpio/gpio$pin/value"
  fi
}

echo "[*] Triggering SX1302 reset sequence..."
set_pin "$SX1302_POWER_EN_PIN" high
sleep_short

set_pin "$SX1302_RESET_PIN" high
sleep_short
set_pin "$SX1302_RESET_PIN" low
sleep_short

set_pin "$SX1261_RESET_PIN" low
sleep_short
set_pin "$SX1261_RESET_PIN" high
sleep_short

set_pin "$AD5338R_RESET_PIN" low
sleep_short
set_pin "$AD5338R_RESET_PIN" high
sleep_short

echo "[+] Concentrator reset complete. GPIO17 is LOW, GPIO18 is HIGH."
exit 0
```

---

## 3. The Temperature Sensor Patch
Standard Semtech HAL reference builds expect a board-integrated STTS751 temperature sensor and halt execution during gateway termination if the sensor is missing. Because RAK2287 concentrator modules lack this chip, closing SPI communication triggers a fatal system halt error.

### Resolution:
Patch the source file `libloragw/src/loragw_hal.c` to handle the close failure of the temperature sensor descriptor (`ts_fd`) as a warning instead of a fatal error:

```c
/* Search for this block in libloragw/src/loragw_hal.c: */
x = i2c_linuxdev_close(ts_fd);
if (x != 0) {
    /* Modify to bypass setting err = LGW_HAL_ERROR */
    printf("WARNING: optional STTS751 temperature sensor close failed; ignored (err=%i)\n", x);
}
```
Recompile the HAL library using `make clean && make`. When successfully patched, querying the hardware EUI via `./chip_id` will yield a correct version code (`0x10`) without throwing a fatal halt.

---

## 4. RF Tuning & Low-Level Verification

### RF Chain Configuration Limits
* **Active TX Path:** **RF Chain 0** (must pass `-c 0` to HAL commands).
* **Disabled TX Path:** **RF Chain 1** (this channel is hardware-disabled for transmit operations on these boards).
* **Transmission parameters:** Frequency **903.9 MHz**, Spreading Factor **SF7** or **SF9**, Bandwidth **125 kHz**, power **14 dBm (25 mW)**.

### HAL Testing Loops
Ensure all duplicate listener and forwarder processes are stopped (`pkill -9 -f test_loragw_hal`) before running diagnostics.

1. **Start Receiver Node (Miner B):**
   ```bash
   ./test_loragw_hal_rx -d /dev/spidev0.0 -r 1250 -a 903.9 -b 903.9 -k 0 -m 1 -j -z 255 -n 1
   ```
2. **Start Transmitter Node (Miner A):**
   ```bash
   ./test_loragw_hal_tx -d /dev/spidev0.0 -k 0 -c 0 -r 1250 -f 903.9 -m LORA -s 7 -b 125 -l 8 -n 10 -z 39 -p 14 -j --pa 1 --pwid 12
   ```

---

## 5. Dynamic Python Client Integration
Once the underlying RF hardware layer is verified, use the dynamic Language-U python transceivers to transmit semantic coordinate matrices. These transceivers pack the data and manage transport over either local sockets or LoRa serial ports.

### A. The Transmitter Setup (Miner A)
Run the transmitter script to compress and packetize coordinates:
```bash
python3 RakMiner-A1.py --mode serial --device /dev/ttyUSB0
```
* **LLD-AC Range Compression:** Packs intent coordinates recursively into an entropy-compressed stream.
* **XOR-FEC frame packaging:** Splices payload into 255-byte frames plus 1 XOR parity packet to defend against channel loss.

### B. The Receiver Setup (Miner B)
Run the receiver script to listen and decode the incoming stream:
```bash
python3 RakMiner-B2.py --mode serial --device /dev/ttyUSB0
```
* **XOR-FEC Recovery:** Reconstructs the original payload lossless even if a frame is dropped.
* **Range Decoding:** Performs system ascent to decode the 6D coordinates and asserts the dynamic verification anchor:
  `[VERIFICATION] Multi-Language runtime FFI structures validated.`

---

## 6. Safety Standards & Troubleshooting

### Safety Warnings
> [!CAUTION]
> **Antenna Load Requirement:** Never run transmit commands (`test_loragw_hal_tx` or `RakMiner-A1.py` in serial mode) without an antenna or a 50-ohm dummy load securely connected to the RF SMA ports. Transmitting into an open load causes high voltage reflection that will permanently burn out the transceivers' power amplifiers.

### Diagnostics Table
| Symptom | Probable Cause | Action |
|:---|:---|:---|
| **Chip version returns `0x00`** | Concentrator power is off or reset pin is floating. | Execute `./reset_lgw.sh` and confirm GPIO17 is driven LOW. |
| **`ERROR: failed to stop the gateway`** | Unpatched temperature sensor close failure. | Re-verify the C source patch in `loragw_hal.c` and rebuild. |
| **RX Node is completely deaf** | Carrier frequency misalignment or TX on Chain 1. | Ensure both nodes are set to 903.9 MHz and TX is using RF Chain 0. |
| **CRC failures on received packets** | Weak signal or antenna pigtail loose. | Check SMA connectors and increase SF or TX power to 14 dBm. |
