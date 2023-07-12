use crate::{
    common::{OssInners, RegionInfo, RegionInfoList},
    error::normal_error,
    send::send_to_oss,
    Error, OssClient,
};
use hyper::{body::to_bytes, Body, Method};

/// 查询地域的EndpPoint信息
///
/// 可以通过 set_regions 方法设置查询特定地域，默认查询全部，具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/345596.html)
///
/// ```
/// let client = OssClient::new("AccessKey ID","AccessKey Secret","oss-cn-beijing.aliyuncs.com");
/// let regions = client.describe_regions()     
///                     .set_regions("oss-cn-hangzhou")     
///                     .send().await;      
/// println!("{:#?}", regions);
/// ```
///
pub struct DescribeRegions {
    client: OssClient,
    querys: OssInners,
}

impl DescribeRegions {
    pub(super) fn new(client: OssClient) -> Self {
        let querys = OssInners::from("regions", "");
        DescribeRegions { client, querys }
    }

    /// 指定查询单个地域信息，此处需要的是Region ID，比如 oss-cn-hangzhou
    pub fn set_regions(mut self, regions: impl ToString) -> Self {
        self.querys.insert("regions", regions);
        self
    }

    /// 发送请求
    pub async fn send(&self) -> Result<Vec<RegionInfo>, Error> {
        //构建http请求
        let response = send_to_oss(
            &self.client,
            None,
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
                let regions: RegionInfoList = serde_xml_rs::from_reader(&*response_bytes)
                    .map_err(|_| Error::OssInvalidResponse(Some(response_bytes)))?;
                Ok(regions.region_info)
            }
            _ => Err(normal_error(response).await),
        }
    }
}
