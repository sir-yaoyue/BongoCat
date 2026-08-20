use encoding_rs::GBK;
use serde::Serialize;
use tauri::command;

const SINA_GOLD_URL: &str = "https://hq.sinajs.cn/list=gds_AU9999";
const SINA_REFERER: &str = "https://finance.sina.com.cn";

/// 国内金价（上海黄金交易所 Au99.99），单位：元/克
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoldPrice {
    name: String,
    price: f64,
    change: f64,
    change_percent: f64,
    time: String,
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

#[command]
pub async fn fetch_gold_price() -> Result<GoldPrice, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|err| format!("初始化网络客户端失败: {err}"))?;

    let bytes = client
        .get(SINA_GOLD_URL)
        .header(reqwest::header::REFERER, SINA_REFERER)
        .header(reqwest::header::USER_AGENT, "Mozilla/5.0")
        .send()
        .await
        .map_err(|err| format!("请求金价接口失败: {err}"))?
        .bytes()
        .await
        .map_err(|err| format!("读取金价数据失败: {err}"))?;

    // 新浪返回 GBK 编码，需先转码为 UTF-8
    let (decoded, _, _) = GBK.decode(&bytes);
    let body = decoded.into_owned();

    // 形如：var hq_str_gds_AU9999="953.20,0,...,沪金99";
    let start = body.find('"').ok_or("金价数据格式异常")? + 1;
    let end = body.rfind('"').ok_or("金价数据格式异常")?;

    if end <= start {
        return Err("金价数据为空".into());
    }

    let fields: Vec<&str> = body[start..end].split(',').collect();

    if fields.len() < 14 {
        return Err("金价数据字段不完整".into());
    }

    let parse = |index: usize| -> Result<f64, String> {
        fields[index]
            .trim()
            .parse::<f64>()
            .map_err(|_| format!("金价字段解析失败: {}", fields[index]))
    };

    // 0=最新价 6=时间 7=昨收 13=名称
    let price = parse(0)?;
    let yesterday_close = parse(7)?;
    let change = round2(price - yesterday_close);
    let change_percent = if yesterday_close != 0.0 {
        round2((change / yesterday_close) * 100.0)
    } else {
        0.0
    };

    Ok(GoldPrice {
        name: fields[13].trim().to_string(),
        price,
        change,
        change_percent,
        time: fields[6].trim().to_string(),
    })
}
