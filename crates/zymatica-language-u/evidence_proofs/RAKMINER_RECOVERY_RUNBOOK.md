# RakMiner-A / RakMiner-B LoRa Recovery Runbook

**Date of recovery:** 2026-06-03  
**Devices:** RakMiner-A and RakMiner-B  
**Goal:** Restore reliable SX1302 LoRa concentrator operation, fix the fatal temperature-sensor shutdown error, prove two-way RF packet flow, and freeze evidence so the process can be repeated if the issue happens again.

---

## 1. Final result

Both miners are working.

### Confirmed

- RakMiner-A detects SX1302 concentrator: **PASS**
- RakMiner-B detects SX1302 concentrator: **PASS**
- Temperature sensor shutdown error: **FIXED**
- Missing STTS751 temperature sensor is now **warning-only**
- B → A LoRa RF packet receive: **PASS**
- A → B LoRa RF packet receive: **PASS**
- Working TX/RX RF chain: **RF chain 0**
- RF chain 1 TX on RakMiner-B: **disabled / do not use**
- Working LoRa settings:
  - Radio: SX1250
  - SPI: `/dev/spidev0.0`
  - Frequency command used for TX: `903.9 MHz`
  - Observed RX packet frequency: `903500000 Hz`
  - SF: `7`
  - Bandwidth: `125 kHz`
  - Coding rate: `CR1`
  - Header: explicit
  - Polarity: non-inverted
  - Payload size used in HAL tests: `39 bytes`
  - TX power: `14 dBm`
  - Preamble: `8 symbols`

### Evidence archives frozen

RakMiner-A:

```text
A_EVIDENCE_DIR=/home/strawberry-z-model-1/POST_REBOOT_TWO_WAY_RF_PROOF_A_20260603T235115Z
A_EVIDENCE_TAR=/home/strawberry-z-model-1/POST_REBOOT_TWO_WAY_RF_PROOF_A_20260603T235115Z.tar.gz
A_SHA256=9c5a6f311cf1bac2467677f880ff154cc01b0c68501f376e43c9059ab30fbccd
```

RakMiner-B:

```text
B_EVIDENCE_DIR=/home/strawberry-z-model-1/POST_REBOOT_TWO_WAY_RF_PROOF_B_20260603T235152Z
B_EVIDENCE_TAR=/home/strawberry-z-model-1/POST_REBOOT_TWO_WAY_RF_PROOF_B_20260603T235152Z.tar.gz
B_SHA256=6f4b3142a59b0ef728485f6ab31ddeee8eacb4a4baa2d68dc408c19177ee757e
```

---

## 2. What was broken

The miners could initialize the SX1302 concentrator, but the HAL tests ended with a fatal shutdown error because the board does not expose the STTS751 temperature sensor expected by the reference HAL.

Bad error pattern before the fix:

```text
INFO: no temperature sensor found on port 0x39
INFO: no temperature sensor found on port 0x3B
INFO: no temperature sensor found on port 0x38
WARNING: no temperature sensor found; continuing without temperature telemetry.
Closing SPI communication interface
ERROR: failed to close I2C temperature sensor device (err=-1)
ERROR: failed to stop the gateway
```

This created confusing false failures even when the SX1302 chip was detected correctly.

The concentrators themselves were alive because both miners showed:

```text
Note: chip version is 0x10 (v1.0)
INFO: concentrator EUI: ...
```

---

## 3. Hardware detection facts

### RakMiner-A

Known reset behavior after fix:

```text
SX1302 reset through GPIO25
SX1261 reset through GPIO22
SX1302 power enable skipped
```

Confirmed chip ID:

```text
Note: chip version is 0x10 (v1.0)
INFO: concentrator EUI: 0x0016c001ff1aadc0
```

### RakMiner-B

Known reset behavior after fix:

```text
SX1302 reset through GPIO17
SX1261 reset through GPIO22
SX1302 power enable skipped
```

B local reset script also showed:

```text
SX1302_RESET_PIN=17
SX1302_POWER_EN_PIN=18
SX1261_RESET_PIN=22
AD5338R_RESET_PIN=13
GPIO17 = output low
GPIO18 = output high
GPIO22 = output high
GPIO13 = output high
```

Confirmed chip ID:

```text
Note: chip version is 0x10 (v1.0)
INFO: concentrator EUI: 0x0016c001ff18afa3
```

---

## 4. What fixed it

The final successful fix was to patch `libloragw/src/loragw_hal.c` so a missing optional STTS751 temperature sensor does **not** set `err = LGW_HAL_ERROR` during `lgw_stop()`.

