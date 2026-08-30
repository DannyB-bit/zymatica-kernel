import * as crypto from "crypto";

// Mock Solana classes in pure JavaScript to test layout serialization/deserialization
class PublicKey {
  constructor(bufferOrString) {
    if (typeof bufferOrString === "string") {
      this.buffer = crypto.randomBytes(32);
    } else {
      this.buffer = bufferOrString;
    }
  }
  toBuffer() {
    return this.buffer;
  }
  toBase58() {
    return this.buffer.toString("hex").substring(0, 44);
  }
}

class CuneiformClientMock {
  constructor(programIdStr) {
    this.programId = new PublicKey(programIdStr);
  }

  getDiscriminator(prefix, name) {
    const hash = crypto.createHash("sha256").update(`${prefix}:${name}`).digest();
    return hash.subarray(0, 8);
  }

  serializeInitializeProgram(treasury, feeLamports) {
    const ixDiscriminator = this.getDiscriminator("global", "initialize_program");
    const feeBuffer = Buffer.alloc(8);
    feeBuffer.writeBigUInt64LE(BigInt(feeLamports));

    return Buffer.concat([
      ixDiscriminator,
      treasury.toBuffer(),
      feeBuffer,
    ]);
  }

  serializeRegisterCoordinates(sessionId, coords, merkleRoot) {
    if (sessionId.length !== 16) throw new Error("Session ID must be 16 bytes.");
    if (coords.length !== 6) throw new Error("Coordinates must be exactly 6 elements.");
    if (merkleRoot.length !== 32) throw new Error("Merkle root must be 32 bytes.");

    const ixDiscriminator = this.getDiscriminator("global", "register_coordinates");

    return Buffer.concat([
      ixDiscriminator,
      sessionId,
      Buffer.from(coords),
      merkleRoot,
    ]);
  }

  deserializeProgramState(data) {
    if (data.length < 80) {
      throw new Error("Invalid program state length. Expected at least 80 bytes.");
    }
    const expectedDiscriminator = this.getDiscriminator("account", "ProgramState");
    const accountDiscriminator = data.subarray(0, 8);
    if (!accountDiscriminator.equals(expectedDiscriminator)) {
      throw new Error("State account discriminator mismatch.");
    }

    const admin = new PublicKey(data.subarray(8, 40));
    const treasury = new PublicKey(data.subarray(40, 72));
    const feeLamports = data.readBigUInt64LE(72);

    return { admin, treasury, feeLamports };
  }

  deserializeRecord(data) {
    if (data.length < 103) {
      throw new Error("Invalid account data length. Expected at least 103 bytes.");
    }

    const expectedDiscriminator = this.getDiscriminator("account", "CoordinateRecord");
    const accountDiscriminator = data.subarray(0, 8);
    if (!accountDiscriminator.equals(expectedDiscriminator)) {
      throw new Error("Account discriminator mismatch.");
    }

    const recAuthority = new PublicKey(data.subarray(8, 40));
    const recSessionId = data.subarray(40, 56);
    const recCoords = Array.from(data.subarray(56, 62));
    const recMerkleRoot = data.subarray(62, 94);
    
    // Read timestamp as 64-bit little endian integer
    const recTimestamp = Number(data.readBigInt64LE(94));
    const recBump = data[102];

    return {
      authority: recAuthority,
      sessionId: recSessionId,
      coords: recCoords,
      merkleRoot: recMerkleRoot,
      timestamp: recTimestamp,
      bump: recBump,
    };
  }
}

class SolanaPayGatewayMock {
  generatePaymentRequest(recipientNode, amount, sessionId) {
    const reference = crypto.randomBytes(32).toString("hex").substring(0, 44);
    const label = encodeURIComponent("Zymatica DePIN Mesh Relay");
    const message = encodeURIComponent(`Reward for routing Cuneiform-U packet (Session: ${sessionId.toString("hex").substring(0, 8)})`);
    const memo = encodeURIComponent(`zymatica:mesh:${sessionId.toString("hex")}`);
    const token = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC

    const url = `solana:${recipientNode.toBase58()}?amount=${amount}&spl-token=${token}&reference=${reference}&label=${label}&message=${message}&memo=${memo}`;

    return {
      url,
      reference,
      memo: `zymatica:mesh:${sessionId.toString("hex")}`,
    };
  }
}

