import urllib.request
import json
import threading
import time
import os
import sys

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(errors="backslashreplace")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(errors="backslashreplace")

# Read API key if configured
api_key = os.environ.get("ZYMATICA_API_KEY", "")

# Base configuration
URL = "http://127.0.0.1:3000/v1/chat/completions"
HEADERS = {
    "Content-Type": "application/json",
}
if api_key:
    HEADERS["Authorization"] = f"Bearer {api_key}"

# Prompts that share a common prefix to stress-test radix KV cache reuse
SHARED_PREFIX = "Context information: Zymatica Engine is a fast C++/Rust runtime for edge devices. "
PROMPTS = [
    SHARED_PREFIX + "How fast is it?",
    SHARED_PREFIX + "Does it support parallel decode?",
    SHARED_PREFIX + "What architectures does it support?",
    SHARED_PREFIX + "What quantization formats are supported?",
    SHARED_PREFIX + "Can it run on a Raspberry Pi 4?",
    SHARED_PREFIX + "How does copy-on-write keep kv cache isolated?",
    SHARED_PREFIX + "Tell me about speculative decoding.",
    SHARED_PREFIX + "What is the memory footprint of Q4?",
    SHARED_PREFIX + "How does radix KV cache share pages?",
    SHARED_PREFIX + "List whitelisted command lists for Field Agent RAG mode."
]

latencies = []
success_count = 0
cancel_count = 0
errors = []

lock = threading.Lock()

def send_request(thread_id, prompt, should_cancel):
    global success_count, cancel_count
    payload = {
        "model": "zymatica-q4",
        "messages": [
            {"role": "user", "content": prompt}
        ],
        "max_tokens": 32,
        "temperature": 0.0,
        "stream": True
    }
    
    start_time = time.time()
    try:
        req = urllib.request.Request(
            URL, 
            data=json.dumps(payload).encode("utf-8"), 
            headers=HEADERS,
            method="POST"
        )
        
        # Open connection
        with urllib.request.urlopen(req, timeout=10) as response:
            first_token_time = None
            bytes_read = 0
            
            # Read streaming response line by line
            for line in response:
                if not line:
                    continue
                decoded_line = line.decode("utf-8").strip()
                print(f"[Thread {thread_id}] Raw line: {decoded_line}")
                if not decoded_line:
                    continue
                if first_token_time is None:
                    first_token_time = time.time() - start_time
                
                # If we want to simulate client cancellation, close the response after some tokens
                bytes_read += len(line)
                if should_cancel and bytes_read > 10:
                    # Cancel the request by closing the connection abruptly
                    response.close()
                    with lock:
                        cancel_count += 1
                    print(f"[Thread {thread_id}] Abruptly cancelled request (reclaiming COW cache pages)...")
                    return
            
            total_time = time.time() - start_time
            with lock:
                success_count += 1
                latencies.append({
                    "thread_id": thread_id,
                    "ttft": first_token_time,
                    "total_time": total_time
                })
            ttft_str = f"{first_token_time:.3f}s" if first_token_time is not None else "N/A"
            print(f"[Thread {thread_id}] Finished successfully in {total_time:.3f}s (ttft={ttft_str}).")
            
    except Exception as e:
        # Ignore errors from intentional cancellation
        if should_cancel:
            with lock:
                cancel_count += 1
            print(f"[Thread {thread_id}] Abruptly cancelled request (reclaiming COW cache pages)...")
            return
        with lock:
            errors.append(f"Thread {thread_id} error: {e}")
        print(f"[Thread {thread_id}] Error: {e}", file=sys.stderr)

def main():
    print("=== Starting Zymatica Engine Server Concurrency & Cancellation Stress Test ===")
    print(f"Target URL: {URL}")
    print(f"Auth token active: {bool(api_key)}")
    
    threads = []
    for i, prompt in enumerate(PROMPTS):
        # Cancel request for threads index 2, 5, 8 to stress cancellation
        should_cancel = i in [2, 5, 8]
        t = threading.Thread(target=send_request, args=(i, prompt, should_cancel))
        threads.append(t)
        t.start()
        
    for t in threads:
        t.join()
        
    print("\n=== Stress Test Results ===")
    print(f"Total concurrent requests: {len(PROMPTS)}")
    print(f"Successful requests completed: {success_count}")
    print(f"Intentionally cancelled requests: {cancel_count}")
    print(f"Failed/Error requests: {len(errors)}")
    
    if errors:
        print("\nErrors encountered:")
        for err in errors:
            print(f" - {err}")
            
    if latencies:
        valid_ttfts = [l["ttft"] for l in latencies if l["ttft"] is not None]
        avg_ttft_str = f"{sum(valid_ttfts) / len(valid_ttfts):.3f}s" if valid_ttfts else "N/A"
        avg_total = sum(l["total_time"] for l in latencies) / len(latencies)
        print(f"Average Time To First Token (TTFT): {avg_ttft_str}")
        print(f"Average Total Duration: {avg_total:.3f}s")
        
    if len(errors) > 0:
        sys.exit(1)
    else:
        print("\n[OK] Concurrency stress test passed successfully!")

if __name__ == "__main__":
    main()