The key block is around `i2c_linuxdev_close(ts_fd)` inside `lgw_stop()`.

### Final desired behavior

After the patch, this is allowed:

```text
WARNING: optional STTS751 temperature sensor close failed; ignored (err=-1)
```

This should **not** appear anymore:

```text
ERROR: failed to stop the gateway
```

---

## 5. Exact final temperature patch command

Run this on **both RakMiner-A and RakMiner-B** if the fatal temperature shutdown error returns.

```bash
cd "$HOME/sx1302_hal" || exit 1

echo "===== FINAL SIMPLE TEMP FIX ====="
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
PATCHDIR="$HOME/sx1302_hal_FINAL_SIMPLE_TEMP_FIX_$STAMP"
mkdir -p "$PATCHDIR"

HALC="libloragw/src/loragw_hal.c"
cp -a "$HALC" "$PATCHDIR/loragw_hal.c.before"

echo "===== BEFORE ====="
nl -ba "$HALC" | sed -n '1224,1232p'

perl -0pi -e 's/(x = i2c_linuxdev_close\(ts_fd\);\s*if \(x != 0\) \{\s*)printf\("[^"]*(?:temperature|STTS751)[^"]*\\n", x\);\s*err = LGW_HAL_ERROR;/$1printf("WARNING: optional STTS751 temperature sensor close failed; ignored (err=%i)\\n", x);\n            \/* patched: do not fail lgw_stop() for missing optional STTS751 temperature sensor *\//s' "$HALC"

echo
echo "===== AFTER ====="
nl -ba "$HALC" | sed -n '1224,1232p'

echo
echo "===== REBUILD ====="
make clean
make

echo
echo "===== TEST CHIP_ID ====="
cd "$HOME/sx1302_hal/util_chip_id" || exit 1

./chip_id -d /dev/spidev0.0 -r 1250 -k 0 2>&1 | tee "$PATCHDIR/chip_id_final_simple_temp_fix.log"

echo
echo "===== RESULT CHECK ====="
grep -aE "chip version|concentrator EUI|temperature|STTS751|Closing SPI|failed to stop|failed to close|ERROR|WARNING" "$PATCHDIR/chip_id_final_simple_temp_fix.log" || true

echo
echo "PATCHDIR=$PATCHDIR"
```

### Successful output after patch

RakMiner-A:

```text
Note: chip version is 0x10 (v1.0)
INFO: concentrator EUI: 0x0016c001ff1aadc0
Closing SPI communication interface
WARNING: optional STTS751 temperature sensor close failed; ignored (err=-1)
```

RakMiner-B:

```text
Note: chip version is 0x10 (v1.0)
INFO: concentrator EUI: 0x0016c001ff18afa3
Closing SPI communication interface
WARNING: optional STTS751 temperature sensor close failed; ignored (err=-1)
```

---

## 6. Post-patch reboot

After patching and rebuilding both miners, both devices were rebooted.

Run on each miner:

```bash
sudo reboot
```

After reboot, confirm chip detection.

Run this on **both A and B**:

```bash
cd "$HOME/sx1302_hal/util_chip_id" || exit 1

echo "===== POST-REBOOT CHIP TEST ====="
hostname
date -u +%Y-%m-%dT%H:%M:%SZ

./chip_id -d /dev/spidev0.0 -r 1250 -k 0 2>&1 | tee "$HOME/POST_REBOOT_CHIP_ID_$(hostname)_$(date +%Y%m%d_%H%M%S).log"
```

Good result:

```text
Note: chip version is 0x10 (v1.0)
INFO: concentrator EUI: ...
Closing SPI communication interface
WARNING: optional STTS751 temperature sensor close failed; ignored (err=-1)
```

Bad result:

```text
ERROR: failed to stop the gateway
```

If the bad result returns, reapply the temperature patch.

---

## 7. Clean HAL-only RF proof commands

Do **not** start with old Phase scripts when debugging basic RF. Use the HAL test tools first.

### B → A test

#### Start receiver on RakMiner-A

```bash
cd "$HOME/sx1302_hal/libloragw" || exit 1

echo "===== A RX SWEEP: LISTENING FOR B ON 903.9 ====="
echo "A_RX_SWEEP_READY_UTC=$(date -u +%Y-%m-%dT%H:%M:%SZ)"

pkill -f test_loragw_hal_rx || true
pkill -f test_loragw_hal_tx || true
sleep 2

timeout 180s stdbuf -oL -eL ./test_loragw_hal_rx \
  -d /dev/spidev0.0 \
  -r 1250 \
  -a 903.9 \
  -b 903.9 \
  -k 0 \
  -m 1 \
  -j \
  -z 255 \
  -n 1 \
  2>&1 | tee "$HOME/A_RX_SWEEP_FROM_B_$(date +%Y%m%d_%H%M%S).log"
```

