use lazy_static::lazy_static;
use std::fs;
use toml;

#[derive(Debug)]
pub struct ServerConfig {
    pub host: String,
    pub port: u32,
    pub thread_numbers: u32,
}

#[derive(Debug)]
pub struct MysqlConfig {
    pub host: String,
    pub port: u32,
    pub username: String,
    pub password: String,
    pub db_name: String,
    pub pool_numbers: u32,
}

#[derive(Debug)]
pub struct EmailConfig {
    pub username: String,
    pub password: String,
    pub gamil: String,
}

// 定义 Config 结构体
#[derive(Debug)]
pub struct Config {
    pub server: ServerConfig,
    pub mysql: MysqlConfig,
    pub email: EmailConfig,
}

impl Config {
    // 创建一个新的 Config 实例
    fn new() -> Self {
        log::info!("Starting to get project configuration");
        // 读取配置文件的内容
        let contents =
            fs::read_to_string("./config.toml").expect("Failed Config init fs::read_to_string");

        // 解析 TOML 配置文件
        let config: toml::Value = contents.parse().expect("Failed Config init contents.parse");

        // 解析服务器配置
        let server_config = ServerConfig {
            host: config["server"]["host"]
                .as_str()
                .unwrap_or("127.0.0.1")
                .to_string(),
            port: config["server"]["port"].as_integer().unwrap_or(8080) as u32,
            thread_numbers: config["server"]["thread_numbers"].as_integer().unwrap_or(80) as u32,
        };

        // 解析 MySQL 配置
        let mysql_config = MysqlConfig {
            host: config["mysql"]["host"]
                .as_str()
                .unwrap_or("localhost")
                .to_string(),
            port: config["mysql"]["port"].as_integer().unwrap_or(3306) as u32,
            username: config["mysql"]["username"]
                .as_str()
                .unwrap_or("root")
                .to_string(),
            password: config["mysql"]["password"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            db_name: config["mysql"]["db_name"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            pool_numbers: config["mysql"]["pool_numbers"].as_integer().unwrap_or(15) as u32,
        };

        let email_config = EmailConfig {
            username: config["email"]["username"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            password: config["email"]["password"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            gamil: config["email"]["gmail"].as_str().unwrap_or("").to_string(),
        };

        log::info!("End to get project configuration");
        Config {
            server: server_config,
            mysql: mysql_config,
            email: email_config,
        }
    }

    // 返回全局的 Config 实例
    pub fn instance() -> &'static Config {
        // 使用 lazy_static 宏定义全局的 Config 实例
        lazy_static! {
            static ref CONFIG: Config = Config::new();
                // .expect("create config is failed");
        }
        &CONFIG
    }
}
