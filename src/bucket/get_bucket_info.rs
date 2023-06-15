use crate::{
    common::{Bucket, BucketList, OssErrorResponse},
    sign::SignRequest,
    Error, OssBucket,
};
use reqwest::Client;

/// 查询存储空间的详细信息
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31968.html)
pub struct GetBucketInfo {
    bucket: OssBucket,
}
impl GetBucketInfo {
    pub(super) fn new(bucket: OssBucket) -> Self {
        GetBucketInfo { bucket }
    }
    /// 发送请求
    pub async fn send(self) -> Result<Bucket, Error> {
        //构造URL
        let url = format!(
            "https://{}.{}/?bucketInfo",
            self.bucket.bucket, self.bucket.client.endpoint
        );
        //发送请求
        let response = Client::new()
            .get(url)
            .sign(
                &self.bucket.client.ak_id,
                &self.bucket.client.ak_secret,
                Some(&self.bucket.bucket),
                None,
            )?
            .send()
            .await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let response_text = response.text().await?;
                let bucket_info: BucketList = serde_xml_rs::from_str(&response_text)?;
                Ok(bucket_info.bucket)
            }
            _ => {
                let response_text = response.text().await?;
                let error_info: OssErrorResponse = serde_xml_rs::from_str(&response_text)?;
                Err(Error::OssError(status_code, error_info))
            }
        }
    }
}
