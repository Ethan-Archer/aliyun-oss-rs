use crate::{
    common::{url_encode, OssInners},
    Error, OssClient,
};
use base64::{engine::general_purpose, Engine};
use chrono::Utc;
use hyper::{client::ResponseFuture, header, Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use ring::hmac;

const EXCLUDED_VALUES: [&str; 84] = [
    "acl",
    "uploads",
    "location",
    "cors",
    "logging",
    "website",
    "referer",
    "lifecycle",
    "delete",
    "append",
    "tagging",
    "objectMeta",
    "uploadId",
    "partNumber",
    "security-token",
    "position",
    "img",
    "style",
    "styleName",
    "replication",
    "replicationProgress",
    "replicationLocation",
    "cname",
    "bucketInfo",
    "comp",
    "qos",
    "live",
    "status",
    "vod",
    "startTime",
    "endTime",
    "symlink",
    "x-oss-process",
    "response-content-type",
    "x-oss-traffic-limit",
    "response-content-language",
    "response-expires",
    "response-cache-control",
    "response-content-disposition",
    "response-content-encoding",
    "udf",
    "udfName",
    "udfImage",
    "udfId",
    "udfImageDesc",
    "udfApplication",
    "comp",
    "udfApplicationLog",
    "restore",
    "callback",
    "callback-var",
    "qosInfo",
    "policy",
    "stat",
    "encryption",
    "versions",
    "versioning",
    "versionId",
    "requestPayment",
    "x-oss-request-payer",
    "sequential",
    "inventory",
    "inventoryId",
    "continuation-token",
    "asyncFetch",
    "worm",
    "wormId",
    "wormExtend",
    "withHashContext",
    "x-oss-enable-md5",
    "x-oss-enable-sha1",
    "x-oss-enable-sha256",
    "x-oss-hash-ctx",
    "x-oss-md5-ctx",
    "transferAcceleration",
    "regionList",
    "cloudboxes",
    "x-oss-ac-source-ip",
    "x-oss-ac-subnet-mask",
    "x-oss-ac-vpc-id",
    "x-oss-ac-forward-allow",
    "metaQuery",
    "resourceGroup",
    "rtc",
];

pub(crate) fn send_to_oss(
    client: &OssClient,
    bucket: Option<&str>,
    object: Option<&str>,
    method: Method,
    querys: Option<&OssInners>,
    headers: Option<&OssInners>,
    body: Body,
) -> Result<ResponseFuture, Error> {
    //初始化时间
    let date = Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();
    //提取header数据
    let mut content_type = String::new();
    let mut canonicalized_ossheaders =
        Vec::with_capacity(headers.map(|headers| headers.len()).unwrap_or(0));
    if let Some(headers) = headers {
        headers.iter().for_each(|(key, value)| {
            if key.starts_with("x-oss-") {
                canonicalized_ossheaders.push(format!("{}:{}", key, value))
            };
            if key.starts_with(&header::CONTENT_TYPE.to_string()) {
                content_type = value.to_string();
            };
        });
    }
    //处理canonicalized_ossheaders
    canonicalized_ossheaders.sort();
    let mut canonicalized_ossheaders = canonicalized_ossheaders.join("\n");
    if !canonicalized_ossheaders.is_empty() {
        canonicalized_ossheaders.push_str("\n")
    }
    //构建sub_resource
    let sub_resource = querys.map_or(String::new(), |querys| {
        let mut sub_resource = querys
            .iter()
            .filter(|(key, _)| EXCLUDED_VALUES.contains(&key.as_str()))
            .map(|(key, value)| {
                if value.to_string().is_empty() {
                    key.to_owned()
                } else {
                    format!("{}={}", key, value)
                }
            })
            .collect::<Vec<_>>();
        sub_resource.sort();
        sub_resource.into_iter().collect::<Vec<_>>().join("&")
    });

    //构建canonicalized_resource
    let mut canonicalized_resource = format!(
        "/{}{}",
        bucket.map_or(String::new(), |v| format!("{}/", v)),
        object.map_or(String::new(), |v| format!("{}", v))
    );
    if !sub_resource.is_empty() {
        canonicalized_resource.push_str(&format!("?{}", sub_resource));
    }
    //生成待签名字符串
    let unsign_str = format!(
        "{}\n\n{}\n{}\n{}{}",
        method, content_type, date, canonicalized_ossheaders, canonicalized_resource
    );
    //计算签名值
    let key_str = hmac::Key::new(
        hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY,
        client.ak_secret.as_bytes(),
    );
    let sign_str = general_purpose::STANDARD.encode(hmac::sign(&key_str, unsign_str.as_bytes()));
    //生成url
    let host = if let Some(bucket) = bucket {
        format!("{}.{}", bucket, client.endpoint)
    } else {
        client.endpoint.to_string()
    };
    let query = querys.map_or(String::new(), |querys| {
        querys
            .iter()
            .map(|(key, value)| {
                let value = value.to_string();
                if value.is_empty() {
                    key.to_string()
                } else {
                    format!("{}={}", key, url_encode(&value))
                }
            })
            .collect::<Vec<_>>()
            .join("&")
    });
    let query_str = if query.is_empty() {
        String::new()
    } else {
        format!("?{}", query)
    };
    let uri = format!(
        "https://{}/{}{}",
        host,
        url_encode(object.unwrap_or_else(|| "")),
        query_str
    );
    //构建http请求
    let mut req = Request::builder().method(method).uri(uri);
    req = req.header(header::DATE, date);
    req = req.header(
        header::AUTHORIZATION,
        format!("OSS {}:{}", client.ak_id, sign_str),
    );
    if let Some(headers) = headers {
        for (key, value) in headers.iter() {
            req = req.header(key, value);
        }
    }
    let request = req.body(body)?;
    let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
    Ok(client.request(request))
}
