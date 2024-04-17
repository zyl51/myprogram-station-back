use actix_web::{post, web, HttpResponse, HttpRequest};
use serde::{Deserialize, Serialize};


use crate::{database::mysql::*, common::token::*};

#[derive(Serialize, Deserialize)]
struct UserStats {
    follower_id: u32,
    following_id: u32,
    count: i32,
}

// 增加关注
#[post("/user_stats/addremove")]
pub async fn add_remove_follow(
    req: HttpRequest,
    user_stats: web::Json<UserStats>,
) -> actix_web::Result<HttpResponse> {
    log::debug!("Start add_remove_follow function");
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("Failed is verif token"));
    }
    // println!("add_remove_follow");

    let UserStats {
        follower_id,
        following_id,
        count,
    } = user_stats.into_inner();

    let my_pool = MysqlPool::instance();

    // println!("{}, {}, {}", follower_id, following_id, count);
    let query1 = format!(
        "
        update user_stats
        set 
            follower_count = follower_count + {}
        where user_id = {};
        ",
        count, follower_id
    );

    let query2 = format!(
        "
        update user_stats
        set
            fans = fans + {}
        where user_id = {};
        ",
        count, following_id,
    );

    let query3 = if count == 1 {
        format!(
            "
            insert into follow
            VALUES
            ({}, {}, NOW())
            ",
            follower_id, following_id
        )
    } else {
        format!(
            "
            delete from follow
            where follower_id = {}
                and 
                following_id = {};
            ",
            follower_id, following_id
        )
    };

    let vec = vec![query1, query2, query3];

    match my_pool.exec_drop(vec, &my_pool.read_write_txopts) {
        Ok(_) => {
            log::info!("add_remove_follow exec_drop successful");
        }
        Err(err) => {
            log::error!("Error add_remove_follow executing query: {:?}", err);
            return Ok(HttpResponse::InternalServerError()
                .body(serde_json::to_string("服务器繁忙").unwrap()));
        }
    };

    log::debug!("End add_remove_follow function");
    Ok(HttpResponse::Ok().body(serde_json::to_string("successful").unwrap()))
}
