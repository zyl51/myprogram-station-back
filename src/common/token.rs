use actix_web::HttpRequest;
use chrono::prelude::*;
use chrono::Duration;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::database::*;

use self::mysql::MysqlPool;

/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    id: u32,
    user_email: String,
    password: String,
    exp: i64,
}

impl Claims {
    pub fn new(id: u32, user_email: &str, password: &str) -> Self {
        let now = Utc::now();
        let exp = now + Duration::try_minutes(60).expect("Invalid minutes");
        // println!("exp: {}", exp);
        Self {
            id,
            user_email: user_email.to_string(),
            password: password.to_string(),
            exp: exp.timestamp(),
        }
    }
    pub fn get_id(&self) -> u32 {
        return self.id;
    }

    pub fn verify(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Claims verify start");
        log::info!("Start executing Claims::verify");
        let my_pool = MysqlPool::instance();

        let query = format!(
            "
            select user_id, email, password, is_ban
            from login 
            where user_id = {}",
            self.id
        );

        let users: Vec<(u32, String, String, bool)> = match my_pool.exec(query, &my_pool.read_only_txopts)
        {
            Ok(result) => result,
            Err(err) => {
                log::error!("Error Claims verif is : {:?}", err);
                return Err(err);
            }
        };

        if users.len() == 0 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "user does not exist",
            )));
        }

        let user = &users[0];

        if user.3 == true {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "user banned",
            )));
        }

        println!("{}, {}, {}", user.0, user.1, user.2);

        let now = Utc::now().timestamp();
        if user.1 != self.user_email || user.2 != self.password || now > self.exp {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to verify Token",
            )));
        }

        log::info!("End executing Claims::verify");
        Ok(())
    }

    pub fn is_ban(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("Start Claims: is_ban function");

        let my_pool = MysqlPool::instance();

        let query = format!(
            "
            select is_ban
            from login 
            where user_id = {}",
            self.id
        );

        let is_ban: Vec<bool> = match my_pool.exec(query, &my_pool.read_only_txopts)
        {
            Ok(result) => result,
            Err(err) => {
                log::error!("Error Claims is_ban is : {:?}", err);
                return Err(err);
            }
        };

        // 不存在该用户
        if is_ban.len() == 0 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "user does not exist",
            )));
        }

        let is_ban = is_ban[0];

        // 如果用户被封禁
        if is_ban == true {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "user banned",
            )));
        }
        
        log::debug!("End Claims: is_ban function");
        Ok(())
    }

    pub fn is_admin(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("Start Claims: is_admin function");

        let my_pool = MysqlPool::instance();
        let query = format!("
            select user_id
            from admin
            where user_id = {};
        ", self.id);

        let admin: Vec<u32> = match my_pool.exec(query, &my_pool.read_only_txopts) {
            Ok(ok) => ok,
            Err(err) => {
                log::error!("Calims: is_admin executing my_pool.exec, err:{:?}", err);
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Server Error",
                )));
            }
        };

        // 不是管理员
        if admin.len() == 0 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Not is admin",
            )));
        }

        log::debug!("End Claims: is_admin function");
        Ok(())
    }
}

pub struct Token;

impl Token {
    // 根据 id，邮箱，密码和时间生成 token
    pub fn get_jwt(claims: &Claims) -> Result<String, Box<dyn std::error::Error>> {
        log::debug!("Start Token::get_jwt");
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("my name is zyl".as_ref()),
        )?;

        // println!("get jwt{:?}", token);
        log::debug!("End Token::get_jwt");
        Ok(token)
    }

    pub fn verif_jwt(req: HttpRequest) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("Start Token::verif_jwt");
        println!("Token verif_jwt");
        // 通过 token 获取 Claims 结构体 
        let user_info = match Token::token_to_claims(req) {
            Ok(ok) => ok, 
            Err(err) => {
                log::error!("Error is Token verif_jwt");
                return Err(err);
            }
        };
        println!("Start verif_jwt user_info.verify");
        if user_info.verify().is_err() {
            log::error!("Failed to verify Token");
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to verify Token",
            )));
        }

        log::debug!("End Token::verif_jwt");
        Ok(())
    }

    pub fn token_to_claims(req: HttpRequest) -> Result<Claims, Box<dyn std::error::Error>> {
        // 获取请求头中的
        let auth_header = req.headers().get("Authorization");
        // println!("{:?}", auth_header);
        // 取出token
        let token = if let Some(auth_header_value) = auth_header {
            if let Ok(auth_str) = auth_header_value.to_str() {
                // 看看是不是 Bearer 开头的字符串
                if auth_str.starts_with("Bearer ") {
                    // 删除掉 "Bearer " 字符串
                    log::info!("delete Bearer ");
                    auth_str.trim_start_matches("Bearer ").to_owned()
                } else {
                    log::error!("No start with Bearer");
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to verify Token",
                    )));
                }
            } else {
                log::error!("auth_str is None");
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to verify Token",
                )));
            }
        } else {
            log::error!("auth_header_value is None");
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to verify Token",
            )));
        };

        // println!("{}", token);

        // 结构出我们的 token 数据
        let token = decode::<Claims>(
            &token,
            &DecodingKey::from_secret("my name is zyl".as_ref()),
            &Validation::default(),
        )?;

        let user_info = token.claims;

        Ok(user_info)
    }
}
