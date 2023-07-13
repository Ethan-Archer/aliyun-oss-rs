use crate::{
    error::normal_error,
    request::{Oss, OssRequest},
    Error,
};
use hyper::{body::to_bytes, Method};
use serde_derive::Deserialize;

// 返回的内容
#[derive(Debug, Deserialize)]
pub(crate) struct Tagging {
    #[serde(rename = "TagSet")]
    pub tag_set: TagSet,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TagSet {
    #[serde(rename = "Tag")]
    pub tags: Option<Vec<Tag>>,
}

#[derive(Debug, Deserialize)]
/// 标签信息
pub struct Tag {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Value")]
    pub value: String,
}

/// 获取文件的标签信息
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/114878.html)
pub struct GetObjectTagging {
    req: OssRequest,
}
impl GetObjectTagging {
    pub(super) fn new(oss: Oss) -> Self {
        let mut req = OssRequest::new(oss, Method::GET);
        req.insert_query("tagging", "");
        GetObjectTagging { req }
    }
    /// 发送请求
    ///
    pub async fn send(self) -> Result<Option<Vec<Tag>>, Error> {
        //构建http请求
        let response = self.req.send_to_oss()?.await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let response_bytes = to_bytes(response.into_body())
                    .await
                    .map_err(|_| Error::OssInvalidResponse(None))?;
                let tagging: Tagging = serde_xml_rs::from_reader(&*response_bytes)
                    .map_err(|_| Error::OssInvalidResponse(Some(response_bytes)))?;
                Ok(tagging.tag_set.tags)
            }
            _ => Err(normal_error(response).await),
        }
    }
}