function runStandaloneTests() {
  console.log("--- Running Standalone Zymatica-Solana Fee Monetization Layout Tests ---");

  const programId = "CunE111111111111111111111111111111111111111";
  const client = new CuneiformClientMock(programId);
  const payGateway = new SolanaPayGatewayMock();

  const admin = new PublicKey(crypto.randomBytes(32));
  const treasury = new PublicKey(crypto.randomBytes(32));
  const feeLamports = 1000000n; // 0.001 SOL (1,000,000 lamports)

  // Test 1: Initialize Program Instruction Serialization
  console.log("\n[Test 1] Verifying Initialize Program Serialization...");
  const serializedInit = client.serializeInitializeProgram(treasury, feeLamports);
  console.log(`Serialized Init Instruction Length: ${serializedInit.length} bytes`);
  
  const expectedInitDiscriminator = client.getDiscriminator("global", "initialize_program");
  const extractedInitDisc = serializedInit.subarray(0, 8);
  const extractedTreasury = new PublicKey(serializedInit.subarray(8, 40));
  const extractedFee = serializedInit.readBigUInt64LE(40);

  if (!extractedInitDisc.equals(expectedInitDiscriminator)) throw new Error("Init discriminator failure");
  if (!extractedTreasury.toBuffer().equals(treasury.toBuffer())) throw new Error("Treasury publickey mismatch");
  if (extractedFee !== feeLamports) throw new Error("Fee lamports mismatch");
  console.log("-> Test 1 Passed.");

  // Test 2: Deserializing Program State Config Account
  console.log("\n[Test 2] Verifying Program State Deserialization...");
  const expectedStateDiscriminator = client.getDiscriminator("account", "ProgramState");
  const mockStateData = Buffer.concat([
    expectedStateDiscriminator,
    admin.toBuffer(),
    treasury.toBuffer(),
    serializedInit.subarray(40, 48) // Contains u64 fee bytes
  ]);

  const state = client.deserializeProgramState(mockStateData);
  console.log("Deserialized Program State:");
  console.log(` - Admin: ${state.admin.toBase58()}`);
  console.log(` - Treasury: ${state.treasury.toBase58()}`);
  console.log(` - Protocol Fee: ${state.feeLamports} lamports`);

  if (!state.admin.toBuffer().equals(admin.toBuffer())) throw new Error("Admin mismatch");
  if (!state.treasury.toBuffer().equals(treasury.toBuffer())) throw new Error("Treasury mismatch");
  if (state.feeLamports !== feeLamports) throw new Error("Fee mismatch");
  console.log("-> Test 2 Passed.");

  // Test 3: Register Coordinates Serialization
  console.log("\n[Test 3] Verifying Register Coordinates Serialization...");
  const sessionId = crypto.randomBytes(16);
  const coords = [12, 34, 2, 1, 99, 5];
  const merkleRoot = crypto.createHash("sha256").update("zymatica-consensus").digest();

  const serializedReg = client.serializeRegisterCoordinates(sessionId, coords, merkleRoot);
  console.log(`Serialized Register Instruction Length: ${serializedReg.length} bytes`);
  
  const expectedRegDiscriminator = client.getDiscriminator("global", "register_coordinates");
  const extractedRegDisc = serializedReg.subarray(0, 8);
  if (!extractedRegDisc.equals(expectedRegDiscriminator)) throw new Error("Register coordinates discriminator mismatch");
  if (serializedReg.length !== 62) throw new Error("Invalid length for register instruction");
  console.log("-> Test 3 Passed.");

  // Test 4: Account Deserialization
  console.log("\n[Test 4] Verifying Account CoordinateRecord Deserialization...");
  const expectedRecordDiscriminator = client.getDiscriminator("account", "CoordinateRecord");
  const timestamp = Math.floor(Date.now() / 1000);
  const timestampBuffer = Buffer.alloc(8);
  timestampBuffer.writeBigInt64LE(BigInt(timestamp));
  const bump = 255;

  const mockRecordData = Buffer.concat([
    expectedRecordDiscriminator,
    admin.toBuffer(), // Using admin as authority
    sessionId,
    Buffer.from(coords),
    merkleRoot,
    timestampBuffer,
    Buffer.from([bump])
  ]);

  const record = client.deserializeRecord(mockRecordData);
  if (!record.authority.toBuffer().equals(admin.toBuffer())) throw new Error("Authority mismatch");
  if (!record.sessionId.equals(sessionId)) throw new Error("Session ID mismatch");
  if (record.coords.join(",") !== coords.join(",")) throw new Error("Coords mismatch");
  if (!record.merkleRoot.equals(merkleRoot)) throw new Error("Merkle root mismatch");
  if (record.timestamp !== timestamp) throw new Error("Timestamp mismatch");
  if (record.bump !== bump) throw new Error("Bump mismatch");
  console.log("-> Test 4 Passed.");

  console.log("\n--- All 4 Standalone tests passed successfully with fee layout verification! ---");
}

runStandaloneTests();
