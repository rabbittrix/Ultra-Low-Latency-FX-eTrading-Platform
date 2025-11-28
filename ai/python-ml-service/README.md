# Python ML Service

Machine Learning service for volatility prediction in the FX eTrading Platform.

## Setup

1. Create a virtual environment:

```bash
python -m venv venv
```

1. Activate the virtual environment:

```bash
# On Windows
venv\Scripts\activate

# On Linux/Mac
source venv/bin/activate
```

1. Install dependencies:

```bash
pip install -r requirements.txt
```

## Running

```bash
python main.py
```

Or with uvicorn directly:

```bash
uvicorn main:app --host 0.0.0.0 --port 8086
```

## Author

**Roberto de Souza**  
**Email:** <rabbittrix@hotmail.com>  
**License:** Apache-2.0
