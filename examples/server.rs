#[macro_use]
extern crate iron;

extern crate cookie_fe;
extern crate session_fe;
extern crate flash_fe;

extern crate router;
extern crate rustc_serialize;
extern crate time;
extern crate rand; 

use iron::prelude::*;
use iron::{status, AroundMiddleware};

use router::Router;

use cookie_fe::{Util as CookieUtil, Builder as CookieBuilder, CookiePair};
use session_fe::{Util as SessionUtil, Builder as SessionBuilder};
use flash_fe::{Util as FlashUtil, Builder as FlashBuilder, Flashable};

use std::collections::{BTreeMap, HashMap};

use rustc_serialize::json::{self, ToJson};
use rustc_serialize::hex::{self, ToHex};

use rand::{thread_rng, Rng};

const KEY: &'static [u8] = b"4b8eee793a846531d6d95dd66ae48319";

pub struct Helper;

#[derive(Clone, Debug)]
pub struct Session {
    flash: Option<HashMap<String,Vec<String>>>
}

impl Flashable for Session {

    fn new() -> Self { Session { flash: None } }

    fn flash(&self) -> Option<HashMap<String,Vec<String>>> {
        self.flash.clone()
    }
    fn set_flash(&mut self, val: Option<HashMap<String,Vec<String>>>) {
        self.flash = val;
    }

}

impl Helper {

    pub fn random() -> String {
        let mut v = [0; 16];
        rand::thread_rng().fill_bytes(&mut v);
        v.to_hex()
    }

    fn key(sid: Option<&'static str>) -> Box<Fn(&mut Request) -> String + Send + Sync> {
        let out = move |req: &mut Request| -> String {
            let jar = req.extensions.get_mut::<CookieUtil>()
                .and_then(|x| x.jar() )
                .expect("No cookie jar");
            let sid = sid.unwrap_or("IRONSID");
            if let Some(cookie) = jar.signed().find(sid) {
                cookie.value
            } else {
                let key = Self::random();
                let cookie = CookiePair::new(sid.to_owned(), key.clone());
                jar.signed().add(cookie);
                key
            }
        };
        Box::new(out)        
    }
}


fn set(req: &mut Request) -> IronResult<Response> {

    let mut res = Response::with((status::Ok, "Set flash"));

    let mut map = HashMap::new();

    map.insert("foo".to_string(), vec!["hello".to_string()]);

    let flash = iexpect!(req.extensions.get_mut::<FlashUtil<Session>>());

    flash.set(Some(map));


    Ok(res)
}

fn get(req: &mut Request) -> IronResult<Response> {

    let mut res = Response::new();

    let flash = iexpect!(req.extensions.get::<FlashUtil<Session>>());

    res
        .set_mut(status::Ok)
        .set_mut(format!("{:?}", flash.get()));

    Ok(res)
}

fn main() {

    let sessioning = SessionBuilder::<Session>::new(Helper::key(None));

    let mut router = Router::new();

    router
        .get("/get", get)
        .get("/set", set);

    let flashed = FlashBuilder::<Session>::new().around(Box::new(router));

    let mut chain = Chain::new(flashed);
    chain.link_before(sessioning);

    let cookied = CookieBuilder(KEY).around(Box::new(chain));

    Iron::new(cookied).http("0.0.0.0:3000").unwrap();
}
