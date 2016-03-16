extern crate iron;
extern crate session_fe;
extern crate rustc_serialize;

use std::sync::{Arc, RwLock};
use std::collections::BTreeMap;

use rustc_serialize::json::{self, ToJson};

use iron::prelude::*;
use iron::{Handler, AroundMiddleware, typemap};

use session_fe::Util as SessionUtil;

#[derive(Clone, Debug)]
pub struct Util {
    pub key: &'static str,
    pub now: Option<json::Json>,
    pub next: Option<json::Json>
}

impl Util {

    pub fn new(key: Option<&'static str>) -> Self {
        Util { 
            key: key.unwrap_or("iron.flash"), 
            now: None,
            next: None
        }
    }

    pub fn rotate_in(&mut self, req: &Request) {
        if let Some(mut obj) = req.extensions.get::<SessionUtil>()
            .and_then(|s| s.get() ) {
            if let Some(flash) = obj.remove(self.key) {
                self.now = Some(flash);
            }
        }  
    }


    pub fn rotate_out(&self, req: &Request) {
        if let Some(sess) = req.extensions.get::<SessionUtil>() {
            if let Some(next) = self.next.clone() {
                let mut map = sess.get()
                    .unwrap_or_else(|| json::Object::new());
                map.insert(self.key.to_owned(), next);
                sess.set(map);
            } else {
                if let Some(mut map) = sess.get() {
                    if map.remove(self.key).is_some() {
                        sess.set(map);
                    }
                }
            }
        }
    }

    pub fn get(&self) -> Option<json::Json> {
        self.now.clone()
    }

    pub fn set(&mut self, value: json::Json) {
        self.next = Some(value);
    }

}

impl typemap::Key for Util { type Value = Self; }

pub struct Builder(pub Option<&'static str>);

struct Rotator<H: Handler> {
    handler: H,
    builder: Builder
}

impl<H: Handler> Handler for Rotator<H> {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {

        let mut util = Util::new(self.builder.0);

        util.rotate_in(req);

        req.extensions.insert::<Util>(util);

        let res = self.handler.handle(req);

        if res.is_ok() {
            if let Some(util) = req.extensions.get::<Util>() {
                util.rotate_out(req);
            }
        }

        res
    }
}

impl AroundMiddleware for Builder {
    fn around(self, handler: Box<Handler>) -> Box<Handler> {
        Box::new(Rotator {
            handler: handler,
            builder: self
        }) as Box<Handler>
    }
}
