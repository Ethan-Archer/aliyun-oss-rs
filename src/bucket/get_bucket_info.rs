use crate::{
    common::{Acl, DataRedundancyType, OssInners, Owner, StorageClass},
    error::normal_error,
    send::send_to_oss,
    Error, OssBucket,
};
use hyper::{body::to_bytes, Body, Method};
use serde_derive::Deserialize;

// 返回内容
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

/// 查询存储空间的详细信息
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31968.html)
pub struct GetBucketInfo {
    bucket: OssBucket,
    querys: OssInners,
}
impl GetBucketInfo {
    pub(super) fn new(bucket: OssBucket) -> Self {
        let querys = OssInners::from("bucketInfo", "");
        GetBucketInfo { bucket, querys }
    }
    /// 发送请求
    pub async fn send(self) -> Result<BucketInfo, Error> {
        //构建http请求
        let response = send_to_oss(
            &self.bucket.client,
            Some(&self.bucket.bucket),
            None,
            Method::GET,
            Some(&self.querys),
            None,
            Body::empty(),
        )?
        .await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let response_bytes = to_bytes(response.into_body())
                    .await
                    .map_err(|_| Error::OssInvalidResponse(None))?;
                let bucket_info: BucketList = serde_xml_rs::from_reader(&*response_bytes)
                    .map_err(|_| Error::OssInvalidResponse(Some(response_bytes)))?;
                Ok(bucket_info.bucket)
            }
            _ => Err(normal_error(response).await),
        }
    }
}
