//! 本地定义的各种数据
//!
//!
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use serde_derive::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::{
    collections::{hash_map::Iter, HashMap},
    fmt,
};

// -------------------------- 公共方法 --------------------------
//编码查询参数值
const URL_ENCODE: &AsciiSet = &NON_ALPHANUMERIC.remove(b'-').remove(b'/');
pub(crate) fn url_encode(input: &str) -> String {
    utf8_percent_encode(input, URL_ENCODE).to_string()
}

// -------------------------- 公共数据 --------------------------

// 迭代器
#[derive(Debug, Deserialize)]
pub(crate) struct OssInners {
    inners: HashMap<String, String>,
}
impl OssInners {
    pub fn new() -> Self {
        let inners = HashMap::with_capacity(10);
        OssInners { inners }
    }
    pub fn from(key: impl ToString, value: impl ToString) -> Self {
        let mut inners = HashMap::with_capacity(10);
        inners.insert(key.to_string(), value.to_string());
        OssInners { inners }
    }
    pub fn insert(&mut self, key: impl ToString, value: impl ToString) {
        self.inners.insert(key.to_string(), value.to_string());
    }
    pub fn len(&self) -> usize {
        self.inners.len()
    }
    pub fn iter(&self) -> Iter<'_, String, String> {
        self.inners.iter()
    }
}

/// 访问权限ACL
#[derive(Debug, Deserialize, Clone)]
pub enum Acl {
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
            Acl::Private => "private",
            Acl::PublicRead => "public-read",
            Acl::PublicReadWrite => "public-read-write",
        };
        write!(f, "{}", value)
    }
}

///存储类型
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

// -------------------------- OSS API 返回的XML结构 --------------------------

// ------ list_buckets ------
/// Bucket基础信息
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BucketBase {
    /// Bucket名称
    pub name: String,
    /// 所在地域
    pub region: String,
    /// 所在地域在oss服务中的标识
    pub location: String,
    ///外网endpoint
    pub extranet_endpoint: String,
    /// 内网endpoint
    pub intranet_endpoint: String,
    /// 存储类型    
    pub storage_class: StorageClass,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct Buckets {
    pub bucket: Option<Vec<BucketBase>>,
}

// 查询存储空间列表的结果集合
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct ListAllMyBucketsResult {
    /// 如果一次查询未穷尽所有存储空间，next_marker则可用于下一次继续查询
    pub next_marker: Option<String>,
    /// 存储空间列表
    pub buckets: Buckets,
}

/// 查询存储空间列表的结果集合
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListAllMyBuckets {
    /// 如果一次查询未穷尽所有存储空间，next_marker则可用于下一次继续查询
    pub next_marker: Option<String>,
    /// 存储空间列表
    pub buckets: Option<Vec<BucketBase>>,
}

// ------ describe_regions ------
/// Region基础信息
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RegionInfo {
    /// 地域ID
    pub region: String,
    /// 地域对应的传输加速Endpoint
    pub accelerate_endpoint: String,
    /// 地域对应的内网Endpoint
    pub internal_endpoint: String,
    /// 地域对应的外网Endpoint
    pub internet_endpoint: String,
}

#[doc(hidden)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct RegionInfoList {
    pub region_info: Vec<RegionInfo>,
}

// ------ list_objects ------
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ObjectsList {
    // 列表继续请求的token
    pub next_continuation_token: Option<String>,
    // 文件列表
    pub contents: Option<Vec<ObjectInfo>>,
    // 分组列表
    pub common_prefixes: Option<Vec<CommonPrefixes>>,
}

