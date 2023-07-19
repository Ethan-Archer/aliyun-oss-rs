use crate::{
    common::Acl,
    error::normal_error,
    request::{Oss, OssRequest},
    Error,
};
use hyper::Method;

/// 设置文件的ACL
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31986.html)
pub struct PutObjectAcl {
    req: OssRequest,
}
impl PutObjectAcl {
    pub(super) fn new(oss: Oss, acl: Acl) -> Self {
        let mut req = OssRequest::new(oss, Method::PUT);
        req.insert_query("acl", "");
        req.insert_header("x-oss-object-acl", acl);
        PutObjectAcl { req }
    }
    /// 发送请求
    ///
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
