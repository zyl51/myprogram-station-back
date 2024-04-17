use actix_multipart::Multipart;
use actix_web::{post, web, HttpRequest, HttpResponse};
use futures::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use std::io::Write;
use uuid::Uuid;

use crate::{common::config::*, common::token::Token, database::mysql::*};

#[derive(Debug, Serialize, Deserialize)]
struct UserName {
    user_id: u32,
    username: String,
}

// 修改用户名
#[post("/userprofile/update_username")]
pub async fn update_userprofile_username(
    req: HttpRequest,
    info: web::Json<UserName>,
) -> actix_web::Result<HttpResponse> {
    log::debug!("Start update_userprofile_username function");
    println!("update_userprofile_username");

    let user_info = match Token::token_to_claims(req) {
        Ok(ok) => ok, 
        Err(err) => {
            log::error!("Error update_userprofile_username is token_to_claims");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    // 获取用户名
    let UserName { user_id, username } = info.into_inner();

    // token 验证
    if user_info.get_id() != user_id || user_info.verify().is_err() {
        log::info!("user_info.get_id() != user_id || user_info.verify().is_err()");
        return Ok(HttpResponse::BadRequest().body("Token verif Farild"));
    }

    // 获取数据库
    let my_pool = MysqlPool::instance();
    let query = format!(
        "
        update user
        set name = '{}'
        where id = {};
    ",
        username, user_id
    );

    match my_pool.exec_drop(vec![query], &my_pool.read_write_txopts) {
        Ok(_) => {
            log::info!("Successful update_userprofile_username executing update user name");
        }
        Err(err) => {
            log::error!("Error update_userprofile_username executing update user name");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    log::debug!("End update_userprofile_username function");
    Ok(HttpResponse::Ok().body(serde_json::to_string(&username).unwrap()))
}

// 更换头像
#[post("/userprofile/update_avatar/{user_id}")]
pub async fn update_userprofile_avatar(
    req: HttpRequest,
    user_id: web::Path<u32>,
    mut payload: Multipart,
) -> actix_web::Result<HttpResponse> {
    log::debug!("Start update_avatar function");
    println!("update_userprofile_avatar");

    // 获取用户 id
    let user_id = user_id.into_inner();
    // token 验证
    let user_info = match Token::token_to_claims(req) {
        Ok(ok) => ok, 
        Err(err) => {
            log::error!("Error update_userprofile_avatar is token_to_claims");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };
    
    // token 验证
    if user_info.get_id() != user_id || user_info.verify().is_err() {
        log::info!("user_info.get_id() != user_id || user_info.verify().is_err()");
        return Ok(HttpResponse::BadRequest().body("Token verif Farild"));
    }
    println!("666");
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
    let url = convert_path_to_url(&file_path);
    // println!("{}", url);
    log::debug!("End update_avatar function");
    Ok(HttpResponse::Ok().body(serde_json::to_string(&url).unwrap()))
    // Ok(HttpResponse::Ok().body(serde_json::to_string(&convert_path_to_url(&file_path)).unwrap()))
}

// 修改路径的函数
fn convert_path_to_url(path: &str) -> String {
    // 假设所有的路径都遵循"./static/image/image-{id}.jpg"的格式
    // 1. 移除文件路径的'./static/image/'部分
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

#[derive(Debug, Serialize, Deserialize)]
struct AvatarUrl {
    user_id: u32,
    avatar_url: String,
}
#[post("/userprofile/update_avatar_url")]
pub async fn update_userprofile_avatar_url(
    req: HttpRequest,
    info: web::Json<AvatarUrl>,
) -> actix_web::Result<HttpResponse> {
    log::debug!("Start update_userprofile_avatar_url function");
    println!("update_userprofile_avatar_url");

    let user_info = match Token::token_to_claims(req) {
        Ok(ok) => ok, 
        Err(err) => {
            log::error!("Error update_userprofile_avatar_url is token_to_claims");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    // 获取用户名
    let AvatarUrl { user_id, avatar_url } = info.into_inner();
    println!("{}", avatar_url);

    // token 验证
    if user_info.get_id() != user_id || user_info.verify().is_err() {
        log::info!("user_info.get_id() != user_id || user_info.verify().is_err()");
        return Ok(HttpResponse::BadRequest().body("Token verif Farild"));
    }

    let my_pool = MysqlPool::instance();

    let query = format!("
        update user
        set avatar_url = '{}'
        where id = {};
    ", avatar_url, user_id);

    // 执行修改语句
    match my_pool.exec_drop(vec![query], &my_pool.read_write_txopts) {
        Ok(_) => {

        },
        Err(err) => {
            log::error!("Error update_userprofile_avatar_url executing my_pool");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    }

    log::debug!("End update_userprofile_avatar_url function");
    Ok(HttpResponse::Ok().body(serde_json::to_string("修改成功").unwrap()))
}
