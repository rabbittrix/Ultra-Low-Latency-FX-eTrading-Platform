"""
AI-driven predictive execution service.

- Primary: ONNX Runtime inference when ``model.onnx`` is present (train via ``train_export.py``).
- Fallback: calibrated logistic-style scoring in NumPy (no external model file).
"""

from __future__ import annotations

import os
from pathlib import Path
from typing import Any

import numpy as np
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field

MODEL_PATH = Path(__file__).resolve().parent / "model.onnx"
_ort_session = None


def _load_onnx() -> Any | None:
    global _ort_session
    if _ort_session is not None:
        return _ort_session
    if not MODEL_PATH.is_file():
        return None
    try:
        import onnxruntime as ort  # type: ignore

        _ort_session = ort.InferenceSession(
            str(MODEL_PATH), providers=["CPUExecutionProvider"]
        )
    except Exception:
        _ort_session = None
    return _ort_session


class VenueFeatures(BaseModel):
    venue_id: str
    spread_bps: float = 0.0
    depth: float = 0.0
    recent_reject_rate: float = 0.0
    latency_ewma_us: float = 0.0
    toxicity_hint: float = 0.0
    mid_move_bps: float = 0.0


class InferRequest(BaseModel):
    instrument: str
    side: str
    quantity: float = Field(gt=0)
    venues: list[VenueFeatures]


class VenueScore(BaseModel):
    venue_id: str
    fill_probability: float
    expected_latency_us: float
    rejection_likelihood: float
    market_impact_bps: float
    score: float


class ExecutionRecommendation(BaseModel):
    ranked_venue_ids: list[str]
    notes: str


class InferResponse(BaseModel):
    venues: list[VenueScore]
    recommendation: ExecutionRecommendation


def _features_matrix(req: InferRequest) -> np.ndarray:
    rows = []
    for v in req.venues:
        x = [
            v.spread_bps / 50.0,
            np.log1p(max(v.depth, 0.0)) / 20.0,
            v.recent_reject_rate,
            v.latency_ewma_us / 1000.0,
            v.toxicity_hint,
            abs(v.mid_move_bps) / 10.0,
            np.log1p(req.quantity) / 25.0,
        ]
        rows.append(x)
    return np.asarray(rows, dtype=np.float32)


def _fallback_scores(req: InferRequest) -> list[VenueScore]:
    X = _features_matrix(req)
    w = np.array(
        [-0.9, 0.25, -1.15, -0.45, -0.95, -0.25, -0.35], dtype=np.float32
    )
    out: list[VenueScore] = []
    for i, v in enumerate(req.venues):
        z = float(w @ X[i] + 0.2)
        fill = float(1.0 / (1.0 + np.exp(-z)))
        fill = max(0.05, min(0.99, fill))
        rej = max(0.0, min(1.0, 1.0 - fill))
        impact = float(max(0.0, v.spread_bps * 0.35 + v.toxicity_hint * 8.0))
        lat = float(max(20.0, v.latency_ewma_us * (1.0 + rej)))
        score = float(fill * 1000.0 - lat - impact * 50.0 - rej * 200.0)
        out.append(
            VenueScore(
                venue_id=v.venue_id,
                fill_probability=fill,
                expected_latency_us=lat,
                rejection_likelihood=rej,
                market_impact_bps=impact,
                score=score,
            )
        )
    return out


def _onnx_scores(req: InferRequest) -> list[VenueScore] | None:
    sess = _load_onnx()
    if sess is None:
        return None
    X = _features_matrix(req)
    input_name = sess.get_inputs()[0].name
    outputs = sess.run(None, {input_name: X})
    prob = None
    for arr in outputs:
        if not isinstance(arr, np.ndarray):
            continue
        if arr.ndim == 2 and arr.shape[1] >= 2 and np.issubdtype(arr.dtype, np.floating):
            prob = np.asarray(arr[:, 1], dtype=np.float64)
            break
        if arr.ndim == 1 and arr.shape[0] == X.shape[0] and np.issubdtype(
            arr.dtype, np.floating
        ):
            prob = np.asarray(arr, dtype=np.float64)
            break
    if prob is None:
        return None
    prob_vec = prob.reshape(-1)
    scores: list[VenueScore] = []
    for i, v in enumerate(req.venues):
        raw = float(prob_vec[i])
        fill = max(0.05, min(0.99, raw))
        rej = max(0.0, min(1.0, 1.0 - fill))
        impact = float(max(0.0, v.spread_bps * 0.35 + v.toxicity_hint * 8.0))
        lat = float(max(20.0, v.latency_ewma_us * (1.0 + rej)))
        score = float(fill * 1000.0 - lat - impact * 50.0 - rej * 200.0)
        scores.append(
            VenueScore(
                venue_id=v.venue_id,
                fill_probability=fill,
                expected_latency_us=lat,
                rejection_likelihood=rej,
                market_impact_bps=impact,
                score=score,
            )
        )
    return scores


app = FastAPI(title="AI Execution Service", version="0.1.0")
app.add_middleware(
    CORSMiddleware,
    allow_origins=os.environ.get("CORS_ORIGINS", "*").split(","),
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/health")
def health() -> dict[str, str]:
    return {"status": "ok", "onnx": str(MODEL_PATH.is_file())}


@app.post("/v1/infer", response_model=InferResponse)
def infer(req: InferRequest) -> InferResponse:
    venues = _onnx_scores(req) or _fallback_scores(req)
    ranked = sorted(venues, key=lambda x: x.score, reverse=True)
    ids = [v.venue_id for v in ranked]
    mode = "onnx" if _load_onnx() else "numpy_fallback"
    return InferResponse(
        venues=ranked,
        recommendation=ExecutionRecommendation(
            ranked_venue_ids=ids,
            notes=f"model={mode}; ranked_by=score",
        ),
    )


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(
        "main:app",
        host=os.environ.get("HOST", "0.0.0.0"),
        port=int(os.environ.get("PORT", "8093")),
        reload=False,
    )
