import wasmModule from "./zymatica_engine.wasm";
import { createClient, instantiateWasm } from "./browser.mjs";

let clientPromise;

async function client() {
  if (!clientPromise) {
    clientPromise = instantiateWasm(wasmModule).then((instance) => createClient(instance.exports));
  }
  return clientPromise;
}

export default {
  async fetch(request) {
    const engine = await client();
    const url = new URL(request.url);
    if (request.method === "GET" && url.pathname === "/healthz") {
      return Response.json({ status: "ok", runtime: "zymatica-edge-wasm" });
    }
    if (request.method === "POST" && url.pathname === "/mcp") {
      const payload = await request.json();
      return Response.json(engine.call(payload));
    }
    return new Response("not found", { status: 404 });
  },
};
