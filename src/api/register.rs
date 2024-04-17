use actix_web::{post, web, HttpResponse};
use lazy_static::*;
use r2d2_mysql::mysql::prelude::Queryable;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

use crate::{common::email::*, common::token::*, database::mysql::*};

// 定义一个全局 Mutex 来存储任务的状态
lazy_static! {
    static ref TASKS: Arc<Mutex<HashMap<String, JoinHandle<()>>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Debug, Serialize, Deserialize)]
struct EmailRequest {
    email: String,
}

// 发送验证码的API
#[post("send/verification")]
pub async fn send_verification_code(
    email: web::Json<EmailRequest>,
) -> actix_web::Result<HttpResponse> {
    log::debug!("Start send_verification_code function");
    let email_receiver = email.into_inner().email;

    // 用于判断数据库中是否已经有这个邮箱注册的用户
    // 获取线程池，这个线程池为单例模式
    let my_pool = MysqlPool::instance();

    // 查询这个邮箱是否已经在用户表中
    let query = format!("select id from user where email = '{}';", email_receiver);
    let user_id: Vec<(u32,)> = match my_pool.exec(query, &my_pool.read_only_txopts) {
        Ok(result) => result,
        Err(err) => {
            log::error!("Error executing query: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    println!("{}", user_id.len());

    // 如果存在返回用户已存在
    if user_id.len() > 0 {
        return Ok(HttpResponse::InternalServerError().body(
            serde_json::to_string("用户已经存在").map_err(|err| {
                log::error!("Error serializing response: {:?}", err);
                actix_web::error::ErrorBadRequest("Error serializing response")
            })?,
        ));
    }

    // println!("{:?}", email_receiver);
    // 生成随机的验证码
    let code = Email::generate_code();
    // 邮件的主题
    let subject = "Program Station 注册验证";
    // let subject = "Program Station 邮箱验证";
    // 你的文件
    let body = format!("你的验证码是: {}， 有限期为五分钟。", code);

    let email = Email::instance();
    // 假设send_email函数已经实现并配置好了
    if email.send_email(&email_receiver, subject, &body).is_ok() {
        // 注册定时函数，五分钟后删除此验证码
        // 检查是否已经存在相同邮箱的任务
        let mut tasks = TASKS.lock().unwrap();
        if let Some(existing_task) = tasks.get(&email_receiver) {
            // 取消旧任务
            existing_task.abort();
        }
        // 此时已经发送成功了，将验证码记录下来用于验证，五分钟
        email.insert(&email_receiver, code)?;
        let email_receiver_clone = email_receiver.clone();
        let handle = tokio::spawn(async move {
            sleep(Duration::from_secs(300)).await;
            // let email = Email::instance();
            let _ = email.remove(&email_receiver_clone);
        });

        // 存储新任务
        tasks.insert(email_receiver, handle);

        // 输出发送成功信息
        log::info!("Email::send_email: is success");
        // println!("Email::send_email: is success");
        // let resp = "Verification code sent.".to_string();
        Ok(HttpResponse::Ok().body(serde_json::to_string("Verification code sent.").unwrap()))
    } else {
        // println!("Email::send_email: is Failed");
        log::error!("Email::send_email: is Failed");
        Ok(HttpResponse::InternalServerError()
            .body(serde_json::to_string("Failed to send verification code.").unwrap()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Register {
    name: String,
    email_receiver: String,
    verification_code: u32,
    password: String,
}

// 验证验证码的函数
#[post("verify/verification")]
pub async fn verify_verification_code(
    register: web::Json<Register>,
) -> actix_web::Result<HttpResponse> {
    log::debug!("Start verify_verification_code function");
    // 获取数据
    let Register {
        name,
        email_receiver,
        verification_code,
        password,
    } = register.into_inner();

    let my_pool = MysqlPool::instance();
    let query = format!("select user_id from login where email = '{}';", email_receiver);
    let count: Vec<(u32,)> = match my_pool.exec(query, &my_pool.read_only_txopts) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("Error verify_verification_code: {:?}", err);
            return Ok(HttpResponse::InternalServerError()
                .body(serde_json::to_string("服务器繁忙").unwrap()));
        }
    };

    if count.len() > 0 {
        return Ok(HttpResponse::InternalServerError()
            .body(serde_json::to_string("用户已经存在").unwrap()));
    }

    // 如果邮箱验证成功
    let email = Email::instance();
    if email
        .verify_verification_code(&email_receiver, verification_code)
        .is_ok()
    {
        // 将用户存入数据库
        // 获取线程池，这个线程池为单例模式
        log::debug!("get my_pool");
        let my_pool = MysqlPool::instance();
        log::debug!("get my_pool get connection");
        let connection = my_pool.get_connection();

        // 如果获取连接失败
        if connection.is_err() {
            log::error!("Error verify_verification_code get connection");
            return Ok(HttpResponse::InternalServerError()
                .body(serde_json::to_string("服务器繁忙").unwrap()));
        }
        let mut connection = connection.unwrap();

        // 读写
        let txopts = my_pool.read_write_txopts.clone();
        log::debug!("get my_pool transaction");
        // 开启事务
        let transaction = connection.start_transaction(txopts);
        if transaction.is_err() {
            log::error!("Error verify_verification_code get transaction");
            return Ok(HttpResponse::InternalServerError()
                .body(serde_json::to_string("服务器繁忙").unwrap()));
        }
        let mut transaction = transaction.unwrap();

        // 将信息插入用户表中
        let insert_user = format!(
            "INSERT INTO user 
                (name, email, avatar_url, register_time)
            VALUES
                ('{}', '{}', '{}', NOW());",
            name, email_receiver, AVATAR_URL
        );
        log::debug!("my_pool transaction.query_drop(insert_user)");
        // 执行插入语句
        match transaction.query_drop(insert_user) {
            Ok(result) => result,
            Err(err) => {
                log::error!(
                    "Error executing transaction.query_drop(insert_user): {:?}",
                    err
                );
                return Ok(HttpResponse::InternalServerError()
                    .body(serde_json::to_string("服务器繁忙").unwrap()));
            }
        };
        // if result.is_err() {
        //     log::error!("Error verify_verification_code executing transaction.query_drop");
        //     return Ok(HttpResponse::InternalServerError()
        //         .body(serde_json::to_string("服务器繁忙").unwrap()));
        // }

        // 获取插入数据的最后一个主键 id
        let user_id: Result<Option<u32>, r2d2_mysql::mysql::Error> =
            transaction.query_first("SELECT LAST_INSERT_ID()");

        // 取出判断
        if user_id.is_err() {
            log::error!("Error verify_verification_code get user_id");
            return Ok(HttpResponse::InternalServerError()
                .body(serde_json::to_string("服务器繁忙").unwrap()));
        }
        let user_id = user_id.unwrap();
        if user_id.is_none() {
            log::error!("None verify_verification_code get user_id");
            return Ok(HttpResponse::InternalServerError()
                .body(serde_json::to_string("服务器繁忙").unwrap()));
        }

        // 获得用户 id
        let user_id = user_id.unwrap();

        // 插入用户关注数和粉丝数
        let insert_user_stats = format!(
            "insert into user_stats
                (user_id, follower_count, fans)
            values
                ({}, 0, 0);",
            user_id
        );
        log::debug!("my_pool transaction.query_drop(insert_user_stats)");
        match transaction.query_drop(insert_user_stats) {
            Ok(result) => result,
            Err(err) => {
                log::error!(
                    "Error executing transaction.query_drop(insert_user_stats): {:?}",
                    err
                );
                return Ok(HttpResponse::InternalServerError()
                    .body(serde_json::to_string("服务器繁忙").unwrap()));
            }
        };

        // 插入用户登录信息表
        let insert_login = format!(
            "INSERT INTO login
                (user_id, email, password)
            VALUES
                ({}, '{}', '{}');",
            user_id, email_receiver, password
        );
        log::debug!("my_pool transaction.query_drop(insert_login)");
        // 如果数据插入成功，事务提交
        if transaction.query_drop(insert_login).is_ok() {
            let success = transaction.commit();
            if success.is_ok() {
                log::info!("Ok verify_verification_code transaction has commit");
                // 在这里进行 token 的获取
                let claims = Claims::new(user_id, &email_receiver, &password);
                let token = Token::get_jwt(&claims);
                if token.is_err() {
                    log::error!("Error verify_verification_code Token get jwt");

                    return Ok(HttpResponse::InternalServerError()
                        .body(serde_json::to_string("服务器繁忙").unwrap()));
                }

                let token = token.unwrap();
                let login_info = UserInfomation {
                    id: user_id,
                    name: name,
                    avatar_url: AVATAR_URL.to_string(), // 默认头像路径
                    follower_count: 0,
                    fans: 0,
                    token: token,
                };

                // 返回用户信息
                let response = serde_json::to_string(&login_info);
                if response.is_err() {
                    log::error!("Error verify_verification_code serde_json: {:?}", response);

                    return Ok(HttpResponse::InternalServerError()
                        .body(serde_json::to_string("服务器繁忙").unwrap()));
                }

                let response = response.unwrap();
                log::info!("End verify_verification_code function");
                // 成功返回信息
                return Ok(HttpResponse::Ok().body(response));
            } else {
                log::error!("Error transaction verify_verification_code commit");
                return Ok(HttpResponse::InternalServerError()
                    .body(serde_json::to_string("服务器繁忙").unwrap()));
            }
        }

        // 数据插入失败，进行数据回滚
        transaction.rollback().expect("Faild transaction rollback");
        // 事务回滚成功，返回服务端繁忙
        log::info!("verify_verification_code transaction is rollback successful");
        return Ok(
            HttpResponse::InternalServerError().body(serde_json::to_string("服务器繁忙").unwrap())
        );
    } else {
        log::info!("verify_verification_code verification code is error");
        Ok(HttpResponse::NotFound().body(serde_json::to_string("验证码错误").unwrap()))
    }
}
