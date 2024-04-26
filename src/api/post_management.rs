use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;

use crate::{common::token::*, database::mysql::*};

// 获取帖子的总数量
#[get("/post_management/get_user_total")]
pub async fn get_post_management_total(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    log::debug!("Start get_post_management_total function");
    // println!("---- get_user_management_total");

    // 验证 token
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("token verify error"));
    }

    let query = format!("select count(*) from post");

    let my_pool = MysqlPool::instance();

    let number: Vec<u32> = my_pool.exec(query, &my_pool.read_only_txopts).unwrap();

    log::debug!("End get_post_management_total function");
    Ok(HttpResponse::Ok().body(serde_json::to_string(&number[0]).unwrap()))
}

#[derive(Debug, Serialize)]
struct MyPost {
    pub id: u32,
    pub title: String,
    pub release_time: String,
    pub cover_url: String,
    pub content: String,
    pub user_id: u32,
}

struct MyReturnPost {
    pub id: u32,
    pub title: String,
    pub release_time: String,
    pub cover_url: String,
    pub content: String,
    pub user_id: u32,
    pub label_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReturnPost {
    pub id: u32,
    pub title: String,
    pub release_time: String,
    pub cover_url: String,
    pub content: String,
    pub user_id: u32,
    pub user_name: String,
    pub label_id: u32,
    pub label_name: String,
}

#[derive(Debug)]
struct MyUser {
    pub user_id: u32,
    pub user_name: String,
}

#[derive(Debug)]
struct MyLabel {
    pub label_id: u32,
    pub label_name: String,
}

// 获得文章列表
#[get("/post_management/get_post_list/{page}")]
pub async fn get_post_management_list(
    req: HttpRequest,
    page: web::Path<u32>,
) -> actix_web::Result<HttpResponse> {
    log::debug!("Start get_post_management_list function");
    let page = *page;

    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::Ok().body("token verify error"));
    }

    // 获取起时下标条数
    let start = (page - 1) * 10;
    let query = format!(
        "SELECT id, title, release_time, cover_url, content_url, user_id, label_id
        FROM post
        order by release_time desc
        LIMIT {}, 20",
        start
    );

    // 获取线程池，这个线程池为单例模式
    let my_pool = MysqlPool::instance();

    // 将查询的值映射到数结构体中
    let posts: Vec<MyReturnPost> = match my_pool.query_map(
        query,
        |(id, title, release_time, cover_url, content_url, user_id, label_id): (
            u32,
            String,
            String,
            String,
            String,
            u32,
            u32,
        )| {
            let content = fs::read_to_string(content_url)
                .expect("get_post_management_list: Failed fs::read_to_string content_url");
            MyReturnPost {
                id,
                title,
                release_time,
                cover_url,
                content,
                user_id,
                label_id,
            }
        },
        &my_pool.read_only_txopts,
    ) {
        Ok(result) => result,
        Err(err) => {
            // eprintln!("Error executing query: {:?}", err);
            log::error!("Error get_post_management_list executing query: {:?}", err);
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
    let users: Vec<MyUser> = match my_pool.query_map(
        query,
        |(user_id, user_name): (u32, String)| MyUser { user_id, user_name },
        &my_pool.read_only_txopts,
    ) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("Error get_post_management_list executing query: {:?}", err);
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

    // 查标签
    let lables_params = posts
        .iter()
        .map(|post| post.label_id.to_string())
        .collect::<Vec<String>>()
        .join(",");

    let query = format!(
        "
        select id, name 
        from label
        where id in ({})
    ",
        lables_params
    );

    let labels_name = my_pool
        .query_map(
            query,
            |(id, name): (u32, String)| MyLabel {
                label_id: id,
                label_name: name,
            },
            &my_pool.read_only_txopts,
        )
        .unwrap();

    let label_map: HashMap<u32, String> = labels_name
        .into_iter()
        .map(|label| (label.label_id, label.label_name))
        .collect();

    // 合并帖子和用户数据
    let result: Vec<ReturnPost> = posts
        .into_iter()
        .map(|post| ReturnPost {
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
            label_id: post.label_id,
            label_name: label_map.get(&post.label_id).cloned().unwrap(),
        })
        .collect();

    let post_jsons = serde_json::to_string(&result).map_err(|err| {
        // eprintln!("Error serializing response: {:?}", err);
        log::error!(
            "Error get_post_management_list serializing response: {:?}",
            err
        );
        actix_web::error::ErrorInternalServerError("Error serializing response")
    })?;

    log::debug!("End get_post_management_list function");
    Ok(HttpResponse::Ok().body(post_jsons))
}

#[derive(Debug, Serialize, Deserialize)]
struct TitleSearchPost {
    search_query: String,
    page: u32,
}

// 通过标题搜素文章
#[get("/post_management/search")]
pub async fn search_post(
    req: HttpRequest,
    info: web::Query<TitleSearchPost>,
) -> actix_web::Result<HttpResponse> {
    println!("---- search_post");

    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("token verify error"));
    }

    let TitleSearchPost { search_query, page } = info.into_inner();

    let start = (page - 1) * 20;
    // 获取线程池，这个线程池为单例模式
    let my_pool = MysqlPool::instance();

    // 查询总数
    let query = format!(
        "
        select count(*)
        from post
        where title like '%{}%';
    ",
        search_query
    );

    let number: Vec<u32> = my_pool.exec(query, &my_pool.read_only_txopts).unwrap();

    let number = number[0];

    if number == 0 {
        return Ok(HttpResponse::Ok().body(
            serde_json::to_string(&serde_json::json!({})).unwrap()
        ));
    }

    let query = format!(
        "SELECT id, title, release_time, cover_url, content_url, user_id, label_id
        FROM post
        where title like '%{}%'
        order by id desc
        LIMIT {}, 20;",
        search_query, start
    );

    // 将查询的值映射到数结构体中
    let posts: Vec<MyReturnPost> = my_pool
        .query_map(
            query,
            |(id, title, release_time, cover_url, content_url, user_id, label_id): (
                u32,
                String,
                String,
                String,
                String,
                u32,
                u32,
            )| {
                let content = fs::read_to_string(content_url)
                    .expect("get_post_management_list: Failed fs::read_to_string content_url");
                MyReturnPost {
                    id,
                    title,
                    release_time,
                    cover_url,
                    content,
                    user_id,
                    label_id,
                }
            },
            &my_pool.read_only_txopts,
        )
        .unwrap();

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
            log::error!("Error get_post_management_list executing query: {:?}", err);
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

    // 查标签
    let lables_params = posts
        .iter()
        .map(|post| post.label_id.to_string())
        .collect::<Vec<String>>()
        .join(",");

    let query = format!(
        "
        select id, name 
        from label
        where id in ({})
    ",
        lables_params
    );

    let labels_name = my_pool
        .query_map(
            query,
            |(id, name): (u32, String)| MyLabel {
                label_id: id,
                label_name: name,
            },
            &my_pool.read_only_txopts,
        )
        .unwrap();

    let label_map: HashMap<u32, String> = labels_name
        .into_iter()
        .map(|label| (label.label_id, label.label_name))
        .collect();

    // 合并帖子和用户数据
    let result: Vec<ReturnPost> = posts
        .into_iter()
        .map(|post| ReturnPost {
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
            label_id: post.label_id,
            label_name: label_map.get(&post.label_id).cloned().unwrap(),
        })
        .collect();

    let post_jsons = serde_json::to_string(&serde_json::json!({
        "total": number,
        "posts": result,
    }))
    .unwrap();

    Ok(HttpResponse::Ok().body(post_jsons))
}

