use lapin::{
    options::{BasicPublishOptions, QueueDeclareOptions},
    types::{AMQPValue, FieldTable},
    BasicProperties, Connection, ConnectionProperties,
};
use prima_tracing::{builder, configure_subscriber, init_subscriber};
use rustic_telemetry_rabbit::consumer;
use std::thread::sleep;
use std::time::Duration;
use tracing::{info, info_span};

#[tokio::main]
async fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    let subscriber = configure_subscriber(
        builder("myapp")
            .with_env("dev".to_string())
            .with_version("1.0".to_string())
            .with_telemetry(
                "http://localhost:55681/v1/traces".to_string(),
                "myapp".to_string(),
            )
            .build(),
    );

    let _guard = init_subscriber(subscriber);

    let span = info_span!("Main");
    let _guard = span.enter();

    info!("Starting my awesome app");

    let addr = std::env::var("AMQP_ADDR")
        .unwrap_or_else(|_| "amqp://lira:lira@127.0.0.1:5672/lira".into());

    let conn = Connection::connect(&addr, ConnectionProperties::default())
        .await
        .unwrap();

    info!("CONNECTED");

    let channel_a = conn.create_channel().await.unwrap();

    let queue = channel_a
        .queue_declare(
            "hello",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    info!(?queue, "Declared queue");

    consumer::start_consumer(&conn).await;

    let payload = b"Hello world!";

    let mut headers = FieldTable::default();
    headers.insert(
        "x-span-id".into(),
        AMQPValue::LongLongInt((span.id().unwrap().into_u64() as i64).into()),
    );

    channel_a
        .basic_publish(
            "",
            "hello",
            BasicPublishOptions::default(),
            payload,
            BasicProperties::default().with_headers(headers),
        )
        .await
        .unwrap()
        .await
        .unwrap();

    sleep(Duration::from_secs(2));
}
