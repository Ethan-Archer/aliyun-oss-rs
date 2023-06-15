//!
//! 阿里云对象存储服务（Object Storage Service，简称OSS），是阿里云对外提供的海量、安全、低成本、高可靠的云存储服务。
//!
//! 没有复杂的结构，仅仅为快速调用而实现，设计遵循极简、实用原则，通过 OssClient - OssBucket - OssObject 三层结构，实现了部份常用API，目前不支持的API在后续会逐步增加。
//!
//! 目前仅实现了少量常用API，后续将逐步增加其他API支持。
//!
//!
//! ##### 初始化
//!  ```
//! let client = OssClient::new(
//! "Your AccessKey ID",
//! "Your AccessKey Secret",
//! "oss-cn-zhangjiakou.aliyuncs.com",
//! );
//!
//! ```
//!
//! ##### 查询存储空间列表
//! ```
//! let buckets = client.list_buckets().set_prefix("rust").send().await;
//!
//! ```
//!
//! ##### 查询存储空间中文件列表
//! ```
//! let bucket = client.bucket("for-rs-test").list_objects()
//!              .set_max_objects(200)
//!              .set_prefix("rust")
//!              .send()
//!              .await;
//! ```
//!
//! ##### 上传文件
//! ```
//! let object = client.bucket("for-rs-test").object("rust.png");
//! let result = object.put_object().send_file("Your File Path").await;
//! ```
//!
//! ##### 获取文件访问地址
//! ```
//! use chrono::{Duration, Local};
//!
//! let date = Local::now().naive_local() + Duration::days(3);
//! let url = object.get_url(date).build().await;
//!
//! ```

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
mod sign;
