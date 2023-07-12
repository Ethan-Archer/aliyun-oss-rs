use hyper::{Body, Method};

use crate::{error::normal_error, send::send_to_oss, Error, OssBucket};

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
        //构建http请求
        let response = send_to_oss(
            &self.bucket.client,
            Some(&self.bucket.bucket),
            None,
            Method::DELETE,
            None,
            None,
            Body::empty(),
        )?
        .await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => Ok(()),
            _ => Err(normal_error(response).await),
        }
    }
}
