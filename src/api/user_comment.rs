use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{common::token::*, database::mysql::*};

#[derive(Debug)]
struct MyUserComment {
    id: u32,
    post_id: u32,
    user_id: u32,
    release_time: String,
    comment: String,
}

#[derive(Debug)]
struct MyUser {
    user_id: u32,
    username: String,
    avatar_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserComment {
    id: u32,
    post_id: u32,
    user_id: u32,
    username: String,
    avatar_url: String,
    release_time: String,
    comment: String,
}

// 获取帖子的评论
#[get("/user_comment/{post_id}")]
pub async fn get_user_comment(post_id: web::Path<u32>) -> actix_web::Result<HttpResponse> {
    log::debug!("Start get_user_comment function ");
    // println!("get_user_comment");
    let post_id = *post_id;

    let my_pool = MysqlPool::instance();
    let query = format!(
        "
        select id, post_id, user_id, release_time, comment
        from user_comment
        where post_id = {}
        ORDER BY release_time DESC;
    ",
        post_id
    );

    // 将查询的值映射到数结构体中
    let user_comments: Vec<MyUserComment> = match my_pool.query_map(
        query,
        |(id, post_id, user_id, release_time, comment): (u32, u32, u32, String, String)| {
            MyUserComment {
                id,
                post_id,
                user_id,
                release_time,
                comment,
            }
        },
        &my_pool.read_only_txopts,
    ) {
        Ok(result) => result,
        Err(err) => {
            // eprintln!("Error get_follow_posts_list executing query: {:?}", err);
            log::error!("get_user_comment executing query: {:?}", err);

            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    let user_ids_params = user_comments
        .iter()
        .map(|user_comment| user_comment.user_id.to_string())
        .collect::<Vec<String>>()
        .join(",");

    // 查询用户数据
    let query = format!(
        "
        select id, name, avatar_url
        from user
        where id in ({});
    ",
        user_ids_params
    );

    let users = match my_pool.query_map(
        query,
        |(id, username, avatar_url): (u32, String, String)| MyUser {
            user_id: id,
            username,
            avatar_url,
        },
        &my_pool.read_only_txopts,
    ) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("Error get username and avatar err:{:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    // 将值映射出来
    let user_map: HashMap<u32, (String, String)> = users
        .iter()
        .map(|user| {
            (
                user.user_id,
                (user.username.clone(), user.avatar_url.clone()),
            )
        })
        .collect();

    // 将这个值输出到结构体构建查询
    let user_comments: Vec<UserComment> = user_comments
        .into_iter()
        .map(|user_comment| UserComment {
            id: user_comment.id,
            post_id: user_comment.post_id,
            user_id: user_comment.user_id,
            username: user_map
                .get(&user_comment.user_id)
                .cloned()
                .unwrap_or_else(|| ("编程驿站一份子".to_string(), String::from("")))
                .0,
            avatar_url: user_map
                .get(&user_comment.user_id)
                .cloned()
                .unwrap_or_else(|| (String::from(""), AVATAR_URL.to_string()))
                .1,
            release_time: user_comment.release_time,
            comment: user_comment.comment,
        })
        .collect();

    let user_comments_jsons = serde_json::to_string(&user_comments).map_err(|err| {
        log::error!("Error serializing response: {:?}", err);
        actix_web::error::ErrorInternalServerError("Error serializing response")
    })?;

    log::debug!("End get_user_comment function ");
    Ok(HttpResponse::Ok().body(user_comments_jsons))
}

#[derive(Debug, Serialize, Deserialize)]
struct SubmitComment {
    post_id: u32,
    user_id: u32,
    username: String,
    avatar_url: String,
    release_time: String,
    comment: String,
}

// 提交用户评论
#[post("/user_comment/submit")]
pub async fn submit_comment(
    req: HttpRequest,
    info: web::Json<SubmitComment>,
) -> actix_web::Result<HttpResponse> {
    log::debug!("Satrt submit_comment");
    println!("submit_comment");
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("Failed is verif token"));
    }
    let SubmitComment {
        post_id,
        user_id,
        username,
        avatar_url,
        release_time,
        comment,
    } = info.into_inner();

    // 插入数据
    let my_pool = MysqlPool::instance();

    // 查询帖对应的作者
    let query = format!(
        "
        select user_id
        from post
        where id = {};
    ",
        post_id
    );

    let recver_id: Vec<u32> = match my_pool.exec(query, &my_pool.read_only_txopts) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("Error add_collect my_pool.exec: {:?}", err);
            return Ok(HttpResponse::InternalServerError().body("Internal Server Error"));
        }
    };

    if recver_id.len() == 0 {
        return Ok(HttpResponse::BadRequest().body("Not this post"));
    }

    let recver_id = recver_id[0];

    // 插入评论
    let query = format!(
        "
        INSERT INTO
            user_comment (
                post_id, user_id, username, avatar_url, release_time, comment
            )
        VALUES (
            {}, {}, '{}', '{}', '{}', '{}'
        );
    ",
        post_id, user_id, username, avatar_url, release_time, comment
    );

    let query2 = format!(
        "
        insert IGNORE into message
            (sender_id, recver_id, post_id, status)
        VALUES
            ({}, {}, {}, 3);
    ",
        user_id, recver_id, post_id
    );

    match my_pool.exec_drop(vec![query2], &my_pool.read_write_txopts) {
        Ok(_) => {
            log::info!("submit comment executing exec_drop successful");
        }
        Err(err) => {
            log::error!("Error submit comment executing exec_drop: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    let comment_id = match my_pool.query_drop(&query, &my_pool.read_write_txopts) {
        Ok(ok) => {
            log::info!("submit_comment executing successful");
            ok
        }
        Err(err) => {
            log::error!("Error query_drop");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    log::debug!("End submit_comment");
    Ok(HttpResponse::Ok().body(serde_json::to_string(&comment_id).unwrap()))
}

#[derive(Debug, Deserialize)]
struct Number {
    comment_id: u32,
}

// 删除用户评论
#[post("/user_comment/delete")]
pub async fn delete_comment(
    req: HttpRequest,
    comment_id: web::Json<Number>,
) -> actix_web::Result<HttpResponse> {
    log::debug!("Start delete_comment function");
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("Failed is verif token"));
    }
    // println!("delete_comment");
    let Number { comment_id } = comment_id.into_inner();
    let my_pool = MysqlPool::instance();
    let query = format!(
        "
        delete from user_comment
        where id = {};
    ",
        comment_id
    );

    match my_pool.exec_drop(vec![query], &my_pool.read_write_txopts) {
        Ok(_) => {
            log::info!("executing delete_comment of my_pool exec_drop successful");
        }
        Err(err) => {
            log::error!("executing delete_comment of my_pool exec_drop: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    Ok(HttpResponse::Ok().body(serde_json::to_string("删除成功").unwrap()))
}
