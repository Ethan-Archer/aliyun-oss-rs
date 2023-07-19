use crate::{
    error::{normal_error, Error},
    request::{Oss, OssRequest},
};
use chrono::NaiveDateTime;
use hyper::Method;

/// 初始化分片上传
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31994.html)
pub struct CopyToPart {
    req: OssRequest,
}
impl CopyToPart {
    pub(super) fn new(
        oss: Oss,
        part_number: u32,
        upload_id: impl ToString,
        copy_source: impl ToString,
    ) -> Self {
        let mut req = OssRequest::new(oss, Method::PUT);
        req.insert_query("partNumber", part_number);
        req.insert_query("uploadId", upload_id);
        req.insert_header("x-oss-copy-source", copy_source);
        CopyToPart { req }
    }
    /// 设置源文件拷贝范围
    ///
    /// 默认拷贝整个文件，文件字节索引是从0开始
    pub fn set_source_range(mut self, start: usize, end: Option<usize>) -> Self {
        self.req.insert_header(
            "x-oss-copy-source-range",
            format!(
                "bytes={}-{}",
                start,
                end.map(|v| v.to_string()).unwrap_or_else(|| String::new())
            ),
        );
        self
    }
    /// 如果指定的时间早于文件实际修改时间，则正常拷贝文件。
    ///
    pub fn set_if_modified_since(mut self, if_modified_since: NaiveDateTime) -> Self {
        self.req.insert_header(
            "x-oss-copy-source-if-modified-since",
            if_modified_since.format("%a, %e %b %Y %H:%M:%S GMT"),
        );
        self
    }
    /// 如果指定的时间等于或者晚于文件实际修改时间，则正常拷贝文件。
    ///
    pub fn set_if_unmodified_since(mut self, if_unmodified_since: NaiveDateTime) -> Self {
        self.req.insert_header(
            "x-oss-copy-source-if-unmodified-since",
            if_unmodified_since.format("%a, %e %b %Y %H:%M:%S GMT"),
        );
        self
    }
    /// 如果源文件的ETag值和您提供的ETag相等，则执行拷贝操作。
    ///
    /// 文件的ETag值用于验证数据是否发生了更改，您可以基于ETag值验证数据完整性。
    pub fn set_if_match(mut self, if_match: impl ToString) -> Self {
        self.req
            .insert_header("x-oss-copy-source-if-match", if_match);
        self
    }
    /// 如果源文件的ETag值和您提供的ETag不相等，则执行拷贝操作。
    ///
    /// 文件的ETag值用于验证数据是否发生了更改，您可以基于ETag值验证数据完整性。
    pub fn set_if_none_match(mut self, if_none_match: impl ToString) -> Self {
        self.req
            .insert_header("x-oss-copy-source-if-none-match", if_none_match);
        self
    }
    /// 拷贝文件内容到分片
    ///
    /// 返回值为ETag
    pub async fn send(self) -> Result<String, Error> {
        //上传文件
        let response = self.req.send_to_oss()?.await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let e_tag = response
                    .headers()
                    .get("ETag")
                    .map(|v| String::from_utf8(v.as_bytes().to_vec()).ok())
                    .flatten()
                    .unwrap_or_else(|| String::new());
                Ok(e_tag)
            }
            _ => Err(normal_error(response).await),
        }
    }
}
