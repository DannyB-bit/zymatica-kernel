import argparse
import hashlib

# Protocol Constants from compress_chirp3.py
SYNC_MARKER    = 0xBB
PKT_SIZE       = 255
TRANSPORT_HDR  = 3
DATA_PER_PKT   = PKT_SIZE - TRANSPORT_HDR  # 252 Bytes

def xor_fec_parity(data_packets):
    """Computes XOR parity byte-by-byte across all data packets."""
    parity = bytearray(DATA_PER_PKT)
    for pkt in data_packets:
        # Extract data segment (excluding transport header)
        data_part = pkt[TRANSPORT_HDR:]
        for idx in range(min(len(data_part), DATA_PER_PKT)):
            parity[idx] ^= data_part[idx]
    return bytes(parity)

def pack_payload(payload_bytes, num_data_packets):
    """Encapsulates payload into N-1 data packets and 1 XOR-FEC parity packet."""
    total_capacity = num_data_packets * DATA_PER_PKT
    
    # Pad payload if it's smaller than the capacity
    if len(payload_bytes) < total_capacity:
        payload_bytes = payload_bytes.ljust(total_capacity, b'\x00')
    elif len(payload_bytes) > total_capacity:
        payload_bytes = payload_bytes[:total_capacity]
        
    data_packets = []
    total_packets = num_data_packets + 1
    
    for idx in range(num_data_packets):
        chunk = payload_bytes[idx * DATA_PER_PKT : (idx + 1) * DATA_PER_PKT]
        header = bytes([SYNC_MARKER, idx, total_packets])
        data_packets.append(header + chunk)
        
    # Generate XOR-parity packet
    parity_data = xor_fec_parity(data_packets)
    parity_header = bytes([SYNC_MARKER, num_data_packets, total_packets])
    parity_packet = parity_header + parity_data
    
    return data_packets + [parity_packet]

def run_proof():
    print("======================================================================")
    print("ZYMATICA | Chirp Packetization & XOR-FEC Transmission Channel Proof")
    print("======================================================================\n")

    # 1. Prepare raw payload
    raw_payload = b"ip zymatica.space | " * 50  # 1000 bytes payload
    payload_hash = hashlib.sha256(raw_payload).hexdigest()
    print(f"[1] Source Payload Prepared:")
    print(f"  - Size: {len(raw_payload)} bytes")
    print(f"  - SHA-256 Checksum: {payload_hash}")

    # 2. Pack payload into chirps
    num_data_pkts = 4
    packets = pack_payload(raw_payload, num_data_pkts)
    print(f"\n[2] Packaging Payload into {len(packets)} LoRa Chirp-3 Packets:")
    for idx, pkt in enumerate(packets):
        ptype = "DATA" if idx < num_data_pkts else "FEC-PARITY"
        print(f"  - Packet {idx}: Sync=0x{pkt[0]:02X}, Idx={pkt[1]}, Total={pkt[2]}, Size={len(pkt)} bytes ({ptype})")

    # 3. Simulate transmission with exactly one lost packet (Packet index 2 is dropped)
    dropped_index = 2
    print(f"\n[3] Simulating Lossy Channel Transmission...")
    print(f"  -> WARNING: Packet index {dropped_index} dropped during transit.")
    
    received_packets = [pkt for idx, pkt in enumerate(packets) if idx != dropped_index]
    
    # 4. Perform XOR-FEC Recovery on the receiver
    print(f"\n[4] Executing Forward Error Correction (XOR-FEC) Reassembler...")
    
    # Identify which packet is missing
    received_indices = {pkt[1] for pkt in received_packets}
    total_packets = received_packets[0][2]
    missing_index = None
    for idx in range(total_packets):
        if idx not in received_indices:
            missing_index = idx
            break
            
    print(f"  -> Detected missing packet index: {missing_index}")
    
    # Recover missing packet by XORing all received packets' payloads
    recovered_data = bytearray(DATA_PER_PKT)
    for pkt in received_packets:
        data_part = pkt[TRANSPORT_HDR:]
        for idx in range(DATA_PER_PKT):
            recovered_data[idx] ^= data_part[idx]
            
    recovered_packet = bytes([SYNC_MARKER, missing_index, total_packets]) + bytes(recovered_data)
    print(f"  -> Packet index {missing_index} reconstructed successfully.")

    # Insert recovered packet back into the buffer
    all_reconstructed_packets = list(received_packets)
    all_reconstructed_packets.append(recovered_packet)
    # Sort by packet index (byte at offset 1)
    all_reconstructed_packets.sort(key=lambda x: x[1])

    # 5. Reassemble and verify payload
    reassembled_payload = bytearray()
    for idx in range(num_data_pkts):
        reassembled_payload.extend(all_reconstructed_packets[idx][TRANSPORT_HDR:])
        
    # Trim padding if necessary to match original length
    reassembled_payload = bytes(reassembled_payload[:len(raw_payload)])
    reassembled_hash = hashlib.sha256(reassembled_payload).hexdigest()
    
    print(f"\n[5] Reassembled Payload Checksum Verification:")
    print(f"  - Original SHA-256:    {payload_hash}")
    print(f"  - Reassembled SHA-256: {reassembled_hash}")
    
    assert payload_hash == reassembled_hash, "Checksum validation failed! Data corrupted."
    print("\n[VERIFICATION] Lossless XOR-FEC reconstruction validated. No data loss.")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Zymatica LoRa FEC Proof")
    parser.add_argument("--test", action="store_true", help="Run test mode")
    args = parser.parse_args()
    run_proof()
