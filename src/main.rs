use futures_lite::StreamExt;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions},
    publisher_confirm::Confirmation,
    types::FieldTable,
    BasicProperties, Connection, ConnectionProperties,
};
use prima_tracing::{builder, configure_subscriber, init_subscriber};
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
                "http://localhost:9411/api/v2/spans".to_string(),
                "myapp".to_string(),
            )
            .build(),
    );

    let _guard = init_subscriber(subscriber);

    let span = info_span!("MySpan");
    let _guard = span.enter();

    info!("Starting my awesome app");

    let addr = std::env::var("AMQP_ADDR")
        .unwrap_or_else(|_| "amqp://lira:lira@127.0.0.1:5672/lira".into());

    let conn = Connection::connect(&addr, ConnectionProperties::default())
        .await
        .unwrap();

    info!("CONNECTED");

    let channel_a = conn.create_channel().await.unwrap();
    let channel_b = conn.create_channel().await.unwrap();

    let queue = channel_a
        .queue_declare(
            "hello",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    info!(?queue, "Declared queue");

    let mut consumer = channel_b
        .basic_consume(
            "hello",
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    tokio::spawn(async move {
        info!("will consume");
        while let Some(delivery) = consumer.next().await {
            let delivery = delivery.expect("error in consumer");
            delivery.ack(BasicAckOptions::default()).await.expect("ack");
        }
    });

    let payload = b"Hello world!";

    let confirm = channel_a
        .basic_publish(
            "",
            "hello",
            BasicPublishOptions::default(),
            payload,
            BasicProperties::default(),
        )
        .await
        .unwrap()
        .await
        .unwrap();
    assert_eq!(confirm, Confirmation::NotRequested);
}
