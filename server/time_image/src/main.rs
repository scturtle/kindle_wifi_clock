use ab_glyph::{FontVec, PxScale};
use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use image::{imageops, GrayImage, ImageFormat, Luma};
use imageproc::drawing::{draw_text_mut, text_size};
use std::{io::Cursor, sync::Arc};

struct AppState {
    font: FontVec,
}

async fn fetch_time_api() -> Result<String, (StatusCode, String)> {
    let url = "https://time.now/developer/api/timezone/Asia/Shanghai";
    reqwest::get(url)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .text()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn get_second() -> Result<String, (StatusCode, String)> {
    let response_text = fetch_time_api().await?;
    let second_str = response_text
        .split(r#""datetime":""#)
        .nth(1)
        .unwrap_or("")
        .split('T')
        .nth(1)
        .unwrap_or("")
        .get(6..8)
        .unwrap_or("0");

    let second = second_str.trim_start_matches('0');
    Ok(if second.is_empty() {
        "0".to_string()
    } else {
        second.to_string()
    })
}

async fn get_image(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let response_text = fetch_time_api().await?;
    let time_str = response_text
        .split(r#""datetime":""#)
        .nth(1)
        .unwrap_or("")
        .split('T')
        .nth(1)
        .unwrap_or("")
        .get(0..5)
        .unwrap_or("00:00");

    let width = 800;
    let height = 600;
    let mut image = GrayImage::from_pixel(width, height, Luma([255]));

    let scale = PxScale::from(360.0);
    let (text_w, text_h) = text_size(scale, &state.font, time_str);
    let x = (width as i32 - text_w as i32) / 2 - 10;
    let y = (height as i32 - text_h as i32) / 2;
    draw_text_mut(&mut image, Luma([0]), x, y, scale, &state.font, time_str);
    // image.save("time.png")?;

    let mut buffer = Cursor::new(Vec::new());
    imageops::rotate90(&image)
        .write_to(&mut buffer, ImageFormat::Png)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to encode image: {}", e),
            )
        })?;

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "image/png".parse().unwrap());
    Ok((headers, buffer.into_inner()))
}

#[tokio::main]
async fn main() {
    let font_data = std::fs::read("font.ttf").expect("font.ttf not found");
    let font = FontVec::try_from_vec(font_data).expect("Invalid font");
    let state = Arc::new(AppState { font });

    let app = Router::new()
        .route("/image", get(get_image))
        .route("/second", get(get_second))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8333").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
