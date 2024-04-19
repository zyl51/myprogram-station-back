use actix_web::{post, web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::{
    common::token::*,
    database::mysql::*,
};

// 创建一个帖子的结构体，用来发送数据
#[derive(Debug, Deserialize, Serialize, Clone)]
struct User {
    id: u32,
    name: String,
    avatar_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Login {
    email: String,
    password: String,
}

#[post("verify/login")]
pub async fn user_login(login: web::Json<Login>) -> actix_web::Result<HttpResponse> {
    log::debug!("Satrt user_login function");
    // 获取用户登录的数据
    let Login { email, password } = login.into_inner();

    // println!("{}, {}", email, password);

    // 检查数据库是不是有这个人
    let my_pool = MysqlPool::instance();
    let query = format!(
        "select user_id, password from login where email = '{}';",
        email
    );

    let user_id_password: Vec<(u32, String)> = match my_pool.exec(query, &my_pool.read_only_txopts)
    {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("Error user_login my_pool exec: {:?}", err);

            return Ok(HttpResponse::InternalServerError()
                .body(serde_json::to_string("服务器繁忙").unwrap()));
        }
    };

    // 没有这个用户
    if user_id_password.len() <= 0 {
        return Ok(HttpResponse::BadRequest().body(serde_json::to_string("用户不存在").unwrap()));
    }

    // 验证密码
    if password != user_id_password[0].1 {
        return Ok(
            HttpResponse::BadRequest().body(serde_json::to_string("用户名或密码错误").unwrap())
        );
    }

    let user_id = user_id_password[0].0;

    // 获取我们的 token 数据
    let claims = Claims::new(user_id, &email, &password);

    if claims.is_ban().is_err() {
        return Ok(
            HttpResponse::InternalServerError().body(serde_json::to_string("用户已被封禁").unwrap())
        );
    }

    log::debug!("Start user_login function executing Token::get_jwt(&claims)");
    let token = Token::get_jwt(&claims);
    if token.is_err() {
        log::error!("Error verify_verification_code Token get jwt: {:?}", token);
        return Ok(
            HttpResponse::InternalServerError().body(serde_json::to_string("服务器繁忙").unwrap())
        );
    }
    let token = token.unwrap();

    // -----------------------------------------------------------
    log::debug!("user_login function get user personal info");
    // 查询用户的个人信息
    let query = format!(
        "SELECT id, name, avatar_url FROM user WHERE id = {}",
        user_id
    );
    let user: Vec<User> = match my_pool.query_map(
        query,
        |(id, name, avatar_url): (u32, String, String)| User {
            id,
            name,
            avatar_url,
        },
        &my_pool.read_only_txopts,
    ) {
        Ok(result) => result,
        Err(err) => {
            log::error!("Error get_user query_map query: {:?}", err);
            // eprintln!("get_user: Error query_map query: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    // 获取到用户数据
    let User {
        id,
        name,
        avatar_url,
    } = user[0].clone();

    // ------------------------------------------------------------
    log::debug!("get the number of followers from users");
    // 获取用户的粉丝数量
    let query = format!(
        "select follower_count, fans from user_stats where user_id = {}",
        id
    );
    let user_stats: Vec<(u32, u32)> = match my_pool.exec(&query, &my_pool.read_only_txopts) {
        Ok(result) => result,
        Err(err) => {
            // eprintln!("Error executing query: {:?}", err);
            log::error!("Error executing user_login function query: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    let avatar_url = if avatar_url.len() != 0 {
        avatar_url
    } else {
        AVATAR_URL.to_string()
    };

    let login_info = UserInfomation {
        id: id,
        name: name,
        avatar_url: avatar_url, // 默认头像路径
        follower_count: user_stats[0].0,
        fans: user_stats[0].1,
        token: token,
    };

    let response = serde_json::to_string(&login_info);
    if response.is_err() {
        log::error!("Error verify_verification_code serde_json: {:?}", response);

        return Ok(
            HttpResponse::InternalServerError().body(serde_json::to_string("服务器繁忙").unwrap())
        );
    }

    let response = response.unwrap();

    log::debug!("End user_login function");
    Ok(HttpResponse::Ok().body(response))
}
