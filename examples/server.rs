extern crate iron;

extern crate cookie_fe;
extern crate session_fe;
extern crate flash_fe;

extern crate router;
extern crate rustc_serialize;
extern crate time;

use iron::prelude::*;
use iron::{status, AroundMiddleware};

use router::Router;

use cookie_fe::{CookieWrapper, WithCookieJar, CookiePair};
use session_fe::{SessionUtil, WithSession};
use flash_fe::{FlashUtil, WithFlash};

use std::collections::{BTreeMap, HashMap};

use rustc_serialize::json::{self, ToJson};

const KEY: &'static [u8] = b"4b8eee793a846531d6d95dd66ae48319";

fn a(req: &mut Request) -> IronResult<Response> {

  let mut res = Response::with((status::Ok, "Set session"));

  let mut map = BTreeMap::new();

  map.insert("foo".to_string(), 23.to_json());

  req.set_flash(map);
  
  Ok(res)
}

fn b(req: &mut Request) -> IronResult<Response> {

  let mut res = Response::new();

  let flash = req.get_flash();

  res
    .set_mut(status::Ok)
    .set_mut(format!("{:?}", flash));

  Ok(res)
}

fn main() {

  let session_util = SessionUtil::new();

  let mut router = Router::new();
  router.get("/a", a);
  router.get("/b", b);
  
  let flashed = FlashUtil::new(None).around(Box::new(router));
  let cookied = CookieWrapper(KEY).around(flashed);

  let mut chain = Chain::new(cookied);

  chain.link_before(session_util);

  Iron::new(chain).http("0.0.0.0:3000").unwrap();
}
