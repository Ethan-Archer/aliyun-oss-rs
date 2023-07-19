use crate::{
    error::normal_error,
    request::{Oss, OssRequest},
    Error,
};
use hyper::Method;

/// 设置文件标签
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/114855.html)
pub struct PutObjectTagging {
    req: OssRequest,
    tags: Vec<(String, String)>,
}
impl PutObjectTagging {
    pub(super) fn new(oss: Oss, tags: Vec<(impl ToString, impl ToString)>) -> Self {
        let mut req = OssRequest::new(oss, Method::PUT);
        req.insert_query("tagging", "");
        PutObjectTagging {
            req,
            tags: tags
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
        }
    }
    /// 新增标签
    pub fn add_tags(mut self, tags: Vec<(impl ToString, impl ToString)>) -> Self {
        self.tags.extend(
            tags.iter()
                .map(|(key, value)| (key.to_string(), value.to_string())),
        );
        self
    }
    /// 发送请求
    ///
    pub async fn send(mut self) -> Result<(), Error> {
        //构建body
        let tag_str = self
            .tags
            .iter()
            .map(|(key, value)| {
                if value.is_empty() {
                    format!("<Tag><Key>{}</Key></Tag>", key)
                } else {
                    format!("<Tag><Key>{}</Key><Value>{}</Value></Tag>", key, value)
                }
            })
            .collect::<Vec<_>>()
            .join("");
        let body = format!("<Tagging><TagSet>{}</TagSet></Tagging>", tag_str);
        self.req.insert_header("Content-Length", body.len());
        self.req.set_body(body.into());
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
