use crate::{
    error::{normal_error, Error},
    request::{Oss, OssRequest},
};
use hyper::Method;

/// 完成分片上传
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31995.html)
pub struct CompleteUpload<'a> {
    req: OssRequest,
    parts: Vec<(&'a str, &'a str)>,
}
impl<'a> CompleteUpload<'a> {
    pub(super) fn new(oss: Oss, upload_id: impl ToString) -> Self {
        let mut req = OssRequest::new(oss, Method::POST);
        req.insert_query("uploadId", upload_id);
        CompleteUpload {
            req,
            parts: Vec::new(),
        }
    }
    /// 新增分片信息
    ///
    /// 数据结构为 (PartNumber,ETag)
    pub fn add_parts(mut self, parts: Vec<(&'a str, &'a str)>) -> Self {
        self.parts.extend(parts);
        self
    }
    /// 完成分片上传
    ///
    pub async fn send(mut self) -> Result<(), Error> {
        // 构建body
        let body = format!(
            "<CompleteMultipartUpload>{}</CompleteMultipartUpload>",
            self.parts
                .iter()
                .map(|(part_num, e_tag)| format!(
                    "<Part><PartNumber>{}</PartNumber><ETag>{}</ETag></Part>",
                    part_num, e_tag
                ))
                .collect::<Vec<_>>()
                .join("")
        );
        let body_len = body.len();
        self.req.set_body(body.into());
        self.req.insert_header("Content-Length", body_len);
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
