use crate::{
    error::{normal_error, Error},
    request::{Oss, OssRequest},
};
use hyper::Method;

/// 删除分片上传数据
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31996.html)
pub struct AbortUpload {
    req: OssRequest,
}

impl AbortUpload {
    pub(super) fn new(oss: Oss, upload_id: impl ToString) -> Self {
        let mut req = OssRequest::new(oss, Method::DELETE);
        req.insert_query("uploadId", upload_id);
        AbortUpload { req }
    }
    /// 完成分片上传
    ///
    pub async fn send(self) -> Result<(), Error> {
        //上传文件
        let response = self.req.send_to_oss()?.await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => Ok(()),
            _ => Err(normal_error(response).await),
        }
    }
}
