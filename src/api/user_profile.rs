use actix_web::{get, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs; // 将 json 字符串解析为结构体

use crate::{common::token::Token, database::mysql::*};

// const IP_PORT: &str = "127.0.0.1:8082";

#[derive(Debug, Serialize)]
struct Number {
    number: u32,
}

// 创建一个帖子的结构体，用来发送数据
#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub avatar_url: String,
    pub follower_count: u64,
    pub fans: u64,
}

// 获取单个用户
#[get("/userprofile/user/{user_id}")]
pub async fn get_userprofile_user(user_id: web::Path<u32>) -> actix_web::Result<HttpResponse> {
    log::info!("Start get_userprofile_user function");
    let user_id = *user_id;

    // println!("get_userprofile_user, user_id: {}", user_id);

    // 获取线程池，这个线程池为单例模式
    let my_pool = MysqlPool::instance();

    let query = format!(
        "SELECT id, name, avatar_url FROM user WHERE id = {};",
        user_id
    );
    let user: Vec<(u32, String, String)> = match my_pool.exec(query, &my_pool.read_only_txopts) {
        Ok(result) => result,
        Err(err) => {
            log::error!("Error get_userprofile_user exec query(user): {:?}", err);
            // eprintln!("get_user: Error query_map query: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    // 查询用户的粉丝数量
    let query = format!(
        "SELECT follower_count, fans FROM user_stats WHERE user_id = {};",
        user_id
    );
    let follow_fans: Vec<(u64, u64)> = match my_pool.exec(query, &my_pool.read_only_txopts) {
        Ok(result) => result,
        Err(err) => {
            log::error!(
                "Error get_userprofile_user exec query(follow_fans): {:?}",
                err
            );
            // eprintln!("get_user: Error query_map query: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    // println!("{}", user[0].2.len());
    let avatar_url: String = if user[0].2.len() <= 1 {
        AVATAR_URL.to_string()
    } else {
        user[0].2.clone()
    };

    let user_profile = User {
        id: user_id,
        name: user[0].1.clone(),
        avatar_url: avatar_url,
        follower_count: follow_fans[0].0,
        fans: follow_fans[0].1,
    };

    // println!("user_id: {}, {:?}", user_id, user);

    let post_jsons = serde_json::to_string(&user_profile).map_err(|err| {
        // eprintln!("Error serializing response: {:?}", err);
        log::error!("Error serializing response: {:?}", err);
        actix_web::error::ErrorInternalServerError("Error serializing response")
    })?;

    log::info!("End get_userprofile_user function");
    Ok(HttpResponse::Ok().body(post_jsons))
}

// 获取个用户的最大帖子的总数量
#[get("/userprofile/totalnumbers/{user_id}")]
pub async fn get_userprofile_post_total_numbers(
    user_id: web::Path<u32>,
) -> actix_web::Result<HttpResponse> {
    log::info!("Start get_userprofile_post_total_numbers function");
    // println!("get_userprofile_post_total_numbers: {}", user_id);
    let user_id = *user_id;
    // 获取线程池，这个线程池为单例模式
    let my_pool = MysqlPool::instance();

    // 事务查询帖子总数量的数据
    let query = format!("SELECT COUNT(*) FROM post where user_id = {};", user_id);
    let numbers: Vec<(u32,)> = match my_pool.exec(&query, &my_pool.read_only_txopts) {
        Ok(result) => result,
        Err(err) => {
            log::error!(
                "Error get_userprofile_post_total_numbers executing query: {:?}",
                err
            );
            // eprintln!("Error executing query: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    let number = numbers[0].0;

    let json_response = serde_json::to_string(&Number { number }).map_err(|err| {
        // eprintln!("Error serializing response: {:?}", err);
        log::error!(
            "Error get_userprofile_post_total_numbers serializing response: {:?}",
            err,
        );
        actix_web::error::ErrorInternalServerError("Error serializing response")
    })?;

    log::info!("End get_userprofile_post_total_numbers function");
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(json_response))
}

// 获取用户收藏的总数量
#[get("/userprofile/totalnumbers_collect/{user_id}")]
pub async fn get_userprofile_totalnumbers_collect(req: HttpRequest, user_id: web::Path<u32>) -> actix_web::Result<HttpResponse> {
    log::debug!("Start get_userprofile_totalnumbers_collect function");
    println!("get_userprofile_totalnumbers_collect");
    
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("Token is failed"));
    }

    let user_id = *user_id;

    let query = format!("select count(*) from collect_post where user_id = {};", user_id);

    let my_pool = MysqlPool::instance();
    let number: Vec<u32> = match my_pool.exec(query, &my_pool.read_only_txopts) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("Error collect numbers executing my_pool.exec");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    log::debug!("End get_userprofile_totalnumbers_collect function");
    Ok(HttpResponse::Ok().body(serde_json::to_string(&number[0]).unwrap()))
}

#[derive(Debug, Serialize, Deserialize)]
struct UserProfile {
    user_id: u32,
    page: u32,
}

// 获取用户收藏的内容
#[get("/userprofile/collect_posts")]
pub async fn get_userprofile_collect_posts(
    req: HttpRequest,
    info: web::Query<UserProfile>,
) -> actix_web::Result<HttpResponse> {
    log::debug!("Start get_userprofile_collect_posts function");
    println!("get_userprofile_collect_posts");
    // Token 验证失败
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("Failed is verif token"));
    }

    let UserProfile { user_id, page } = info.into_inner();

    let start = (page - 1) * 10;

    let my_pool = MysqlPool::instance();
    let query = format!("select post_id from collect_post where user_id = {};", user_id);
    let post_ids: Vec<u32> = match my_pool.exec(query, &my_pool.read_only_txopts) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("Error get_userprofile_collect_posts executing my_pool.exec");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    let query = format!(
        "SELECT id, title, release_time, cover_url, content_url, user_id, user_name
        FROM post 
        WHERE id IN ({})
        LIMIT {}, 10",
        post_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(", "),
        start
    );

    let posts: Vec<Post> = match my_pool.query_map(
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
                .expect("get_userprofile_collect_posts: Failed fs::read_to_string content_url");
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
            log::error!(
                "get_userprofile_collect_posts: Error query_map query: {:?}",
                err
            );
            // eprintln!("get_search: Error query_map query: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    log::debug!("End get_userprofile_collect_posts function");
    Ok(HttpResponse::Ok().body(serde_json::to_string(&posts).unwrap()))
}

// 获取该用户的所有帖子
#[get("/userprofile/posts")]
pub async fn get_userprofile_posts(
    info: web::Query<UserProfile>,
) -> actix_web::Result<HttpResponse> {
    log::info!("Satrt get_userprofile_posts function");
    let UserProfile { user_id, page } = info.into_inner();
    let start = (page - 1) * 10;
    // println!(
    //     "get_userprofile_posts: {}, page: {}, start: {}",
    //     user_id, page, start
    // );

    // 获取线程池，这个线程池为单例模式
    let my_pool = MysqlPool::instance();

    let query = format!(
        "SELECT id, title, release_time, cover_url, content_url, user_id, user_name
        FROM post 
        WHERE user_id = {}
        LIMIT {}, 10",
        user_id, start
    );

    let posts: Vec<Post> = match my_pool.query_map(
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
                .expect("get_userprofile_posts: Failed fs::read_to_string content_url");
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
            log::error!("get_userprofile_posts: Error query_map query: {:?}", err);
            // eprintln!("get_search: Error query_map query: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    let post_jsons = serde_json::to_string(&json!(&posts)).map_err(|err| {
        log::error!(
            "Error get_userprofile_posts serializing response: {:?}",
            err
        );
        // eprintln!("Error serializing response: {:?}", err);
        actix_web::error::ErrorInternalServerError("Error serializing response")
    })?;

    log::info!("End get_userprofile_posts function");
    Ok(HttpResponse::Ok().body(post_jsons))
}