#### Transmit from RakMiner-B on RF chain 0

```bash
cd "$HOME/sx1302_hal/libloragw" || exit 1

echo "===== B TX RF CHAIN 0 ====="
echo "B_TX_SWEEP_UTC=$(date -u +%Y-%m-%dT%H:%M:%SZ)"

pkill -f test_loragw_hal_tx || true
sleep 1

timeout 90s ./test_loragw_hal_tx \
  -d /dev/spidev0.0 \
  -k 0 \
  -c 0 \
  -r 1250 \
  -f 903.9 \
  -m LORA \
  -s 7 \
  -b 125 \
  -l 8 \
  -n 10 \
  -z 39 \
  -p 14 \
  -j \
  --pa 1 \
  --pwid 12 \
  2>&1 | tee "$HOME/B_TX_SWEEP_CHAIN0_$(date +%Y%m%d_%H%M%S).log"
```

#### Good B → A evidence observed

A received packets from B:

```text
----- LoRa packet -----
  size:     39
  status:   0x01
  datr:     7
  codr:     1
  rf_chain  0
  freq_hz   903500000
  snr_avg:  -7.8
  rssi_chan:139.0
  rssi_sig :132.0
  crc:      0x0000
Received 2 packets
```

A also received another packet:

```text
----- LoRa packet -----
  size:     39
  status:   0x01
  datr:     7
  codr:     1
  rf_chain  0
  freq_hz   903500000
  snr_avg:  -7.5
  rssi_chan:139.0
  rssi_sig :133.0
  crc:      0x0000
Received 1 packets
```

### A → B test

#### Start receiver on RakMiner-B

```bash
cd "$HOME/sx1302_hal/libloragw" || exit 1

echo "===== B RX: WAITING FOR A TX ON WORKING RF SETUP ====="
echo "B_RX_READY_UTC=$(date -u +%Y-%m-%dT%H:%M:%SZ)"

pkill -f test_loragw_hal_rx || true
pkill -f test_loragw_hal_tx || true
sleep 2

timeout 180s stdbuf -oL -eL ./test_loragw_hal_rx \
  -d /dev/spidev0.0 \
  -r 1250 \
  -a 903.9 \
  -b 903.9 \
  -k 0 \
  -m 1 \
  -j \
  -z 255 \
  -n 1 \
  2>&1 | tee "$HOME/B_RX_FROM_A_WORKING_SETUP_$(date +%Y%m%d_%H%M%S).log"
```

#### Transmit from RakMiner-A on RF chain 0

```bash
cd "$HOME/sx1302_hal/libloragw" || exit 1

echo "===== A TX TO B: RF CHAIN 0 ONLY ====="
echo "A_TX_UTC=$(date -u +%Y-%m-%dT%H:%M:%SZ)"

pkill -f test_loragw_hal_tx || true
sleep 1

timeout 90s ./test_loragw_hal_tx \
  -d /dev/spidev0.0 \
  -k 0 \
  -c 0 \
  -r 1250 \
  -f 903.9 \
  -m LORA \
  -s 7 \
  -b 125 \
  -l 8 \
  -n 10 \
  -z 39 \
  -p 14 \
  -j \
  --pa 1 \
  --pwid 12 \
  2>&1 | tee "$HOME/A_TX_TO_B_CHAIN0_$(date +%Y%m%d_%H%M%S).log"
```

#### Good A → B evidence observed

B received packet from A:

```text
----- LoRa packet -----
  size:     39
  status:   0x01
  datr:     7
  codr:     1
  rf_chain  0
  freq_hz   903500000
  snr_avg:  -7.2
  rssi_chan:140.0
  rssi_sig :134.0
  crc:      0x0000
Received 1 packets
```

A transmitted:

```text
Sending 10 LoRa packets on 903900000 Hz
Nb packets sent: 10 (1)
```

---

## 8. RF chain finding

RF chain 0 is the working transmit chain.

RakMiner-B RF chain 1 TX failed:

```text
ERROR: SELECTED RF_CHAIN IS DISABLED FOR TX ON SELECTED BOARD
ERROR: failed to send packet
Nb packets sent: 0 (1)
```

