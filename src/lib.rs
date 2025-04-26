use std::collections::HashMap;
use worker::*;

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
  console_error_panic_hook::set_once();
  const ALLOWED_ORIGIN: &str = "https://fashni.github.io";
  match req.headers().get("Origin").unwrap() {
    Some(origin) => {
      if origin != ALLOWED_ORIGIN {
        return Response::error("403 Unauthorized", 403);
      }
    },
    None => {
      return Response::error("403 Unauthorized", 403);
    }
  };

  let url: Url = req.url().unwrap();
  let params: HashMap<_, _> = url.query_pairs().into_owned().collect();
  let city: &String = match params.get("city") {
    Some(c) => c,
    None => {
      return Response::ok(
        "{\"cod\": \"400\", \"message\": \"missing city param\"}"
      ).map(|mut resp: Response| {
        resp.headers_mut().set("Content-Type", "application/json").unwrap();
        resp.headers_mut().set("Access-Control-Allow-Origin", ALLOWED_ORIGIN).unwrap();
        resp.with_status(400)
      })
    }
  };

  let api_key: String = env.var("OPENWEATHER_API_KEY").unwrap().to_string();
  let api_url: String = format!(
    "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}",
    urlencoding::encode(city),
    api_key
  );

  let client: Fetch = Fetch::Url(api_url.parse().unwrap());
  let mut res: Response = client.send().await?;
  let text: String = res.text().await?;
  Response::ok(text).map(|mut resp: Response| {
    resp.headers_mut().set("Content-Type", "application/json").unwrap();
    resp.headers_mut().set("Access-Control-Allow-Origin", ALLOWED_ORIGIN).unwrap();
    resp
  })
}
