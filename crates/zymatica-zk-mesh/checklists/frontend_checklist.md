# ZK-LoRaWAN Frontend Development Checklist

This checklist outlines the stack, features, security implementations, and deployment steps for the ZK-LoRaWAN dApp dashboard and DePIN explorer at `zymatica.space`.

---

## 1. Technology Stack & Bootstrapping

- [ ] **Core Framework:** Next.js 14+ (App Router) with React 18+ and TypeScript.
- [ ] **Styling System:** Vanilla CSS designed with premium dark mode, subtle HSL gradients, and glassmorphism cards.
- [ ] **Web3 Integration:**
  - `@solana/web3.js` and `@solana/safe-token`
  - `@solana/wallet-adapter-base`
  - `@solana/wallet-adapter-react`
  - `@solana/wallet-adapter-react-ui` (with custom glassmorphism overrides)
- [ ] **Geospatial Mapping:** `mapbox-gl` and `react-map-gl` for rendering physical gateway hotspots.
- [ ] **Metrics & Charts:** `recharts` or `chart.js` for dynamic pool stats and transaction histories.

---

## 2. Web3 Developer Portal

- [ ] **Multi-Wallet Connector:** Support for Phantom, Solflare, and Backpack wallets.
- [ ] **Shielded Pool Escrow Deposit:**
  - Display current on-chain pool reserve balance.
  - Form to deposit SOL, executing the `deposit_shielded` transaction on-chain.
- [ ] **Client-Side Key Generation Tool:**
  - Securely derive secp256k1 keypairs locally using Web Crypto API.
  - Downloadable backup JSON keyfile (matching user-identity metadata).
  - Computation of the 8-character hex identity commitment ("LoRa Phone Number") to display to the user.
- [ ] **Firmware Whitelist Dashboard:**
  - Administrative dashboard gated behind the `ADMIN_AUTHORITY` signature.
  - Table of active approved firmware hashes.
  - Interface to submit new firmware whitelists to the Registry PDA.
- [ ] **QR Code Node Onboarding:** Smartphone camera scanner integration in the portal allowing developers/users to scan a QR code printed on the physical device casing to instantly extract the ATECC608A public key and register it on Solana.

---

## 3. DePIN Network Explorer (`explorer.zymatica.space`)

- [ ] **Interactive Gateway Map:**
  - Renders active gateways as neon green nodes using Mapbox GL.
  - Heatmap layer showing message density and routing channels.
  - Decoupled from private wallet addresses (displaying only Gateway ID and location tags).
- [ ] **Anonymized Telemetry Feed:**
  - Live table streaming incoming packet headers: Frequency, Spreading Factor, RSSI, SNR, and Frame Counter.
  - Indicators confirming packet validity (e.g., "ZK Proof Passed", "Registry Whitelist Match").
- [ ] **Dynamic Metrics Dashboard:**
  - TVL (Total Value Locked) in the `ShieldedEscrowPool`.
  - Total packets routed vs. total transaction fees paid.
  - Real-time chart showing daily average verification times (targeting < 1.5ms).

---

## 4. Gateway Operator Claim Interface

- [ ] **Operator Status Panel:**
  - Displays gateway router status, uptime history, and total packets routed.
  - Shows accumulated claimable rewards in lamports and equivalent USD.
- [ ] **One-Click Rewards Claim:**
  - Interactive "Claim Earnings" button executing the Anchor program claim instruction.
  - Automatic deduction of the 2% developer fee split on-chain.
  - Success modal displaying the transaction confirmation signature.

---

## 5. Security & RPC Proxy Layer

- [ ] **Next.js Serverless RPC Proxy (`/api/rpc`):**
  - Configure the Solana `Connection` to request endpoints via local API routes.
  - Server-side redirect of `/api/rpc` traffic to the private RPC provider (Helius/Triton) using secure, non-exposed environment variables.
- [ ] **Content Security Policy (CSP):**
  - Configure `next.config.js` to restrict wallet-adapter script execution to authorized domains.
  - Block unauthorized frame-ancestors to prevent clickjacking scams.
- [ ] **CORS and XSS Mitigations:**
  - Enforce strict CORS on all API routes.
  - Prevent cross-site scripting vectors targeting local key generation operations.
- [ ] **RPC Node Latency & Connection Health Monitor:** Real-time visual indicator in the portal footer showing current RPC status and latency ping times, with automatic browser-side switching to fallback RPC nodes if the primary node lag exceeds 2.5 seconds.

---

## 6. ZK client-side Verification Tool (Crucial Addition)

- [ ] **WASM Compiled Verifier Sandbox:**
  - Compile the Rust Groth16 verifying logic to WebAssembly (`wasm-pack`).
  - A client-side sandbox page where developers can upload a serialized proof file (`proof.bin`), input verification variables, and execute the bilinear pairing checks locally in the browser before deploying over radio links.

---

## 7. Offline Progressive Web App (PWA) Support (Crucial Addition)

- [ ] **Local Gateway Configuration (PWA):**
  - Configure service workers to allow the dashboard to run completely offline.
  - Support local Bluetooth / Local Wi-Fi AP communication so gateway operators in remote locations with zero internet backhaul can connect to their physical RAK miners and view telemetry logs locally in the browser interface.

---

## 8. Vercel Hosting & Domain Configuration

- [ ] **Vercel Project Setup:**
  - Link the Vercel dashboard to the GitHub repository.
  - Configure automated deployment triggers for the `master` branch.
- [ ] **Environment Variable Vault:**
  - Inject private Solana Devnet/Mainnet RPC keys.
  - Inject private Mapbox API access tokens.
- [ ] **DNS & Subdomain Mapping:**
  - Map primary domain `zymatica.space`.
  - Map `explorer.zymatica.space` to the explorer sub-route.
  - Map `dev.zymatica.space` to the test portal.
