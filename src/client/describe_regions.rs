use crate::{
    common::{RegionInfo, RegionInfoList},
    error::normal_error,
    sign::SignRequest,
    Error, OssClient,
};
use reqwest::Client;

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
    regions: Option<String>,
}

impl DescribeRegions {
    pub(super) fn new(client: &OssClient) -> Self {
        DescribeRegions {
            client: client.clone(),
            regions: None,
        }
    }

    /// 指定查询单个地域信息，此处需要的是Region ID，比如 oss-cn-hangzhou
    pub fn set_regions(mut self, regions: &str) -> Self {
        self.regions = Some(regions.to_owned());
        self
    }

    /// 发送请求
    pub async fn send(&self) -> Result<Vec<RegionInfo>, Error> {
        //构建http请求
        let mut url = format!("https://{}/?regions", self.client.endpoint);
        if let Some(regions) = &self.regions {
            url.push_str("=");
            url.push_str(&regions);
        }
        let req = Client::new().get(url);
        //发送请求
        let response = req
            .sign(&self.client.ak_id, &self.client.ak_secret, None, None)?
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
                let regions: RegionInfoList = serde_xml_rs::from_reader(&*response_bytes)
                    .map_err(|_| Error::OssInvalidResponse(Some(response_bytes.into())))?;
                Ok(regions.region_info)
            }
            _ => Err(normal_error(response).await),
        }
    }
}
