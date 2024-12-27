use std::{any::Any, sync::Arc};

use dioxus::prelude::*;

pub mod database;
mod handlers;
pub mod schema;

use handlers::{dioxus_handler, ws_echo_server};

#[derive(Debug, Clone)]
pub struct MyContext {
    pub title: String,
}

// The entry point for the server
#[cfg(feature = "server")]
pub async fn init(app: fn() -> Element) {
    use axum::{routing::get, Extension};
    use handlers::health_check;

    tracing_subscriber::fmt::init();

    let database = database::init().await;
    let database_clone = database.clone();

    let context = MyContext {
        title: "Dioxus Context".to_string(),
    };

    // Get the address the server should run on. If the CLI is running, the CLI proxies fullstack into the main address
    // and we use the generated address the CLI gives us
    let address = dioxus_cli_config::fullstack_address_or_localhost();

    let provider_1 = move || Box::new(context.clone()) as Box<dyn Any>;
    let provider_2 = move || Box::new(42u32) as Box<dyn Any>;
    let provider_3 = move || Box::new(database.clone()) as Box<dyn Any>;

    let cfg = ServeConfigBuilder::default().context_providers(Arc::new(vec![
        Box::new(provider_1),
        Box::new(provider_2),
        Box::new(provider_3),
    ]));

    // Set up the axum router
    let router = axum::Router::new()
        // You can add a dioxus application to the router with the `serve_dioxus_application` method
        // This will add a fallback route to the router that will serve your component and server functions
        .serve_dioxus_application(cfg, app)
        .route("/_health", get(health_check))
        .route("/_dioxus", get(dioxus_handler))
        .route("/echo", get(ws_echo_server))
        .layer(Extension(database_clone));

    // Finally, we can launch the server
    let router = router.into_make_service();
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
