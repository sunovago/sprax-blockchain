from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select
from app.core.dependencies import get_db, get_current_user
from app.db.models.watchlist import Watchlist, WatchlistAsset
from app.core.response import ApiResponse
import uuid

router = APIRouter()


class AddAssetRequest(BaseModel):
    symbol: str


@router.get("")
async def list_watchlists(user: dict = Depends(get_current_user), db: AsyncSession = Depends(get_db)):
    user_id = uuid.UUID(user["sub"])
    result = await db.execute(select(Watchlist).where(Watchlist.user_id == user_id))
    watchlists = result.scalars().all()
    out = []
    for wl in watchlists:
        assets_r = await db.execute(select(WatchlistAsset).where(WatchlistAsset.watchlist_id == wl.id))
        assets = assets_r.scalars().all()
        out.append({"id": str(wl.id), "name": wl.name, "assets": [a.symbol for a in assets]})
    return ApiResponse.ok(out)


@router.post("")
async def create_watchlist(user: dict = Depends(get_current_user), db: AsyncSession = Depends(get_db)):
    user_id = uuid.UUID(user["sub"])
    wl = Watchlist(user_id=user_id)
    db.add(wl)
    await db.commit()
    return ApiResponse.ok({"id": str(wl.id), "name": wl.name})


@router.post("/assets")
async def add_asset(
    req: AddAssetRequest,
    user: dict = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    user_id = uuid.UUID(user["sub"])
    # Get default watchlist
    result = await db.execute(select(Watchlist).where(Watchlist.user_id == user_id).limit(1))
    wl = result.scalar_one_or_none()
    if not wl:
        wl = Watchlist(user_id=user_id)
        db.add(wl)
        await db.flush()
    asset = WatchlistAsset(watchlist_id=wl.id, symbol=req.symbol.upper())
    db.add(asset)
    await db.commit()
    return ApiResponse.ok({"added": req.symbol.upper()})


@router.delete("/assets/{symbol}")
async def remove_asset(
    symbol: str,
    user: dict = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    user_id = uuid.UUID(user["sub"])
    result = await db.execute(select(Watchlist).where(Watchlist.user_id == user_id).limit(1))
    wl = result.scalar_one_or_none()
    if not wl:
        raise HTTPException(404, "Watchlist not found")
    asset_r = await db.execute(
        select(WatchlistAsset).where(
            WatchlistAsset.watchlist_id == wl.id,
            WatchlistAsset.symbol == symbol.upper(),
        )
    )
    asset = asset_r.scalar_one_or_none()
    if not asset:
        raise HTTPException(404, "Asset not in watchlist")
    await db.delete(asset)
    await db.commit()
    return ApiResponse.ok({"removed": symbol.upper()})
