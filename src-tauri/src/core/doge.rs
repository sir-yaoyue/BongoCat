use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::Value;
use tauri::command;

/// 主数据源：OKX（欧易）现货 DOGE-USDT 24h ticker（用户参照的行情源）
/// 注意：OKX CDN 会拦截 rustls 的 TLS 指纹，本模块的客户端使用
/// native-tls（Windows Schannel）绕开该限制。
const OKX_TICKER_URL: &str = "https://okx.com/api/v5/market/ticker?instId=DOGE-USDT";
/// 备用数据源：HTX（火币）DOGEUSDT 合并成交（国内免代理可达）
const HTX_MERGED_URL: &str = "https://api.huobi.pro/market/detail/merged?symbol=dogeusdt";
/// 备用数据源：Binance DOGE/USDT
/// 最新价走轻量端点（权重 2），可支撑 1 秒轮询；
/// 24h 开盘价走 ticker/24hr（权重 80），缓存 60 秒，
/// 合计约 200 权重/分钟，远低于 Binance 1200/分钟限额。
const BINANCE_PRICE_URL: &str = "https://api.binance.com/api/v3/ticker/price?symbol=DOGEUSDT";
const BINANCE_24H_URL: &str = "https://api.binance.com/api/v3/ticker/24hr?symbol=DOGEUSDT";
const BINANCE_OPEN_TTL: Duration = Duration::from_secs(60);
/// 末位备用源：Kraken DOGEUSD
const KRAKEN_TICKER_URL: &str = "https://api.kraken.com/0/public/Ticker?pair=DOGEUSD";

/// 全局复用的 HTTP 客户端（连接池 + 系统代理），避免每秒轮询时重复 TLS 握手。
/// 使用 native-tls（Windows Schannel）而非 rustls：指纹与普通 Windows 客户端
/// 一致，可正常访问 OKX CDN；短超时保证单源失败时快速切换到备源，
/// 使 1 秒轮询不被慢源拖住。
fn http_client() -> Result<reqwest::Client, String> {
    static CLIENT: OnceLock<Result<reqwest::Client, String>> = OnceLock::new();

    CLIENT
        .get_or_init(|| {
            let mut builder = reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(1))
                .timeout(Duration::from_secs(2))
                .pool_max_idle_per_host(4)
                .use_native_tls();

            if let Some(proxy) = system_proxy_url() {
                builder = builder.proxy(
                    reqwest::Proxy::all(proxy)
                        .map_err(|err| format!("初始化代理失败: {err}"))?
                        .no_proxy(reqwest::NoProxy::from_string(
                            "localhost,127.*,10.*,172.*,192.168.*,*.local",
                        )),
                );
            }

            builder.build().map_err(|err| format!("初始化网络客户端失败: {err}"))
        })
        .clone()
}

/// 读取系统代理（Windows 下为 WinINET 设置，如 FlClash 的 127.0.0.1:7890）。
/// reqwest 默认直连，海外行情接口在国内网络下必须走系统代理才可达。
fn system_proxy_url() -> Option<String> {
    static PROXY: OnceLock<Option<String>> = OnceLock::new();

    PROXY.get_or_init(|| {
        #[cfg(windows)]
        {
            use winreg::enums::HKEY_CURRENT_USER;
            use winreg::RegKey;

            let hkey = RegKey::predef(HKEY_CURRENT_USER);
            let key = hkey
                .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Internet Settings")
                .ok()?;
            let enabled: u32 = key.get_value("ProxyEnable").ok()?;

            if enabled != 1 {
                return None;
            }

            let server: String = key.get_value("ProxyServer").ok()?;

            if server.is_empty() {
                return None;
            }

            return Some(if server.contains("://") {
                server
            } else {
                format!("http://{server}")
            });
        }

        #[cfg(not(windows))]
        {
            std::env::var("HTTPS_PROXY")
                .ok()
                .or_else(|| std::env::var("HTTP_PROXY").ok())
        }
    })
    .clone()
}

/// GET 并解析 JSON，统一网络错误文案
async fn get_json(client: &reqwest::Client, source: &str, url: &str) -> Result<Value, String> {
    client
        .get(url)
        .header(reqwest::header::USER_AGENT, "Mozilla/5.0")
        .send()
        .await
        .map_err(|err| format!("请求 {source} 行情接口失败: {err}"))?
        .json()
        .await
        .map_err(|err| format!("解析 {source} 行情数据失败: {err}"))
}

/// DOGE/USDT 行情，单位：美元
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DogePrice {
    name: String,
    price: f64,
    change: f64,
    change_percent: f64,
    time: String,
}

