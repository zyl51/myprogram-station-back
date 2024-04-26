use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{common::token::*, database::mysql::*};

#[get("/user_management/get_user_total")]
pub async fn get_user_management_total(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    log::debug!("Start get_user_management_total function");
    // println!("---- get_user_management_total");

    // 验证 token
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("token verify error"));
    }

    let query = format!("select count(*) from user");

    let my_pool = MysqlPool::instance();

    let number: Vec<u32> = match my_pool.exec(query, &my_pool.read_only_txopts) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("get_user_management_total: my_pool.exec, err:{:?}", err);
            return Ok(HttpResponse::InternalServerError().body("Internal Server Error"));
        }
    };

    log::debug!("End get_user_management_total function");
    Ok(HttpResponse::Ok().body(serde_json::to_string(&number[0]).unwrap()))
}

#[derive(Debug, Serialize)]
struct MyUser {
    user_id: u32,
    email: String,
    username: String,
    avatar_url: String,
    register_time: String,
}

#[derive(Debug, Serialize)]
struct User {
    user_id: u32,
    email: String,
    username: String,
    avatar_url: String,
    register_time: String,
    is_ban: bool,
    is_admin: bool,
}

#[derive(Debug)]
struct MyLogin {
    user_id: u32,
    is_ban: bool,
}

pub fn is_admin(user_id: u32) -> Result<bool, ()> {

    let my_pool = MysqlPool::instance();
    let query = format!("
        select user_id
        from admin
        where user_id = {};
    ", user_id);

    let admin: Vec<u32> = my_pool.exec(query, &my_pool.read_only_txopts).unwrap();

    Ok(admin.len() != 0)
}

#[get("/user_management/get_user_list/{page}")]
pub async fn get_user_management_list(
    req: HttpRequest,
    page: web::Path<u32>,
) -> actix_web::Result<HttpResponse> {
    log::debug!("Start get_user_management_list function");
    // println!("--- get_user_management_list");

    // token 验证
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("token verify error"));
    }

    let page = *page;
    let satrt = (page - 1) * 20;

    let my_pool = MysqlPool::instance();
    let query = format!(
        "
        select id, email, name, avatar_url, register_time
        from user
        limit {}, 20;
    ",
        satrt
    );

    // 获取用户的信息数据
    let users: Vec<MyUser> = match my_pool.query_map(
        query,
        |(user_id, email, username, avatar_url, register_time): (u32, String, String, String, String)| MyUser {
            user_id,
            email,
            username,
            avatar_url,
            register_time,
        },
        &my_pool.read_only_txopts,
    ) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("get_user_management_list: my_pool.query_map, err:{:?}", err);
            return Ok(HttpResponse::InternalServerError().body("Internal Server Error"));
        }
    };

    // 创建用户的 id 参数
    let user_id_params = users
        .iter()
        .map(|user| user.user_id.to_string())
        .collect::<Vec<String>>()
        .join(",");

    let query = format!(
        "
        select user_id, is_ban
        from login
        where user_id in ({});
    ",
        user_id_params
    );

    // 查询用户是否被封禁
    let logins: Vec<MyLogin> = match my_pool.query_map(
        query,
        |(user_id, is_ban): (u32, bool)| MyLogin { user_id, is_ban },
        &my_pool.read_only_txopts,
    ) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("get_user_management_list: executing query_map, err:{:?}", err);
            return Ok(HttpResponse::InternalServerError().body("Internal Server Error"));
        }
    };

    let logins_map: HashMap<u32, bool> = logins
        .into_iter()
        .map(|login| (login.user_id, login.is_ban))
        .collect();

    let users: Vec<User> = users
        .into_iter()
        .map(|user| User{
            user_id: user.user_id,
            email: user.email,
            username: user.username,
            avatar_url: user.avatar_url,
            register_time: user.register_time,
            is_ban: logins_map
                .get(&user.user_id)
                .cloned()
                .unwrap_or_default(),
            is_admin: is_admin(user.user_id).unwrap(),
        })
        .collect();

    log::debug!("End get_user_management_list function");
    Ok(HttpResponse::Ok().body(serde_json::to_string(&users).unwrap()))
}


