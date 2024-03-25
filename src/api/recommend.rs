use actix_web::{get, web, HttpResponse};
use r2d2_mysql::mysql::{
    prelude::Queryable,
    TxOpts, IsolationLevel, AccessMode
};
use serde::{Deserialize, Serialize};
use std::fs; // 将 json 字符串解析为结构体

use crate::database::mysql::*;

// const IP_PORT: &str = "127.0.0.1:8082";

#[derive(Debug, Serialize)]
struct Number {
    number: u32,
}

// 推荐列表的总数
#[get("/recommend/totalnumbers")]
pub async fn get_recommend_post_total_numbers() -> actix_web::Result<HttpResponse> {
    // 获取线程池，这个线程池为单例模式
    let pool = MysqlPool::instance().lock()
        .expect("get_recommend_post_total_numbers: Failed get mysql pool lock");
    // 获取连接
    let mut connection = pool.get_connection()
        .expect("get_recommend_post_total_numbers: Failed get mysql connection");

    // 释放掉这个数据库连接池的锁
    drop(pool);

    // 设置事务的配置
    let opts = TxOpts::default()
        .set_with_consistent_snapshot(true) // 开启事务快照
        .set_isolation_level(Some(IsolationLevel::RepeatableRead))  // 设置事务的隔离级别
        .set_access_mode(Some(AccessMode::ReadOnly));   // 只允许可读

    // 开启事务
    let mut transaction = connection.start_transaction(opts)
        .expect("get_recommend_post_total_numbers: Failed start_transaction");

    // 事务查询帖子总数量的数据
    let query = "SELECT COUNT(*) FROM post;";
    let numbers: Vec<(u32,)> = transaction.exec(query, ())
        .expect("get_recommend_post_total_numbers: Failed exec total_numbers");

    let number = Number { number: numbers[0].0 };
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

// 获取推荐页中的第 page 页帖子列表
#[get("/recommend/postlist/{page}")]
pub async fn get_recommend_posts_list(_: web::Path<u32>) -> actix_web::Result<HttpResponse> {
    // println!("get_recommend_posts_list, {:?}", env::current_dir());
    // 获取线程池，这个线程池为单例模式
    let pool = MysqlPool::instance().lock()
        .expect("get_recommend_posts_list: Failed get mysql pool lock");
    // 获取连接
    let mut connection = pool.get_connection()
        .expect("get_recommend_posts_list: Failed get mysql connection");

    // 释放掉这个数据库连接池的锁
    drop(pool);

    // 设置事务的配置
    let opts = TxOpts::default()
        .set_with_consistent_snapshot(true) // 开启事务快照
        .set_isolation_level(Some(IsolationLevel::RepeatableRead))  // 设置事务的隔离级别
        .set_access_mode(Some(AccessMode::ReadOnly));   // 只允许可读

    // 开启事务
    let mut transaction = connection.start_transaction(opts)
        .expect("get_recommend_posts_list: Failed start_transaction");

    // 获取帖子列表
    let posts: Vec<Post> = transaction.query_map(
        "SELECT id, title, release_time, cover_url, content_url, user_id, user_name FROM post",
        |(id, title, release_time, cover_url, content_url, user_id, user_name)
        : (u32, String, String, String, String, u32, String)| {
            let content = fs::read_to_string(content_url)
                .expect("get_recommend_posts_list: Failed fs::read_to_string content_url");
            Post { id, title, release_time, cover_url, content, user_id, user_name }
        },
    ).expect("get_recommend_posts_list: Failed transaction.query_map");

    // 提交事务
    transaction.commit().expect("get_recommend_posts_list: Failed tarnsaction commit");

    // 将获取到的数据转换成 json 字符串
    let post_jsons = serde_json::to_string(&posts).expect("Failed to serialize posts");

    
    // 发送报文
    Ok(HttpResponse::Ok().body(post_jsons))
}