#[derive(Debug, Serialize, Deserialize)]
struct LabelPost {
    label_id: u32,
    page: u32,
}

#[get("/post_management/label_post")]
pub async fn label_get_post(
    req: HttpRequest,
    info: web::Query<LabelPost>,
) -> actix_web::Result<HttpResponse> {
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("token verify error"));
    }

    let LabelPost { label_id, page } = info.into_inner();

    let start = (page - 1) * 20;
    // 获取线程池，这个线程池为单例模式
    let my_pool = MysqlPool::instance();

    // 查询总数
    let query = format!(
        "
        select count(*)
        from post
        where label_id = {};
    ",
        label_id
    );

    let number: Vec<u32> = my_pool.exec(query, &my_pool.read_only_txopts).unwrap();

    let number = number[0];
    
    if number == 0 {
        return Ok(HttpResponse::Ok().body(
            serde_json::to_string(&serde_json::json!({})).unwrap()
        ));
    }

    // 查询文章
    let query = format!(
        "
        select id, title, release_time, cover_url, content_url, user_id, label_id
        from post
        where label_id = {}
        order by id desc
        LIMIT {}, 20;
    ",
        label_id, start
    );

    // 将查询的值映射到数结构体中
    let posts: Vec<MyReturnPost> = my_pool
        .query_map(
            query,
            |(id, title, release_time, cover_url, content_url, user_id, label_id): (
                u32,
                String,
                String,
                String,
                String,
                u32,
                u32,
            )| {
                let content = fs::read_to_string(content_url)
                    .expect("get_post_management_list: Failed fs::read_to_string content_url");
                MyReturnPost {
                    id,
                    title,
                    release_time,
                    cover_url,
                    content,
                    user_id,
                    label_id,
                }
            },
            &my_pool.read_only_txopts,
        )
        .unwrap();

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
            log::error!("Error get_post_management_list executing query: {:?}", err);
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

    // 查标签
    let lables_params = posts
        .iter()
        .map(|post| post.label_id.to_string())
        .collect::<Vec<String>>()
        .join(",");

    let query = format!(
        "
        select id, name 
        from label
        where id in ({})
    ",
        lables_params
    );

    let labels_name = my_pool
        .query_map(
            query,
            |(id, name): (u32, String)| MyLabel {
                label_id: id,
                label_name: name,
            },
            &my_pool.read_only_txopts,
        )
        .unwrap();

    let label_map: HashMap<u32, String> = labels_name
        .into_iter()
        .map(|label| (label.label_id, label.label_name))
        .collect();

    // 合并帖子和用户数据
    let result: Vec<ReturnPost> = posts
        .into_iter()
        .map(|post| ReturnPost {
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
            label_id: post.label_id,
            label_name: label_map.get(&post.label_id).cloned().unwrap(),
        })
        .collect();

    let post_jsons = serde_json::to_string(&serde_json::json!({
        "total": number,
        "posts": result,
    }))
    .unwrap();

    Ok(HttpResponse::Ok().body(post_jsons))
}


