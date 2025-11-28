//! gRPC service implementation for matching engine

use fx_core::{MatchingEngine, Order};
use fx_proto::fx::etrading::{
    matching_engine_service_server::MatchingEngineService, CancelOrderRequest, Order as ProtoOrder,
    OrderResponse as ProtoOrderResponse, OrderType as ProtoOrderType, Side as ProtoSide,
    Trade as ProtoTrade,
};
use fx_utils::{OrderType, Price, Quantity, Side};
use parking_lot::Mutex;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct MatchingEngineGrpcService {
    engine: Arc<Mutex<MatchingEngine>>,
}

impl MatchingEngineGrpcService {
    pub fn new(engine: Arc<Mutex<MatchingEngine>>) -> Self {
        Self { engine }
    }
}

#[tonic::async_trait]
impl MatchingEngineService for MatchingEngineGrpcService {
    async fn submit_order(
        &self,
        request: Request<ProtoOrder>,
    ) -> Result<Response<ProtoOrderResponse>, Status> {
        let proto_order = request.into_inner();

        let order_id = Uuid::parse_str(&proto_order.order_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid order ID: {}", e)))?;

        let side = match proto_order.side() {
            ProtoSide::Buy => Side::Buy,
            ProtoSide::Sell => Side::Sell,
        };

        let order_type = match proto_order.order_type() {
            ProtoOrderType::Market => OrderType::Market,
            ProtoOrderType::Limit => OrderType::Limit,
            ProtoOrderType::Stop => OrderType::Stop,
            ProtoOrderType::Ioc => OrderType::IoC,
            ProtoOrderType::Fok => OrderType::FoK,
        };

        let price = if proto_order.price == 0 {
            None
        } else {
            Some(Price(proto_order.price))
        };

        let order = Arc::new(Order::new(
            order_id,
            proto_order.instrument,
            side,
            order_type,
            Quantity(proto_order.quantity),
            price,
        ));

        let mut engine_guard = self.engine.lock();
        let match_result = engine_guard.match_order(order);

        let proto_trades: Vec<ProtoTrade> = match_result
            .trades
            .iter()
            .map(|t| ProtoTrade {
                trade_id: t.id.to_string(),
                order_id: match_result.order.id.to_string(),
                instrument: t.instrument.clone(),
                side: match t.buy_order_id == match_result.order.id {
                    true => ProtoSide::Buy as i32,
                    false => ProtoSide::Sell as i32,
                },
                quantity: t.quantity.0,
                price: t.price.0,
                timestamp_ns: t.timestamp_ns as i64,
            })
            .collect();

        Ok(Response::new(ProtoOrderResponse {
            success: true,
            message: if proto_trades.is_empty() {
                "Order placed".to_string()
            } else {
                format!("Order matched with {} trades", proto_trades.len())
            },
            trades: proto_trades,
        }))
    }

    async fn cancel_order(
        &self,
        request: Request<CancelOrderRequest>,
    ) -> Result<Response<ProtoOrderResponse>, Status> {
        let req = request.into_inner();
        let order_id = Uuid::parse_str(&req.order_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid order ID: {}", e)))?;

        let mut engine_guard = self.engine.lock();
        let success = engine_guard.cancel_order(order_id);

        Ok(Response::new(ProtoOrderResponse {
            success,
            message: if success {
                "Order cancelled".to_string()
            } else {
                "Order not found".to_string()
            },
            trades: vec![],
        }))
    }
}
