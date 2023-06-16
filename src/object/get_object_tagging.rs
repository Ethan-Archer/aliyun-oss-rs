use crate::{common::OssErrorResponse, sign::SignRequest, Error, OssObject};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::Client;

/// 删除指定文件
///
/// 无论要删除的文件是否存在，删除成功后均会返回204状态码
///
/// 如果Object类型为软链接，使用此接口只会删除该软链接
///
/// 在开启版本控制的情况下，上传文件和删除文件的逻辑都变得复杂，建议详细阅读阿里云官方文档
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31982.html)
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
    pub async fn send(self) -> Result<String, Error> {
        //对文件名进行urlencode
        let filename_str = utf8_percent_encode(&self.object.object, NON_ALPHANUMERIC).to_string();
        //构造URL
        let url = format!(
            "https://{}.{}/{}?tagging",
            self.object.bucket, self.object.client.endpoint, filename_str
        );
        //构建请求
        let mut req = Client::new().get(url);
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
                let response_text = response.text().await?;
                Ok(response_text)
            }
            _ => {
                let response_text = response.text().await?;
                let error_info: OssErrorResponse = serde_xml_rs::from_str(&response_text)?;
                Err(Error::OssError(status_code, error_info))
            }
        }
    }
}
