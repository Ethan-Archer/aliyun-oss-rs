//!
//! 阿里云对象存储服务（Object Storage Service，简称OSS），是阿里云对外提供的海量、安全、低成本、高可靠的云存储服务。
//!
//! 设计遵循极简、实用原则，尽可能通过链式操作，OssClient - OssBucket - OssObject - Operation 层级结构，实现了部分常用API，目前不支持的API在后续会逐步增加。
//!
//! #### 提醒
//! - 暂不支持版本控制功能，如你的存储空间已经开启了版本控制，可能会出现功能和数据不全的情况
//! - 大部份方法的参数的字符合法性未进行校验，需要严格按照OSS要求传参，否则可能会产生本地或远程错误
//!
//! ## 使用方法
//! ##### 初始化
//!  ```
//! let client = OssClient::new(
//! "Your AccessKey ID",
//! "Your AccessKey Secret",
//! );
//!
//! ```
//!
//! ##### 查询存储空间列表
//! ```
//! let bucket_list = client.list_buckets().set_prefix("rust").send().await;
//!
//! ```
//!
//! ##### 查询存储空间中文件列表
//! ```
//! let bucket = client.bucket("for-rs-test","oss-cn-zhangjiakou.aliyuncs.com");
//! let files = bucket.list_objects().send().await;
//! ```
//!
//! ##### 上传文件
//! ```
//! let object = bucket.object("rust.png");
//! let result = object.put_object().send_file("Your File Path").await;
//! ```
//!
//! ##### 获取文件访问地址
//! ```
//! use chrono::{Duration, Local};
//!
//! let date = Local::now().naive_local() + Duration::days(3);
//! let url = object.get_object_url().url(date);
//!
//! ```
//!

#[doc(inline)]
pub use crate::bucket::OssBucket;
#[doc(inline)]
pub use crate::client::OssClient;
#[doc(inline)]
pub use crate::error::Error;
#[doc(inline)]
pub use crate::object::OssObject;

pub mod bucket;
pub mod client;
pub mod common;
mod error;
pub mod object;
mod request;
