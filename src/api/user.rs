use actix_web::{get, post, web, HttpResponse, HttpRequest};
use serde::{Deserialize, Serialize};
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use std::io::Write;
use std::fs; // 将 json 字符串解析为结构体
use uuid::Uuid;

use crate::{database::mysql::*, common::token::*, common::config::*};

// const IP_PORT: &str = "127.0.0.1:8082";

// 头像获取函数
#[get("/avatar/{user_id}")]
pub async fn get_avatar(user_id: web::Path<String>) -> actix_web::Result<HttpResponse> {
    log::info!("Start get_avatar function");
    // println!("get_avatar");
    let user_id = (*user_id).clone();
    let avatar_path = format!("./static/avatar/avatar-{}.jpg", user_id);

    let avatar_data = match fs::read(avatar_path) {
        Ok(bytes) => bytes,
        Err(err) => {
            return {
                log::error!("Error get_avatar read avatar: {:?}", err);
                Err(actix_web::error::ErrorInternalServerError(err))
            }
        }
    };

    log::info!("End get_avatar function");
    // 返回包含图片数据的 HTTP 响应
    Ok(HttpResponse::Ok()
        .content_type("image/jpeg") // 指定图片的 MIME 类型
        .body(avatar_data))
}

// 上传帖子封面的函数
#[post("/avatar/submit_avatar")]
async fn submit_avatar(req: HttpRequest, mut payload: Multipart) -> actix_web::Result<HttpResponse> {
    log::debug!("Start submit_avatar function");
    println!("submit_avatar");
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("Failed is verif token"));
    }
    // 用于存储文件的路径
    let mut file_path = String::new();
    // 用于遍历上传的文件流中的每一个字段
    // payload.try_next().await 从文件流中异步获取下一个字段
    while let Ok(Some(mut field)) = payload.try_next().await {
        // 获取字段的内容描述信息，用于提取文件名等信息。
        // let content_disposition = field.content_disposition();
        // 从内容描述信息中提取文件名。
        // let filename = content_disposition.get_filename().unwrap();
        file_path = format!("./static/avatar/avatar-{}.jpg", Uuid::new_v4());
        let file_path_clone = file_path.clone();

        // 用 web::block 来阻塞地创建文件，并将文件对象存储在 f 中
        let mut f = web::block(move || std::fs::File::create(&file_path_clone))
            .await
            .unwrap();
        // 这是内部的 while 循环，用于读取字段中的每个数据块（chunk）
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            if let Ok(ref mut file) = f {
                file.write_all(&data)?;
            }
        }
    }
    log::debug!("End submit_avatar function");
    let url = convert_path_to_url(&file_path);
    // println!("{}", url);
    Ok(HttpResponse::Ok().body(url))
    // Ok(HttpResponse::Ok().body(serde_json::to_string(&convert_path_to_url(&file_path)).unwrap()))
}

// 修改路径的函数
fn convert_path_to_url(path: &str) -> String {
    // 假设所有的路径都遵循"./static/cover/cover-{id}.jpg"的格式
    // 1. 移除文件路径的'./static/cover/'部分
    let trimmed_path = path.trim_start_matches("./static/avatar/");

    // 2. 移除文件扩展名'.jpg'
    let without_extension = trimmed_path.trim_end_matches(".jpg");

    // 3. 提取ID（这一步在这个简单示例中可能不是必需的，取决于你是否需要对ID做进一步处理）
    let id = without_extension.trim_start_matches("avatar-");

    let ip = Config::instance().server.host.clone();
    let port = Config::instance().server.port;
    // 4. 拼接新的URL
    let new_url = format!("https://{}:{}/api/avatar/{}", ip, port, id);

    new_url
}

// 创建一个帖子的结构体，用来发送数据
#[derive(Debug, Deserialize, Serialize)]
struct User {
    id: u32,
    name: String,
    avatar_url: String,
}

// 获取单个用户
#[get("/user/{user_id}")]
pub async fn get_user(user_id: web::Path<u32>) -> actix_web::Result<HttpResponse> {
    log::info!("Start get_user function");
    let user_id = *user_id;

    // 获取线程池，这个线程池为单例模式
    let my_pool = MysqlPool::instance();

    let query = format!(
        "SELECT id, name, avatar_url FROM user WHERE id = {}",
        user_id
    );
    let user: Vec<User> = match my_pool.query_map(
        query,
        |(id, name, avatar_url): (u32, String, String)| {
            let avatar_url = if avatar_url.len() <= 1 {
                AVATAR_URL.to_string()
            } else {
                avatar_url
            };
            User {
                id,
                name,
                avatar_url,
            }
        },
        &my_pool.read_only_txopts,
    ) {
        Ok(result) => result,
        Err(err) => {
            log::error!("Error get_user query_map query: {:?}", err);
            // eprintln!("get_user: Error query_map query: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    // println!("user_id: {}, {:?}", user_id, user);

    let post_jsons = serde_json::to_string(&user[0]).map_err(|err| {
        // eprintln!("Error serializing response: {:?}", err);
        log::error!("Error serializing response: {:?}", err);
        actix_web::error::ErrorInternalServerError("Error serializing response")
    })?;

    log::info!("End get_user function");
    Ok(HttpResponse::Ok().body(post_jsons))
}
