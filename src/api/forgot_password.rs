use actix_web::{post, web, HttpResponse};
use lazy_static::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

use crate::{common::email::*, database::mysql::*};

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
#[post("/send_forgotPassword/verification")]
pub async fn send_forgot_password(
    email: web::Json<EmailRequest>,
) -> actix_web::Result<HttpResponse> {
    log::debug!("Start send_forgot_password function");
    println!("send_forgot_password");
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

    // 如果用户不存在
    if user_id.len() == 0 {
        return Ok(HttpResponse::InternalServerError().body(
            serde_json::to_string("用户不存在").map_err(|err| {
                log::error!("Error serializing response: {:?}", err);
                actix_web::error::ErrorBadRequest("Error serializing response")
            })?,
        ));
    }

    // println!("{:?}", email_receiver);
    // 生成随机的验证码
    let code = Email::generate_code();
    // 邮件的主题
    let subject = "Program Station 找回密码验证";
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
struct ForgotPassword {
    email_receiver: String,
    verification_code: u32,
    password: String,
}

// 验证验证码的函数
#[post("/verify_forgotPassword/verification")]
pub async fn verify_forgot_password(
    register: web::Json<ForgotPassword>,
) -> actix_web::Result<HttpResponse> {
    log::debug!("Start verify_forgot_password function");
    println!("verify_forgot_password");
    // 获取数据
    let ForgotPassword {
        email_receiver,
        verification_code,
        password,
    } = register.into_inner();

    // 如果邮箱验证成功
    let email = Email::instance();
    if email
        .verify_verification_code(&email_receiver, verification_code)
        .is_ok()
    {
        let my_pool = MysqlPool::instance();
        let query = format!("
            update login
            set password = '{}'
            where email = '{}';
        ", password, email_receiver);

        // 执行这个修改数据代码
        match my_pool.exec_drop(vec![query], &my_pool.read_write_txopts) {
            Ok(_) => {
                log::info!("verify_forgot_password exec_drop successful");
            }
            Err(err) => {
                log::error!("Error verify_forgot_password executing query: {:?}", err);
                return Ok(HttpResponse::InternalServerError()
                    .body(serde_json::to_string("服务器繁忙").unwrap()));
            }
        };

        Ok(HttpResponse::Ok().body(serde_json::to_string("重置成功").unwrap()))
    } else {
        log::info!("verify_forgot_password verification code is error");
        Ok(HttpResponse::NotFound().body(serde_json::to_string("验证码错误").unwrap()))
    }
}
