mod auth;
mod sync;
mod data;

use std::sync::Arc;

use crate::auth::Credentials;
use crate::auth::UserBackend;
use crate::sync::BroadcastMap;
use crate::sync::DocumentRepository;
use axum::extract::State;
use axum::http::header;
use axum::http::HeaderValue;
use axum::http::Method;
use axum::{
    extract::{ws::WebSocket, Path, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_login::login_required;
use axum_login::tower_sessions::cookie::SameSite;
use axum_login::tracing::trace;
use axum_login::{
    tower_sessions::{cookie::time::Duration, Expiry, MemoryStore, SessionManagerLayer},
    AuthManagerLayerBuilder, AuthSession, AuthzBackend,
};
use axum_ycrdt_websocket::ws::AxumSink;
use axum_ycrdt_websocket::ws::AxumStream;

use futures_util::StreamExt;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing::Level;

#[derive(Clone)]
struct AppState {
    room_state: Arc<Mutex<BroadcastMap>>,
    doc_repo: Arc<DocumentRepository>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_http_only(false)
        .with_expiry(Expiry::OnInactivity(Duration::days(1)));

    let backend = UserBackend::new();
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();
    let cors_layer = CorsLayer::new()
        .allow_origin(["http://localhost:3000".parse::<HeaderValue>().unwrap()])
        .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS])
        .allow_private_network(true)
        .allow_credentials(true)
        .allow_headers([header::ACCEPT, header::AUTHORIZATION, header::COOKIE, header::CONTENT_TYPE]);

    let room_state = Arc::new(Mutex::new(BroadcastMap::new()));
    let doc_repo = Arc::new(DocumentRepository::new());

    let app_state = Arc::new(AppState {
        room_state,
        doc_repo,
    });

    let app = Router::new()
        .route("/:item", get(ws_handler))
        .with_state(app_state)
        .route("/hello", get(hello))
        .route_layer(login_required!(UserBackend, login_url = "/login"))
        .route("/login", post(login))
        .layer(auth_layer)
        .layer(cors_layer);
        

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn hello() -> impl IntoResponse {
    (StatusCode::OK, "hello").into_response()
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(item): Path<String>,
    State(app_state): State<Arc<AppState>>,
    auth_session: AuthSession<UserBackend>,
) -> impl IntoResponse {
    info!("{:?}", auth_session.user);
    let room_state = app_state.room_state.clone();
    let doc_repo = app_state.doc_repo.clone();
    trace!(item);
    if !auth_session
        .backend
        .get_user_permissions(&auth_session.user.unwrap())
        .await
        .unwrap()
        .contains(&item)
    {
        return (StatusCode::FORBIDDEN, "no access to this doc").into_response();
    }
    let doc_id: i64 = item.parse().unwrap();
    ws.on_upgrade(move |socket| handle_socket(socket, doc_id, room_state, doc_repo))
}

async fn handle_socket(
    socket: WebSocket,
    doc_id: i64,
    room_state: Arc<Mutex<BroadcastMap>>,
    doc_state: Arc<DocumentRepository>,
) {
    let (sender, receiver) = socket.split();
    let sender = Arc::new(Mutex::new(AxumSink::from(sender)));
    let receiver = AxumStream::from(receiver);

    // let mut inner_room_state = room_state.lock().await;
    // let bcast = inner_room_state.get_room(doc_id, doc_state).await.unwrap();

    let bcast = room_state
        .lock()
        .await
        .get_room(doc_id, doc_state)
        .await
        .unwrap();
    let sub = bcast.subscribe(sender, receiver);

    match sub.completed().await {
        Ok(_) => println!("broadcasting for channel finished successfully"),
        Err(e) => eprintln!("broadcaing for channel finished abruptly: {}", e),
    }
}

async fn login(
    mut auth_session: AuthSession<UserBackend>,
    Json(creds): Json<Credentials>,
) -> impl IntoResponse {
    info!("{:?}", creds);
    let user = match auth_session.authenticate(creds.clone()).await {
        Ok(Some(user)) => user,
        Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    (StatusCode::OK, "login success").into_response()
}
