use lazy_static::lazy_static;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rand::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::common::config::*;

pub struct Email {
    pub email_verification: Arc<Mutex<HashMap<String, u32>>>,
    mailer: SmtpTransport,
}

impl Email {
    // 创建一个新的邮箱实例
    fn new() -> Self {
        log::info!("Starting to get Email configuration");
        // 获取邮箱配置
        let email = &Config::instance().email;
        // 想使用的 SMTP 用户的用户名和密码
        let creds = Credentials::new(email.username.to_owned(), email.password.to_owned());

        // 打开与 gmail 的远程连接,
        let mailer = SmtpTransport::relay(&email.gamil)
            .expect("Failed SmtpTransport::relay")
            .credentials(creds)
            .build();

        log::info!("End to get Email configuration");
        Email {
            email_verification: Arc::new(Mutex::new(HashMap::new())),
            mailer,
        }
    }

    // 单例模式
    pub fn instance() -> &'static Self {
        lazy_static! {
            static ref EMAIL: Email = Email::new();
        }
        &EMAIL
    }

    // 用于发送邮件
    pub fn send_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Start executing function Email::send_email");
        // 获取邮件配置
        let email = &Config::instance().email;

        // 构建发送的消息
        let email_message = Message::builder()
            .from(email.username.parse()?) // 邮件发送者
            .to(to.parse()?) // 邮件接收者
            .subject(subject) // 邮件的主题
            .header(ContentType::TEXT_HTML) // 邮件的类型，这里是 HTML
            .body(body.to_string())?; // 邮件的内容

        // 发送邮件
        match self.mailer.send(&email_message) {
            Ok(_) => {
                // log::info!("Email send_email function successfully!");
                // 在这里可以添加成功发送邮件后的逻辑
                return Ok(());
            }
            Err(err) => {
                // log::error!("Failed Email send_email fuction: {}", err);
                return Err(Box::new(err));
                // 在这里可以添加处理失败发送邮件后的逻辑
            }
        }
    }

    // 用于验证邮件
    pub fn verify_verification_code(
        &self,
        email_receiver: &str,
        verification_code: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Start executing function Email::verify_verification_code");

        let verification_hash = self
            .email_verification
            .lock()
            .expect("Failed Email::verify_verification_code function get verification_hash lock");

        println!(
            "{:?}, {:?}",
            verification_hash.get(email_receiver),
            verification_code
        );
        if verification_hash.get(email_receiver) == Some(&verification_code) {
            Ok(())
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Verification failed",
            )))
        }
    }

    // 插入验证码
    pub fn insert(
        &self,
        email_receiver: &str,
        code: u32,
    ) -> Result<(), Box<dyn std::error::Error + '_>> {
        log::info!("Start executing function Email::insert: {}", email_receiver);
        let mut verification_hash = self.email_verification.lock()?;

        println!("start {:?}", verification_hash);
        verification_hash.insert(email_receiver.to_string(), code);
        println!("end {:?}", verification_hash);
        // .ok_or("verification_hash insert error occurred")?;

        log::info!("End executing function Email::insert");
        Ok(())
    }

    // 删除验证码
    pub fn remove(&self, email_receiver: &str) -> Result<(), Box<dyn std::error::Error + '_>> {
        log::info!("Start executing function Email::remove: {}", email_receiver);
        let mut verification_hash = self.email_verification.lock()?;
        verification_hash.remove(&email_receiver.to_string());
        Ok(())
    }

    // 获取随机验证码
    pub fn generate_code() -> u32 {
        let verification = rand::thread_rng().gen_range(100000..1000000);
        verification
    }
}
