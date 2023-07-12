use crate::{
    common::{BucketStat, OssInners},
    error::normal_error,
    send::send_to_oss,
    Error, OssBucket,
};
use hyper::{body::to_bytes, Body, Method};

/// 获取指定存储空间的存储容量以及文件数量
///
/// 获取的数据并非是实时数据，延时可能超过一个小时
///
/// 获取到的存储信息的时间点不保证是最新的，即后一次调用该接口返回的LastModifiedTime字段值可能比前一次调用该接口返回的LastModifiedTime字段值小
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/426056.html)
pub struct GetBucketStat {
    bucket: OssBucket,
    querys: OssInners,
}
impl GetBucketStat {
    pub(super) fn new(bucket: OssBucket) -> Self {
        let querys = OssInners::from("stat", "");
        GetBucketStat { bucket, querys }
    }

    /// 发送请求
    pub async fn send(self) -> Result<BucketStat, Error> {
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
                let bucket_stat: BucketStat = serde_xml_rs::from_reader(&*response_bytes)
                    .map_err(|_| Error::OssInvalidResponse(Some(response_bytes)))?;
                Ok(bucket_stat)
            }
            _ => Err(normal_error(response).await),
        }
    }
}
