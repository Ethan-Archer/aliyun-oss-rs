use crate::{common::OssErrorResponse, sign::SignRequest, Error, OssBucket};
use reqwest::Client;

/// 删除某个存储空间
///
/// 为了防止误删除的发生，OSS不允许删除一个非空的Bucket
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31973.html)
pub struct DelBucket {
    bucket: OssBucket,
}
impl DelBucket {
    pub(super) fn new(bucket: OssBucket) -> Self {
        DelBucket { bucket }
    }

    pub async fn send(self) -> Result<(), Error> {
        //构造URL
        let url = format!(
            "https://{}.{}",
            self.bucket.bucket, self.bucket.client.endpoint
        );
        //发送请求
        let response = Client::new()
            .delete(url)
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
            code if code.is_success() => Ok(()),
            _ => {
                let body = response.text().await?;
                let error_info: OssErrorResponse = serde_xml_rs::from_str(&body)?;
                Err(Error::OssError(status_code, error_info))
            }
        }
    }
}
