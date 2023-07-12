//! 文件是 OSS 存储数据的基本单元，文件由元信息、用户数据和文件名（Key）组成，文件由存储空间内部唯一的Key来标识。

pub use self::append_object::AppendObject;
pub use self::copy_object::CopyObject;
pub use self::del_object::DelObject;
pub use self::get_object::GetObject;
pub use self::get_object_meta::GetObjectMeta;
pub use self::get_object_tagging::GetObjectTagging;
pub use self::get_object_url::GetObjectUrl;
#[doc(hidden)]
pub use self::oss_object::OssObject;
pub use self::put_object::PutObject;

mod append_object;
mod copy_object;
mod del_object;
mod get_object;
mod get_object_meta;
mod get_object_tagging;
mod get_object_url;
mod oss_object;
mod put_object;
