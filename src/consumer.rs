use futures_lite::StreamExt;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions},
    types::FieldTable,
    Connection,
};
use tracing::{info, Span};

pub async fn start_consumer(span: &Span, conn: &Connection) {
    info!("Starting my awesome consumer");

    let channel = conn.create_channel().await.unwrap();

    let mut consumer = channel
        .basic_consume(
            "hello",
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    tokio::spawn(async move {
        info!(parent: span, "will consume");

        while let Some(delivery) = consumer.next().await {
            let delivery = delivery.expect("error in consumer");
            delivery.ack(BasicAckOptions::default()).await.expect("ack");
            let headers = delivery.properties.headers();
            info!("headers: {:?}", headers);
        }
    });
}