/// Object文件信息
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ObjectInfo {
    /// Object路径
    pub key: String,
    /// Object最后修改时间
    pub last_modified: String,
    /// ETag在每个Object生成时创建，用于标识一个Object的内容，ETag值可以用于检查Object内容是否发生变化，不建议使用ETag值作为Object内容的MD5校验数据完整性的依据。
    pub e_tag: String,
    #[serde(rename = "Type")]
    pub type_field: String,
    /// Object大小，单位为字节
    pub size: u64,
    /// Object的存储类型
    pub storage_class: StorageClass,
    /// Object的解冻状态
    pub restore_info: Option<String>,
    /// Bucket拥有者信息
    pub owner: Option<Owner>,
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

/// 分组列表
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CommonPrefixes {
    /// 前缀
    pub prefix: String,
}

// ------ get_bucket_info ------
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct BucketList {
    pub bucket: BucketInfo,
}

/// 存储空间详细信息
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BucketInfo {
    /// 访问跟踪状态
    pub access_monitor: String,
    /// 备注信息
    pub comment: String,
    /// 创建日期
    pub creation_date: String,
    /// 跨区域复制状态
    pub cross_region_replication: String,
    /// 数据容灾类型
    pub data_redundancy_type: DataRedundancyType,
    /// 外网EndPoint
    pub extranet_endpoint: String,
    /// 内网EndPoint
    pub intranet_endpoint: String,
    /// 所在地域
    pub location: String,
    /// 名称
    pub name: String,
    /// 资源组
    pub resource_group_id: String,
    /// 存储类型
    pub storage_class: StorageClass,
    /// 传输加速状态
    pub transfer_acceleration: String,
    /// 所有者信息
    pub owner: Owner,
    /// 访问权限
    pub access_control_list: AccessControlList,
    /// 服务端加密信息
    pub server_side_encryption_rule: ServerSideEncryptionRule,
    /// 日志信息
    pub bucket_policy: BucketPolicy,
}

/// 存储空间的访问权限信息
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AccessControlList {
    ///访问权限
    pub grant: Acl,
}

/// 存储空间的服务端加密信息
#[derive(Debug, Deserialize)]
pub struct ServerSideEncryptionRule {
    /// 服务端默认加密方式
    #[serde(rename = "SSEAlgorithm")]
    pub sse_algorithm: String,
}

/// 存储空间的日志信息
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BucketPolicy {
    /// 存储日志记录的存储空间名称
    pub log_bucket: String,
    /// 存储日志文件的目录
    pub log_prefix: String,
}

// ------ get_bucket_info ------
/// 存储空间的容量信息
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BucketStat {
    /// 总存储容量，单位字节
    pub storage: u64,
    /// 总文件数量
    pub object_count: u64,
    /// 已经初始化但还未完成（Complete）或者还未中止（Abort）的Multipart Upload数量
    pub multipart_upload_count: u64,
    /// Live Channel的数量
    pub live_channel_count: u64,
    /// 获取到的存储信息的时间点，格式为时间戳，单位为秒
    pub last_modified_time: u64,
    /// 标准存储类型的存储容量，单位字节
    pub standard_storage: u64,
    /// 标准存储类型的文件数量
    pub standard_object_count: u64,
    /// 低频存储类型的计费存储容量，单位字节
    pub infrequent_access_storage: u64,
    /// 低频存储类型的实际存储容量，单位字节
    pub infrequent_access_real_storage: u64,
    /// 低频存储类型的文件数量
    pub infrequent_access_object_count: u64,
    /// 归档存储类型的计费存储容量，单位字节
    pub archive_storage: u64,
    /// 归档存储类型的实际存储容量，单位字节
    pub archive_real_storage: u64,
    /// 归档存储类型的文件数量
    pub archive_object_count: u64,
    /// 冷归档存储类型的计费存储容量，单位字节
    pub cold_archive_storage: u64,
    /// 冷归档存储类型的实际存储容量，单位字节
    pub cold_archive_real_storage: u64,
    /// 冷归档存储类型的文件数量
    pub cold_archive_object_count: u64,
    /// 预留空间使用容量
    pub reserved_capacity_storage: u64,
    /// 预留空间使用的文件数量
    pub reserved_capacity_object_count: u64,
    /// 深度冷归档存储类型的计费存储容量，单位字节
    pub deep_cold_archive_storage: u64,
    /// 深度冷归档存储类型的实际存储容量，单位字节
    pub deep_cold_archive_real_storage: u64,
    /// 深度冷归档存储类型的文件数量
    pub deep_cold_archive_object_count: u64,
}

// ------ get_object_meta ------
/// 文件meta信息
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ObjectMeta {
    /// 文件大小，单位字节
    pub content_length: Option<String>,
    /// 用于标识一个文件的内容
    pub e_tag: Option<String>,
    /// 文件最后访问时间
    pub last_access_time: Option<String>,
    /// 文件最后修改时间
    pub last_modified: Option<String>,
    /// 版本id
    pub version_id: Option<String>,
}

// ------ get_object_tagging ------
#[derive(Debug, Deserialize)]
pub(crate) struct Tagging {
    #[serde(rename = "TagSet")]
    pub tag_set: TagSet,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TagSet {
    #[serde(rename = "Tag")]
    pub tags: Option<Vec<Tag>>,
}

#[derive(Debug, Deserialize)]
/// 标签信息
pub struct Tag {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Value")]
    pub value: String,
}

// ------ copy_object ------
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CopyObjectResult {
    pub e_tag: String,
    pub last_modified: String,
    pub version_id: Option<String>,
}

// ------ append_object ------
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AppendObjectResult {
    pub next_position: Option<String>,
    pub crc64ecma: Option<String>,
    pub version_id: Option<String>,
}

// ------ del_object ------
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DelObjectResult {
    /// 删除标记，含义请详细查看阿里云官方文档
    pub delete_marker: Option<bool>,
    /// 成功删除的文件的版本id
    pub version_id: Option<String>,
}

// ------ get_versions ------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListVersionsResult {
    pub is_truncated: bool,
    pub next_version_id_marker: Option<String>,
    pub next_key_marker: Option<String>,
    pub delete_marker: Option<Vec<DeleteMarker>>,
    pub version: Option<Vec<Version>>,
    pub common_prefixes: Option<Vec<CommonPrefixes>>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ObjectVersionsResult {
    pub delete_marker: Option<Vec<DeleteMarker>>,
    pub version: Option<Vec<Version>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteMarker {
    pub key: String,
    pub version_id: String,
    pub is_latest: bool,
    pub last_modified: String,
    pub owner: Owner,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Version {
    pub key: String,
    pub version_id: String,
    pub is_latest: bool,
    pub last_modified: String,
    pub e_tag: String,
    pub type_: String,
    pub size: u64,
    pub storage_class: StorageClass,
    pub owner: Owner,
}

// ------ put_object ------
#[derive(Debug)]
pub struct PutObjectResult {
    /// ETa值
    pub e_tag: Option<String>,
    /// 成功上传的文件的版本id
    pub version_id: Option<String>,
}
