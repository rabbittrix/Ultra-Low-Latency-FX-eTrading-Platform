//! Gateway API definitions

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(),
    components(schemas()),
    tags(
        (name = "gateway", description = "FX eTrading Gateway API"),
        (name = "market-data", description = "Market Data endpoints"),
        (name = "trading", description = "Trading endpoints"),
    ),
    info(
        title = "FX eTrading Platform API",
        description = "Ultra-Low-Latency FX eTrading Platform REST API",
        version = "0.1.0",
        contact(
            name = "Roberto de Souza",
            email = "rabbittrix@hotmail.com"
        ),
        license(
            name = "Apache-2.0",
            url = "https://www.apache.org/licenses/LICENSE-2.0"
        )
    )
)]
pub struct GatewayApi;
