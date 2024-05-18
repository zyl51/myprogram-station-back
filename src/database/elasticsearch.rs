use elasticsearch::{
    http::transport::{SingleNodeConnectionPool, TransportBuilder},
    Elasticsearch,
};
use lazy_static::lazy_static;
use serde_json::Value;
use std::sync::{Arc, Mutex};

use crate::common::config::*;

pub struct ElasticsearchPool {
    pool: Arc<Mutex<SingleNodeConnectionPool>>,
}

impl ElasticsearchPool {
    fn new() -> Self {
        // 获取 ES 数据库的配置
        let elasticsearch_config = &Config::instance().elasticsearch;

        // 创建数据库的 url
        let url = format!(
            "
            http://{}:{}
        ",
            elasticsearch_config.host.as_str(),
            elasticsearch_config.port
        );

        // 创建 ES 数据库的一个连接池
        let conn_pool = SingleNodeConnectionPool::new(url.parse().unwrap());

        ElasticsearchPool {
            pool: Arc::new(Mutex::new(conn_pool)),
        }
    }

    // 获取 ES 数据库连接实例
    pub fn instance() -> &'static Self {
        lazy_static! {
            static ref ELASTICSEARCHPOOL: ElasticsearchPool = ElasticsearchPool::new();
        }
        &ELASTICSEARCHPOOL
    }

    pub fn get_elasticsearch(&self) -> Result<Elasticsearch, Box<dyn std::error::Error + '_>> {
        // 创建一个 ES 数据库连接池
        let pool = self.pool.lock()?.clone();
        // 构建不带身份验证的传输对象
        let transport = TransportBuilder::new(pool).disable_proxy().build()?;

        // 使用传输对象创建Elasticsearch客户端
        let client = Elasticsearch::new(transport);

        Ok(client)
    }

    pub async fn add_or_update_article(
        &self,
        post_id: u32,
        label_id: u32,
        title: String,
        content: String,
    ) -> Result<(), Box<dyn std::error::Error + '_>> {
        let client = self.get_elasticsearch()?;

        let request_body = serde_json::json!({
            "label": label_id.to_string(),
            "title": title,
            "content": content,
        });

        // 获取 ES 数据库的配置
        let elasticsearch_config = &Config::instance().elasticsearch;

        // 插入文档，如果文章 id 已经存在，则是对文章进行修改
        let response = client.index(elasticsearch::IndexParts::IndexId(elasticsearch_config.index.as_str(), &post_id.to_string()))
            .body(request_body)
            .send()
            .await?;

        if response.status_code().is_success() {
            log::info!("success executing Elasticsearch add_or_update_article");
        } else {
            log::error!("executing Elasticsearch add_or_update_article");
        }

        Ok(())
    }


    pub async fn delete_article(&self, post_id: u32) -> Result<(), Box<dyn std::error::Error + '_>> {
        let client = self.get_elasticsearch()?;

        // 获取 ES 连接库配置
        let elasticsearch_config = &Config::instance().elasticsearch;

        let response = client
            .delete(elasticsearch::DeleteParts::IndexId(elasticsearch_config.index.as_str(), &post_id.to_string()))
            .send()
            .await?;

        if response.status_code().is_success() {
            log::info!("success delete article {}", post_id);
        } else {
            log::error!("executing Elasticsearch delete_article");
        }

        Ok(())
    }


    pub async fn search_article(&self, start: u32, search_query: &str) -> Result<Value, Box<dyn std::error::Error + '_>> {
        let client = self.get_elasticsearch()?;

        // 获取 ES 连接库配置
        let elasticsearch_config = &Config::instance().elasticsearch;

        // 构建查询语句
        let query = serde_json::json!({
            "query":{
                "multi_match":{
                    "fields" : [ "content", "title"],
                    "query" : search_query,
                },
            },
            "from": start,
            "size": 20,
        });

        
        let response = client
            .search(elasticsearch::SearchParts::Index(&[elasticsearch_config.index.as_str()]))
            .body(query)
            .send()
            .await?;

        let response_body: Value = response.json().await?;

        Ok(response_body)
    }
}
