use crate::{
    common::{ObjectMeta, OssInners},
    send::send_to_oss,
    Error, OssObject,
};
use base64::{engine::general_purpose, Engine};
use bytes::Bytes;
use hyper::{Body, Method};

/// 获取文件的Meta信息
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31985.html)
pub struct GetObjectMeta {
    object: OssObject,
    querys: OssInners,
}
impl GetObjectMeta {
    pub(super) fn new(object: OssObject) -> Self {
        let querys = OssInners::from("objectMeta", "");
        GetObjectMeta { object, querys }
    }
    /// 设置版本id
    ///
    /// 只有开启了版本控制时才需要设置
    ///
    pub fn set_version_id(mut self, version_id: impl ToString) -> Self {
        self.querys.insert("versionId", version_id);
        self
    }
    /// 发送请求
    ///
    pub async fn send(self) -> Result<ObjectMeta, Error> {
        //构建http请求
        let response = send_to_oss(
            &self.object.client,
            Some(&self.object.bucket),
            Some(&self.object.object),
            Method::HEAD,
            Some(&self.querys),
            None,
            Body::empty(),
        )?
        .await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let headers = response.headers();
                let content_length = headers
                    .get("Content-Length")
                    .and_then(|header| header.to_str().ok().map(|s| s.to_owned()));
                let e_tag = headers.get("ETag").and_then(|header| {
                    header.to_str().ok().map(|s| s.trim_matches('"').to_owned())
                });
                let last_access_time = headers
                    .get("x-oss-last-access-time")
                    .and_then(|header| header.to_str().ok().map(|s| s.to_owned()));
                let last_modified = headers
                    .get("Last-Modified")
                    .and_then(|header| header.to_str().ok().map(|s| s.to_owned()));
                let version_id = headers
                    .get("x-oss-version-id")
                    .and_then(|header| header.to_str().ok().map(|s| s.to_owned()));
                Ok(ObjectMeta {
                    content_length,
                    e_tag,
                    last_access_time,
                    last_modified,
                    version_id,
                })
            }
            _ => {
                let x_oss_error = response.headers().get("x-oss-err").and_then(|header| {
                    general_purpose::STANDARD
                        .decode(header)
                        .ok()
                        .map(|v| Bytes::from(v))
                });
                Err(Error::OssError(status_code, x_oss_error))
            }
        }
    }
}
