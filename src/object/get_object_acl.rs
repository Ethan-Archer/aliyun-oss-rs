use crate::{
    common::Acl,
    error::normal_error,
    request::{Oss, OssRequest},
    Error,
};
use hyper::{body::to_bytes, Method};
use serde_derive::Deserialize;

// 返回的内容
/// 文件ACL信息
#[derive(Debug, Deserialize)]
struct AccessControlPolicy {
    #[serde(rename = "AccessControlList")]
    access_control_list: AccessControlList,
}

#[derive(Debug, Deserialize)]
struct AccessControlList {
    #[serde(rename = "Grant")]
    grant: Acl,
}

/// 获取文件的ACL信息
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31987.html)
pub struct GetObjectAcl {
    req: OssRequest,
}
impl GetObjectAcl {
    pub(super) fn new(oss: Oss) -> Self {
        let mut req = OssRequest::new(oss, Method::GET);
        req.insert_query("acl", "");
        GetObjectAcl { req }
    }
    /// 发送请求
    ///
    pub async fn send(self) -> Result<Acl, Error> {
        //构建http请求
        let response = self.req.send_to_oss()?.await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let response_bytes = to_bytes(response.into_body())
                    .await
                    .map_err(|_| Error::OssInvalidResponse(None))?;
                let acl: AccessControlPolicy = serde_xml_rs::from_reader(&*response_bytes)
                    .map_err(|_| Error::OssInvalidResponse(Some(response_bytes)))?;
                Ok(acl.access_control_list.grant)
            }
            _ => Err(normal_error(response).await),
        }
    }
}
