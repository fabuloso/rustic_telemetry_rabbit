use futures_lite::StreamExt;
use lapin::message::Delivery;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions},
    types::FieldTable,
    Connection,
};
use tracing::{info, info_span, Id};

pub async fn start_consumer(conn: &Connection) {
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
        while let Some(delivery) = consumer.next().await {
            let delivery = delivery.expect("error in consumer");
            delivery.ack(BasicAckOptions::default()).await.expect("ack");
            let span = info_span!("consumer");
            span.follows_from(extract_span_id(&delivery).unwrap());
            let _guard = span.enter();
            info!("I'm in the new span!");
        }
    });
}

fn extract_span_id(delivery: &Delivery) -> Option<Id> {
    for (key, value) in delivery.properties.headers().as_ref().unwrap() {
        if key.to_string() == "x-span-id" {
            return Some(Id::from_u64(value.as_long_long_int().unwrap() as u64));
        }
    }
    None
}
