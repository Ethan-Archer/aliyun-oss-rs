use crate::{common::OssInners, error::normal_error, send::send_to_oss, Error, OssObject};
use hyper::{Body, Method};

/// 删除指定文件
///
/// 删除文件时，不会检查文件是否存在，只要请求合法，都会返回成功
///
/// 返回成功时，如果开启了版本控制，则返回内容有意义，删除标记和版本id的含义，请仔细阅读阿里云官方文档
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31982.html)
pub struct DelObject {
    object: OssObject,
    querys: OssInners,
}
impl DelObject {
    pub(super) fn new(object: OssObject) -> Self {
        DelObject {
            object,
            querys: OssInners::new(),
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
        let response = send_to_oss(
            &self.object.client,
            Some(&self.object.bucket),
            Some(&self.object.object),
            Method::DELETE,
            Some(&self.querys),
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
