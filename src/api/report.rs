use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{common::token::*, database::mysql::*};

#[derive(Debug, Deserialize)]
struct SubmitReport {
    user_id: u32,
    post_id: u32,
    report_reason: String,
}

// 提交举报的文章
#[post("/report/submit")]
pub async fn submit_report(
    req: HttpRequest,
    info: web::Json<SubmitReport>,
) -> actix_web::Result<HttpResponse> {
    log::debug!("Start submit_report function");
    // println!("--- submit_report");

    // 验证 token 数据
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("Failed token verif"));
    }

    // 获得用户的举报信息
    let SubmitReport {
        user_id,
        post_id,
        report_reason,
    } = info.into_inner();
    let my_pool = MysqlPool::instance();

    let query = format!(
        "
        insert into report_info 
            (user_id, post_id, release_time, report_reason)
        VALUES
            ({}, {}, NOW(), '{}');
    ",
        user_id, post_id, report_reason
    );

    match my_pool.exec_drop(vec![query], &my_pool.read_write_txopts) {
        Ok(_) => {
            log::info!("submit_report: my_pool.exec_drop a new data");
        }
        Err(err) => {
            log::error!("submit_report: Error my_pool.exec_drop err:{:?}", err);
        }
    }

    log::debug!("End submit_report function");
    Ok(HttpResponse::Ok().body(serde_json::to_string("插入成功").unwrap()))
}



#[get("/report/get_total")]
pub async fn get_report_total(req: HttpRequest)
-> actix_web::Result<HttpResponse> {
    log::debug!("Start get_report_total function");
    // println!("--- get_report_total");

    // 验证身份
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("token verify error"));
    }

    let my_pool = MysqlPool::instance();

    let query = format!("
        select count(*) from report_info;
    ");

    let number: Vec<u32> = match my_pool.exec(query, &my_pool.read_only_txopts) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("get_report_total my_pool.exec, err: {:?}", err);
            return Ok(HttpResponse::InternalServerError().body("Internal Server Error"));
        }
    };

    log::debug!("End get_report_total function");
    Ok(HttpResponse::Ok().body(serde_json::to_string(&number[0]).unwrap()))
}

#[derive(Debug)]
struct MyUser {
    user_id: u32,
    user_name: String,
    avatar_url: String,
}

#[derive(Debug)]
struct MyPost {
    post_id: u32,
    title: String,
}

#[derive(Debug, Serialize)]
struct MyReportInfo {
    id: u32,
    user_id: u32,
    post_id: u32,
    release_time: String,
    report_reason: String,
}

#[derive(Debug, Serialize)]
struct ReportInfo {
    id: u32,
    user_id: u32,
    username: String,
    avatar_url: String,
    post_id: u32,
    title: String,
    release_time: String,
    report_reason: String,
}

#[get("/report/get")]
pub async fn get_report(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    log::debug!("Start get_report function");
    // println!("--- get_report");

    // 进行token验证
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("Failed token verify"));
    }

    // 查询举报数据
    let my_pool = MysqlPool::instance();

    let query = format!(
        "
        select id, user_id, post_id, release_time, report_reason
        from report_info;
    "
    );

    let user_reports = match my_pool.query_map(
        query,
        |(id, user_id, post_id, release_time, report_reason): (u32, u32, u32, String, String)| {
            MyReportInfo {
                id,
                user_id,
                post_id,
                release_time,
                report_reason,
            }
        },
        &my_pool.read_only_txopts,
    ) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("Error get_report: my_pool exec, err: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    println!("user_reports.len: {}", user_reports.len());

    if user_reports.len() == 0 {
        return Ok(HttpResponse::Ok().body(serde_json::to_string(&user_reports).unwrap()));
    }

    // 构建查询用户信息的参数
    let user_ids_params = user_reports
        .iter()
        .map(|user_report| user_report.user_id.to_string())
        .collect::<Vec<String>>()
        .join(",");

    // 构建用户数据查询语句
    let query = format!(
        "
        select id, name, avatar_url
        from user
        where id in ({});
    ",
        user_ids_params
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
            log::error!("Error get_report executing query: {:?}", err);
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

    // 查询用户帖子的信息
    let titles_params = user_reports
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

    // 将这个值输出到结构体构建查询
    let user_reports: Vec<ReportInfo> = user_reports
        .into_iter()
        .map(|user_report| ReportInfo {
            id: user_report.id,
            user_id: user_report.user_id,
            username: user_map
                .get(&user_report.user_id)
                .cloned()
                .unwrap_or_else(|| ("编程驿站一份子".to_string(), String::from("")))
                .0,
            avatar_url: user_map
                .get(&user_report.user_id)
                .cloned()
                .unwrap_or_else(|| (String::from(""), AVATAR_URL.to_string()))
                .1,
            post_id: user_report.post_id,
            title: post_map
                .get(&user_report.post_id)
                .cloned()
                .unwrap_or_else(|| "编程驿站的小文章".to_string()),
            release_time: user_report.release_time,
            report_reason: user_report.report_reason,
        })
        .collect();

    // 将这个值进行序列化
    let user_reports_jsons = serde_json::to_string(&user_reports).map_err(|err| {
        log::error!("Error serializing response: {:?}", err);
        actix_web::error::ErrorInternalServerError("Error serializing response")
    })?;

    log::debug!("End get_report function");
    Ok(HttpResponse::Ok().body(user_reports_jsons))
}

// 删除用户的举报信息
#[post("/report/delete/{report_id}")]
pub async fn delete_report(
    req: HttpRequest,
    report_id: web::Path<u32>,
) -> actix_web::Result<HttpResponse> {
    log::debug!("Start delete_report function");
    println!("delete_report");

    // 进行 token 验证
    let user_info = match Token::token_to_claims(req) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("Error update_userprofile_avatar is token_to_claims");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    if user_info.verify().is_err() {
        return Ok(HttpResponse::BadRequest().body("Token verif Farild"));
    }

    // 不是管理员
    if user_info.is_admin().is_err() {
        return Ok(HttpResponse::BadRequest().body("不是管理员"));
    }

    let report_id = *report_id;
    let query = format!("
        delete from report_info
        where id = {};
    ", report_id);

    let my_pool = MysqlPool::instance();

    match my_pool.exec_drop(vec![query], &my_pool.read_write_txopts) {
        Ok(_) => {
            log::info!("");
        },
        Err(err) => {
            log::error!("delete report, err: {:?}", err);
            return Ok(HttpResponse::InternalServerError().body("Internal Server Error"));
        }
    }

    log::debug!("End delete_report function");
    Ok(HttpResponse::Ok().body(serde_json::to_string("删除成功").unwrap()))
}
