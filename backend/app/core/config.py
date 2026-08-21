from functools import lru_cache
from typing import Literal
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        case_sensitive=False,
        extra="ignore",
    )

    # App
    app_env: Literal["local", "testnet", "production"] = "local"
    app_name: str = "SPRX Backend"
    secret_key: str = "change-this-in-production"
    debug: bool = False

    # Database
    database_url: str = "postgresql+asyncpg://sprax:sprax@localhost:5432/sprax"
    database_url_sync: str = "postgresql://sprax:sprax@localhost:5432/sprax"

    # Redis
    redis_url: str = "redis://localhost:6379/0"

    # Sprax RPC
    sprax_rpc_url: str = "http://localhost:26657"
    sprax_chain_id: str = "sprax-testnet-1"
    sprax_decimals: int = 18

    # Market
    market_provider: str = "coingecko"
    coingecko_api_key: str = ""
    coingecko_base_url: str = "https://api.coingecko.com/api/v3"

    # FX
    fx_provider: str = "exchangerate"
    fx_api_key: str = ""
    fx_base_url: str = "https://v6.exchangerate-api.com/v6"

    # CORS
    cors_origins: str = "http://localhost:3000,http://localhost:5173"

    @property
    def cors_origins_list(self) -> list[str]:
        return [o.strip() for o in self.cors_origins.split(",")]

    # Rate Limits
    rate_limit_public: str = "60/minute"
    rate_limit_search: str = "30/minute"
    rate_limit_auth: str = "10/minute"
    rate_limit_write: str = "20/minute"

    # JWT
    jwt_algorithm: str = "HS256"
    jwt_access_token_expire_minutes: int = 60
    jwt_refresh_token_expire_days: int = 30

    # Celery
    celery_broker_url: str = "redis://localhost:6379/1"
    celery_result_backend: str = "redis://localhost:6379/2"

    # Indexer
    indexer_poll_interval_seconds: int = 2
    indexer_batch_size: int = 10

    # Feature Flags
    feature_perps_enabled: bool = False
    feature_mainnet_enabled: bool = False
    feature_price_alerts_enabled: bool = True
    feature_staking_enabled: bool = True
    feature_markets_enabled: bool = True
    feature_discover_enabled: bool = True

    # Chain constants
    sprax_address_prefix: str = "sprax1"
    sprax_valoper_prefix: str = "spraxvaloper1"
    sprax_atto_per_sprx: int = 10**18
    sprax_default_fee_atto: str = "1000000000000000"  # 0.001 SPRX
    sprax_default_gas: int = 200_000


@lru_cache
def get_settings() -> Settings:
    return Settings()


settings = get_settings()
