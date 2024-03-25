use actix_web::{get, web, HttpResponse};
use r2d2_mysql::mysql::{prelude::Queryable, AccessMode, IsolationLevel, TxOpts};
use serde::{Deserialize, Serialize};
use std::fs; // 将 json 字符串解析为结构体

use crate::database::mysql::*;

#[derive(Debug, Serialize)]
struct Number {
    number: u32,
}

#[get("/follow/totalnumbers/{user_id}")]
pub async fn get_follow_post_total_numbers(
    user_id: web::Path<u32>,
) -> actix_web::Result<HttpResponse> {
    // println!("{}", user_id);
    // 获取线程池，这个线程池为单例模式
    let pool = MysqlPool::instance()
        .lock()
        .expect("get_follow_post_total_numbers: Failed get mysql pool lock");
    // 获取连接
    let mut connection = pool
        .get_connection()
        .expect("get_follow_post_total_numbers: Failed get mysql connection");

    // 释放掉这个数据库连接池的锁
    drop(pool);

    // 设置事务的配置
    let opts = TxOpts::default()
        .set_with_consistent_snapshot(true) // 开启事务快照
        .set_isolation_level(Some(IsolationLevel::RepeatableRead)) // 设置事务的隔离级别
        .set_access_mode(Some(AccessMode::ReadOnly)); // 只允许可读

    // 开启事务
    let mut transaction = connection
        .start_transaction(opts)
        .expect("get_follow_post_total_numbers: Failed start_transaction");

    // 事务查询帖子总数量的数据
    let query = format!("SELECT COUNT(*) FROM post where user_id = {};", user_id);
    let numbers: Vec<(u32,)> = transaction
        .exec(query, ())
        .expect("get_follow_post_total_numbers: Failed exec total_numbers");
    // println!("{:?}", numbers);

    let number = Number {
        number: numbers[0].0,
    };
    let json_response = serde_json::to_string(&number)?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(json_response))
}

// 创建一个帖子的结构体，用来发送数据
#[derive(Debug, Deserialize, Serialize)]
struct Post {
    id: u32,
    title: String,
    release_time: String,
    cover_url: String,
    content: String,
    user_id: u32,
    user_name: String,
}

// 用于存储客户端寻求的用户 id 和 page 页数
#[derive(serde::Deserialize)]
struct FollowPost {
    user_id: u32,
    page: u32,
}

// 获取自己关注的帖子列表
#[get("/follow/postlist")]
pub async fn get_follow_posts_list(
    info: web::Query<FollowPost>,
) -> actix_web::Result<HttpResponse> {
    let FollowPost { user_id, page } = info.into_inner();
    print!("{}, {} ", user_id, page);

    // 获取线程池，这个线程池为单例模式
    let pool = MysqlPool::instance().lock()
        .expect("get_follow_posts_list: Failed get mysql pool lock");
    // 获取连接
    let mut connection = pool.get_connection()
        .expect("get_follow_posts_list: Failed get mysql connection");

    // 释放掉这个数据库连接池的锁
    drop(pool);

    // 设置事务的配置
    let opts = TxOpts::default()
        .set_with_consistent_snapshot(true) // 开启事务快照
        .set_isolation_level(Some(IsolationLevel::RepeatableRead)) // 设置事务的隔离级别
        .set_access_mode(Some(AccessMode::ReadOnly)); // 只允许可读

    // 开启事务
    let mut transaction = connection
        .start_transaction(opts)
        .expect("get_follow_posts_list: Failed start_transaction");

    let start = (page - 1) * 10;
    let query = format!("
            SELECT post.id, post.title, post.release_time, post.cover_url, post.content_url, post.user_id, post.user_name FROM post
            JOIN follow ON post.user_id = follow.following_id 
            WHERE follow.follower_id = {}
            ORDER BY post.release_time DESC
            LIMIT {}, 10;
    ", user_id, start);

    // let posts: Vec<String> = transaction.exec(query, ()).unwrap();


    let posts: Vec<Post> = transaction.query_map(
        query,
        |(id, title, release_time, cover_url, content_url, user_id, user_name)
        : (u32, String, String, String, String, u32, String)| {
            let content = fs::read_to_string(content_url)
                .expect("get_follow_posts_list: Failed fs::read_to_string content_url");
            Post { id, title, release_time, cover_url, content, user_id, user_name }
        },
    ).expect("get_follow_posts_list: Failed transaction.query_map");

    // println!("----{:?}", posts);

    let post_jsons = serde_json::to_string(&posts).expect("Failed to serialize posts");

    Ok(HttpResponse::Ok().body(post_jsons))
}
