use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_keyvalue::{GetResponse, KeyValue, KeyValueSender, SetRequest};
use wasmcloud_interface_logging::{error, info};
use wasmcloud_interface_messaging::{
    MessageSubscriber, MessageSubscriberReceiver, Messaging, MessagingSender, PubMessage,
    SubMessage,
};

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, MessageSubscriber)]
struct DistKvActor {}

#[async_trait]
impl MessageSubscriber for DistKvActor {
    async fn handle_message(&self, ctx: &Context, msg: &SubMessage) -> RpcResult<()> {
        info!("Received message: {:?}", msg);
        // Split subject into parts for matching
        let subj = msg.subject.split('.').collect::<Vec<&str>>();
        match (subj.first(), subj.get(1)) {
            // wasmkv.get.<key>
            (Some(&"wasmkv"), Some(&"get")) if msg.reply_to.is_some() && subj.get(2).is_some() => {
                // Publish reply with the retrieved payload
                MessagingSender::new()
                    .publish(
                        ctx,
                        &PubMessage {
                            subject: msg.reply_to.clone().unwrap(),
                            reply_to: None,
                            body: get(ctx, subj.get(2).unwrap()).await?,
                        },
                    )
                    .await?;
            }
            // wasmkv.set.<key>
            (Some(&"wasmkv"), Some(&"set")) if subj.get(2).is_some() => {
                set(ctx, subj.get(2).unwrap(), &msg.body).await?;
            }
            (first, second) => error!(
                "Invalid distkv operation, ignoring: {:?}.{:?}",
                first, second
            ),
        };
        Ok(())
    }
}

/// Gets a value at `key`, returning an empty vector if nothing is found
async fn get(ctx: &Context, key: &str) -> RpcResult<Vec<u8>> {
    match KeyValueSender::new().get(ctx, key).await {
        Ok(GetResponse {
            value,
            exists: true,
        }) => Ok(value.as_bytes().to_vec()),
        Ok(GetResponse { exists: false, .. }) => Ok(vec![]),
        _ => Err(RpcError::ActorHandler("".to_string())),
    }
}

/// Sets a value at `key`
async fn set(ctx: &Context, key: &str, value: &[u8]) -> RpcResult<()> {
    KeyValueSender::new()
        .set(
            ctx,
            &SetRequest {
                key: key.to_string(),
                value: String::from_utf8(value.to_vec())
                    .map_err(|e| RpcError::Ser(format!("{}", e)))?,
                expires: 0,
            },
        )
        .await
}
