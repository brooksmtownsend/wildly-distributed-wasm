use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_httpserver::{
    HeaderMap, HttpRequest, HttpResponse, HttpServer, HttpServerReceiver,
};
use wasmcloud_interface_logging::{debug, info, warn};
use wasmcloud_interface_messaging::{Messaging, MessagingSender, PubMessage, RequestMessage};
use wild_wasm_interface::*;

const UI_ACTOR: &str = "MD7C625SXR64K4SBW7YOSOXVKREECAMS4LRQO4MUNSIHLFGIAZCAPWUO";

#[derive(Serialize, Deserialize)]
struct InputTodo {
    title: String,
    order: Option<i32>,
}
#[derive(Serialize, Deserialize)]
struct UpdateTodo {
    title: Option<String>,
    completed: Option<bool>,
    order: Option<i32>,
}
#[derive(Debug, Serialize, Deserialize)]
struct Todo {
    url: String,
    title: String,
    completed: bool,
    order: i32,
}

impl Todo {
    fn new(url: String, title: String, order: i32) -> Self {
        Self {
            url,
            title,
            completed: false,
            order,
        }
    }

    fn update(self, update: UpdateTodo) -> Todo {
        Todo {
            url: self.url,
            title: update.title.unwrap_or(self.title),
            completed: update.completed.unwrap_or(self.completed),
            order: update.order.unwrap_or(self.order),
        }
    }
}

async fn create_todo(ctx: &Context, input: InputTodo) -> Result<Todo> {
    info!("Creating a todo...");

    //TODO: this won't work for any title that's not a single word or whatever
    let todo = Todo::new(
        format!("/api/{}", input.title.replace(" ", "_")),
        input.title,
        input.order.unwrap_or(0),
    );

    MessagingSender::new()
        .publish(
            ctx,
            &PubMessage {
                subject: format!("wasmkv.set.{}", todo.url.clone()),
                reply_to: None,
                body: serde_json::to_vec(&todo)?,
            },
        )
        .await
        .map_err(|e| anyhow!(e))?;

    Ok(todo)
}

async fn update_todo(ctx: &Context, url: &str, update: UpdateTodo) -> Result<Todo> {
    info!("Updating a todo...");

    let todo = get_todo(ctx, url).await?;
    let todo = todo.update(update);

    // this assumes update method can't change the url. Otherwise, we'd need to delete and add.
    MessagingSender::new()
        .publish(
            ctx,
            &PubMessage {
                subject: format!("wasmkv.set.{}", url),
                reply_to: None,
                body: serde_json::to_vec(&todo)?,
            },
        )
        .await
        .map_err(|e| anyhow!(e))?;

    Ok(todo)
}

async fn get_all_todos(ctx: &Context) -> Result<Vec<Todo>> {
    info!("Getting all todos...");

    let resp = MessagingSender::new()
        .request(
            ctx,
            &RequestMessage {
                subject: format!("wasmkv.get"),
                body: vec![],
                timeout_ms: 1000,
            },
        )
        .await?;

    // Deserialize into vec of strings, map to Todo object
    let todos = serde_json::from_slice::<Vec<Vec<u8>>>(&resp.body)?
        .iter()
        .filter_map(|todo_slice| {
            serde_json::from_slice::<Todo>(todo_slice)
                .map(|todo| Todo {
                    title: todo.title.replace("_", ""),
                    ..todo
                })
                .ok()
        })
        .collect();

    Ok(todos)
}

async fn get_todo(ctx: &Context, todo: &str) -> Result<Todo> {
    info!("Getting a todo... {}", todo);
    let resp = MessagingSender::new()
        .request(
            ctx,
            &RequestMessage {
                subject: format!("wasmkv.get./api/{}", todo),
                body: vec![],
                timeout_ms: 1000,
            },
        )
        .await?;

    info!("Resp: {:?}", resp);

    let todo = serde_json::from_slice::<Todo>(&resp.body)?;
    let todo = Todo {
        title: todo.title.replace("_", " "),
        ..todo
    };

    Ok(todo)
}

