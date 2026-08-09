//! Parquet tick writer/reader.

use crate::traits::TickStore;
use arrow::array::{Int64Array, StringArray, UInt8Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use fx_smc_common::{Px, Qty, Side, SmcError, SymbolId, Tick, TsNanos};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Filesystem Parquet backend (`{root}/{dataset}.parquet`).
#[derive(Debug, Clone)]
pub struct ParquetTickStore {
    root: PathBuf,
}

impl ParquetTickStore {
    /// Create a store rooted at `root` (created if missing).
    ///
    /// # Errors
    /// Returns [`SmcError::Io`] if the directory cannot be created.
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, SmcError> {
        let root = root.into();
        std::fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    fn path_for(&self, dataset: &str) -> PathBuf {
        let safe: String = dataset
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        self.root.join(format!("{safe}.parquet"))
    }

    fn schema() -> Schema {
        Schema::new(vec![
            Field::new("symbol", DataType::Utf8, false),
            Field::new("ts_ns", DataType::Int64, false),
            Field::new("bid", DataType::Int64, false),
            Field::new("ask", DataType::Int64, false),
            Field::new("bid_qty", DataType::Int64, false),
            Field::new("ask_qty", DataType::Int64, false),
            Field::new("aggressor", DataType::UInt8, false),
        ])
    }

    fn aggressor_code(side: Option<Side>) -> u8 {
        match side {
            None => 0,
            Some(Side::Buy) => 1,
            Some(Side::Sell) => 2,
        }
    }

    fn aggressor_from_code(code: u8) -> Result<Option<Side>, SmcError> {
        match code {
            0 => Ok(None),
            1 => Ok(Some(Side::Buy)),
            2 => Ok(Some(Side::Sell)),
            other => Err(SmcError::InvalidData(format!(
                "unknown aggressor code {other}"
            ))),
        }
    }
}

impl TickStore for ParquetTickStore {
    fn write_ticks(&self, dataset: &str, ticks: &[Tick]) -> Result<(), SmcError> {
        let path = self.path_for(dataset);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let n = ticks.len();
        let symbols: Vec<&str> = ticks.iter().map(|t| t.symbol.as_str()).collect();
        let ts: Vec<i64> = ticks.iter().map(|t| t.ts_ns.0).collect();
        let bid: Vec<i64> = ticks.iter().map(|t| t.bid.0).collect();
        let ask: Vec<i64> = ticks.iter().map(|t| t.ask.0).collect();
        let bid_qty: Vec<i64> = ticks.iter().map(|t| t.bid_qty.0).collect();
        let ask_qty: Vec<i64> = ticks.iter().map(|t| t.ask_qty.0).collect();
        let agg: Vec<u8> = ticks
            .iter()
            .map(|t| Self::aggressor_code(t.aggressor))
            .collect();

        let batch = RecordBatch::try_new(
            Arc::new(Self::schema()),
            vec![
                Arc::new(StringArray::from(symbols)),
                Arc::new(Int64Array::from(ts)),
                Arc::new(Int64Array::from(bid)),
                Arc::new(Int64Array::from(ask)),
                Arc::new(Int64Array::from(bid_qty)),
                Arc::new(Int64Array::from(ask_qty)),
                Arc::new(UInt8Array::from(agg)),
            ],
        )
        .map_err(|e| SmcError::Store(format!("record batch: {e}")))?;

        let file = File::create(&path)?;
        let props = WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .build();
        let mut writer = ArrowWriter::try_new(file, Arc::new(Self::schema()), Some(props))
            .map_err(|e| SmcError::Store(format!("parquet writer: {e}")))?;
        writer
            .write(&batch)
            .map_err(|e| SmcError::Store(format!("parquet write: {e}")))?;
        writer
            .close()
            .map_err(|e| SmcError::Store(format!("parquet close: {e}")))?;

        tracing::debug!(?path, rows = n, "wrote parquet ticks");
        let _ = path;
        Ok(())
    }

    fn read_ticks(&self, dataset: &str) -> Result<Vec<Tick>, SmcError> {
        let path = self.path_for(dataset);
        read_ticks_from_path(&path)
    }
}

fn read_ticks_from_path(path: &Path) -> Result<Vec<Tick>, SmcError> {
    let file =
        File::open(path).map_err(|e| SmcError::Store(format!("open {}: {e}", path.display())))?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|e| SmcError::Store(format!("parquet open: {e}")))?;
    let reader = builder
        .build()
        .map_err(|e| SmcError::Store(format!("parquet reader: {e}")))?;

    let mut out = Vec::new();
    for batch_res in reader {
        let batch = batch_res.map_err(|e| SmcError::Store(format!("parquet batch: {e}")))?;
        let symbol = batch
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| SmcError::Store("symbol column".into()))?;
        let ts = batch
            .column(1)
            .as_any()
            .downcast_ref::<Int64Array>()
            .ok_or_else(|| SmcError::Store("ts_ns column".into()))?;
        let bid = batch
            .column(2)
            .as_any()
            .downcast_ref::<Int64Array>()
            .ok_or_else(|| SmcError::Store("bid column".into()))?;
        let ask = batch
            .column(3)
            .as_any()
            .downcast_ref::<Int64Array>()
            .ok_or_else(|| SmcError::Store("ask column".into()))?;
        let bid_qty = batch
            .column(4)
            .as_any()
            .downcast_ref::<Int64Array>()
            .ok_or_else(|| SmcError::Store("bid_qty column".into()))?;
        let ask_qty = batch
            .column(5)
            .as_any()
            .downcast_ref::<Int64Array>()
            .ok_or_else(|| SmcError::Store("ask_qty column".into()))?;
        let agg = batch
            .column(6)
            .as_any()
            .downcast_ref::<UInt8Array>()
            .ok_or_else(|| SmcError::Store("aggressor column".into()))?;

        for i in 0..batch.num_rows() {
            out.push(Tick {
                symbol: SymbolId::new(symbol.value(i)),
                ts_ns: TsNanos(ts.value(i)),
                bid: Px(bid.value(i)),
                ask: Px(ask.value(i)),
                bid_qty: Qty(bid_qty.value(i)),
                ask_qty: Qty(ask_qty.value(i)),
                aggressor: ParquetTickStore::aggressor_from_code(agg.value(i))?,
            });
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fx_smc_common::{Px, Qty, SymbolId, Tick, TsNanos};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn round_trip_parquet() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("smc_parquet_{nanos}"));
        let store = ParquetTickStore::new(&dir).expect("store");
        let ticks = vec![
            Tick {
                symbol: SymbolId::new("EURUSD"),
                ts_ns: TsNanos(10),
                bid: Px(100),
                ask: Px(101),
                bid_qty: Qty(1),
                ask_qty: Qty(2),
                aggressor: Some(Side::Buy),
            },
            Tick {
                symbol: SymbolId::new("EURUSD"),
                ts_ns: TsNanos(20),
                bid: Px(99),
                ask: Px(100),
                bid_qty: Qty(3),
                ask_qty: Qty(4),
                aggressor: None,
            },
        ];
        store.write_ticks("demo", &ticks).expect("write");
        let back = store.read_ticks("demo").expect("read");
        assert_eq!(back, ticks);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
