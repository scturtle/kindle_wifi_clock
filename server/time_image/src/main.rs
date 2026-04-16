use ab_glyph::{FontVec, PxScale};
use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use chrono::Timelike;
use image::{imageops, GrayImage, ImageFormat, Luma};
use imageproc::drawing::{draw_text_mut, text_size};
use rsntp::{AsyncSntpClient, SynchronizationResult};
use std::{io::Cursor, sync::Arc};

struct AppState {
    font: FontVec,
}

/// Fetches time from NTP server using rsntp and converts it to Asia/Shanghai DateTime
async fn get_shanghai_time() -> Result<chrono::DateTime<chrono_tz::Tz>, (StatusCode, String)> {
    let result: SynchronizationResult = AsyncSntpClient::new()
        .synchronize("162.159.200.123:123")
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("NTP Error: {}", e),
            )
        })?;

    let utc_datetime = result.datetime().into_chrono_datetime().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Invalid NTP date: {}", e),
        )
    })?;

    Ok(utc_datetime.with_timezone(&chrono_tz::Asia::Shanghai))
}

async fn get_second() -> Result<impl IntoResponse, (StatusCode, String)> {
    let now = get_shanghai_time().await?;
    Ok(now.second().to_string())
}

async fn get_image(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let now = get_shanghai_time().await?;
    let time_str = format!("{:02}:{:02}", now.hour(), now.minute());

    let width = 800;
    let height = 600;
    let mut image = GrayImage::from_pixel(width, height, Luma([255]));

    let scale = PxScale::from(360.0);
    let (text_w, text_h) = text_size(scale, &state.font, &time_str);

    let x = (width as i32 - text_w as i32) / 2 - 10;
    let y = (height as i32 - text_h as i32) / 2;

    draw_text_mut(&mut image, Luma([0]), x, y, scale, &state.font, &time_str);

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
    println!("Server running on http://0.0.0.0:8333");
    axum::serve(listener, app).await.unwrap();
}
