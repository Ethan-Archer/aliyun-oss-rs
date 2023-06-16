use crate::{
    common::{ObjectMeta, OssErrorResponse},
    sign::SignRequest,
    Error, OssObject,
};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::Client;

/// 获取文件的Meta信息
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31985.html)
pub struct GetObjectMeta {
    object: OssObject,
    version_id: Option<String>,
}
impl GetObjectMeta {
    pub(super) fn new(object: OssObject) -> Self {
        GetObjectMeta {
            object,
            version_id: None,
        }
    }
    /// 设置版本id
    ///
    /// 只有开启了版本控制时才需要设置
    ///
    pub fn set_version_id(mut self, version_id: &str) -> Self {
        self.version_id = Some(version_id.to_owned());
        self
    }
    /// 发送请求
    ///
    pub async fn send(self) -> Result<ObjectMeta, Error> {
        //对文件名进行urlencode
        let filename_str = utf8_percent_encode(&self.object.object, NON_ALPHANUMERIC).to_string();
        //构造URL
        let url = format!(
            "https://{}.{}/{}?objectMeta",
            self.object.bucket, self.object.client.endpoint, filename_str
        );
        //构建请求
        let mut req = Client::new().head(url);
        //插入版本id
        if let Some(version_id) = self.version_id {
            req = req.query(&[("versionId", version_id)]);
        }
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
                let headers = response.headers();
                let content_length = headers
                    .get("Content-Length")
                    .and_then(|header| header.to_str().ok().map(|s| s.to_owned()));
                let e_tag = headers
                    .get("ETag")
                    .and_then(|header| header.to_str().ok().map(|s| s.to_owned()));
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
                let response_text = response.text().await?;
                let error_info: OssErrorResponse = serde_xml_rs::from_str(&response_text)?;
                Err(Error::OssError(status_code, error_info))
            }
        }
    }
}
