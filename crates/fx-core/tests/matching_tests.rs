//! Unit tests for matching engine

use fx_core::{MatchingEngine, Order};
use fx_utils::{OrderType, Price, Quantity, Side};
use std::sync::Arc;
use uuid::Uuid;

#[test]
fn test_matching_engine_creation() {
    let engine = MatchingEngine::new("EURUSD".to_string());
    assert_eq!(engine.orderbook().instrument(), "EURUSD");
}

#[test]
fn test_simple_match() {
    let mut engine = MatchingEngine::new("EURUSD".to_string());

    // Add a sell order
    let sell_order = Arc::new(Order::new(
        Uuid::new_v4(),
        "EURUSD".to_string(),
        Side::Sell,
        OrderType::Limit,
        Quantity(1_000_000),
        Some(Price(10_850)), // 1.0850 with 4 decimals
    ));
    engine.orderbook_mut().add_order(sell_order.clone());

    // Add a buy order that matches
    let buy_order = Arc::new(Order::new(
        Uuid::new_v4(),
        "EURUSD".to_string(),
        Side::Buy,
        OrderType::Limit,
        Quantity(1_000_000),
        Some(Price(10_850)),
    ));

    let result = engine.match_order(buy_order);
    assert_eq!(result.trades.len(), 1);
    assert!(result.order.is_filled());
}

#[test]
fn test_partial_match() {
    let mut engine = MatchingEngine::new("EURUSD".to_string());

    // Add a sell order for 500k
    let sell_order = Arc::new(Order::new(
        Uuid::new_v4(),
        "EURUSD".to_string(),
        Side::Sell,
        OrderType::Limit,
        Quantity(500_000),
        Some(Price(10_850)),
    ));
    engine.orderbook_mut().add_order(sell_order.clone());

    // Add a buy order for 1M (should partially fill)
    let buy_order = Arc::new(Order::new(
        Uuid::new_v4(),
        "EURUSD".to_string(),
        Side::Buy,
        OrderType::Limit,
        Quantity(1_000_000),
        Some(Price(10_850)),
    ));

    let result = engine.match_order(buy_order);
    assert_eq!(result.trades.len(), 1);
    assert!(!result.order.is_filled());
    assert_eq!(result.order.remaining_quantity.0, 500_000);
}

#[test]
fn test_no_match_price_too_low() {
    let mut engine = MatchingEngine::new("EURUSD".to_string());

    // Add a sell order at 1.0850
    let sell_order = Arc::new(Order::new(
        Uuid::new_v4(),
        "EURUSD".to_string(),
        Side::Sell,
        OrderType::Limit,
        Quantity(1_000_000),
        Some(Price(10_850)),
    ));
    engine.orderbook_mut().add_order(sell_order.clone());

    // Add a buy order at 1.0840 (too low to match)
    let buy_order = Arc::new(Order::new(
        Uuid::new_v4(),
        "EURUSD".to_string(),
        Side::Buy,
        OrderType::Limit,
        Quantity(1_000_000),
        Some(Price(10_840)),
    ));

    let result = engine.match_order(buy_order);
    assert_eq!(result.trades.len(), 0);
    assert!(!result.order.is_filled());
}

#[test]
fn test_market_order_matches() {
    let mut engine = MatchingEngine::new("EURUSD".to_string());

    // Add a sell order
    let sell_order = Arc::new(Order::new(
        Uuid::new_v4(),
        "EURUSD".to_string(),
        Side::Sell,
        OrderType::Limit,
        Quantity(1_000_000),
        Some(Price(10_850)),
    ));
    engine.orderbook_mut().add_order(sell_order.clone());

    // Add a market buy order (no price limit)
    let buy_order = Arc::new(Order::new(
        Uuid::new_v4(),
        "EURUSD".to_string(),
        Side::Buy,
        OrderType::Market,
        Quantity(1_000_000),
        None,
    ));

    let result = engine.match_order(buy_order);
    assert_eq!(result.trades.len(), 1);
    assert!(result.order.is_filled());
}

#[test]
fn test_trade_log_storage() {
    let mut engine = MatchingEngine::new("EURUSD".to_string());

    // Add a sell order
    let sell_order = Arc::new(Order::new(
        Uuid::new_v4(),
        "EURUSD".to_string(),
        Side::Sell,
        OrderType::Limit,
        Quantity(1_000_000),
        Some(Price(10_850)),
    ));
    engine.orderbook_mut().add_order(sell_order.clone());

    // Add a buy order that matches
    let buy_order = Arc::new(Order::new(
        Uuid::new_v4(),
        "EURUSD".to_string(),
        Side::Buy,
        OrderType::Limit,
        Quantity(1_000_000),
        Some(Price(10_850)),
    ));

    let result = engine.match_order(buy_order.clone());
    assert_eq!(result.trades.len(), 1);

    // Check trade log
    let trade_log = engine.trade_log();
    let trades = trade_log.get_trades();
    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].quantity.0, 1_000_000);
}

#[test]
fn test_audit_log_storage() {
    let mut engine = MatchingEngine::new("EURUSD".to_string());

    let buy_order = Arc::new(Order::new(
        Uuid::new_v4(),
        "EURUSD".to_string(),
        Side::Buy,
        OrderType::Limit,
        Quantity(1_000_000),
        Some(Price(10_850)),
    ));

    engine.match_order(buy_order.clone());

    // Check audit log
    let audit_log = engine.audit_log();
    let events = audit_log.get_events();
    assert!(!events.is_empty());
    assert!(events.iter().any(|e| e.order_id == buy_order.id));
}
