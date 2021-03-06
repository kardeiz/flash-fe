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

use cookie_fe::Builder as CookieBuilder;
use session_fe::{Builder as SessionBuilder, helpers as session_helpers};
use flash_fe::{Util as FlashUtil, Builder as FlashBuilder, Flashable};

use std::collections::HashMap;

const KEY: &'static [u8] = b"4b8eee793a846531d6d95dd66ae48319";

#[derive(Clone, Debug)]
pub struct Flash(HashMap<&'static str, Vec<&'static str>>);

#[derive(Clone, Debug)]
pub struct Session {
    flash: Option<Flash>
}

impl Flashable for Session {

    type Object = Flash;

    fn new() -> Self { Session { flash: None } }

    fn flash(&self) -> Option<&Self::Object> {
        self.flash.as_ref()
    }

    fn set_flash(&mut self, val: Option<Self::Object>) {
        self.flash = val;
    }

}

fn set(req: &mut Request) -> IronResult<Response> {

    let res = Response::with((status::Ok, "Set flash"));

    let mut map = HashMap::new();

    map.insert("foo", vec!["hello"]);

    let flash = iexpect!(req.extensions.get_mut::<FlashUtil<Session>>());

    flash.set(Some(Flash(map)));


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

    let sessioning = SessionBuilder::<Session>::new(session_helpers::key_gen(None));

    let mut router = Router::new();

    router
        .get("/get", get)
        .get("/set", set);

    let flashed = FlashBuilder::<Session>::new().around(Box::new(router));

    let mut chain = Chain::new(flashed);
    chain.link_before(sessioning);

    let cookied = CookieBuilder::new(KEY).around(Box::new(chain));

    Iron::new(cookied).http("0.0.0.0:3000").unwrap();
}
