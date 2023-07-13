use crate::{
    error::normal_error,
    request::{Oss, OssRequest},
    Error,
};
use hyper::Method;

/// 删除某个存储空间
///
/// 为了防止误删除的发生，OSS不允许删除一个非空的Bucket
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31973.html)
pub struct DelBucket {
    req: OssRequest,
}
impl DelBucket {
    pub(super) fn new(oss: Oss) -> Self {
        DelBucket {
            req: OssRequest::new(oss, Method::DELETE),
        }
    }

    pub async fn send(self) -> Result<(), Error> {
        //构建http请求
        let response = self.req.send_to_oss()?.await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => Ok(()),
            _ => Err(normal_error(response).await),
        }
    }
}
