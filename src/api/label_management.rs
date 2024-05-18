use actix_web::{post, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::common::token::Token;
use crate::database::mysql::MysqlPool;

#[derive(Debug, Deserialize, Serialize)]
struct AddLabel {
    label_name: String,
}

#[post("/add_label")]
pub async fn managemrnt_add_label(req: HttpRequest, info: web::Json<AddLabel>)
-> actix_web::Result<HttpResponse> {

    println!("---- managemrnt_add_label");

    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("token verify error"));
    }

    let AddLabel { label_name } = info.into_inner();

    let query = format!("
        insert into label
            (name)
        VALUES
            ('{}')
    ", label_name);

    let my_pool = MysqlPool::instance();

    // 增加新标签
    let label_id = my_pool.query_drop(&query, &my_pool.read_write_txopts).unwrap();

    Ok(HttpResponse::Ok().body(serde_json::to_string(&label_id).unwrap()))
}

#[derive(Debug, Deserialize)]
struct DeleteLabel {
    label_id: u32,
}

#[post("/delete_label")]
pub async fn managemrnt_delete_label(req: HttpRequest, info: web::Json<DeleteLabel>)
-> actix_web::Result<HttpResponse> {

    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::BadRequest().body("token verify error"));
    }

    let DeleteLabel { label_id } = info.into_inner();

    let query = format!("
        select count(*)
        from post
        where label_id = {}
    ", label_id);

    let my_pool = MysqlPool::instance();

    let number: Vec<u32> = my_pool.exec(query, &my_pool.read_only_txopts).unwrap();

    if number[0] > 0 {
        return Ok(HttpResponse::Forbidden().body("forbid delete"));
    }

    let query = format!("
        delete from label
        where id = {};
    ", label_id);

    my_pool.exec_drop(vec![query], &my_pool.read_write_txopts).unwrap();

    Ok(HttpResponse::Ok().body("delete success"))
}