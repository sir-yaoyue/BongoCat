use encoding_rs::GBK;
use serde::Serialize;
use tauri::command;

const SINA_INDEX_URL: &str = "https://hq.sinajs.cn/list=s_sh000001";
const SINA_REFERER: &str = "https://finance.sina.com.cn";

/// 上证指数（000001），单位：点
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockIndex {
    name: String,
    price: f64,
    change: f64,
    change_percent: f64,
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

#[command]
pub async fn fetch_stock_index() -> Result<StockIndex, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|err| format!("初始化网络客户端失败: {err}"))?;

    let bytes = client
        .get(SINA_INDEX_URL)
        .header(reqwest::header::REFERER, SINA_REFERER)
        .header(reqwest::header::USER_AGENT, "Mozilla/5.0")
        .send()
        .await
        .map_err(|err| format!("请求上证指数接口失败: {err}"))?
        .bytes()
        .await
        .map_err(|err| format!("读取上证指数数据失败: {err}"))?;

    // 新浪返回 GBK 编码，需先转码为 UTF-8
    let (decoded, _, _) = GBK.decode(&bytes);
    let body = decoded.into_owned();

    // 形如：var hq_str_s_sh000001="上证指数,3990.3037,7.6502,0.19,...";
    let start = body.find('"').ok_or("上证指数数据格式异常")? + 1;
    let end = body.rfind('"').ok_or("上证指数数据格式异常")?;

    if end <= start {
        return Err("上证指数数据为空".into());
    }

    let fields: Vec<&str> = body[start..end].split(',').collect();

    if fields.len() < 4 {
        return Err("上证指数数据字段不完整".into());
    }

    let parse = |index: usize| -> Result<f64, String> {
        fields[index]
            .trim()
            .parse::<f64>()
            .map_err(|_| format!("上证指数字段解析失败: {}", fields[index]))
    };

    // 0=名称 1=当前点数 2=涨跌额 3=涨跌幅
    Ok(StockIndex {
        name: fields[0].trim().to_string(),
        price: parse(1)?,
        change: round2(parse(2)?),
        change_percent: round2(parse(3)?),
    })
}
