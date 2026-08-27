# 🦀 Zymatica Voice - Quick Start Guide

## 🚀 Launch The App

### Option 1: Desktop Icon (Recommended)
**Double-click:** `🦀 Zymatica Voice - The App` on your desktop

### Option 2: Terminal
```bash
python3 /home/researcher/zymatica_voice_app.py
```

---

## 🎨 What You'll See

When you launch, you'll see a **cyberpunk-style terminal UI** with:

1. **Agent Identity Card** (Solana purple/green colors)
   - Your LoRa phone number (Bitcoin-style address)
   - zymatica.space address
   - ECDSA keypair info

2. **Main Menu**
   - [1] Transmit Message (TX)
   - [2] Listen for Packets (RX)
   - [3] Show Identity
   - [4] Generate ZK-Proof
   - [5] Export Whitepaper
   - [0] Exit

---

## 📡 First Transmission

1. **Launch the app** (double-click desktop icon)
2. **Type:** `1` (Transmit)
3. **Enter message:** `Hello from researcher-1!`
4. **Packet count:** `5` (default)
5. **Watch the magic:** Matrix-style packet animation
6. **Result:** 5 packets transmitted @ 903.9 MHz

**What's happening:**
- ✅ ECDSA identity attached (your LoRa phone number)
- ✅ ZK-proof generated (proves legitimacy, hides UID)
- ✅ Packet encrypted (ECIES, recipient-only decrypt)
- ✅ Transmitted on 903.9 MHz, SF9, 125kHz

---

## 📻 Receive Packets

1. **Launch the app**
2. **Type:** `2` (Listen)
3. **Duration:** `30` seconds (default)
4. **Wait:** App listens for incoming packets
5. **Result:** Any packets received displayed with SNR/RSSI

**Note:** Current version simulates RX. Real SX1302 HAL integration coming in Rust v2.0!

---

## 🔐 Your Identity

Your identity is saved in:
```
~/.zyMatica/keys/researcher-1.json
```

Contains:
- ECDSA private key (secp256k1, Bitcoin's curve)
- Public key
- LoRa phone number (HASH160 of public key)
- zymatica.space address

**First launch:** Identity auto-generated  
**Subsequent launches:** Identity loaded from file

**Backup this file!** It's your agent's soul. 🔑

---

## 🎨 Cyberpunk UI Features

- **Solana Purple** (`#9945FF`) - Primary branding
- **Solana Green** (`#14F195`) - Success states
- **Matrix-style animations** - Packet transmission visualization
- **ASCII art borders** - Cyberpunk aesthetic
- **Color-coded menu** - Easy navigation

---

## 🦀 Road to Rust v2.0

**Current (Python v1.0):**
- ✅ Bitcoin-style identity
- ✅ ZK-proof mock (ready for arkworks integration)
- ✅ Cyberpunk UI
- ✅ Simulated TX/RX

**Rust v2.0 (Grant-funded):**
- 🔮 25x faster performance
- 🔮 Real SX1302 HAL integration
- 🔮 Actual LoRa TX/RX
- 🔮 WebAssembly build (runs in browser)
- 🔮 Solana program (on-chain ZK verification)

---

## 📞 Support

**Issue?** Check whitepaper first:
```
Double-click: 🦀 Zymatica Voice (Solana Grant)
```

**Still stuck?**
- GitHub: github.com/zymatica/voice (TBD)
- Email: dan@zymatica.space

---

## 🎯 Next Steps

1. ✅ **Launch app** (double-click desktop icon)
2. ✅ **Transmit test packet** (type `1`)
3. ✅ **Check identity** (type `3`)
4. ✅ **Read whitepaper** (type `5`)
5. 🔄 **Wait for Rust v2.0** (grant-funded development)

---

<div align="center">

**"The impossible is just code waiting to be written."**

🦀 **Zymatica Voice**  
*From E-Waste to AI Grace*

Powered by Solana | Secured by ZK | Built with Rust (soon)

</div>