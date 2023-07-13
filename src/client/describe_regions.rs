use crate::{
    error::normal_error,
    request::{Oss, OssRequest},
    Error,
};
use hyper::{body::to_bytes, Method};
use serde_derive::Deserialize;

// 返回内容
/// Region基础信息
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RegionInfo {
    /// 地域ID
    pub region: String,
    /// 地域对应的传输加速Endpoint
    pub accelerate_endpoint: String,
    /// 地域对应的内网Endpoint
    pub internal_endpoint: String,
    /// 地域对应的外网Endpoint
    pub internet_endpoint: String,
}

#[doc(hidden)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct RegionInfoList {
    pub region_info: Vec<RegionInfo>,
}

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
    req: OssRequest,
}

impl DescribeRegions {
    pub(super) fn new(oss: Oss) -> Self {
        let mut req = OssRequest::new(oss, Method::GET);
        req.insert_query("regions", "");
        DescribeRegions { req }
    }

    /// 指定查询单个地域信息，此处需要的是Region ID，比如 oss-cn-hangzhou
    pub fn set_regions(mut self, regions: impl ToString) -> Self {
        self.req.insert_query("regions", regions);
        self
    }

    /// 指定从哪个EndPoint发起查询
    ///
    /// 默认为 oss.aliyuncs.com ，如你的网络无法访问，可以设置一个你可以访问的EndPoint
    pub fn set_endpoint(mut self, endpoint: impl ToString) -> Self {
        self.req.set_endpoint(endpoint);
        self
    }

    /// 发送请求
    pub async fn send(self) -> Result<Vec<RegionInfo>, Error> {
        //构建http请求
        let response = self.req.send_to_oss()?.await?;
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
