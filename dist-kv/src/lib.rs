use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_keyvalue::{
    GetResponse, KeyValue, KeyValueSender, SetAddRequest, SetDelRequest, SetRequest,
};
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
        match (subj.first(), subj.get(1), subj.get(2), msg.reply_to.clone()) {
            // wasmkv.get
            (Some(&"wasmkv"), Some(&"get"), None, Some(reply_to)) => {
                reply(
                    ctx,
                    reply_to,
                    serde_json::to_vec(&get_all(ctx).await?)
                        .map_err(|_| RpcError::Ser("Failed to serialize all todos".to_string()))?,
                )
                .await
            }
            // wasmkv.get.<key>
            (Some(&"wasmkv"), Some(&"get"), Some(key), Some(reply_to)) => {
                reply(
                    ctx,
                    reply_to,
                    get(ctx, key).await.map(|todo| todo.as_bytes().to_vec())?,
                )
                .await
            }
            // wasmkv.set.<key>
            (Some(&"wasmkv"), Some(&"set"), Some(key), _) => set(ctx, key, &msg.body).await,
            // wasmky.del   (delete all)
            (Some(&"wasmkv"), Some(&"del"), None, _) => delete_all(ctx).await,
            // wasmky.del.key
            (Some(&"wasmkv"), Some(&"del"), Some(key), _) => delete(ctx, key).await,
            (first, second, _, _) => {
                error!(
                    "Invalid distkv operation, ignoring: {:?}.{:?}",
                    first, second
                );
                Ok(())
            }
        }
    }
}

/// Gets a value at `key`, returning an empty vector if nothing is found
async fn get(ctx: &Context, key: &str) -> RpcResult<String> {
    match KeyValueSender::new().get(ctx, key).await {
        Ok(GetResponse {
            value,
            exists: true,
        }) => Ok(value),
        Ok(GetResponse { exists: false, .. }) => Ok("".to_string()),
        _ => Err(RpcError::ActorHandler("".to_string())),
    }
}

/// Gets all TODOs
async fn get_all(ctx: &Context) -> RpcResult<Vec<Vec<u8>>> {
    let urls = KeyValueSender::new().set_query(ctx, "all_urls").await?;

    let mut result = Vec::new();
    for url in urls {
        result.push(get(ctx, &url).await?.as_bytes().to_vec())
    }

    Ok(result)
}

async fn delete_all(ctx: &Context) -> RpcResult<()> {
    let kv = KeyValueSender::new();
    let urls = kv.set_query(ctx, "all_urls").await?;
    for key in urls {
        kv.del(ctx, &key).await?;
    }
    kv.set_clear(ctx, "all_urls").await?;
    Ok(())
}

async fn delete(ctx: &Context, key: &str) -> RpcResult<()> {
    let kv = KeyValueSender::new();
    kv.del(ctx, key).await?;
    let _ = kv
        .set_del(
            ctx,
            &SetDelRequest {
                set_name: "all_urls".to_string(),
                value: key.to_string(),
            },
        )
        .await?;
    Ok(())
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
        .await?;

    KeyValueSender::new()
        .set_add(
            ctx,
            &SetAddRequest {
                set_name: "all_urls".to_string(),
                value: key.to_string(),
            },
        )
        .await?;

    Ok(())
}

async fn reply(ctx: &Context, subject: String, body: Vec<u8>) -> RpcResult<()> {
    MessagingSender::new()
        .publish(
            ctx,
            &PubMessage {
                subject,
                reply_to: None,
                body,
            },
        )
        .await
}