// 封禁获取解禁用户
#[post("/user_management/ban_user/{user_id}")]
pub async fn ban_user(req: HttpRequest, user_id: web::Path<u32>)
-> actix_web::Result<HttpResponse> {
    log::debug!("Start ban_user function");

    println!("--- ban_user");

    // 验证
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("token verify error"));
    }

    // 获取用户 id
    let user_id = *user_id;

    let query = format!("
        update login
        set is_ban = 1 - is_ban
        where user_id = {};
    ", user_id);

    let my_pool = MysqlPool::instance();

    // 执行语句修改
    match my_pool.exec_drop(vec![query], &my_pool.read_write_txopts) {
        Ok(_) => {
            log::info!("ban_user: executing exec_drop");
        },
        Err(err) => {
            log::error!("ban_user: execytuting exec_drop, err:{:?}", err);
            return Ok(HttpResponse::InternalServerError().body("Error"));
        }
    };

    log::debug!("End ban_user function");
    Ok(HttpResponse::Ok().body(serde_json::to_string("修改成功").unwrap()))
}

#[derive(Debug, Deserialize)]
struct MyUpdateUserInfo {
    user_id: u32,
    username: String,
    email: String,
    is_admin: bool,
}

// 修改用户信息
#[post("/user_management/update_userinfo")]
pub async fn update_user_info(req: HttpRequest, info: web::Json<MyUpdateUserInfo>)
 -> actix_web::Result<HttpResponse> {

    println!("----- update_user_info");

    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("Token verify error"));
    }

    let MyUpdateUserInfo {
        user_id,
        username,
        email,
        is_admin,
    } = info.into_inner();

    let my_pool = MysqlPool::instance();

    let update_user = format!("
        update user
        set name = '{}', email = '{}'
        where id = {}; 
    ", username, email, user_id);

    let update_login = format!("
        update login
        set email = '{}'
        where user_id = {};
    ", email, user_id);

    let mut vec = vec![update_user, update_login];

    if is_admin {
        let query = format!("
            insert IGNORE into admin
                (user_id)
            VALUES
                ({})
        ", user_id);

        vec.push(query);
    } else {
        let query = format!("
            delete IGNORE from admin
            where user_id = {};
        ", user_id);

        vec.push(query);
    }

    my_pool.exec_drop(vec, &my_pool.read_write_txopts).unwrap();

    Ok(HttpResponse::Ok().body(serde_json::to_string("成功").unwrap()))
 }


// 通过用户名搜素用户
 #[get("/user_management/search/{search_query}")]
 pub async fn search_user(req: HttpRequest, search_query: web::Path<String>)
 -> actix_web::Result<HttpResponse> {

    println!("---- search_user");

    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("token verify error"));
    }

    let search_query = (*search_query).clone();

    // if search_query.len() == 0 {
    //     return get_user_management_list();
    // }

    let my_pool = MysqlPool::instance();
    let query = format!(
        "
        select id, email, name, avatar_url, register_time
        from user
        where name like '%{}%';
    ",
        search_query
    );

    // 获取用户的信息数据
    let users: Vec<MyUser> = my_pool.query_map(
        query,
        |(user_id, email, username, avatar_url, register_time): (u32, String, String, String, String)| MyUser {
            user_id,
            email,
            username,
            avatar_url,
            register_time,
        },
        &my_pool.read_only_txopts,
    ).unwrap();

    if users.len() == 0 {
        return Ok(HttpResponse::Ok().body(serde_json::to_string(&users).unwrap()));
    }

    // 创建用户的 id 参数
    let user_id_params = users
        .iter()
        .map(|user| user.user_id.to_string())
        .collect::<Vec<String>>()
        .join(",");

    let query = format!(
        "
        select user_id, is_ban
        from login
        where user_id in ({});
    ",
        user_id_params
    );

    // 查询用户是否被封禁
    let logins: Vec<MyLogin> = my_pool.query_map(
        query,
        |(user_id, is_ban): (u32, bool)| MyLogin { user_id, is_ban },
        &my_pool.read_only_txopts,
    ).unwrap();

    let logins_map: HashMap<u32, bool> = logins
        .into_iter()
        .map(|login| (login.user_id, login.is_ban))
        .collect();

    let users: Vec<User> = users
        .into_iter()
        .map(|user| User{
            user_id: user.user_id,
            email: user.email,
            username: user.username,
            avatar_url: user.avatar_url,
            register_time: user.register_time,
            is_ban: logins_map
                .get(&user.user_id)
                .cloned()
                .unwrap_or_default(),
            is_admin: is_admin(user.user_id).unwrap(),
        })
        .collect();

    // println!("{:?}", users);

    Ok(HttpResponse::Ok().body(serde_json::to_string(&users).unwrap()))
 }