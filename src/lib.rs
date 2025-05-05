use std::collections::HashMap;
use worker::*;

const BASE_URL: &str = "https://api.openweathermap.org/data/2.5/weather";

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
  console_error_panic_hook::set_once();
  let allowed_origin: String = env.var("ALLOWED_ORIGIN").unwrap().to_string();
  let set_headers = |mut resp: Response, status_code: u16| {
    resp.headers_mut().set("Content-Type", "application/json").unwrap();
    resp.headers_mut().set("Access-Control-Allow-Origin", &allowed_origin).unwrap();
    resp.with_status(status_code)
  };

  match req.headers().get("Origin").unwrap() {
    Some(origin) if origin == allowed_origin => (),
    _ => return Response::error("403 Unauthorized", 403),
  };

  let url: Url = req.url().unwrap();
  let params: HashMap<_, _> = url.query_pairs().into_owned().collect();

  let api_key: String = env.var("OPENWEATHER_API_KEY").unwrap().to_string();
  let api_url: String = match (params.get("city"), params.get("lat"), params.get("lon")) {
    (Some(city), None, None) => {
      format!(
        "{}?q={}&appid={}",
        BASE_URL, urlencoding::encode(city), api_key
      )
    },
    (None, Some(lat), Some(lon)) => {
      format!(
        "{}?lat={}&lon={}&appid={}",
        BASE_URL, urlencoding::encode(lat), urlencoding::encode(lon), api_key
      )
    },
    _ => {
      return Response::ok(
        r#"{"cod": "400", "message": "invalid parameters"}"#
      ).map(|resp: Response| set_headers(resp, 400))
    }
  };

  let client: Fetch = Fetch::Url(api_url.parse().unwrap());
  let mut res: Response = client.send().await?;
  let text: String = res.text().await?;
  Response::ok(text).map(|resp: Response| set_headers(resp, 200))
}
