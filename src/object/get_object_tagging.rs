use crate::{common::OssInners, error::normal_error, send::send_to_oss, Error, OssObject};
use hyper::{body::to_bytes, Body, Method};
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
    object: OssObject,
    querys: OssInners,
}
impl GetObjectTagging {
    pub(super) fn new(object: OssObject) -> Self {
        let querys = OssInners::from("tagging", "");
        GetObjectTagging { object, querys }
    }
    /// 发送请求
    ///
    pub async fn send(self) -> Result<Option<Vec<Tag>>, Error> {
        //构建http请求
        let response = send_to_oss(
            &self.object.client,
            Some(&self.object.bucket),
            Some(&self.object.object),
            Method::GET,
            Some(&self.querys),
            None,
            Body::empty(),
        )?
        .await?;
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