Therefore:

```text
Use -c 0 for TX.
Do not use -c 1 for TX on this board.
```

---

## 9. Evidence freeze commands

### Freeze evidence on RakMiner-A

```bash
cd "$HOME" || exit 1

OUT="$HOME/POST_REBOOT_TWO_WAY_RF_PROOF_A_$(date -u +%Y%m%dT%H%M%SZ)"
mkdir -p "$OUT"

cp -a "$HOME"/POST_REBOOT_CHIP_ID_RakMiner-A_*.log "$OUT/" 2>/dev/null || true
cp -a "$HOME"/A_RX_SWEEP_FROM_B_*.log "$OUT/" 2>/dev/null || true
cp -a "$HOME"/A_TX_TO_B_CHAIN0_*.log "$OUT/" 2>/dev/null || true
cp -a "$HOME/sx1302_hal/libloragw/test_loragw_hal_rx" "$OUT/" 2>/dev/null || true
cp -a "$HOME/sx1302_hal/libloragw/test_loragw_hal_tx" "$OUT/" 2>/dev/null || true

cat > "$OUT/README_A_TWO_WAY_RF_PROOF.md" <<'EOF'
# RakMiner-A Two-Way RF Proof

Confirmed after reboot and temperature HAL patch.

## A-side evidence

- RakMiner-A SX1302 chip detection: PASS, chip version 0x10
- Temperature sensor missing is warning-only, not fatal
- B to A RF receive: PASS
- A to B RF transmit: PASS
- Working RF chain: 0
- RF chain 1 TX should not be used
- Observed receive frequency: 903500000 Hz
- LoRa parameters: SF7, BW125, CR1, explicit header, non-inverted polarity
EOF

tar -czf "$OUT.tar.gz" -C "$HOME" "$(basename "$OUT")"

echo "A_EVIDENCE_DIR=$OUT"
echo "A_EVIDENCE_TAR=$OUT.tar.gz"
sha256sum "$OUT.tar.gz"
```

### Freeze evidence on RakMiner-B

```bash
cd "$HOME" || exit 1

OUT="$HOME/POST_REBOOT_TWO_WAY_RF_PROOF_B_$(date -u +%Y%m%dT%H%M%SZ)"
mkdir -p "$OUT"

cp -a "$HOME"/POST_REBOOT_CHIP_ID_RakMiner-B_*.log "$OUT/" 2>/dev/null || true
cp -a "$HOME"/B_TX_SWEEP_CHAIN*.log "$OUT/" 2>/dev/null || true
cp -a "$HOME"/B_RX_FROM_A_WORKING_SETUP_*.log "$OUT/" 2>/dev/null || true
cp -a "$HOME"/B_POST_REBOOT_TX_TO_A_*.log "$OUT/" 2>/dev/null || true
cp -a "$HOME/sx1302_hal/libloragw/test_loragw_hal_rx" "$OUT/" 2>/dev/null || true
cp -a "$HOME/sx1302_hal/libloragw/test_loragw_hal_tx" "$OUT/" 2>/dev/null || true

cat > "$OUT/README_B_TWO_WAY_RF_PROOF.md" <<'EOF'
# RakMiner-B Two-Way RF Proof

Confirmed after reboot and temperature HAL patch.

## B-side evidence

- RakMiner-B SX1302 chip detection: PASS, chip version 0x10
- Temperature sensor missing is warning-only, not fatal
- B to A RF transmit: PASS
- A to B RF receive: PASS
- Working RF chain: 0
- RF chain 1 TX disabled on selected board
- Observed receive frequency: 903500000 Hz
- LoRa parameters: SF7, BW125, CR1, explicit header, non-inverted polarity
EOF

tar -czf "$OUT.tar.gz" -C "$HOME" "$(basename "$OUT")"

echo "B_EVIDENCE_DIR=$OUT"
echo "B_EVIDENCE_TAR=$OUT.tar.gz"
sha256sum "$OUT.tar.gz"
```

---

## 10. Archive verification commands

### Verify A archive

```bash
cd "$HOME" || exit 1

A_TAR="$HOME/POST_REBOOT_TWO_WAY_RF_PROOF_A_20260603T235115Z.tar.gz"

echo "===== VERIFY A ARCHIVE ====="
ls -la "$A_TAR"
sha256sum "$A_TAR"
tar -tzf "$A_TAR" | sed -n '1,120p'

echo
echo "===== A README INSIDE ARCHIVE ====="
tar -xOzf "$A_TAR" POST_REBOOT_TWO_WAY_RF_PROOF_A_20260603T235115Z/README_A_TWO_WAY_RF_PROOF.md
```

