use futures_lite::StreamExt;
use lapin::options::{BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Channel, Connection, ConnectionProperties};
use tokio::sync::mpsc;

use super::config::QueueConfig;
use super::error::QueueError;

const CONSUMER_CHANNEL_BUFFER: usize = 64;

pub struct RabbitMQ {
    config: QueueConfig,
    channel: Option<Channel>,
}

impl RabbitMQ {
    pub fn new(config: QueueConfig) -> Self {
        Self {
            config,
            channel: None,
        }
    }

    pub async fn connect(&mut self) -> Result<(), QueueError> {
        let dsn = self.config.dsn();
        let conn = Connection::connect(&dsn, ConnectionProperties::default()).await?;
        self.channel = Some(conn.create_channel().await?);
        Ok(())
    }

    pub async fn consume(&self) -> Result<mpsc::Receiver<lapin::message::Delivery>, QueueError> {
        let channel = self
            .channel
            .as_ref()
            .ok_or(QueueError::ChannelUnavailable)?;

        let mut args = FieldTable::default();
        args.insert(
            "x-dead-letter-exchange".into(),
            lapin::types::AMQPValue::LongString(self.config.dlx.clone().into()),
        );

        let queue = channel
            .queue_declare(
                &self.config.consumer_queue_name,
                QueueDeclareOptions {
                    durable: true,
                    ..QueueDeclareOptions::default()
                },
                args,
            )
            .await
            .map_err(|e| QueueError::QueueDeclare {
                queue: self.config.consumer_queue_name.clone(),
                source: e,
            })?;

        let mut consumer = channel
            .basic_consume(
                queue.name().as_str(),
                &self.config.consumer_name,
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| QueueError::ConsumerRegister {
                consumer: self.config.consumer_name.clone(),
                source: e,
            })?;

        let (tx, rx) = mpsc::channel(CONSUMER_CHANNEL_BUFFER);

        tokio::spawn(async move {
            while let Some(Ok(delivery)) = consumer.next().await {
                tracing::info!("incoming new message");
                if tx.send(delivery).await.is_err() {
                    tracing::warn!("receiver dropped, stopping consumer");
                    break;
                }
            }
            tracing::info!("RabbitMQ consumer stream closed");
        });

        Ok(rx)
    }

    pub async fn notify(
        &self,
        message: &str,
        content_type: &str,
        exchange: &str,
        routing_key: &str,
    ) -> Result<(), QueueError> {
        let channel = self
            .channel
            .as_ref()
            .ok_or(QueueError::ChannelUnavailable)?;

        channel
            .basic_publish(
                exchange,
                routing_key,
                BasicPublishOptions::default(),
                message.as_bytes(),
                BasicProperties::default().with_content_type(content_type.into()),
            )
            .await
            .map_err(QueueError::Publish)?
            .await
            .map_err(QueueError::Publish)?;

        Ok(())
    }

    pub async fn notify_default(&self, message: &str) -> Result<(), QueueError> {
        self.notify(
            message,
            "application/json",
            &self.config.notification_exchange,
            &self.config.notification_routing_key,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> QueueConfig {
        QueueConfig {
            user: "guest".to_string(),
            password: "guest".to_string(),
            host: "localhost".to_string(),
            port: "5672".to_string(),
            vhost: "/".to_string(),
            consumer_queue_name: "videos".to_string(),
            consumer_name: "test-consumer".to_string(),
            dlx: "dlx".to_string(),
            notification_exchange: "amq.direct".to_string(),
            notification_routing_key: "jobs".to_string(),
        }
    }

    #[test]
    fn new_creates_without_channel() {
        let rmq = RabbitMQ::new(test_config());
        assert!(rmq.channel.is_none());
    }

    #[tokio::test]
    async fn consume_without_connect_returns_channel_unavailable() {
        let rmq = RabbitMQ::new(test_config());
        let result = rmq.consume().await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("channel unavailable"));
    }

    #[tokio::test]
    async fn notify_without_connect_returns_channel_unavailable() {
        let rmq = RabbitMQ::new(test_config());
        let result = rmq
            .notify("test", "application/json", "amq.direct", "jobs")
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("channel unavailable"));
    }

    #[tokio::test]
    #[ignore = "requires running RabbitMQ instance"]
    async fn connect_to_rabbitmq() {
        let mut rmq = RabbitMQ::new(test_config());
        let result = rmq.connect().await;
        assert!(result.is_ok());
        assert!(rmq.channel.is_some());
    }

    #[tokio::test]
    #[ignore = "requires running RabbitMQ instance"]
    async fn roundtrip_consume_and_notify() {
        let mut rmq = RabbitMQ::new(test_config());
        rmq.connect().await.unwrap();

        let mut rx = rmq.consume().await.unwrap();

        rmq.notify(r#"{"video_id":"123"}"#, "application/json", "", "videos")
            .await
            .unwrap();

        let delivery = tokio::time::timeout(std::time::Duration::from_secs(5), rx.recv())
            .await
            .unwrap()
            .unwrap();

        let body = String::from_utf8(delivery.data.clone()).unwrap();
        assert!(body.contains("123"));

        delivery
            .ack(lapin::options::BasicAckOptions::default())
            .await
            .unwrap();
    }
}
