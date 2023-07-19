use crate::{
    common::{Acl, StorageClass},
    error::normal_error,
    request::{Oss, OssRequest},
    Error,
};
use hyper::Method;

/// 新增软链接
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/45126.html)
pub struct PutSymlink {
    req: OssRequest,
}
impl PutSymlink {
    pub(super) fn new(oss: Oss, symlink_target: impl ToString) -> Self {
        let mut req = OssRequest::new(oss, Method::PUT);
        req.insert_query("symlink", "");
        req.insert_header("x-oss-symlink-target", symlink_target);
        PutSymlink { req }
    }
    /// 设置文件的访问权限
    pub fn set_acl(mut self, acl: Acl) -> Self {
        self.req.insert_header("x-oss-object-acl", acl);
        self
    }
    /// 设置文件的存储类型
    pub fn set_storage_class(mut self, storage_class: StorageClass) -> Self {
        self.req.insert_header("x-oss-storage-class", storage_class);
        self
    }
    /// 不允许覆盖同名文件
    pub fn forbid_overwrite(mut self) -> Self {
        self.req.insert_header("x-oss-forbid-overwrite", "true");
        self
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
