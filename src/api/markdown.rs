use actix_web::{web, get, post, HttpResponse, HttpRequest};
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use std::io::Write;
use std::fs;
use uuid::Uuid;

use crate::common::{token::*, config::*};

// 上传图片
#[post("/markdown/submit_image")]
async fn submit_image(req: HttpRequest, mut payload: Multipart) -> actix_web::Result<HttpResponse> {
    log::debug!("Start submit_image function");
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("Failed is verif token"));
    }
    // println!("submit_image");
    // 用于存储文件的路径
    let mut file_path = String::new();
    // 用于遍历上传的文件流中的每一个字段
    // payload.try_next().await 从文件流中异步获取下一个字段
    while let Ok(Some(mut field)) = payload.try_next().await {
        // 获取字段的内容描述信息，用于提取文件名等信息。
        // let content_disposition = field.content_disposition();
        // 从内容描述信息中提取文件名。
        // let filename = content_disposition.get_filename().unwrap();
        file_path = format!("./static/image/image-{}.jpg", Uuid::new_v4());
        let file_path_clone = file_path.clone();

        // 用 web::block 来阻塞地创建文件，并将文件对象存储在 f 中
        let mut f = web::block(move || std::fs::File::create(&file_path_clone)).await.unwrap();
        // 这是内部的 while 循环，用于读取字段中的每个数据块（chunk）
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            if let Ok(ref mut file) = f {
                file.write_all(&data)?;
            }
        }
    }
    log::debug!("End submit_image function");
    let url = convert_path_to_url(&file_path);
    // println!("{}", url);
    Ok(HttpResponse::Ok().body(url))
    // Ok(HttpResponse::Ok().body(serde_json::to_string(&convert_path_to_url(&file_path)).unwrap()))
}

// 修改路径的函数
fn convert_path_to_url(path: &str) -> String {
    // 假设所有的路径都遵循"./static/image/image-{id}.jpg"的格式
    // 1. 移除文件路径的'./static/image/'部分
    let trimmed_path = path.trim_start_matches("./static/image/");

    // 2. 移除文件扩展名'.jpg'
    let without_extension = trimmed_path.trim_end_matches(".jpg");

    // 3. 提取ID（这一步在这个简单示例中可能不是必需的，取决于你是否需要对ID做进一步处理）
    let id = without_extension.trim_start_matches("image-");

    let ip = Config::instance().server.host.clone();
    let port = Config::instance().server.port;
    // 4. 拼接新的URL
    let new_url = format!("https://{}:{}/api/image/{}", ip, port, id);

    new_url
}

// 获取图片
#[get("/image/{uu_id}")]
pub async fn get_image(req: HttpRequest, uu_id: web::Path<String>) -> actix_web::Result<HttpResponse> {
    log::info!("Start get_image function");
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("Failed is verif token"));
    }
    // println!("get_image");
    let uu_id = (*uu_id).clone();
    let image_path = format!("./static/image/image-{}.jpg", uu_id);

    let image_data = match fs::read(image_path) {
        Ok(bytes) => bytes,
        Err(err) => {
            log::error!("Error get_image read image_path: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    log::info!("End get_image function");
    // 返回包含图片数据的 HTTP 响应
    Ok(HttpResponse::Ok()
        .content_type("image/jpeg") // 指定图片的 MIME 类型
        .body(image_data))
}