# AI Execution Service (optional / offline training)

Venue scoring for the execution pipeline is **embedded in Rust** by default
(`fx-ai-execution` in-process logistic scorer). No Python venv is required for
`npm run dev:stack` or production execution.

This FastAPI app remains for:

- Training / exporting `model.onnx` (`train_export.py`)
- Optional remote inference when you set:
  - `AI_EXECUTION_MODE=http`
  - `AI_EXECUTION_URL=http://127.0.0.1:8093`
  - and for local stack: `DEV_STACK_WITH_AI=1`

## Run (optional remote)

```bash
python -m venv .venv
# Windows: .venv\Scripts\activate
source .venv/bin/activate
pip install -r requirements.txt
python main.py
```

Default listen port: **8093** (`PORT` env overrides).

## Train / export ONNX

```bash
pip install -r requirements.txt
python train_export.py
```

See repository root `README.md` and `docs/API.md` for endpoint details.
