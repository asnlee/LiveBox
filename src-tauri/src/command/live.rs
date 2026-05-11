use crate::command::model::LiveInfo;
use crate::command::runner::DouYinReq;
use tauri::{AppHandle, Manager};

// 自定义函数
#[tauri::command]
pub async fn greet_you(name: &str) -> Result<String, String> {
    println!("调用了greet_you");
    Ok(format!("Hello, {}! You've been greeted from Rust!", name))
}

#[tauri::command]
pub async fn get_live_html(url: &str) -> Result<LiveInfo, String> {
    // let response = reqwest::get(live_url).await.unwrap();
    // println!("调用了get_live_html");
    let mut live_req = DouYinReq::new(url);
    // 获取直播间room_id和主播信息
    let result = live_req.get_room_info().await;
    match result {
        Ok(info) => Ok(info),
        Err(_) => Err("This failed!".into()),
    }
}

#[tauri::command]
pub async fn open_window(
    handle: AppHandle,
    app_url: String,
    app_name: String,
    platform: String,
    user_agent: String,
    _resize: bool,
    width: f64,
    height: f64,
    _js_content: String,
) {
    let window_label = "previewWeb";
    println!("Opening docs in external window: {}, {}", app_url, platform);
    if let Some(existing_window) = handle.get_window(window_label) {
        let _ = existing_window.set_title(&app_name);
        let _ = existing_window.eval(&format!("window.location.replace('{}')", app_url));
    } else {
        let _window = tauri::WindowBuilder::new(
            &handle,
            window_label,
            tauri::WindowUrl::External(app_url.parse().unwrap()),
        )
        .title(app_name.clone())
        .inner_size(width, height)
        .user_agent(user_agent.as_str())
        .initialization_script(include_str!("../inject/websocket.js"))
        .center()
        .build()
        .unwrap();
    }
}

#[tauri::command]
pub async fn upload_file(file_name: String, file_data: Vec<u8>) -> Result<String, String> {
    println!("[upload_file] start, file_name: {}, file_size: {}", file_name, file_data.len());
    let client = reqwest::Client::new();

    let response = client
        .post(format!("https://html2web.codepoem.top/api/cos/files/mall/{}", file_name))
        .header("Authorization", "asnlee")
        .body(file_data)
        .send()
        .await
        .map_err(|e| {
            println!("[upload_file] request error: {}", e);
            e.to_string()
        })?;

    println!("[upload_file] response status: {}", response.status());
    let json: serde_json::Value = response.json().await.map_err(|e| {
        println!("[upload_file] parse json error: {}", e);
        e.to_string()
    })?;
    println!("[upload_file] response json: {:?}", json);

    let result = json.get("message")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Failed to get message from response".to_string());

    result
}
