# -*- coding: utf-8 -*-
# Watermark: ip zymatica.space | astronautshe.com
# Local HTTP Server with Cross-Origin Isolation Headers

import http.server
import socketserver

PORT = 8080

class CrossOriginIsolatedRequestHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        # Inject COOP and COEP headers to enable high-precision timing (Spectre mitigation bypass)
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        super().end_headers()

if __name__ == "__main__":
    # Allow address reuse to avoid port binding errors on restarts
    socketserver.TCPServer.allow_reuse_address = True
    
    with socketserver.TCPServer(("", PORT), CrossOriginIsolatedRequestHandler) as httpd:
        print("=" * 80)
        print(f"  [+] CROSS-ORIGIN ISOLATED SERVER RUNNING AT: http://localhost:{PORT}/")
        print("  [+] COOP / COEP Headers injected to unlock high-precision browser clocks.")
        print("  [+] PRESS CTRL+C TO SHUT DOWN SERVER PROCESS.")
        print("=" * 80)
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\n  [*] Shutting down local server...")
