use hyper::{body::to_bytes, Body, Method};

use crate::{
    common::{BucketInfo, BucketList, OssInners},
    error::normal_error,
    send::send_to_oss,
    Error, OssBucket,
};

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
