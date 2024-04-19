use actix_web::{get, post, web, HttpResponse, HttpRequest};
use serde::{Deserialize, Serialize};

use crate::{database::mysql::*, common::token::*};

#[derive(Debug, Serialize, Deserialize)]
struct LikeCollect {
    user_id: u32,
    post_id: u32,
}

#[derive(Debug, Serialize)]
struct ResLikeCollect {
    like: bool,
    collect: bool,
}

// 获取用户是否点赞和收藏
#[get("/like_collect/get")]
pub async fn get_like_collect(req: HttpRequest, info: web::Query<LikeCollect>) -> actix_web::Result<HttpResponse> {
    log::debug!("Start get_like_collect function");
    println!("get_like_collect");
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("Failed is verif token"));
    }
    let LikeCollect { user_id, post_id } = info.into_inner();

    let get_like = format!("
        select count(*) from like_post 
        where user_id = {} and post_id = {};
    ", user_id, post_id);
    let get_collect = format!("
        select count(*) from collect_post 
        where user_id = {} and post_id = {};
    ", user_id, post_id);

    // let qeurys = vec![get_like, get_collect];

    let my_pool = MysqlPool::instance();
    let like: Vec<u32> = match my_pool.exec(get_like, &my_pool.read_only_txopts) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("Error get_like_collect executing my_pool.exec: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };
    let collect: Vec<u32> = match my_pool.exec(get_collect, &my_pool.read_only_txopts) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("Error get_like_collect executing my_pool.exec: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    let like = if like[0] == 0 {
        false
    } else {
        true
    };

    let collect = if collect[0] == 0 {
        false
    } else {
        true
    };

    let res = ResLikeCollect { like, collect };

    log::debug!("End get_like_collect function");
    Ok(HttpResponse::Ok().body(serde_json::to_string(&res).unwrap()))
}


#[post("/like_collect/add_like")]
pub async fn add_like(req: HttpRequest, info: web::Json<LikeCollect>) -> actix_web::Result<HttpResponse> {
    log::debug!("Start add_like function");
    // println!("add_like");
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("Failed is verif token"));
    }
    let LikeCollect { user_id, post_id } = info.into_inner();

    let my_pool = MysqlPool::instance();

    // 查询帖对应的作者
    let query = format!("
        select user_id
        from post
        where id = {};
    ", post_id);

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

    let query1 = format!("
        insert into like_post
            (user_id, post_id)
        VALUES
            ({}, {})
    ", user_id, post_id);

    let query2 = format!("
        update post
        set like_numbers = like_numbers + 1
        where id = {};
    ", post_id);

    let query3 = format!("
        insert into message
            (sender_id, recver_id, post_id, status)
        VALUES
            ({}, {}, {}, 1);
    ", user_id, recver_id, post_id);

    match my_pool.exec_drop(vec![query1, query2, query3], &my_pool.read_write_txopts) {
        Ok(_) => {
            log::info!("add_like executing exec_drop successful");
        }
        Err(err) => {
            log::error!("Error add_like executing exec_drop: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    log::debug!("End add_like function");

    Ok(HttpResponse::Ok().body(serde_json::to_string("点赞成功").unwrap()))
}


// 取消点赞
#[post("/like_collect/sub_like")]
pub async fn sub_like(req: HttpRequest, info: web::Json<LikeCollect>) -> actix_web::Result<HttpResponse> {
    log::debug!("Start sub_like function");
    // println!("sub_like");
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("Failed is verif token"));
    }

    let LikeCollect { user_id, post_id } = info.into_inner();

    let query1 = format!("
        delete from like_post 
        where user_id = {} and post_id = {}
    ", user_id, post_id);

    let query2 = format!("
        update post
        set like_numbers = like_numbers - 1
        where id = {};
    ", post_id);

    let my_pool = MysqlPool::instance();

    match my_pool.exec_drop(vec![query1, query2], &my_pool.read_write_txopts) {
        Ok(_) => {
            log::info!("sub_like executing exec_drop successful");
        }
        Err(err) => {
            log::error!("Error sub_like executing exec_drop: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    log::debug!("End sub_like function");

    Ok(HttpResponse::Ok().body(serde_json::to_string("取消点赞成功").unwrap()))
}

// 增加收藏
#[post("/like_collect/add_collect")]
pub async fn add_collect(req: HttpRequest, info: web::Json<LikeCollect>) -> actix_web::Result<HttpResponse> {
    log::debug!("Start add_collect function");
    // println!("add_collect");
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("Failed is verif token"));
    }

    let LikeCollect { user_id, post_id } = info.into_inner();

    let my_pool = MysqlPool::instance();

    let query = format!("
        select user_id
        from post
        where id = {};
    ", post_id);

    // 查询帖对应的作者
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

    let query1 = format!("
        insert into collect_post
            (user_id, post_id)
        VALUES
            ({}, {});
    ", user_id, post_id);

    let query2 = format!("
        update post
        set collect_numbers = collect_numbers + 1
        where id = {};
    ", post_id);

    let query3 = format!("
        insert into message
            (sender_id, recver_id, post_id, status)
        VALUES
            ({}, {}, {}, 2);
    ", user_id, recver_id, post_id);

    match my_pool.exec_drop(vec![query1, query2, query3], &my_pool.read_write_txopts) {
        Ok(_) => {
            log::info!("add_collect executing exec_drop successful");
        }
        Err(err) => {
            log::error!("Error add_collect executing exec_drop: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    log::debug!("End add_collect function");

    Ok(HttpResponse::Ok().body(serde_json::to_string("收藏成功").unwrap()))
}



#[post("/like_collect/sub_collect")]
pub async fn sub_collect(req: HttpRequest, info: web::Json<LikeCollect>) -> actix_web::Result<HttpResponse> {
    log::debug!("Start sub_like function");
    // println!("sub_collect");
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("Failed is verif token"));
    }
    let LikeCollect { user_id, post_id } = info.into_inner();

    let query1 = format!("
        delete from collect_post 
        where user_id = {} and post_id = {};
    ", user_id, post_id);

    let query2 = format!("
        update post
        set collect_numbers = collect_numbers - 1
        where id = {};
    ", post_id);

    let my_pool = MysqlPool::instance();

    match my_pool.exec_drop(vec![query1, query2], &my_pool.read_write_txopts) {
        Ok(_) => {
            log::info!("sub_collect executing exec_drop successful");
        }
        Err(err) => {
            log::error!("Error sub_collect executing exec_drop: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    log::debug!("End sub_collect function");

    Ok(HttpResponse::Ok().body(serde_json::to_string("取消收藏成功").unwrap()))
}
