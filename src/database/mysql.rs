use lazy_static::lazy_static;
use r2d2::Pool;
use r2d2_mysql::{
    // 导入数据库配置，
    mysql::{prelude::*, AccessMode, IsolationLevel, OptsBuilder, TxOpts},
    MySqlConnectionManager,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::common::config::*;

// 定义默认头像和文章默认封面
pub const AVATAR_URL: &str = "https://127.0.0.1:8082/api/avatar/0";
// const COVER_URL: &str = "";

// 返回用户的登录信息
#[derive(Debug, Deserialize, Serialize)]
pub struct UserInfomation {
    pub id: u32,
    pub name: String,
    pub avatar_url: String,
    pub follower_count: u32,
    pub fans: u32,
    pub token: String,
}

// 返回帖子的结构体
#[derive(Debug, Deserialize, Serialize)]
pub struct Post {
    pub id: u32,
    pub title: String,
    pub release_time: String,
    pub cover_url: String,
    pub content: String,
    pub user_id: u32,
    pub user_name: String,
}

pub struct MysqlPool {
    pool: Arc<Mutex<Pool<MySqlConnectionManager>>>,
    pub read_only_txopts: TxOpts,
    pub read_write_txopts: TxOpts,
}

impl MysqlPool {
    fn new() -> Self {
        // Mysql 的连接配置
        let mysql = &Config::instance().mysql;
        let opts = OptsBuilder::new()
            .ip_or_hostname(Some(mysql.host.as_str()))
            .user(Some(mysql.username.as_str()))
            .pass(Some(mysql.password.as_str()))
            .db_name(Some(mysql.db_name.as_str()));

        // 创建 Mysql 连接管理器
        let manager = MySqlConnectionManager::new(opts);

        // 创建数据库连接池 max_size 为连接池的最大数量
        let pool = Pool::builder()
            .max_size(15)
            .build(manager)
            .expect("Database connection pool builder failed");

        let read_only_txopts = TxOpts::default()
            .set_with_consistent_snapshot(true) // 开启事务快照
            .set_isolation_level(Some(IsolationLevel::RepeatableRead)) // 设置事务的隔离级别
            .set_access_mode(Some(AccessMode::ReadOnly)); // 只允许可读

        let read_write_txopts = TxOpts::default()
            .set_with_consistent_snapshot(true) // 开启事务快照
            .set_isolation_level(Some(IsolationLevel::RepeatableRead)) // 设置事务的隔离级别
            .set_access_mode(Some(AccessMode::ReadWrite)); // 允许读写

        MysqlPool {
            pool: Arc::new(Mutex::new(pool)),
            read_only_txopts,
            read_write_txopts,
        }
    }

    // 获取数据库连接池实例
    pub fn instance() -> &'static Self {
        lazy_static! {
            static ref MYSQLPOOL: MysqlPool = MysqlPool::new();
        }
        &MYSQLPOOL
    }

    // 获取数据库连接
    pub fn get_connection(
        &self,
    ) -> Result<r2d2::PooledConnection<MySqlConnectionManager>, Box<dyn std::error::Error + '_>>
    {
        let pool = self.pool.lock()?;
        // 将 r2d2::Error 类型的错误转换为 Box<dyn std::error::Error> 类型的错误
        pool.get().map_err(move |err| err.into())
    }

    // 封装数据库的 exec 方法
    pub fn exec<T, S>(
        &self,
        query: S,        // 查询语句
        txopts: &TxOpts, // 只读 或 读写
    ) -> Result<Vec<T>, Box<dyn std::error::Error + '_>>
    where
        S: AsStatement + std::fmt::Debug,
        T: FromRow,
    {
        // 获取连接
        let mut connection = self.get_connection()?;

        // 开启事务， 只允许读操作
        let mut transaction = connection.start_transaction(*txopts)?;

        // println!("{:?}", query);
        log::info!("my_pool exec function execution query: {:?}", query);

        // 匹配正确和错误
        match transaction.exec(query, ()) {
            Ok(result) => {
                // 提交事务
                transaction.commit()?;
                log::info!("my_pool exec transaction has been commit");
                return Ok(result);
            }
            Err(err) => {
                // 这里好像回滚和不回滚都是一样的
                log::error!("my_pool exec function encountered an error: {:?}", err);
                transaction.rollback()?;
                return Err(Box::new(err));
            }
        }
    }

    // 封装数据库的 query_map 方法
    pub fn query_map<T, F, Q, U>(
        &self,
        query: Q,
        f: F,
        txopts: &TxOpts,
    ) -> Result<Vec<U>, Box<dyn std::error::Error + '_>>
    where
        Q: AsRef<str> + std::fmt::Debug,
        T: FromRow,
        F: FnMut(T) -> U,
    {
        // 获取连接
        let mut connection = self.get_connection()?;

        // 开启事务， 允许读写操作
        let mut transaction = connection.start_transaction(*txopts)?;

        log::info!("my_pool query_map function execution query: {:?}", query);
        // 匹配正确和错误
        match transaction.query_map(query, f) {
            Ok(result) => {
                // 提交事务
                transaction.commit()?;
                log::info!("my_pool query_map transaction has been commit");
                Ok(result)
            }
            Err(err) => {
                // 这里好像回滚和不回滚都是一样的
                log::error!("my_pool query_map function encountered an error: {:?}", err);
                transaction.rollback()?;
                Err(Box::new(err))
            }
        }
    }

    pub fn exec_drop<S>(
        &self,
        querys: Vec<S>,
        txopts: &TxOpts,
    ) -> Result<(), Box<dyn std::error::Error + '_>>
    where
        S: AsStatement + std::fmt::Debug,
    {
        let mut connection = self.get_connection()?;

        // log::info!("my_pool exec_drop function execution query: {:?}", querys);
        let mut transaction = connection.start_transaction(*txopts)?;

        for query in querys {
            match transaction.exec_drop(query, ()) {
                Ok(_) => {
                    log::debug!("my_pool exec_drop function execution query successful");
                }
                Err(err) => {
                    log::error!("my_pool exec_drop function encountered an error: {:?}", err);
                    transaction.rollback()?;
                    return Err(Box::new(err));
                }
            };
        }
        transaction.commit()?;
        Ok(())
    }

    // 执行插入操作
    pub fn query_drop(
        &self,
        query: &str,
        txopts: &TxOpts,
    ) -> Result<u32, Box<dyn std::error::Error + '_>> {
        // 获取连接
        let mut connection = self.get_connection()?;

        // 开启事务， 允许读写操作
        let mut transaction = connection.start_transaction(*txopts)?;

        // 执行插入语句，匹配正确和错误
        match transaction.query_drop(query) {
            Ok(_) => {
                // 提交事务
                // 执行查询以获取最后插入的自增主键值
                let result: Option<u32> = transaction.query_first("SELECT LAST_INSERT_ID()")?;

                match result {
                    Some(value) => {
                        // 查询成功提交事务和返回正确值
                        transaction.commit()?;
                        return Ok(value);
                    }
                    None => {
                        // 事务进行回滚
                        transaction.rollback()?;
                        return Err("Last insert id not found".into());
                    }
                };
            }
            Err(err) => {
                // 这里好像回滚和不回滚都是一样的
                transaction.rollback()?;
                return Err(Box::new(err));
            }
        }
    }
}
