use crate::{
    error::normal_error,
    request::{Oss, OssRequest},
    Error,
};
use hyper::Method;

/// 删除指定文件
///
/// 删除文件时，不会检查文件是否存在，只要请求合法，都会返回成功
///
/// 返回成功时，如果开启了版本控制，则返回内容有意义，删除标记和版本id的含义，请仔细阅读阿里云官方文档
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31982.html)
pub struct DelObject {
    req: OssRequest,
}
impl DelObject {
    pub(super) fn new(oss: Oss) -> Self {
        DelObject {
            req: OssRequest::new(oss, Method::DELETE),
        }
    }
    /// 发送请求
    ///
    /// 在开启了版本控制的情况下，返回值才有意义
    ///
    /// - 返回值 0 - x-oss-delete-marker标记
    /// - 返回值 1 - 版本ID，删除时如果未指定版本ID，则此返回值代表新增删除标记的版本ID，否则代表你主动指定的版本ID
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
