use actix_cors::Cors;
use actix_files::Files;
use actix_web::{http, web, App, HttpResponse, HttpServer};
use log::info;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

mod api;
mod common;
mod database;

// use database::mysql::*;
use common::config::*;

use api::{
    follow::*, login::*, post::*, recommend::*, register::*, search::*, user::*, user_profile::*,
    user_stats::*, user_comment::*, like_collect::*, markdown::*, token::*, forgot_password::*,
    user_profile_psersonal::*, report::*, user_management::*, post_management::*, label::*,
    label_management::*,
};

async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!(
            "D:/Web_lesson/vue/myprogram-station/dist/index.html"
        ))
}

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {

    // 初始化日志文件
    log4rs::init_file("log4rs.yaml", Default::default()).expect("Failed log4rs init");
    // 获取服务端配置
    let ip = Config::instance().server.host.clone();
    let port = Config::instance().server.port;
    let ip_port = format!("{}:{}", ip, port);
    let thread_numbers = Config::instance().server.thread_numbers;

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

    info!("Start SSL configuration");
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())
        .expect("Failed to execute function SSL builder");
    // 这行代码设置了私钥文件，"key.pem" 是私钥文件的路径
    // SslFiletype::PEM 表示私钥文件的格式为 PEM 格式。
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .expect("Failed to execute builder set_private_key_file");
    // 这行代码设置了证书链文件，"cert.pem" 是证书链文件的路径
    builder
        .set_certificate_chain_file("cert.pem")
        .expect("Failed to execute builder set_certificate_chain_file");
    info!("End SSL configuration");
    info!("Start HttpServer");
    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .supports_credentials()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::CONTENT_TYPE,
            ])
            .max_age(3600);

        App::new()
            // .wrap(
            //     Cors::default()
            //     .allow_any_origin()
            //     .allowed_methods(vec!["GET", "POST", "OPTIONS"])
            // ) // 添加这一行，允许跨域请求
            .wrap(cors)
            .service(
                web::scope("/api")
                    .service(get_recommend_post_total_numbers)
                    .service(get_recommend_posts_list)
                    .service(get_follow_relationships)
                    .service(get_follow_posts_list)
                    .service(get_follow_post_total_numbers)
                    .service(get_search)
                    .service(get_cover)
                    .service(get_image)
                    .service(get_post)
                    .service(submit_cover)
                    .service(submit_post)
                    .service(delete_post)
                    .service(get_avatar)
                    .service(get_user)
                    .service(get_is_admin)
                    .service(send_verification_code)
                    .service(verify_verification_code)
                    .service(user_login)
                    .service(get_userprofile_post_total_numbers)
                    .service(get_userprofile_posts)
                    .service(get_userprofile_user)
                    .service(get_userprofile_totalnumbers_collect)
                    .service(get_userprofile_collect_posts)
                    .service(get_message_total)
                    .service(get_message)
                    .service(update_message_read)
                    .service(update_userprofile_username)
                    .service(update_userprofile_avatar)
                    .service(update_userprofile_avatar_url)
                    .service(update_userprofile_password)
                    .service(add_remove_follow)
                    .service(get_user_comment)
                    .service(submit_comment)
                    .service(delete_comment)
                    .service(get_like_collect)
                    .service(add_like)
                    .service(sub_like)
                    .service(add_collect)
                    .service(sub_collect)
                    .service(submit_image)
                    .service(token_get_userinfo)
                    .service(send_forgot_password)
                    .service(verify_forgot_password)
                    .service(submit_report)
                    .service(get_report_total)
                    .service(get_report)
                    .service(delete_report)
                    .service(get_user_management_total)
                    .service(get_user_management_list)
                    .service(ban_user)
                    .service(update_user_info)
                    .service(search_user)
                    .service(get_post_management_total)
                    .service(get_post_management_list)
                    .service(management_update_post_info)
                    .service(search_post)
                    .service(label_get_post)
                    .service(title_label_get_post)
                    .service(
                        web::scope("/label")
                        .service(get_label_list)
                        .service(delete_label)
                        .service(add_label)
                    )
                    .service(
                        web::scope("/management_label")
                        .service(managemrnt_add_label)
                        .service(managemrnt_delete_label)
                    )
            )
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
    .workers(thread_numbers as usize)
    .bind_openssl(ip_port, builder)?
    .run()
    .await
}
