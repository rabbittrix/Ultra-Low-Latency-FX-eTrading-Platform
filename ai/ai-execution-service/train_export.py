"""
Train a simple logistic model on synthetic execution data and export to ONNX.

Run from this directory:
  python -m venv .venv && .venv\\Scripts\\activate  # Windows
  pip install -r requirements.txt
  python train_export.py

Produces ``model.onnx`` next to ``main.py``.
"""

from __future__ import annotations

from typing import cast

import numpy as np
from onnx import ModelProto
from sklearn.linear_model import LogisticRegression
from sklearn.model_selection import train_test_split
from skl2onnx import convert_sklearn
from skl2onnx.common.data_types import FloatTensorType


def synthetic(n: int = 5000, seed: int = 42) -> tuple[np.ndarray, np.ndarray]:
    rng = np.random.default_rng(seed)
    X = rng.normal(size=(n, 7)).astype(np.float32)
    # Synthetic fill label: liquidity improves with depth, worsens with toxicity/latency/spread
    z = (
        -0.9 * X[:, 0]
        + 0.25 * X[:, 1]
        -1.15 * X[:, 2]
        -0.45 * X[:, 3]
        -0.95 * X[:, 4]
        -0.25 * X[:, 5]
        -0.35 * X[:, 6]
        + 0.2
    )
    p = 1.0 / (1.0 + np.exp(-z))
    y = (rng.random(n) < p).astype(np.int64)
    return X, y


def main() -> None:
    X, y = synthetic()
    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.2, random_state=0
    )
    clf = LogisticRegression(max_iter=200)
    clf.fit(X_train, y_train)
    acc = float((clf.predict(X_test) == y_test).mean())
    print(f"holdout accuracy (synthetic): {acc:.3f}")

    converted = convert_sklearn(
        clf,
        initial_types=[("float_input", FloatTensorType([None, 7]))],
        target_opset=12,
    )
    # skl2onnx>=1.20 returns (ModelProto, Topology); older versions return ModelProto only.
    proto = cast(ModelProto, converted[0] if isinstance(converted, tuple) else converted)
    out_path = __import__("pathlib").Path(__file__).resolve().parent / "model.onnx"
    with open(out_path, "wb") as f:
        f.write(proto.SerializeToString())
    print(f"wrote {out_path}")


if __name__ == "__main__":
    main()