#[derive(Debug, Serialize, Deserialize)]
struct TitleLabelPost {
    search_query: String,
    label_id: u32,
    page: u32,
}

#[get("/post_management/title_label_post")]
pub async fn title_label_get_post(
    req: HttpRequest,
    info: web::Query<TitleLabelPost>,
) -> actix_web::Result<HttpResponse> {

    println!("title_label_get_post");
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("token verify error"));
    }

    let TitleLabelPost { search_query, label_id, page } = info.into_inner();

    let start = (page - 1) * 20;
    // 获取线程池，这个线程池为单例模式
    let my_pool = MysqlPool::instance();

    // 查询总数
    let query = format!(
        "
        select count(*)
        from post
        where label_id = {} and title like '%{}%';
    ",
        label_id, search_query
    );

    let number: Vec<u32> = my_pool.exec(query, &my_pool.read_only_txopts).unwrap();

    let number = number[0];
    
    if number == 0 {
        return Ok(HttpResponse::Ok().body(
            serde_json::to_string(&serde_json::json!({})).unwrap()
        ));
    }

    // 查询文章
    let query = format!(
        "
        select id, title, release_time, cover_url, content_url, user_id, label_id
        from post
        where label_id = {} and title like '%{}%';
        LIMIT {}, 20;
    ",
        label_id, search_query, start
    );

    // 将查询的值映射到数结构体中
    let posts: Vec<MyReturnPost> = my_pool
        .query_map(
            query,
            |(id, title, release_time, cover_url, content_url, user_id, label_id): (
                u32,
                String,
                String,
                String,
                String,
                u32,
                u32,
            )| {
                let content = fs::read_to_string(content_url)
                    .expect("get_post_management_list: Failed fs::read_to_string content_url");
                MyReturnPost {
                    id,
                    title,
                    release_time,
                    cover_url,
                    content,
                    user_id,
                    label_id,
                }
            },
            &my_pool.read_only_txopts,
        )
        .unwrap();

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
            log::error!("Error get_post_management_list executing query: {:?}", err);
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

    // 查标签
    let lables_params = posts
        .iter()
        .map(|post| post.label_id.to_string())
        .collect::<Vec<String>>()
        .join(",");

    let query = format!(
        "
        select id, name 
        from label
        where id in ({})
    ",
        lables_params
    );

    let labels_name = my_pool
        .query_map(
            query,
            |(id, name): (u32, String)| MyLabel {
                label_id: id,
                label_name: name,
            },
            &my_pool.read_only_txopts,
        )
        .unwrap();

    let label_map: HashMap<u32, String> = labels_name
        .into_iter()
        .map(|label| (label.label_id, label.label_name))
        .collect();

    // 合并帖子和用户数据
    let result: Vec<ReturnPost> = posts
        .into_iter()
        .map(|post| ReturnPost {
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
            label_id: post.label_id,
            label_name: label_map.get(&post.label_id).cloned().unwrap(),
        })
        .collect();

    let post_jsons = serde_json::to_string(&serde_json::json!({
        "total": number,
        "posts": result,
    }))
    .unwrap();

    Ok(HttpResponse::Ok().body(post_jsons))
}
