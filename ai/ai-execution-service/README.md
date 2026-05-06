# AI Execution Service

FastAPI service used by `execution-engine` for per-venue scores. Uses ONNX Runtime when `model.onnx` is present; otherwise a NumPy fallback.

## Run

```bash
python -m venv .venv
# Windows: .venv\Scripts\activate
source .venv/bin/activate
pip install -r requirements.txt
python main.py
```

Default listen port: **8093** (`PORT` env overrides). Point Rust at it with `AI_EXECUTION_URL` (default `http://127.0.0.1:8093`).

## Train / export ONNX

```bash
pip install -r requirements.txt
python train_export.py
```

See repository root `README.md` and `docs/API.md` for endpoint details.
