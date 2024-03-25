use actix_cors::Cors;
use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

mod api;
mod database;

// use database::mysql::*;

use api::{
    follow::*,
    post::*,
    recommend::*,
    user::*,
};

const IP_PORT: &str = "127.0.0.1:8082";

async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!(
            "D:/Web_lesson/vue/myprogram-station/dist/index.html"
        ))
}

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    // solve();
    /*
    req: 表示执行证书请求操作。
    -x509: 表示生成自签名的 X.509 证书。
    -newkey rsa:4096: 生成一个新的 RSA 密钥，并指定密钥的长度为 4096 位。
    -nodes: 表示生成的私钥文件不使用密码保护，也就是不设置密码。
    -keyout key.pem: 指定生成的私钥文件的输出路径和文件名为 key.pem。
    -out cert.pem: 指定生成的证书文件的输出路径和文件名为 cert.pem。
    -days 365: 指定证书的有效期为 365 天。
    -subj '/CN=localhost': 指定证书的主题，这里指定了证书的通用名称（CN）为 localhost，表示该证书适用于本地主机。
    */
    // 创建 SSL 上下文
    // 加载TLS密钥
    // 要创建用于测试的自签名临时证书，请执行以下操作：
    // openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'

    // 这行代码创建了一个 SSL/TLS 加密器，使用了中间级别的安全配置
    // mozilla_intermediate 是一个 SSL/TLS 安全配置的预定义配置之一
    // SslMethod::tls() 指定了 TLS 协议的版本,现在是 1.2版本
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    // 这行代码设置了私钥文件，"key.pem" 是私钥文件的路径
    // SslFiletype::PEM 表示私钥文件的格式为 PEM 格式。
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    // 这行代码设置了证书链文件，"cert.pem" 是证书链文件的路径
    builder.set_certificate_chain_file("cert.pem").unwrap();

    HttpServer::new(|| {
        App::new()
            .service(
                web::scope("/api")
                    .service(get_recommend_post_total_numbers)
                    .service(get_recommend_posts_list)
                    .service(get_follow_posts_list)
                    .service(get_follow_post_total_numbers)
                    .service(get_cover)
                    .service(get_post)
                    .service(get_avatar)
                    .service(get_user),
            )
            .wrap(Cors::default().allow_any_origin()) // 添加这一行，允许跨域请求
            .service(
                Files::new("/", "D:/Web_lesson/vue/myprogram-station/dist")
                    .index_file("index.html"),
            )
            .route("/", web::get().to(index))
            .route("/recommend/", web::get().to(index))
            .route("/follow/", web::get().to(index))
            .route("/login/", web::get().to(index))
            .route("/register/", web::get().to(index))
            .route("/editor/", web::get().to(index))
            .default_service(web::get().to(index))
    })
    .bind_openssl(IP_PORT, builder)?
    .run()
    .await
}
