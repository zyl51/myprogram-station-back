use actix_multipart::Multipart;
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use futures::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use uuid::Uuid;

use crate::{
    common::{config::*, token::*},
    database::mysql::*,
};

// const IP_PORT: &str = "127.0.0.1:8082";

// 上传帖子封面的函数
#[post("/cover/submit_cover")]
async fn submit_cover(req: HttpRequest, mut payload: Multipart) -> actix_web::Result<HttpResponse> {
    log::debug!("Start submit_cover function");
    println!("submit_cover");
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
        file_path = format!("./static/cover/cover-{}.jpg", Uuid::new_v4());
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
    log::debug!("End submit_cover function");
    let url = convert_path_to_url(&file_path);
    // println!("{}", url);
    Ok(HttpResponse::Ok().body(url))
    // Ok(HttpResponse::Ok().body(serde_json::to_string(&convert_path_to_url(&file_path)).unwrap()))
}

// 修改路径的函数
fn convert_path_to_url(path: &str) -> String {
    // 假设所有的路径都遵循"./static/cover/cover-{id}.jpg"的格式
    // 1. 移除文件路径的'./static/cover/'部分
    let trimmed_path = path.trim_start_matches("./static/cover/");

    // 2. 移除文件扩展名'.jpg'
    let without_extension = trimmed_path.trim_end_matches(".jpg");

    // 3. 提取ID（这一步在这个简单示例中可能不是必需的，取决于你是否需要对ID做进一步处理）
    let id = without_extension.trim_start_matches("cover-");

    let ip = Config::instance().server.host.clone();
    let port = Config::instance().server.port;
    // 4. 拼接新的URL
    let new_url = format!("https://{}:{}/api/cover/{}", ip, port, id);

    new_url
}

