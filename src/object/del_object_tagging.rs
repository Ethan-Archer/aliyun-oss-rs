use crate::{
    error::normal_error,
    request::{Oss, OssRequest},
    Error,
};
use hyper::Method;

/// 清空文件标签
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/114879.html)
pub struct DelObjectTagging {
    req: OssRequest,
}
impl DelObjectTagging {
    pub(super) fn new(oss: Oss) -> Self {
        let mut req = OssRequest::new(oss, Method::DELETE);
        req.insert_query("tagging", "");
        DelObjectTagging { req }
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
