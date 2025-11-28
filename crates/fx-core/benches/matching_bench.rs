//! Latency benchmarks for matching engine

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fx_core::{MatchingEngine, Order};
use fx_utils::{OrderType, Price, Quantity, Side};
use std::sync::Arc;
use uuid::Uuid;

fn bench_match_order(c: &mut Criterion) {
    let mut engine = MatchingEngine::new("EURUSD".to_string());

    // Pre-populate orderbook with some orders
    for i in 0..10 {
        let sell_order = Arc::new(Order::new(
            Uuid::new_v4(),
            "EURUSD".to_string(),
            Side::Sell,
            OrderType::Limit,
            Quantity(1_000_000),
            Some(Price(10_850 + i * 10)),
        ));
        engine.orderbook_mut().add_order(sell_order);
    }

    c.bench_function("match_order", |b| {
        b.iter(|| {
            let buy_order = Arc::new(Order::new(
                Uuid::new_v4(),
                "EURUSD".to_string(),
                Side::Buy,
                OrderType::Limit,
                Quantity(black_box(1_000_000)),
                Some(Price(black_box(10_900))),
            ));
            engine.match_order(black_box(buy_order));
        })
    });
}

fn bench_match_market_order(c: &mut Criterion) {
    let mut engine = MatchingEngine::new("EURUSD".to_string());

    // Pre-populate orderbook
    for i in 0..10 {
        let sell_order = Arc::new(Order::new(
            Uuid::new_v4(),
            "EURUSD".to_string(),
            Side::Sell,
            OrderType::Limit,
            Quantity(1_000_000),
            Some(Price(10_850 + i * 10)),
        ));
        engine.orderbook_mut().add_order(sell_order);
    }

    c.bench_function("match_market_order", |b| {
        b.iter(|| {
            let buy_order = Arc::new(Order::new(
                Uuid::new_v4(),
                "EURUSD".to_string(),
                Side::Buy,
                OrderType::Market,
                Quantity(black_box(1_000_000)),
                None,
            ));
            engine.match_order(black_box(buy_order));
        })
    });
}

criterion_group!(benches, bench_match_order, bench_match_market_order);
criterion_main!(benches);
