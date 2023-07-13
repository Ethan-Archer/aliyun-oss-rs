use crate::{
    error::normal_error,
    request::{Oss, OssRequest},
    Error,
};
use hyper::{body::to_bytes, Method};
use serde_derive::Deserialize;

// 返回内容
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

/// 获取指定存储空间的存储容量以及文件数量
///
/// 获取的数据并非是实时数据，延时可能超过一个小时
///
/// 获取到的存储信息的时间点不保证是最新的，即后一次调用该接口返回的LastModifiedTime字段值可能比前一次调用该接口返回的LastModifiedTime字段值小
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/426056.html)
pub struct GetBucketStat {
    req: OssRequest,
}
impl GetBucketStat {
    pub(super) fn new(oss: Oss) -> Self {
        let mut req = OssRequest::new(oss, Method::GET);
        req.insert_query("stat", "");
        GetBucketStat { req }
    }

    /// 发送请求
    pub async fn send(self) -> Result<BucketStat, Error> {
        //构建http请求
        let response = self.req.send_to_oss()?.await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let response_bytes = to_bytes(response.into_body())
                    .await
                    .map_err(|_| Error::OssInvalidResponse(None))?;
                let bucket_stat: BucketStat = serde_xml_rs::from_reader(&*response_bytes)
                    .map_err(|_| Error::OssInvalidResponse(Some(response_bytes)))?;
                Ok(bucket_stat)
            }
            _ => Err(normal_error(response).await),
        }
    }
}
