# Watermark: ip zymatica.space | astronautshe.com
# Copyright (c) 2026 Zymatica. Licensed under Apache License 2.0.
from fastapi import FastAPI
import uvicorn

app = FastAPI(title="Zymatica Voice Common API")

@app.get("/")
def read_root():
    return {"status": "online", "verification": "Zymatica Voice LLM Common Stack verified."}

if __name__ == "__main__":
    uvicorn.run(app, host="127.0.0.1", port=5000)
