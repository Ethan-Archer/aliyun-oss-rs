use crate::{
    common::{ListVersionsResult, ObjectVersionsResult, OssInners, Version},
    error::normal_error,
    send::send_to_oss,
    Error, OssClient,
};
use hyper::{body::to_bytes, Body, Method};

/// 列举文件的历史版本信息
///
/// 默认获取前1000条历史版本
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/112467.html)
pub struct ListObjectVersions<'a> {
    client: OssClient,
    bucket: &'a str,
    object: &'a str,
    querys: OssInners,
}

impl<'a> ListObjectVersions<'a> {
    pub(super) fn new(client: OssClient, bucket: &'a str, object: &'a str) -> Self {
        let mut querys = OssInners::from("versions", "");
        querys.insert("prefix", object);
        querys.insert("max-keys", "1000");
        ListObjectVersions {
            client,
            bucket,
            object,
            querys,
        }
    }
    // 设置请求的开始版本号
    fn set_version_id(&mut self, version_id: impl ToString) {
        self.querys.insert("version-id-marker", version_id);
        self.querys.insert("key-marker", self.object);
    }
    // 发送请求
    async fn send_req(&mut self) -> Result<ListVersionsResult, Error> {
        //构建http请求
        let response = send_to_oss(
            &self.client,
            Some(&self.bucket),
            None,
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
                let mut versions_result: ListVersionsResult =
                    serde_xml_rs::from_reader(&*response_bytes)
                        .map_err(|_| Error::OssInvalidResponse(Some(response_bytes)))?;
                let versions = versions_result.version.map(|versions| {
                    let modified_versions: Vec<Version> = versions
                        .into_iter()
                        .filter(|version| version.key == self.object)
                        .map(|version| {
                            let modified_e_tag = version.e_tag.replace("\"", "");
                            Version {
                                e_tag: modified_e_tag,
                                ..version
                            }
                        })
                        .collect();
                    modified_versions
                });
                let del_markers = versions_result.delete_marker.map(|mut markers| {
                    markers.retain(|marker| marker.key == self.object);
                    markers
                });
                versions_result.version = versions;
                versions_result.delete_marker = del_markers;
                Ok(versions_result)
            }
            _ => Err(normal_error(response).await),
        }
    }
    /// 发送请求
    ///
    pub async fn send(&mut self) -> Result<ObjectVersionsResult, Error> {
        let response = self.send_req().await?;
        let mut next_version_id_marker = response.next_version_id_marker;
        let mut delete_marker = response.delete_marker;
        let mut version = response.version;
        while let Some(next_version_id) = next_version_id_marker {
            self.set_version_id(next_version_id);
            let new_response = self.send_req().await?;
            if let Some(new_delete_marker) = new_response.delete_marker {
                delete_marker = delete_marker.map(|mut v| {
                    v.extend(new_delete_marker);
                    v
                });
            }
            if let Some(new_version) = new_response.version {
                version = version.map(|mut v| {
                    v.extend(new_version);
                    v
                });
            }
            next_version_id_marker = new_response.next_version_id_marker;
        }
        Ok(ObjectVersionsResult {
            version,
            delete_marker,
        })
    }
}