// 封面获取函数
#[get("/cover/{post_id}")]
pub async fn get_cover(post_id: web::Path<String>) -> actix_web::Result<HttpResponse> {
    log::info!("Start get_cover function");
    // println!("get_cover");
    let post_id = (*post_id).clone();
    // https://127.0.0.1:8082/api/cover/dae497c5-417a-43ca-a915-9e0c6cb42a1e
    let cover_path = format!("./static/cover/cover-{}.jpg", post_id);

    let cover_data = match fs::read(cover_path) {
        Ok(bytes) => bytes,
        Err(err) => {
            log::error!("Error get_cover read cover_path: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    log::info!("End get_cover function");
    // 返回包含图片数据的 HTTP 响应
    Ok(HttpResponse::Ok()
        .content_type("image/jpeg") // 指定图片的 MIME 类型
        .body(cover_data))
}

// 获取单个帖子
#[get("/post/{post_id}")]
pub async fn get_post(post_id: web::Path<u32>) -> actix_web::Result<HttpResponse> {
    log::info!("Start get_post function");
    let post_id = *post_id;

    // 获取线程池，这个线程池为单例模式
    let my_pool = MysqlPool::instance();
    // 获取起时下标条数
    let query = format!(
        "SELECT id, title, release_time, 
        cover_url, content_url, user_id, user_name FROM post where id = {}",
        post_id
    );

    // 获取帖子列表
    let post: Vec<Post> = match my_pool.query_map(
        query,
        |(id, title, release_time, cover_url, content_url, user_id, user_name): (
            u32,
            String,
            String,
            String,
            String,
            u32,
            String,
        )| {
            let content = fs::read_to_string(content_url)
                .expect("Error get_post: Failed fs::read_to_string content_url");
            Post {
                id,
                title,
                release_time,
                cover_url,
                content,
                user_id,
                user_name,
            }
        },
        &my_pool.read_only_txopts,
    ) {
        Ok(result) => result,
        Err(err) => {
            // eprintln!("get_post: Error query_map query: {:?}", err);
            log::error!("get_post: Error query_map query: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    // 没有这个帖子
    if post.len() == 0 {
        return Ok(HttpResponse::BadRequest().body(serde_json::to_string("没有此帖子").unwrap()));
    }

    // 解析错误
    let post_jsons = serde_json::to_string(&post[0]).map_err(|err| {
        // eprintln!("get_post: Error serializing response: {:?}", err);
        log::error!("get_post: Error serializing response: {:?}", err);
        actix_web::error::ErrorInternalServerError("Error serializing response")
    })?;

    log::info!("End get_post function");
    Ok(HttpResponse::Ok().body(post_jsons))
}

#[derive(Debug, Serialize, Deserialize)]
struct PostId {
    post_id: u32,
}
// 删除一个帖子
#[post("/delete_post")]
pub async fn delete_post(
    req: HttpRequest,
    post_id: web::Json<PostId>,
) -> actix_web::Result<HttpResponse> {
    log::debug!("Start delete_post function");
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("Failed is verif token"));
    }
    // println!("delete_post");
    // return Ok(HttpResponse::Ok().body(serde_json::to_string("删除成功").unwrap()));
    let PostId { post_id } = post_id.into_inner();

    let my_pool = MysqlPool::instance();

    // 删除收藏的用户
    let delete_collect = format!(
        "
        delete from collect_post
        where post_id = {};
    ",
        post_id
    );

    // 删除用户的评论
    let delete_user_comment = format!(
        "
        delete from user_comment
        where post_id = {};
    ",
        post_id
    );

    // 删除对应的帖子
    let delete_post = format!(
        "
        delete from post
        where id = {};
    ",
        post_id
    );

    let querys = vec![delete_collect, delete_user_comment, delete_post];

    match my_pool.exec_drop(querys, &my_pool.read_write_txopts) {
        Ok(_) => {
            log::info!("executing delete_post of my_pool exec_drop successful");
        }
        Err(err) => {
            log::error!("executing delete_post of my_pool exec_drop: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    log::debug!("End delete_post function");
    Ok(HttpResponse::Ok().body(serde_json::to_string("删除成功").unwrap()))
}

#[derive(Debug, Serialize, Deserialize)]
struct SubmitPost {
    title: String,
    cover_url: String,
    content: String,
    user_id: u32,
    user_name: String,
}

// 上传一个帖子
#[post("/post/submit_post")]
pub async fn submit_post(
    req: HttpRequest,
    info: web::Json<SubmitPost>,
) -> actix_web::Result<HttpResponse> {
    log::debug!("Start submit_post function");
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("Failed is verif token"));
    }
    // println!("submit_post");
    let SubmitPost {
        title,
        cover_url,
        content,
        user_id,
        user_name,
    } = info.into_inner();

    // 写入文件
    let uu_id = Uuid::new_v4();
    let content_url = format!("./static/content/content-{}.md", uu_id);
    match fs::write(content_url.clone(), content) {
        Ok(_) => {
            log::info!("submit_post fs::write successful");
        }
        Err(err) => {
            log::error!("Error submit_post fs::write: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    }

    // 执行插入语句
    let query = format!(
        "
        insert into post
            (title, release_time, cover_url, content_url, user_id, user_name)
        VALUES
            ('{}', NOW(), '{}', '{}', {}, '{}');
    ",
        title, cover_url, content_url, user_id, user_name
    );

    let my_pool = MysqlPool::instance();
    let post_id = match my_pool.query_drop(&query, &my_pool.read_write_txopts) {
        Ok(ok) => {
            log::info!("submit_post exec_drop successful");
            ok
        }
        Err(err) => {
            log::error!("Error submit_post executing query: {:?}", err);
            return Ok(HttpResponse::InternalServerError()
                .body(serde_json::to_string("服务器繁忙").unwrap()));
        }
    };

    log::debug!("End submit_post function");
    Ok(HttpResponse::Ok().body(serde_json::to_string(&post_id).unwrap()))
}