fn round5(value: f64) -> f64 {
    (value * 100_000.0).round() / 100_000.0
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn build_price(price: f64, open: f64, time: String) -> DogePrice {
    let change = round5(price - open);
    let change_percent = if open != 0.0 {
        round2(change / open * 100.0)
    } else {
        0.0
    };

    DogePrice {
        name: "DOGE/USDT".into(),
        price,
        change,
        change_percent,
        time,
    }
}

/// 主数据源：OKX（欧易）现货 DOGE-USDT 24h ticker
/// （last=最新成交价，open24h=24h 开盘价，ts=毫秒时间戳）
async fn fetch_from_okx(client: &reqwest::Client) -> Result<DogePrice, String> {
    let body = get_json(client, "OKX", OKX_TICKER_URL).await?;

    if body["code"] != "0" {
        return Err(format!("OKX 返回错误: {}", body["code"]));
    }

    let ticker = &body["data"][0];

    let parse = |key: &str| -> Result<f64, String> {
        ticker[key]
            .as_str()
            .ok_or(format!("OKX 缺少 {key} 字段"))?
            .parse()
            .map_err(|_| format!("OKX {key} 字段解析失败"))
    };

    Ok(build_price(
        parse("last")?,
        parse("open24h")?,
        ticker["ts"].to_string(),
    ))
}

/// 备用数据源：HTX（火币）DOGEUSDT 合并成交
async fn fetch_from_htx(client: &reqwest::Client) -> Result<DogePrice, String> {
    let body = get_json(client, "HTX", HTX_MERGED_URL).await?;

    if body.get("status").and_then(|status| status.as_str()) != Some("ok") {
        return Err(format!(
            "HTX 返回错误: {}",
            body.get("status").and_then(|status| status.as_str()).unwrap_or("unknown")
        ));
    }

    let tick = &body["tick"];
    let close = tick["close"].as_f64().ok_or("HTX 缺少 close 字段")?;
    let open = tick["open"].as_f64().ok_or("HTX 缺少 open 字段")?;

    Ok(build_price(
        close,
        open,
        tick.get("time").and_then(|time| time.as_str()).unwrap_or_default().to_string(),
    ))
}

/// 备用数据源：Binance DOGE/USDT。
/// 最新价每秒实时拉取；24h 开盘价 60 秒内复用缓存，
/// 避免高频调用 ticker/24hr（权重 80）触发限频。
async fn fetch_from_binance(client: &reqwest::Client) -> Result<DogePrice, String> {
    static OPEN_CACHE: OnceLock<Mutex<Option<(f64, Instant)>>> = OnceLock::new();
    let open_cache = OPEN_CACHE.get_or_init(Default::default);

    let mut open = open_cache
        .lock()
        .unwrap()
        .as_ref()
        .and_then(|(value, at)| (at.elapsed() < BINANCE_OPEN_TTL).then_some(*value));

    if open.is_none() {
        let body = get_json(client, "Binance", BINANCE_24H_URL).await?;
        let value = body["openPrice"]
            .as_str()
            .ok_or("Binance 缺少 openPrice 字段")?
            .parse::<f64>()
            .map_err(|_| "Binance openPrice 字段解析失败".to_string())?;
        open = Some(value);
        *open_cache.lock().unwrap() = Some((value, Instant::now()));
    }

    let body = get_json(client, "Binance", BINANCE_PRICE_URL).await?;
    let price = body["price"]
        .as_str()
        .ok_or("Binance 缺少 price 字段")?
        .parse::<f64>()
        .map_err(|_| "Binance price 字段解析失败".to_string())?;

    Ok(build_price(price, open.unwrap(), String::new()))
}

/// 末位备用源：Kraken DOGEUSD ticker
/// （c[0]=最新成交价，p[0]=24h 加权均价，用均价作为涨跌基准）
async fn fetch_from_kraken(client: &reqwest::Client) -> Result<DogePrice, String> {
    let body = get_json(client, "Kraken", KRAKEN_TICKER_URL).await?;

    let errors = body["error"].as_array().ok_or("Kraken 缺少 error 字段")?;
    if !errors.is_empty() {
        return Err(format!("Kraken 返回错误: {}", body["error"]));
    }

    let pair = &body["result"]["XDGUSD"];

    let parse = |key: &str| -> Result<f64, String> {
        pair[key][0]
            .as_str()
            .ok_or(format!("Kraken 缺少 {key} 字段"))?
            .parse()
            .map_err(|_| format!("Kraken {key} 字段解析失败"))
    };

    Ok(build_price(parse("c")?, parse("p")?, String::new()))
}

#[command]
pub async fn fetch_doge_price() -> Result<DogePrice, String> {
    let client = http_client()?;

    match fetch_from_okx(&client).await {
        Ok(price) => Ok(price),
        Err(okx_err) => match fetch_from_htx(&client).await {
            Ok(price) => Ok(price),
            Err(htx_err) => match fetch_from_binance(&client).await {
                Ok(price) => Ok(price),
                Err(binance_err) => fetch_from_kraken(&client)
                    .await
                    .map_err(|kraken_err| {
                        format!("获取 DOGE 行情失败 (OKX: {okx_err}; HTX: {htx_err}; Binance: {binance_err}; Kraken: {kraken_err})")
                    }),
            },
        },
    }
}
