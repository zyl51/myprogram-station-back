use lazy_static::lazy_static;
use std::sync::Mutex;
use r2d2::Pool;
use r2d2_mysql::{
    mysql::OptsBuilder, MySqlConnectionManager,
};

pub struct MysqlPool {
    pool: Pool<MySqlConnectionManager>,
}

impl MysqlPool {
    fn new() -> Self {
        // Mysql 的连接配置
        let opts = OptsBuilder::new()
            .ip_or_hostname(Some("localhost"))
            .user(Some("zyl"))
            .pass(Some("password"))
            .db_name(Some("program_station"));

        // 创建 Mysql 连接管理器
        let manager = MySqlConnectionManager::new(opts);

        // 创建数据库连接池 max_size 为连接池的最大数量
        let pool = Pool::builder().max_size(15).build(manager).unwrap();

        MysqlPool { pool }
    }

    pub fn instance() -> &'static Mutex<Self> {
         lazy_static! {
            static ref INSTANCE: Mutex<MysqlPool> = Mutex::new(MysqlPool::new());
         }
         &INSTANCE
    }

    pub fn get_connection(&self) -> Result<r2d2::PooledConnection<MySqlConnectionManager>, r2d2::Error> {
        // let instance_lock = MysqlPool::instance().lock().expect("Failed get mysql pool");
        // let connection = instance_lock.pool.get()?;
        // drop(instance_lock); // 释放锁
        // Ok(connection)
        self.pool.get()
    }

    
}

// cargo test -- --nocapture
#[cfg(test)]
mod mysql_tests {
    use super::*;
    use r2d2_mysql::mysql::{
        prelude::Queryable,
        TxOpts, IsolationLevel, AccessMode
    };


    // 映射到结构体中
    #[test]
    fn test_connection_quert_map() {
        // 获取数据库连接池实例
        let pool = MysqlPool::instance().lock()
            .expect("Failed to acquire lock");
        
        // 获取数据库连接
        let mut connection = pool.get_connection()
            .expect("Failed to get connection");
        drop(pool); // 释放锁

        // 执行 SQL 查询操作
        #[allow(dead_code)]
        #[derive(Debug)]
        struct Post {
            id: usize,
            title: String,
            release_time: String,
            cover_url: String,
            content_url: String,
            user_id: usize,
            user_name: String,
        }
        let posts: Vec<Post> = connection.query_map(
            "SELECT * FROM post",
            |(id, title, release_time, cover_url, content_url, user_id, user_name)| {
                Post { id, title, release_time, cover_url, content_url, user_id, user_name }
            },
        ).unwrap();

        for post in posts {
            println!("query_map post: {:?}", post);
        }
        
        assert!(true);
    }

    // 查询分组
    #[test]
    fn test_connection_exec() {
        // println!("test_connection_exec");
        let pool = MysqlPool::instance().lock()
            .expect("Failed to acquire lock");
        let mut connection = pool.get_connection()
            .expect("Failed to get connection");
        
        // 释放锁
        drop(pool);
        
        let query = "SELECT title FROM post";
        let result: Vec<(String,)> = connection.exec(query, ())
            .expect("connection exec");

        // println!("error");
        for res in result {
            println!("exec res:{:?}", res.0);
        }
        assert!(true);
    }

    // 使用 Mysql 事务代码
    #[test]
    fn test_transaction() {
        let pool = MysqlPool::instance().lock()
            .expect("Failed to acquire lock");
        
        let mut connection = pool.get_connection()
            .expect("Failed to get connection");

        drop(pool);
        // 创建事务的的配置
        let opts = TxOpts::default()
            .set_with_consistent_snapshot(true)
            .set_isolation_level(Some(IsolationLevel::RepeatableRead))
            .set_access_mode(Some(AccessMode::ReadOnly));
        println!("{:?}", opts);

        // 开启 Mysql 事务
        let mut transaction = connection.start_transaction(opts)
            .expect("Failed satrt_transaction");

        // 使用事务进行数据的读取
        let query = "SELECT id, title FROM post";
        let result: Vec<(u32, String,)> = transaction.exec(query, ())
            .expect("connection exec");

        for res in result {
            println!("{}, {}", res.0, res.1);
        }

        // 提交事务
        transaction.commit().unwrap();
        // 回滚事务
        // transaction.rollback().unwrap();
    }
}