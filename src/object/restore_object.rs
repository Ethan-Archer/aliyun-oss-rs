use crate::{
    common::RestoreTier,
    error::normal_error,
    request::{Oss, OssRequest},
    Error,
};
use hyper::Method;

/// 解冻归档文件
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/52930.html)
pub struct RestoreObject {
    req: OssRequest,
    days: Option<u32>,
    tier: Option<RestoreTier>,
}
impl RestoreObject {
    pub(super) fn new(oss: Oss) -> Self {
        let mut req = OssRequest::new(oss, Method::POST);
        req.insert_query("restore", "");
        RestoreObject {
            req,
            days: None,
            tier: None,
        }
    }
    /// 设置解冻天数
    ///
    pub fn set_days(mut self, days: u32) -> Self {
        self.days = Some(days);
        self
    }
    /// 设置解冻优先级
    ///
    pub fn set_tier(mut self, tier: RestoreTier) -> Self {
        self.tier = Some(tier);
        self
    }
    /// 发送请求
    ///
    pub async fn send(mut self) -> Result<(), Error> {
        //构建Body
        let days_str = self
            .days
            .map(|v| format!("<Days>{}</Days>", v))
            .unwrap_or_else(|| String::new());
        let tier_str = self
            .tier
            .map(|v| format!("<JobParameters><Tier>{}</Tier></JobParameters>", v))
            .unwrap_or_else(|| String::new());
        if !days_str.is_empty() || !tier_str.is_empty() {
            let body_str = format!("<RestoreRequest>{}{}</RestoreRequest>", days_str, tier_str);
            self.req.set_body(body_str.into());
        }
        //构建http请求
        let response = self.req.send_to_oss()?.await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => Ok(()),
            _ => Err(normal_error(response).await),
        }
    }
}
