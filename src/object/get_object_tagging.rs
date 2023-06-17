use crate::{
    common::{Tag, Tagging},
    error::normal_error,
    sign::SignRequest,
    Error, OssObject,
};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::Client;

/// 获取文件的标签信息
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/114878.html)
pub struct GetObjectTagging {
    object: OssObject,
}
impl GetObjectTagging {
    pub(super) fn new(object: OssObject) -> Self {
        GetObjectTagging { object }
    }
    /// 发送请求
    ///
    /// 在开启了版本控制的情况下，返回值才有意义
    ///
    /// - 返回值 0 - x-oss-delete-marker标记
    /// - 返回值 1 - 版本ID，删除时如果未指定版本ID，则此返回值代表新增删除标记的版本ID，否则代表你主动指定的版本ID
    pub async fn send(self) -> Result<Vec<Tag>, Error> {
        //对文件名进行urlencode
        let filename_str = utf8_percent_encode(&self.object.object, NON_ALPHANUMERIC).to_string();
        //构造URL
        let url = format!(
            "https://{}.{}/{}?tagging",
            self.object.bucket, self.object.client.endpoint, filename_str
        );
        //构建请求
        let req = Client::new().get(url);
        //发送请求
        let response = req
            .sign(
                &self.object.client.ak_id,
                &self.object.client.ak_secret,
                Some(&self.object.bucket),
                Some(&self.object.object),
            )?
            .send()
            .await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let response_bytes = response
                    .bytes()
                    .await
                    .map_err(|_| Error::OssInvalidResponse(None))?;
                let tagging: Tagging = serde_xml_rs::from_reader(&*response_bytes)
                    .map_err(|_| Error::OssInvalidResponse(Some(response_bytes.into())))?;
                Ok(tagging.tag_set.tags)
            }
            _ => Err(normal_error(response).await),
        }
    }
}
