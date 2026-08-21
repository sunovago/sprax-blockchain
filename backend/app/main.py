from contextlib import asynccontextmanager
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from fastapi.middleware.trustedhost import TrustedHostMiddleware
from slowapi import _rate_limit_exceeded_handler
from slowapi.errors import RateLimitExceeded
import asyncio
import uuid

from app.core.config import settings
from app.core.logging import configure_logging
from app.core.exceptions import (
    SpraxError, sprax_exception_handler,
    http_exception_handler, unhandled_exception_handler,
)
from app.core.rate_limit import limiter, rate_limit_exceeded_handler
from app.api.router import api_router
from app.websocket.routes import ws_router
from fastapi import HTTPException
from starlette.requests import Request
from starlette.responses import JSONResponse

configure_logging()


@asynccontextmanager
async def lifespan(app: FastAPI):
    from app.websocket.manager import get_ws_manager
    manager = get_ws_manager()
    # Start heartbeat task
    heartbeat_task = asyncio.create_task(manager.heartbeat_loop())
    yield
    heartbeat_task.cancel()
    try:
        await heartbeat_task
    except asyncio.CancelledError:
        pass


app = FastAPI(
    title="SPRX Backend API",
    description="SPRX Protocol Backend — Blockchain, Markets, Explorer, Portfolio, Validators, Perps",
    version="1.0.0",
    docs_url="/docs" if settings.debug else None,
    redoc_url="/redoc" if settings.debug else None,
    lifespan=lifespan,
)

# CORS
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.cors_origins_list,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Rate limiting
app.state.limiter = limiter
app.add_exception_handler(RateLimitExceeded, rate_limit_exceeded_handler)

# Exception handlers
app.add_exception_handler(SpraxError, sprax_exception_handler)
app.add_exception_handler(HTTPException, http_exception_handler)
app.add_exception_handler(Exception, unhandled_exception_handler)

# Request ID middleware
@app.middleware("http")
async def add_request_id(request: Request, call_next):
    request_id = str(uuid.uuid4())
    request.state.request_id = request_id
    response = await call_next(request)
    response.headers["X-Request-ID"] = request_id
    return response

# Routes
app.include_router(api_router)
app.include_router(ws_router)

# Root health
@app.get("/health")
async def health_root():
    return {"status": "ok", "service": "sprax-backend", "version": "1.0.0"}


@app.get("/ready")
async def ready_root():
    from app.api.v1.health import ready
    return await ready()
