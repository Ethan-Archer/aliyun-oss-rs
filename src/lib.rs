//!
//! ** 目前库正在快速开发和调整中，所有功能都可以正常使用，但v1.0版本之前的所有调整都不保证向后兼容，请谨慎使用 **
//! 阿里云对象存储服务（Object Storage Service，简称OSS），是阿里云对外提供的海量、安全、低成本、高可靠的云存储服务。
//!
//! 没有复杂的结构，仅仅为快速调用而实现，设计遵循极简、实用原则，通过 OssClient - OssBucket - OssObject 三层结构，实现了部份常用API，目前不支持的API在后续会逐步增加。
//!
//! 目前仅实现了少量常用API，后续将逐步增加其他API支持。
//!
//! ## 使用方法
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
//!
//!
//! ## OSS返回结果的处理
//! 阿里云这套接口，是由无数人分开写，而后拼凑的，缺少统一的规范，导致返回格式乱七八糟，错误处理也很奇怪，所以处理返回结果的时候需要特别留意。
//!
//! 当阿里云返回成功，即2xx状态码，并且返回内容被正确解析的情况下，会正常返回，否则将返回如下错误码：
//! ```
//! use bytes::Bytes;
//!
//! // [aliyun_oss_rs::Error](enum.Error.html) 中只有这两个错误码是请求阿里云成功之后会返回的
//! // 其他错误都是本地或网络错误
//! #[error("OSS返回了成功，但消息体解析失败，请自行解析")]
//! OssInvalidResponse(Option<Bytes>),
//! #[error("OSS返回了错误，HTTP状态码：{0}，错误内容：\n{1:?}")]
//! OssError(reqwest::StatusCode, Option<Bytes>),
//! ```
//!
//! #### OssInvalidResponse
//!
//! 这个错误码代表阿里云正常返回，并且是请求成功的2xx状态，但是解析返回内容失败了，如果你需要自行解析返回内容，可以读取后面返回的`Option<Bytes>`，网络失败之类的情况下，可能会出现为Noned的情况
//!
//! #### OssError
//!
//! 当阿里云返回了非2xx状态码的时候，会尝试去获取返回内容，并返回给调用者，网络失败之类的情况下，可能会出现为Noned的情况。阿里云的返回内容，绝大部分时候是未经任何编码的XM，但是有部分例外，需要留心相应的方法说明
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
mod send;
