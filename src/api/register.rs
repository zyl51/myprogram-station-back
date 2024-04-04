use actix_web::{post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

use crate::{common::email::*, database::mysql::*};

#[derive(Debug, Serialize, Deserialize)]
struct EmailRequest {
    email: String,
}

// 发送验证码的API
#[post("send/verification")]
pub async fn send_verification_code(
    email: web::Json<EmailRequest>,
) -> actix_web::Result<HttpResponse> {
    let email_receiver = email.into_inner().email;

    // 用于判断数据库中是否已经有这个邮箱注册的用户
    // 获取线程池，这个线程池为单例模式
    let my_pool = MysqlPool::instance();

    // 查询这个邮箱是否已经在用户表中
                                // "SELECT id FROM user WHERE email = {};"
    // println!("start");
    let query = format!("select id from user where email = '{}';", email_receiver);
    let user_id: Vec<(u32,)> = match my_pool.exec(query, &my_pool.read_only_txopts) {
        Ok(result) => result,
        Err(err) => {
            eprintln!("Error executing query: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };


    // 如果存在返回用户已存在
    if user_id.len() > 0 {
        return Ok(HttpResponse::InternalServerError().body(
            serde_json::to_string("用户已经存在").map_err(|err| {
                eprintln!("Error serializing response: {:?}", err);
                actix_web::error::ErrorBadRequest("Error serializing response")
            })?,
        ));
    }

    println!("{:?}", email_receiver);
    // 生成随机的验证码
    let code = Email::generate_code();
    // 邮件的主题
    let subject = "Program Station 邮箱验证";
    // let subject = "Program Station 邮箱验证";
    // 你的文件
    let body = format!("你的验证码是: {}", code);

    let email = Email::instance();
    // 假设send_email函数已经实现并配置好了
    if email.send_email(&email_receiver, subject, &body).is_ok() {
        // 此时已经发送成功了，将验证码记录下来用于验证
        email.insert(&email_receiver, code)?;
        // 注册定时函数，五分钟后删除此验证码
        tokio::spawn(async move {

            sleep(Duration::from_secs(300)).await;
            // let email = Email::instance();
            let _ = email.remove(&email_receiver);
        });

        // 输出发送成功信息
        println!("Email::send_email: is success");
        // let resp = "Verification code sent.".to_string();
        Ok(HttpResponse::Ok().body(serde_json::to_string("Verification code sent.").unwrap()))
    } else {
        println!("Email::send_email: is Failed");
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
pub async fn verify_verification_code(register: web::Json<Register>) 
-> actix_web::Result<HttpResponse> {
    // 获取数据
    let Register {
        name,
        email_receiver,
        verification_code,
        password,
    } = register.into_inner();

    println!(
        "{}, {}, {}, {}",
        name, email_receiver, verification_code, password
    );

    // 如果成功，
    let email = Email::instance();
    if email.verify_verification_code(&email_receiver, verification_code).is_ok()
    {
        // 将用户存入数据库
        // 获取线程池，这个线程池为单例模式
        let my_pool = MysqlPool::instance();

        // 查询这个邮箱是否已经在用户表中
        let insert_user = format!(
            "INSERT INTO user 
                (name, email, register_time)
            VALUES
                ('{}', '{}', NOW())",
            name, email_receiver
        );

        // 执行插入
        match my_pool.query_drop(&insert_user, &my_pool.read_write_txopts) {
            Ok(user_id) => {
                // 返回成功
                let insert_login = format!(
                    "INSERT INTO login
                        (user_id, email, password)
                    VALUES
                        ({}, '{}', '{}')",
                    user_id, email_receiver, password
                );

                match my_pool.query_drop(&insert_login, &my_pool.read_write_txopts) {
                    Ok(_) => {
                        return Ok(HttpResponse::Ok().body(serde_json::to_string("注册成功").unwrap()));
                    }
                    Err(err) => {
                        eprintln!("{}", err);
                        return Ok(HttpResponse::InternalServerError()
                            .body(serde_json::to_string("服务器繁忙").unwrap()));
                    }
                }
            }
            Err(err) => {
                // 插入失败
                eprintln!("{}", err);
                return Ok(HttpResponse::InternalServerError()
                    .body(serde_json::to_string("服务器繁忙").unwrap()));
            }
        }
    } else {
        Ok(HttpResponse::NotFound().body(serde_json::to_string("验证码错误").unwrap()))
    }
}
