use crate::{common::BucketStat, error::normal_error, sign::SignRequest, Error, OssBucket};
use reqwest::Client;

/// 获取指定存储空间的存储容量以及文件数量
///
/// 获取的数据并非是实时数据，延时可能超过一个小时
///
/// 获取到的存储信息的时间点不保证是最新的，即后一次调用该接口返回的LastModifiedTime字段值可能比前一次调用该接口返回的LastModifiedTime字段值小
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/426056.html)
pub struct GetBucketStat {
    bucket: OssBucket,
}
impl GetBucketStat {
    pub(super) fn new(bucket: OssBucket) -> Self {
        GetBucketStat { bucket }
    }

    /// 发送请求
    pub async fn send(self) -> Result<BucketStat, Error> {
        //构造URL
        let url = format!(
            "https://{}.{}/?stat",
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
                let response_bytes = response
                    .bytes()
                    .await
                    .map_err(|_| Error::OssInvalidResponse(None))?;
                let bucket_stat: BucketStat = serde_xml_rs::from_reader(&*response_bytes)
                    .map_err(|_| Error::OssInvalidResponse(Some(response_bytes.into())))?;
                Ok(bucket_stat)
            }
            _ => Err(normal_error(response).await),
        }
    }
}