### Verify B archive

```bash
cd "$HOME" || exit 1

B_TAR="$HOME/POST_REBOOT_TWO_WAY_RF_PROOF_B_20260603T235152Z.tar.gz"

echo "===== VERIFY B ARCHIVE ====="
ls -la "$B_TAR"
sha256sum "$B_TAR"
tar -tzf "$B_TAR" | sed -n '1,120p'

echo
echo "===== B README INSIDE ARCHIVE ====="
tar -xOzf "$B_TAR" POST_REBOOT_TWO_WAY_RF_PROOF_B_20260603T235152Z/README_B_TWO_WAY_RF_PROOF.md
```

Verified B archive contents included:

```text
POST_REBOOT_TWO_WAY_RF_PROOF_B_20260603T235152Z/
POST_REBOOT_TWO_WAY_RF_PROOF_B_20260603T235152Z/B_TX_SWEEP_CHAIN1_20260603_194332.log
POST_REBOOT_TWO_WAY_RF_PROOF_B_20260603T235152Z/B_POST_REBOOT_TX_TO_A_20260603_193843.log
POST_REBOOT_TWO_WAY_RF_PROOF_B_20260603T235152Z/B_RX_FROM_A_WORKING_SETUP_20260603_194556.log
POST_REBOOT_TWO_WAY_RF_PROOF_B_20260603T235152Z/README_B_TWO_WAY_RF_PROOF.md
POST_REBOOT_TWO_WAY_RF_PROOF_B_20260603T235152Z/POST_REBOOT_CHIP_ID_RakMiner-B_20260603_193527.log
POST_REBOOT_TWO_WAY_RF_PROOF_B_20260603T235152Z/test_loragw_hal_rx
POST_REBOOT_TWO_WAY_RF_PROOF_B_20260603T235152Z/test_loragw_hal_tx
POST_REBOOT_TWO_WAY_RF_PROOF_B_20260603T235152Z/B_TX_SWEEP_CHAIN0_20260603_194323.log
POST_REBOOT_TWO_WAY_RF_PROOF_B_20260603T235152Z/B_POST_REBOOT_TX_TO_A_20260603_193733.log
```

---

## 11. If this breaks again

Follow this order. Do **not** jump straight to Language U / Phase scripts.

### Step 1: verify chip

Run on each miner:

```bash
cd "$HOME/sx1302_hal/util_chip_id" || exit 1
./chip_id -d /dev/spidev0.0 -r 1250 -k 0
```

If no `0x10`, fix reset GPIO / SPI first.

### Step 2: check temperature patch

Bad:

```text
ERROR: failed to stop the gateway
```

Fix: rerun the final temperature patch command.

### Step 3: run clean HAL-only RF test

Do B → A, then A → B using the commands above.

### Step 4: only after HAL RF passes, run Language U / Phase scripts

Known-good RF layer:

```text
/dev/spidev0.0
RF chain 0
903.9 MHz TX command
Observed RX freq_hz 903500000
SF7
BW125
CR1
Explicit header
Non-inverted polarity
39-byte payload tests
```

### Step 5: avoid these mistakes

- Do not use RF chain 1 for TX.
- Do not assume the missing STTS751 temperature sensor is fatal.
- Do not trust old Phase scripts until HAL-only RF passes.
- Do not paste terminal prompts into the shell.
- Do not transmit without a 915 MHz antenna or 50-ohm load.
- Do not keep repeating the same test if the log shows only `Waiting for packets...`.
- If packets are missing, do RF chain / antenna / pigtail / parameter testing.

---

## 12. Bottom line

The miners were fixed by:

1. Proving both SX1302 concentrators were detectable with chip `0x10`.
2. Identifying the missing STTS751 temperature sensor as a false fatal shutdown problem.
3. Patching `loragw_hal.c` so temperature sensor close failure does not set HAL failure.
4. Rebuilding the HAL on both miners.
5. Rebooting both miners.
6. Verifying chip ID survived reboot.
7. Running clean HAL-only LoRa tests.
8. Proving B → A RF packets.
9. Proving A → B RF packets.
10. Freezing evidence archives with SHA256 hashes.

Current safe working command family:

```text
test_loragw_hal_rx
test_loragw_hal_tx
RF chain 0
SF7
BW125
903.9 MHz TX command
Observed packet receive at 903500000 Hz
```
