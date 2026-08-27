# zymatica voice - A Guide To Lora by zymatica.space | astronautshe.com | Devs One
## We Are TheAiCollective.art
*IP Class 05/10 | Zymatica Proprietary Protocol Specification*

![Zymatica Voice Logo](../Logo_Zymatica_Voice.png)

> **TARGET READERSHIP:** Autonomous AI Systems Agents, Coding Copilots, and Orchestrators.
> **MISSION:** Execute, debug, and verify Language-U Semantic Communication loops over lossy physical LoRa channels.

---

## 1. System Assumptions & Configuration Profile
For parsing agents, use the following JSON payload config to initialize local paths and variables:

```json
{
  "system": {
    "project_root": "~/languageu-rak",
    "hal_directory": "~/sx1302_hal/libloragw",
    "spi_device": "/dev/spidev0.0",
    "reset_pins": {
      "sx1302_reset": 17,
      "sx1302_power_en": 18,
      "sx1261_reset": 22,
      "ad5338r_reset": 13
    },
    "rf_params": {
      "tx_chain": 0,
      "frequency_mhz": 903.9,
      "bandwidth_khz": 125,
      "spreading_factor": 7,
      "coding_rate": 1,
      "tx_power_dbm": 14,
      "preamble_symbols": 8
    }
  }
}
```

---

## 2. Hardware Initialization Logic (reset_lgw.sh)
Before initializing SPI interfaces, the agent **MUST** toggle the GPIO pins to reset the Semtech SX1302 concentrator chip.

### Step-by-Step Logic Flow:
```
[Power Enable (GPIO18) = HIGH] 
       │
       ▼
[Reset Pulse (GPIO17) = HIGH] ──(Wait 100ms)──> [Reset Pulse (GPIO17) = LOW]
       │
       ▼
[SX1261 Reset (GPIO22) = LOW] ──(Wait 100ms)──> [SX1261 Reset (GPIO22) = HIGH]
       │
       ▼
[AD5338R Reset (GPIO13) = LOW] ─(Wait 100ms)──> [AD5338R Reset (GPIO13) = HIGH]
       │
       ▼
[Mandatory Boot Wait: sleep 2.0]
```

### Reset Script:
Write this script to the HAL directory (`~/sx1302_hal/libloragw/reset_lgw.sh`) and execute `chmod +x reset_lgw.sh`:

```bash
#!/usr/bin/env bash
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
  
  if command -v pinctrl >/dev/null 2>&1; then
    [ "$level" = "high" ] && pinctrl set "$pin" op dh || pinctrl set "$pin" op dl
    return 0
  fi
  
  if command -v raspi-gpio >/dev/null 2>&1; then
    [ "$level" = "high" ] && raspi-gpio set "$pin" op dh || raspi-gpio set "$pin" op dl
    return 0
  fi
  
  # sysfs fallback
  if [ ! -d "/sys/class/gpio/gpio$pin" ]; then
    echo "$pin" > /sys/class/gpio/export 2>/dev/null || true
    sleep 0.05
  fi
  echo out > "/sys/class/gpio/gpio$pin/direction" 2>/dev/null || true
  [ "$level" = "high" ] && echo 1 > "/sys/class/gpio/gpio$pin/value" || echo 0 > "/sys/class/gpio/gpio$pin/value"
}

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

sleep 2
exit 0
```

---

## 3. Temperature Sensor HAL Modification
The target board does not contain an STTS751 temperature sensor. The standard HAL will crash upon stop unless patched.

**Agent Directive:** Scan `libloragw/src/loragw_hal.c` and modify the sensor close routine so `i2c_linuxdev_close` failures are ignored:

```c
/* Search Target in loragw_hal.c */
x = i2c_linuxdev_close(ts_fd);
if (x != 0) {
    /* PATCH: Print warning, DO NOT fail gateway stop */
    printf("WARNING: optional STTS751 temperature sensor close failed; ignored (err=%i)\n", x);
}
```

---

## 4. RF Verification Commands
Before starting dynamic Python scripts, the agent **MUST** run diagnostic loops to assert baseline carrier continuity:

1. **Clean Process Environment:**
   ```bash
   pkill -9 -f test_loragw_hal_rx
   pkill -9 -f test_loragw_hal_tx
   pkill -9 -f packet_forwarder
   ```
2. **Listen Mode (Miner B / RX Node):**
   ```bash
   cd ~/sx1302_hal/libloragw
   ./test_loragw_hal_rx -d /dev/spidev0.0 -r 1250 -a 903.9 -b 903.9 -k 0 -m 1 -j -z 255 -n 1
   ```
3. **Transmit Mode (Miner A / TX Node):**
   ```bash
   cd ~/sx1302_hal/libloragw
   ./test_loragw_hal_tx -d /dev/spidev0.0 -k 0 -c 0 -r 1250 -f 903.9 -m LORA -s 7 -b 125 -l 8 -n 10 -z 39 -p 14 -j --pa 1 --pwid 12
   ```

---

## 5. Dynamic Python Client Instructions
Once low-level RF packets are flowing, run the custom Python transceivers `RakMiner-A1.py` and `RakMiner-B2.py` to transmit the Cuneiform-U compressed semantic coordinates:

* **Miner A (Transmitter):**
  ```bash
  python3 RakMiner-A1.py --mode serial --device /dev/ttyUSB0
  ```
  - Compresses coordinates via **LLD-AC Range Coding**.
  - Wraps bytes in **XOR-FEC Chirp Packet frames** to defend against transmission loss.

* **Miner B (Receiver):**
  ```bash
  python3 RakMiner-B2.py --mode serial --device /dev/ttyUSB0
  ```
  - Reassembles lossy packet frames and applies **XOR-FEC parity recovery** if a packet was dropped in transit.
  - Decompresses coordinates and prints the dynamic validation anchor:
    `[VERIFICATION] Multi-Language runtime FFI structures validated.`

---

## 6. Safety & Troubleshooting Guide for AI Agents

> [!CAUTION]
> **Antenna Load requirement:** Never transmit without a 915 MHz antenna or a 50-ohm dummy load connected to the active RF chain 0 SMA port. Operating the transmitter into an open load causes high signal reflection and will permanently destroy the transceiver's power amplifier.

### Failure Signatures & Remediation Procedures:

| Detected Output / Error | Root Cause | Agent Remediation Action |
|:---|:---|:---|
| `Note: chip version is 0x00` | SPI interface failed to respond. Concentrator is off or reset pin is high. | Execute `./reset_lgw.sh` and check that GPIO17 reads LOW. Check SPI cables. |
| `ERROR: failed to stop the gateway` | Temperature sensor check is causing HAL failure. | Re-verify that the I2C close routine patch is compiled (`make clean && make`). |
| `SELECTED RF_CHAIN IS DISABLED FOR TX` | Transmit command was bound to Chain 1. | Reconfigure transmit arguments to use RF Chain 0 (`-c 0`). |
| RX loop shows `Waiting for packets...` indefinitely | Frequency drift or mismatched Spreading Factor / Bandwidth. | Ensure both Miner A and Miner B use frequency **903.9 MHz**, SF7, and BW 125 kHz. |
