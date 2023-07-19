use crate::{
    error::normal_error,
    request::{Oss, OssRequest},
    Error,
};
use hyper::Method;
use percent_encoding::percent_decode;

/// 获取软链接
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/45146.html)
pub struct GetSymlink {
    req: OssRequest,
}
impl GetSymlink {
    pub(super) fn new(oss: Oss) -> Self {
        let mut req = OssRequest::new(oss, Method::GET);
        req.insert_query("symlink", "");
        GetSymlink { req }
    }
    /// 发送请求
    ///
    pub async fn send(self) -> Result<String, Error> {
        //构建http请求
        let response = self.req.send_to_oss()?.await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let target = response
                    .headers()
                    .get("x-oss-symlink-target")
                    .map(|v| v.as_bytes())
                    .unwrap_or_else(|| "".as_bytes());
                let target_decode = percent_decode(target)
                    .decode_utf8()
                    .map_err(|_| Error::OssInvalidResponse(None))?;
                Ok(target_decode.into_owned())
            }
            _ => Err(normal_error(response).await),
        }
    }
}
