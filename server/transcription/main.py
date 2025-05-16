from loguru import logger
logger.info("import sys"); import sys
logger.info("import asyncio"); import asyncio
logger.info("from aiohttp import web"); from aiohttp import web
logger.info("import os"); import os
logger.info("from typing import Optional"); from typing import Optional
logger.info("import torch"); import torch
logger.info("import whisperx"); import whisperx
logger.info("from whisperx.asr import FasterWhisperPipeline"); from whisperx.asr import FasterWhisperPipeline

APP_VERSION = "1.0.0"

# Model states
IDLE_MODEL_LOADED = "IDLE_MODEL_LOADED"
IDLE_MODEL_UNLOADED = "IDLE_MODEL_UNLOADED"
TRANSCRIBING = "TRANSCRIBING"

class AppState:
    def __init__(self):
        self.model: Optional[FasterWhisperPipeline] = None
        self.model_state = IDLE_MODEL_UNLOADED
        self.transcribe_path: Optional[str] = None
        self.model_name = "large-v2" if torch.cuda.is_available() else "small.en"
        self.device = "cuda" if torch.cuda.is_available() else "cpu"
        self.language = "en"
        self.transcribe_task: Optional[asyncio.Task] = None

    def status(self):
        if self.model_state == TRANSCRIBING:
            return {"model_state": f"{TRANSCRIBING}({self.transcribe_path})"}
        return {"model_state": self.model_state}

state = AppState()

def assert_correct():
    def decorator(handler):
        async def wrapper(request):
            try:
                if state.model is None:
                    assert state.model_state == IDLE_MODEL_UNLOADED, f"Model is not unloaded, but state is {state.model_state}"
                if state.model is not None:
                    assert state.model_state == IDLE_MODEL_LOADED, f"Model is not loaded, but state is {state.model_state}"
            except AssertionError as e:
                logger.error(f"Assertion failed: {e}")
                return web.json_response({"error": str(e)}, status=500)
            return await handler(request)
        return wrapper
    return decorator

def require_auth(api_key):
    def decorator(handler):
        async def wrapper(request):
            if request.headers.get("Authorization") != api_key:
                logger.warning("Unauthorized access attempt")
                return web.json_response({"error": "Unauthorized"}, status=401)
            return await handler(request)
        return wrapper
    return decorator

async def handle_version(request):
    logger.info("Handling version request")
    return web.json_response({"version": APP_VERSION})

async def handle_status(request):
    logger.info("Handling status request")
    return web.json_response(state.status())

async def handle_load_model(request):
    logger.info("Handling load model request")
    if state.model is not None:
        logger.info("Model already loaded")
        return web.json_response({"status": "already loaded"})
    state.model = whisperx.load_model(
        state.model_name, device=state.device, language=state.language
    )
    state.model_state = IDLE_MODEL_LOADED
    logger.info(f"Model loaded: {state.model_name}")
    return web.json_response({"status": "model loaded"})

async def handle_unload_model(request):
    logger.info("Handling unload model request")
    if state.model is None:
        logger.info("Model already unloaded")
        return web.json_response({"status": "already unloaded"})
    state.model = None
    state.model_state = IDLE_MODEL_UNLOADED
    logger.info("Model unloaded")
    return web.json_response({"status": "model unloaded"})

async def handle_start_transcribe(request):
    logger.info("Handling start transcribe request")
    model = state.model
    if model is None:
        logger.error("Model not loaded")
        return web.json_response({"error": "model not loaded"}, status=400)
    if state.model_state == TRANSCRIBING:
        logger.warning("Already transcribing")
        return web.json_response({"error": "already transcribing"}, status=400)
    data = await request.json()
    path = data.get("path")
    if not path or not os.path.exists(path):
        logger.error(f"Invalid path: {path}")
        return web.json_response({"error": "invalid path"}, status=400)
    state.model_state = TRANSCRIBING
    state.transcribe_path = path
    logger.info(f"Transcription started for path: {path}")

    async def transcribe(): 
        try:
            result = model.transcribe(path)
            logger.info(f"Transcription completed for path: {path}")
            # Save or process result as needed
        except Exception as e:
            logger.exception(f"Transcription failed for path {path}: {e}")
        finally:
            state.model_state = IDLE_MODEL_LOADED
            state.transcribe_path = None

    state.transcribe_task = asyncio.create_task(transcribe())
    return web.json_response({"status": "transcription started"})

async def handle_stop(request):
    logger.info("Handling stop request")
    # Graceful shutdown
    await request.app.shutdown()
    await request.app.cleanup()
    # Stop the event loop after a short delay to allow response to be sent
    asyncio.get_event_loop().call_later(0.5, asyncio.get_event_loop().stop)
    logger.info("Shutting down")
    return web.json_response({"status": "shutting down"})

def make_app(api_key):
    app = web.Application()
    app.add_routes([
        web.get("/version", assert_correct()(require_auth(api_key)(handle_version))),
        web.get("/status", assert_correct()(require_auth(api_key)(handle_status))),
        web.post("/load_model", assert_correct()(require_auth(api_key)(handle_load_model))),
        web.post("/unload_model", assert_correct()(require_auth(api_key)(handle_unload_model))),
        web.post("/start_transcribe", assert_correct()(require_auth(api_key)(handle_start_transcribe))),
        web.post("/stop", assert_correct()(require_auth(api_key)(handle_stop))),
    ])
    return app

def main():
    if len(sys.argv) < 3:
        print("Usage: python app.py <PORT> <API_KEY>")
        sys.exit(1)
    try:
        port = int(sys.argv[1])
    except ValueError:
        print("Error: Port must be an integer.")
        sys.exit(1)
    api_key = sys.argv[2]
    logger.info("Starting app")
    app = make_app(api_key)
    web.run_app(app, port=port)
    logger.info("App stopped")

if __name__ == "__main__":
    main()
