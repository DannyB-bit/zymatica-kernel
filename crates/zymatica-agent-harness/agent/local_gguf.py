"""
Local GGUF LLM Inference Engine using llama_cpp with CUDA acceleration.
"""
import os
import glob
import logging

_LLAMA_INSTANCES = {}

def find_local_gguf_model(model_name_or_dir: str) -> str:
    """Find a GGUF model file on disk corresponding to the model slug or directory."""
    search_paths = [
        os.path.join(os.getcwd(), "models", model_name_or_dir),
        os.path.join(os.getcwd(), "models"),
        os.path.expanduser("~/.cache/huggingface"),
        os.path.expanduser("~/.ollama/models"),
    ]
    
    for base in search_paths:
        if os.path.isfile(base) and base.endswith(".gguf"):
            return base
        if os.path.isdir(base):
            ggufs = glob.glob(os.path.join(base, "**", "*.gguf"), recursive=True)
            if ggufs:
                return ggufs[0]
    return ""

def generate_local_gguf_completion(messages: list, model_name: str = "gemma-4-e4b-compressed", max_tokens: int = 1024) -> str:
    """Execute real local GGUF model inference via llama_cpp."""
    try:
        import llama_cpp
    except ImportError:
        logging.warning("llama_cpp module not installed.")
        return ""

    gguf_path = find_local_gguf_model(model_name)
    if not gguf_path:
        return ""

    if gguf_path not in _LLAMA_INSTANCES:
        logging.info("Loading GGUF model from %s...", gguf_path)
        try:
            _LLAMA_INSTANCES[gguf_path] = llama_cpp.Llama(
                model_path=gguf_path,
                n_gpu_layers=-1,  # Offload all layers to CUDA GPU if available
                n_ctx=2048,
                verbose=False
            )
        except Exception as e:
            logging.warning("Failed to initialize llama_cpp with GPU: %s", e)
            try:
                _LLAMA_INSTANCES[gguf_path] = llama_cpp.Llama(
                    model_path=gguf_path,
                    n_gpu_layers=0,  # Fallback to CPU
                    n_ctx=2048,
                    verbose=False
                )
            except Exception as ex:
                logging.error("Failed to load GGUF model on CPU: %s", ex)
                return ""

    llm = _LLAMA_INSTANCES[gguf_path]
    try:
        resp = llm.create_chat_completion(
            messages=messages,
            max_tokens=max_tokens,
            temperature=0.7
        )
        return resp["choices"][0]["message"]["content"]
    except Exception as err:
        logging.error("GGUF inference execution error: %s", err)
        return ""
