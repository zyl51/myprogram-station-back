use actix_web::{get, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, HashMap};
use std::fs; // 将 json 字符串解析为结构体

use crate::{common::token::*, database::mysql::*};

#[derive(Debug, Serialize, Deserialize)]
struct UserRelationships {
    follower_id: u32,
    following_id: u32,
}

// 是否有关注关系
#[get("/follow/relationships")]
pub async fn get_follow_relationships(
    req: HttpRequest,
    info: web::Query<UserRelationships>,
) -> actix_web::Result<HttpResponse> {
    log::debug!("Start get_follow_relationships");
    println!("get_follow_relationships");
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("Failed is verif token"));
    }
    // println!("get_follow_relationships");

    let UserRelationships {
        follower_id,
        following_id,
    } = info.into_inner();

    let query = format!(
        "
        select count(*) 
        from follow
        where follower_id = {} and following_id = {};",
        follower_id, following_id
    );

    let my_pool = MysqlPool::instance();

    let numbers: Vec<(u32,)> = match my_pool.exec(query, &my_pool.read_only_txopts) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("Error get_follow_relationships my_pool exec: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    println!("follower_id: {}, following_id: {}, numbers[0].0: {}", 
        follower_id, following_id, numbers[0].0);
    if numbers[0].0 == 0 {
        let flag = false;
        return Ok(HttpResponse::Ok().body(serde_json::to_string(&flag).unwrap()));
    }

    log::debug!("End get_follow_relationships");
    let flag = true;
    Ok(HttpResponse::Ok().body(serde_json::to_string(&flag).unwrap()))
}

#[derive(Debug, Serialize)]
struct Number {
    number: u32,
}

// 关注列表的总数量
#[get("/follow/totalnumbers/{user_id}")]
pub async fn get_follow_post_total_numbers(
    req: HttpRequest,
    user_id: web::Path<u32>,
) -> actix_web::Result<HttpResponse> {
    log::info!("Start executing get_follow_post_total_numbers function");
    // println!("get_follow_post_total_numbers");
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("Failed is verif token"));
    }
    // println!("get_follow_post_total_numbers");
    let user_id = *user_id;
    // println!("{}", user_id);
    // 获取线程池，这个线程池为单例模式
    let my_pool = MysqlPool::instance();

    // 事务查询帖子总数量的数据
    let query = format!(
        "SELECT COUNT(*) FROM follow where follower_id = {};",
        user_id
    );

    log::info!("Executing MySQL statement: {:?}", query);
    let numbers: Vec<(u32,)> = match my_pool.exec(&query, &my_pool.read_only_txopts) {
        Ok(result) => result,
        Err(err) => {
            // eprintln!("Error executing query: {:?}", err);
            log::error!(
                "Error executing get_follow_post_total_numbers function query: {:?}",
                err
            );
            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    let number = numbers[0].0;

    let json_response = serde_json::to_string(&Number { number }).map_err(|err| {
        log::error!(
            "Error get_follow_post_total_numbers serializing response: {:?}",
            err
        );
        actix_web::error::ErrorInternalServerError("Error serializing response")
    })?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(json_response))
}

// 用于存储客户端寻求的用户 id 和 page 页数
#[derive(serde::Deserialize)]
struct FollowPost {
    user_id: u32,
    page: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct MyPost {
    pub id: u32,
    pub title: String,
    pub release_time: String,
    pub cover_url: String,
    pub content: String,
    pub user_id: u32,
}

#[derive(Debug)]
struct MyUser {
    pub user_id: u32,
    pub user_name: String,
}

// 获取自己关注的帖子列表
#[get("/follow/postlist")]
pub async fn get_follow_posts_list(
    req: HttpRequest,
    info: web::Query<FollowPost>,
) -> actix_web::Result<HttpResponse> {
    log::info!("Start get_follow_posts_list function");
    println!("get_follow_posts_list");
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("Failed is verif token"));
    }
    let FollowPost { user_id, page } = info.into_inner();
    // print!("{}, {} ", user_id, page);

    // 获取线程池，这个线程池为单例模式
    let my_pool = MysqlPool::instance();

    let start = (page - 1) * 10;
    let query = format!(
        "
            SELECT post.id, post.title, post.release_time, post.cover_url, 
                post.content_url, post.user_id FROM post
            JOIN follow ON post.user_id = follow.following_id 
            WHERE follow.follower_id = {}
            ORDER BY post.release_time DESC
            LIMIT {}, 10;
    ",
        user_id, start
    );

    // println!("user_id: {}, start: {}", user_id, start);

    // 将查询的值映射到数结构体中
    let posts: Vec<MyPost> = match my_pool.query_map(
        query,
        |(id, title, release_time, cover_url, content_url, user_id): (
            u32,
            String,
            String,
            String,
            String,
            u32,
        )| {
            let content =
                fs::read_to_string(content_url).expect("Error get_follow_posts_list: read content");

            MyPost {
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
            // eprintln!("Error get_follow_posts_list executing query: {:?}", err);
            log::error!("get_follow_posts_list executing query: {:?}", err);

            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    if posts.len() == 0 {
        return Ok(HttpResponse::Ok().body(serde_json::to_string(&posts).unwrap()));
    }

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
    let users: Vec<MyUser> = match my_pool.query_map(
        query,
        |(user_id, user_name): (u32, String)| MyUser { user_id, user_name },
        &my_pool.read_only_txopts,
    ) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("Error get_recommend_posts_list executing query: {:?}", err);
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

    let post_jsons = serde_json::to_string(&result).map_err(|err| {
        log::error!("Error serializing response: {:?}", err);
        actix_web::error::ErrorInternalServerError("Error serializing response")
    })?;

    log::info!("End get_follow_posts_list function");
    Ok(HttpResponse::Ok().body(post_jsons))
}
