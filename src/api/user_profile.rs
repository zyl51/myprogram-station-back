use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};
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
pub async fn get_userprofile_totalnumbers_collect(
    req: HttpRequest,
    user_id: web::Path<u32>,
) -> actix_web::Result<HttpResponse> {
    log::debug!("Start get_userprofile_totalnumbers_collect function");
    println!("get_userprofile_totalnumbers_collect");

    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("Token is failed"));
    }

    let user_id = *user_id;

    let query = format!(
        "select count(*) from collect_post where user_id = {};",
        user_id
    );

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
    let query = format!(
        "select post_id from collect_post where user_id = {};",
        user_id
    );
    let post_ids: Vec<u32> = match my_pool.exec(query, &my_pool.read_only_txopts) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("Error get_userprofile_collect_posts executing my_pool.exec");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    let query = format!(
        "SELECT id, title, release_time, cover_url, content_url, user_id
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

    let posts: Vec<MyPosts> = match my_pool.query_map(
        query,
        |(id, title, release_time, cover_url, content_url, user_id): (
            u32,
            String,
            String,
            String,
            String,
            u32,
        )| {
            let content = fs::read_to_string(content_url)
                .expect("get_userprofile_collect_posts: Failed fs::read_to_string content_url");
            MyPosts {
                id,
                title,
                release_time,
                cover_url,
                content,
                user_id,
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

     // 将用户的 id 提取出来并且去重
     let user_ids: HashSet<u32> = posts.iter().map(|post| post.user_id).collect();

     // 构建数据库的查询参数
     let params = user_ids
         .iter()
         .map(|user_id| user_id.to_string())
         .collect::<Vec<String>>()
         .join(",");
 
     // 构建查询语句
     let query = format!(
         "
         select id, name
         from user
         where id in ({});
     ",
         params
     );
 
     // 通过 user_id 和 user_id 和 user_name 查出来
     let users: Vec<MyUsers> = match my_pool.query_map(
         query,
         |(user_id, user_name): (u32, String)| MyUsers { user_id, user_name },
         &my_pool.read_only_txopts,
     ) {
         Ok(ok) => ok,
         Err(err) => {
             log::error!("Error get_userprofile_collect_posts executing query_map, err: {:?}", err);
             return Err(actix_web::error::ErrorInternalServerError(
                 "Internal Server Error",
             ));
         }
     };
 
     // 将用户数据映射到 HashMap 中
     let user_map: HashMap<u32, String> = users
         .into_iter()
         .map(|user| (user.user_id, user.user_name))
         .collect();
 
     // 合并帖子和用户数据
     let result: Vec<Post> = posts
         .into_iter()
         .map(|post| Post {
             id: post.id,
             title: post.title,
             release_time: post.release_time,
             cover_url: post.cover_url,
             content: post.content,
             user_id: post.user_id,
             user_name: user_map
                 .get(&post.user_id)
                 .cloned()
                 .unwrap_or_else(|| "编程驿站一份子".to_string()),
         })
         .collect();

    log::debug!("End get_userprofile_collect_posts function");
    Ok(HttpResponse::Ok().body(serde_json::to_string(&result).unwrap()))
}

#[derive(Debug)]
struct MyPosts {
    pub id: u32,
    pub title: String,
    pub release_time: String,
    pub cover_url: String,
    pub content: String,
    pub user_id: u32,
}

#[derive(Debug)]
struct MyUsers {
    user_id: u32,
    user_name: String,
}

// 获取该用户的所有帖子
#[get("/userprofile/posts")]
pub async fn get_userprofile_posts(
    info: web::Query<UserProfile>,
) -> actix_web::Result<HttpResponse> {
    log::info!("Satrt get_userprofile_posts function");
    let UserProfile { user_id, page } = info.into_inner();
    let start = (page - 1) * 10;

    // 获取线程池，这个线程池为单例模式
    let my_pool = MysqlPool::instance();

    // 构建查询语句
    let query = format!(
        "SELECT id, title, release_time, cover_url, content_url, user_id
        FROM post 
        WHERE user_id = {}
        LIMIT {}, 10",
        user_id, start
    );

    // 获取文章的信息
    let posts: Vec<MyPosts> = match my_pool.query_map(
        query,
        |(id, title, release_time, cover_url, content_url, user_id): (
            u32,
            String,
            String,
            String,
            String,
            u32,
        )| {
            let content = fs::read_to_string(content_url)
                .expect("get_userprofile_posts: Failed fs::read_to_string content_url");
            MyPosts {
                id,
                title,
                release_time,
                cover_url,
                content,
                user_id,
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

     // 将用户的 id 提取出来并且去重
     let user_ids: HashSet<u32> = posts.iter().map(|post| post.user_id).collect();

     // 构建数据库的查询参数
     let params = user_ids
         .iter()
         .map(|user_id| user_id.to_string())
         .collect::<Vec<String>>()
         .join(",");
 
     // 构建查询语句
     let query = format!(
         "
         select id, name
         from user
         where id in ({});
     ",
         params
     );
 
     // 通过 user_id 和 user_id 和 user_name 查出来
     let users: Vec<MyUsers> = match my_pool.query_map(
         query,
         |(user_id, user_name): (u32, String)| MyUsers { user_id, user_name },
         &my_pool.read_only_txopts,
     ) {
         Ok(ok) => ok,
         Err(err) => {
             log::error!("Error get_userprofile_posts executing query_map, err: {:?}", err);
             return Err(actix_web::error::ErrorInternalServerError(
                 "Internal Server Error",
             ));
         }
     };
 
     // 将用户数据映射到 HashMap 中
     let user_map: HashMap<u32, String> = users
         .into_iter()
         .map(|user| (user.user_id, user.user_name))
         .collect();
 
     // 合并帖子和用户数据
     let result: Vec<Post> = posts
         .into_iter()
         .map(|post| Post {
             id: post.id,
             title: post.title,
             release_time: post.release_time,
             cover_url: post.cover_url,
             content: post.content,
             user_id: post.user_id,
             user_name: user_map
                 .get(&post.user_id)
                 .cloned()
                 .unwrap_or_else(|| "编程驿站一份子".to_string()),
         })
         .collect();

    let post_jsons = serde_json::to_string(&json!(&result)).map_err(|err| {
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



// 获取用户的消息数量
#[get("/userprofile/message_total/{user_id}")]
pub async fn get_message_total(
    req: HttpRequest,
    user_id: web::Path<u32>,
) -> actix_web::Result<HttpResponse> {
    log::debug!("Start get_message_total function");
    // println!("--- get_message_total");

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

    // 查询用户的数量
    let query = format!("
        select count(*)
        from message 
        where recver_id = {};
    ", user_id);

    let my_pool = MysqlPool::instance();

    let numbers: Vec<u32> = match my_pool.exec(query, &my_pool.read_only_txopts) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("Error get message total, err:{:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    log::debug!("End get_message_total function");
    Ok(HttpResponse::Ok().body(serde_json::to_string(&numbers[0]).unwrap()))
}

#[derive(Debug, Serialize)]
struct Message {
    id: u32,
    sender_id: u32,
    recver_id: u32,
    sender_name: String,
    sender_avatar_url: String,
    post_id: u32,
    title: String,
    status: u32,
}

#[derive(Debug, Serialize)]
struct MyMessage {
    pub id: u32,
    pub sender_id: u32,
    pub recver_id: u32,
    pub post_id: u32,
    pub status: u32,
}

struct MyUser {
    user_id: u32,
    user_name: String,
    avatar_url: String,
}

struct MyPost {
    post_id: u32,
    title: String,
}

// 获取用户的消息
#[get("/userprofile/message/{user_id}")]
pub async fn get_message(
    req: HttpRequest,
    user_id: web::Path<u32>,
) -> actix_web::Result<HttpResponse> {
    log::debug!("Start get_message function");
    println!("--- get_message");

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

    // 查询用户的信息
    let query = format!(
        "
        select id, sender_id, recver_id, post_id, status
        from message 
        where recver_id = {}
        order by id desc;
    ",
        user_id
    );

    let my_pool = MysqlPool::instance();
    let my_messages: Vec<MyMessage> = match my_pool.query_map(
        query,
        |(id, sender_id, recver_id, post_id, status): (u32, u32, u32, u32, u32)| MyMessage {
            id,
            sender_id,
            recver_id,
            post_id,
            status,
        },
        &my_pool.read_only_txopts,
    ) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("Error get message err:{:?}", err);
            return Ok(HttpResponse::InternalServerError().body("Internal Server Error"));
        }
    };

    if my_messages.len() == 0 {
        return Ok(HttpResponse::Ok().body(serde_json::to_string(&my_messages).unwrap()))
    }

    // 查询用户名
    let usernames_params = my_messages
        .iter()
        .map(|message| message.sender_id.to_string())
        .collect::<Vec<String>>()
        .join(",");

    // 构建查询语句
    let query = format!(
        "
        select id, name, avatar_url
        from user
        where id in ({});
    ",
        usernames_params
    );

    // 通过 user_id 和 user_id 和 user_name 查出来
    let users: Vec<MyUser> = match my_pool.query_map(
        query,
        |(user_id, user_name, avatar_url): (u32, String, String)| MyUser {
            user_id,
            user_name,
            avatar_url,
        },
        &my_pool.read_only_txopts,
    ) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("Error get message executing query: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    // 将用户数据映射到 HashMap 中
    let user_map: HashMap<u32, (String, String)> = users
        .into_iter()
        .map(|user| (user.user_id, (user.user_name, user.avatar_url)))
        .collect();

    let titles_params = my_messages
        .iter()
        .map(|message| message.post_id.to_string())
        .collect::<Vec<String>>()
        .join(",");

    // 构建查询语句
    let query = format!(
        "
        select id, title
        from post
        where id in ({});
    ",
        titles_params
    );

    // 通过 user_id 和 user_id 和 user_name 查出来
    let posts: Vec<MyPost> = match my_pool.query_map(
        query,
        |(post_id, title): (u32, String)| MyPost { post_id, title },
        &my_pool.read_only_txopts,
    ) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("Error get message executing query: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    // 将帖子数据映射到 HashMap 中
    let post_map: HashMap<u32, String> = posts
        .into_iter()
        .map(|post| (post.post_id, post.title))
        .collect();

    // 合并帖子和用户数据
    let result: Vec<Message> = my_messages
        .into_iter()
        .map(|message| Message {
            id: message.id,
            sender_id: message.sender_id,
            recver_id: message.recver_id,
            sender_name: user_map
                .get(&message.sender_id)
                .cloned()
                .unwrap_or_else(|| ("编程驿站一份子".to_string(), "".to_string()))
                .0,
            sender_avatar_url: user_map
                .get(&message.sender_id)
                .cloned()
                .unwrap_or_else(|| ("编程驿站一份子".to_string(), "".to_string()))
                .1,
            post_id: message.post_id,
            title: post_map
                .get(&message.post_id)
                .cloned()
                .unwrap_or_else(|| "编程驿站的小文章".to_string()),
            status: message.status,
        })
        .collect();

    let message_jsons = serde_json::to_string(&result).map_err(|err| {
        // eprintln!("Error serializing response: {:?}", err);
        log::error!("Error get message serializing response: {:?}", err);
        actix_web::error::ErrorInternalServerError("Error serializing response")
    })?;

    log::debug!("End get_message function");
    Ok(HttpResponse::Ok().body(message_jsons))
}

// 删除消息
#[post("/userprofile/update_message/{message_id}")]
pub async fn update_message_read(
    req: HttpRequest,
    message_id: web::Path<u32>,
) -> actix_web::Result<HttpResponse> {
    log::debug!("Start update_message_read function");
    println!("delete message");

    let message_id = message_id.into_inner();

    let my_pool = MysqlPool::instance();

    // 查询用户 id
    let query = format!("select recver_id from message where id = {}", message_id);

    let user_id: Vec<u32> = match my_pool.exec(query, &my_pool.read_only_txopts) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("update message read: {:?}", err);
            return Ok(HttpResponse::InternalServerError().body("Internal Server Error"));
        }
    };

    // 没有此消息
    if user_id.len() == 0 {
        return Ok(HttpResponse::BadRequest().body("Error request"));
    }

    // 获取用户 id
    let user_id = user_id[0];

    // token 验证
    let user_info = match Token::token_to_claims(req) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("Error update_message_read is token_to_claims");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    // token 验证
    if user_info.get_id() != user_id || user_info.verify().is_err() {
        log::info!("user_info.get_id() != user_id || user_info.verify().is_err()");
        return Ok(HttpResponse::BadRequest().body("Token verif Farild"));
    }

    let query = format!(
        "
        delete from message
        where id = {};
    ",
        message_id
    );

    match my_pool.exec_drop(vec![query], &my_pool.read_write_txopts) {
        Ok(_) => {
            log::info!("update message my_pool.exec_drop");
        }
        Err(err) => {
            log::error!("Error update message read: {:?}", err);
        }
    };

    log::debug!("End update_message_read function");
    Ok(HttpResponse::Ok().body(serde_json::to_string("删除成功").unwrap()))
}