async fn delete_all_todos(ctx: &Context) -> Result<()> {
    info!("Deleting all todos...");
    let _ = MessagingSender::new()
        .request(
            ctx,
            &RequestMessage {
                subject: format!("wasmkv.del"),
                body: vec![],
                timeout_ms: 1000,
            },
        )
        .await?;
    Ok(())
}

async fn delete_todo(ctx: &Context, url: &str) -> Result<()> {
    info!("Deleting a todo...");
    let _ = MessagingSender::new()
        .request(
            ctx,
            &RequestMessage {
                subject: format!("wasmkv.del.{}", url),
                body: vec![],
                timeout_ms: 1000,
            },
        )
        .await?;
    Ok(())
}

async fn handle_request(ctx: &Context, req: &HttpRequest) -> RpcResult<HttpResponse> {
    debug!("incoming req: {:?}", req);

    //TODO: better way to trim both ends
    let trimmed_path: Vec<&str> = req
        .path
        .trim_start_matches('/')
        .trim_end_matches('/')
        .split('/')
        .collect();
    info!("Segments: {:?}", trimmed_path);
    match (req.method.as_ref(), trimmed_path.as_slice()) {
        ("POST", ["api"]) => match serde_json::from_slice(&req.body) {
            Ok(input) => match create_todo(ctx, input).await {
                Ok(todo) => HttpResponse::json(todo, 200),
                Err(e) => Err(RpcError::ActorHandler(format!("creating todo: {:?}", e))),
            },
            Err(e) => Ok(HttpResponse::bad_request(format!(
                "malformed body: {:?}",
                e
            ))),
        },

        ("GET", ["api"]) => match get_all_todos(ctx).await {
            Ok(todos) => HttpResponse::json(todos, 200),
            Err(e) => Err(RpcError::ActorHandler(format!("getting all todos: {}", e))),
        },

        ("GET", ["api", todo]) => match get_todo(ctx, todo).await {
            Ok(todo) => HttpResponse::json(todo, 200),
            Err(_) => Ok(HttpResponse::not_found()),
        },

        ("PATCH", [url]) => match serde_json::from_slice(&req.body) {
            Ok(update) => match update_todo(ctx, url, update).await {
                Ok(todo) => HttpResponse::json(todo, 200),
                Err(e) => Err(RpcError::ActorHandler(format!("updating todo: {}", e))),
            },
            Err(e) => Ok(HttpResponse::bad_request(format!(
                "malformed body: {:?}",
                e
            ))),
        },

        ("DELETE", ["api"]) => match delete_all_todos(ctx).await {
            Ok(_) => Ok(HttpResponse::default()),
            Err(e) => Err(RpcError::ActorHandler(format!("deleting all todos: {}", e))),
        },

        ("DELETE", [url]) => match delete_todo(ctx, url).await {
            Ok(_) => Ok(HttpResponse::default()),
            Err(e) => Err(RpcError::ActorHandler(format!("deleting todo: {}", e))),
        },

        ("GET", _) => {
            info!(
                "Got unrecognized path {}. Assuming this is an asset request",
                req.path
            );
            let sender = UiSender::to_actor(UI_ACTOR);
            let resp = sender.get_asset(ctx, &req.path).await?;
            if !resp.found {
                debug!("Asset {} was not found", req.path);
                return Ok(HttpResponse::not_found());
            }
            debug!("Got {} bytes for {}", resp.asset.len(), req.path);
            let mut header = HeaderMap::new();
            if let Some(content_type) = resp.content_type {
                debug!(
                    "Found content type of {}, setting Content-Type header",
                    content_type
                );
                header.insert("Content-Type".to_string(), vec![content_type]);
            }
            Ok(HttpResponse {
                status_code: 200,
                header,
                body: resp.asset,
            })
        }
        (_, _) => {
            warn!("no route for this request: {:?}", req);
            Ok(HttpResponse::not_found())
        }
    }
}

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, HttpServer)]
struct TodoActor {}

/// Implementation of HttpServer trait methods
#[async_trait]
impl HttpServer for TodoActor {
    async fn handle_request(&self, ctx: &Context, req: &HttpRequest) -> RpcResult<HttpResponse> {
        handle_request(ctx, req).await
    }
}
