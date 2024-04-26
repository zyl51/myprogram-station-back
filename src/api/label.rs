use actix_web::{get, post, web, HttpRequest, HttpResponse};
use r2d2_mysql::mysql::prelude::Queryable;
use serde::{Deserialize, Serialize};

use crate::{common::token::*, database::mysql::*};

#[derive(Debug, Serialize, Deserialize)]
struct Label {
    id: u32,
    label_name: String,
}

#[get("/get_label_list")]
pub async fn get_label_list(req: HttpRequest)
-> actix_web::Result<HttpResponse> 
{
    println!("----- get_label_list");
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::Ok().body("toekn verify error"));
    }

    let my_pool = MysqlPool::instance();
    
    let query = format!("
        select id, name
        from label;
    ");
    let labels = my_pool.query_map(
        query,
        |(id, name): (u32, String)| Label {
            id,
            label_name: name,
        },
        &my_pool.read_only_txopts
    ).unwrap();

    Ok(HttpResponse::Ok().body(serde_json::to_string(&labels).unwrap()))
}

// 删除标签
#[post("/delete/{label_id}")]
pub async fn delete_label(req: HttpRequest, label_id: web::Path<u32>)
-> actix_web::Result<HttpResponse>
{
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::Ok().body("token verify error"));
    }

    // 获得标签 id
    let label_id = *label_id;

    let query = format!("
        delete ignore
        from label
        where id = {};
    ", label_id);

    let my_pool = MysqlPool::instance();
    my_pool.exec_drop(vec![query], &my_pool.read_write_txopts).unwrap();

    Ok(HttpResponse::Ok().body(serde_json::to_string("删除成功").unwrap()))
}

// 增加标签
#[post("/add/{label}")]
pub async fn add_label(req: HttpRequest, label: web::Path<String>)
-> actix_web::Result<HttpResponse>
{
    if Token::verif_jwt(req).is_err() {
        return Ok(HttpResponse::Ok().body("token verify error"));
    }

    // 获得标签 id
    let label = (*label).clone();

    let my_pool = MysqlPool::instance();
    let query = format!("
        select count(*)
        from name = '{}';
    ", label);

    // 查询条数
    let number:Vec<u32> = my_pool.exec(query, &my_pool.read_only_txopts).unwrap();

    if number[0] == 0 {
        return Ok(HttpResponse::BadRequest().body("标签已经存在"));
    }

    // 增加标签
    let query = format!("
        insert into label
            (label)
        VALUES
            ('{}')
    ", label);

    let mut connection = my_pool.get_connection().unwrap();
    let mut transaction = connection.start_transaction(my_pool.read_write_txopts).unwrap();

    transaction.exec_drop(query, ()).unwrap();
    let label_id: Vec<u32> = transaction.exec("SELECT LAST_INSERT_ID();", ()).unwrap();
    transaction.commit().unwrap();

    Ok(HttpResponse::Ok().body(serde_json::to_string(&label_id).unwrap()))
}