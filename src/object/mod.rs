//! 对象是 OSS 存储数据的基本单元，对象由元信息、用户数据和文件名（Key）组成，对象由存储空间内部唯一的Key来标识。

pub use self::append_object::AppendObject;
pub use self::del_object::DelObject;
pub use self::get_object_tagging::GetObjectTagging;
pub use self::get_url::GetUrl;
#[doc(hidden)]
pub use self::oss_object::OssObject;
pub use self::put_object::PutObject;

mod append_object;
mod del_object;
mod get_object_tagging;
mod get_url;
mod oss_object;
mod put_object;
