//! 公共数据定义
//!
//!
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use serde_derive::{Deserialize, Serialize};
use std::fmt;

// -------------------------- 公共方法 --------------------------
//编码查询参数值
const URL_ENCODE: &AsciiSet = &NON_ALPHANUMERIC.remove(b'-').remove(b'/');
pub(crate) fn url_encode(input: &str) -> String {
    utf8_percent_encode(input, URL_ENCODE).to_string()
}

//检测metadata中key是否合规
pub(crate) fn invalid_metadata_key(input: &str) -> bool {
    for c in input.chars() {
        if !c.is_ascii_alphanumeric() && c != '-' {
            return true;
        }
    }
    false
}

// -------------------------- 公共数据 --------------------------

/// 访问权限ACL
#[derive(Debug, Deserialize, Clone)]
pub enum Acl {
    /// 仅用于文件ACL，代表文件ACL继承存储空间ACL
    #[serde(rename = "default")]
    Default,
    /// 私有，读写请求全部需要经过授权
    #[serde(rename = "private")]
    Private,
    /// 公共读，存储空间中的文件可以被匿名读取，但无法写入文件
    #[serde(rename = "public-read")]
    PublicRead,
    /// 公共读写，存储空间中的文件可以被匿名读取和写入
    #[serde(rename = "public-read-write")]
    PublicReadWrite,
}
impl fmt::Display for Acl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Acl::Default => "default",
            Acl::Private => "private",
            Acl::PublicRead => "public-read",
            Acl::PublicReadWrite => "public-read-write",
        };
        write!(f, "{}", value)
    }
}

///存储类型
#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum StorageClass {
    /// 标准存储
    Standard,
    /// 低频访问
    IA,
    /// 归档存储
    Archive,
    /// 冷归档存储
    ColdArchive,
    /// 深度冷归档存储
    DeepColdArchive,
}
impl fmt::Display for StorageClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageClass::Standard => f.write_str("Standard"),
            StorageClass::IA => f.write_str("IA"),
            StorageClass::Archive => f.write_str("Archive"),
            StorageClass::ColdArchive => f.write_str("ColdArchive"),
            StorageClass::DeepColdArchive => f.write_str("DeepColdArchive"),
        }
    }
}

///数据容灾类型
#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum DataRedundancyType {
    ///本地冗余LRS将您的数据冗余存储在同一个可用区的不同存储设备上，可支持两个存储设备并发损坏时，仍维持数据不丢失，可正常访问。
    LRS,
    ///同城冗余ZRS采用多可用区（AZ）内的数据冗余存储机制，将用户的数据冗余存储在同一地域（Region）的多个可用区。当某个可用区不可用时，仍然能够保障数据的正常访问。
    ZRS,
}
impl fmt::Display for DataRedundancyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataRedundancyType::LRS => f.write_str("LRS"),
            DataRedundancyType::ZRS => f.write_str("ZRS"),
        }
    }
}

///解冻优先级
#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum RestoreTier {
    /// 高优先级
    Expedited,
    /// 标准
    Standard,
    /// 批量
    Bulk,
}
impl fmt::Display for RestoreTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RestoreTier::Standard => f.write_str("Standard"),
            RestoreTier::Expedited => f.write_str("Expedited"),
            RestoreTier::Bulk => f.write_str("Bulk"),
        }
    }
}

/// http头，cache_control
#[derive(Debug, Clone)]
pub enum CacheControl {
    NoCache,
    NoStore,
    Public,
    Private,
    MaxAge(u32),
}
impl fmt::Display for CacheControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheControl::NoCache => f.write_str("no-cache"),
            CacheControl::NoStore => f.write_str("no-store"),
            CacheControl::Public => f.write_str("public"),
            CacheControl::Private => f.write_str("private"),
            CacheControl::MaxAge(val) => {
                f.write_str("max-age=")?;
                f.write_fmt(format_args!("{}", val))
            }
        }
    }
}

/// http头，content-disposition
#[derive(Debug, Clone)]
pub enum ContentDisposition {
    Inline,
    Attachment,
    AttachmentWithNewName(String),
}
impl fmt::Display for ContentDisposition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentDisposition::Inline => f.write_str("inline"),
            ContentDisposition::AttachmentWithNewName(file_name) => {
                let content_disposition_value = format!(
                    "attachment;filename=\"{0}\";filename*=UTF-8''{0}",
                    url_encode(file_name)
                );
                f.write_str(&content_disposition_value)
            }
            ContentDisposition::Attachment => f.write_str("attachment"),
        }
    }
}

/// 所有者信息
#[derive(Debug, Deserialize)]
pub struct Owner {
    /// 用户ID
    #[serde(rename = "ID")]
    pub id: u64,
    /// 用户名称
    #[serde(rename = "DisplayName")]
    pub display_name: String,
}
