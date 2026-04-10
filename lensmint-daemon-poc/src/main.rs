use axum::{
    extract::State,
    routing::{get, post},
    Json, Router, response::IntoResponse,
};
use base64::{engine::general_purpose::STANDARD as BASE64_STD, Engine as _};
use ed25519_dalek::{Signer, SigningKey, Signature};
use image_hasher::{HashAlg, HasherConfig};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::fs::File;          // 核心修改：使用标准库的文件读取
use std::io::Read;          // 核心修改：使用标准库的 Read 特征
use tokio::net::TcpListener;

// --- 共享的硬件状态 (存储设备私钥) ---
struct AppState {
    signing_key: SigningKey,
}

#[derive(Deserialize)]
struct CaptureRequest {
    image_path: String,
}

#[derive(Serialize)]
struct MintPayload {
    phash_hex: String,
    device_pubkey: String,
    signature: String,
    status: String,
}

#[tokio::main]
async fn main() {
    // 1. 初始化设备身份：直接读取 Linux 内核的硬件熵池！(0 依赖，最硬核的 IoT 写法)
    let mut secret = [0u8; 32];
    let mut urandom = File::open("/dev/urandom").expect("OS RNG failure: /dev/urandom not found");
    urandom.read_exact(&mut secret).expect("Failed to read OS entropy");

    let signing_key = SigningKey::from_bytes(&secret);
    let verifying_key = signing_key.verifying_key();
    
    let shared_state = Arc::new(AppState { signing_key });

    // 2. 配置本地 IPC API 路由
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/internal/capture", post(process_capture))
        .with_state(shared_state);

    // 3. 启动异步网络监听
    let listener = TcpListener::bind("127.0.0.1:3030").await.unwrap();
    println!("🚀 LensMint Edge Daemon started on http://127.0.0.1:3030");
    println!("🔐 Hardware Identity (Pubkey): {}", hex::encode(verifying_key.as_bytes()));

    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "LensMint Hardware Daemon is running securely."
}

// --- 核心业务 API: 模拟捕获、提取 pHash 并签名 ---
async fn process_capture(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CaptureRequest>,
) -> impl IntoResponse {
    
    println!("\n📸 [EVENT] Received capture request for: {}", payload.image_path);

    // 1. 读取传过来的任意图片
    let img_result = image::open(&payload.image_path);
    if img_result.is_err() {
        println!("❌ Error: Image not found at path.");
        return Json(MintPayload {
            phash_hex: "".to_string(),
            device_pubkey: "".to_string(),
            signature: "".to_string(),
            status: "Error: Image not found".to_string(),
        });
    }
    let img = img_result.unwrap();

    // 2. 提取抗压的 pHash
    let hasher = HasherConfig::new().hash_alg(HashAlg::Mean).hash_size(8, 8).to_hasher();
    let phash = hasher.hash_image(&img);
    let phash_base64 = BASE64_STD.encode(phash.as_bytes());

    // 3. 硬件身份签名
    let signature: Signature = state.signing_key.sign(phash.as_bytes());
    let sig_hex = hex::encode(signature.to_bytes());
    let pubkey_hex = hex::encode(state.signing_key.verifying_key().to_bytes());

    println!("✅ pHash extracted and cryptographically signed.");

    Json(MintPayload {
        phash_hex: phash_base64,
        device_pubkey: pubkey_hex,
        signature: sig_hex,
        status: "Success".to_string(),
    })
}