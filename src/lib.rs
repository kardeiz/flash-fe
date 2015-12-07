extern crate iron;
extern crate cookie_fe;
extern crate session_fe;
extern crate rustc_serialize;

use std::borrow::ToOwned;
use std::collections::BTreeMap;

use std::sync::{Arc, RwLock};

use rustc_serialize::json::{self, ToJson};

use iron::prelude::*;
use iron::{Handler, AroundMiddleware, typemap};

use cookie_fe::{CookieWrapper, WithCookieJar, CookiePair};
use session_fe::{SessionUtil, WithSession};

#[derive(Clone, Debug)]
pub struct FlashUtil {
  pub _key: Option<String>,
  pub now: Arc<RwLock<Option<json::Object>>>,
  pub next: Arc<RwLock<Option<json::Object>>>
}

impl FlashUtil {
  
  pub fn new(_key: Option<String>) -> Self {
    FlashUtil { 
      _key: _key, 
      now: Arc::new(RwLock::new(None)), 
      next: Arc::new(RwLock::new(None))
    }
  }

  pub fn key(&self) -> String {
    self._key.clone()
      .unwrap_or_else(|| "flash".to_string() )
  }

  pub fn rotate_in(&self, req: &Request) -> FlashUtil {
    let flash = req.get_session().as_ref()
      .and_then(|session| session.get(&self.key()))
      .and_then(|flash| flash.as_object() )
      .map(|flash| flash.to_owned() );
    FlashUtil { 
      _key: self._key.clone(), // as_ref().map(|x| x.clone()),
      now: Arc::new(RwLock::new(flash)),
      next: Arc::new(RwLock::new(None))
    }
  }

  pub fn rotate_out(&self, req: &Request) {
    let flash = self.next.read().ok()
      .and_then(|x| (*x).clone() );

    let mut session = req.get_session()
      .unwrap_or_else(|| BTreeMap::new() );
    session.insert(self.key(), flash.to_json());
    req.set_session(session);
  }

}


impl typemap::Key for FlashUtil { type Value = FlashUtil; }

struct FlashRotator<H: Handler> {
  handler: H,
  flash_util: FlashUtil
}

impl<H: Handler> Handler for FlashRotator<H> {
  fn handle(&self, req: &mut Request) -> IronResult<Response> {
    
    let flash_util = self.flash_util.rotate_in(req);

    req.extensions.insert::<FlashUtil>(flash_util);

    let res = self.handler.handle(req);

    if res.is_ok() {
      req.extensions.get::<FlashUtil>().unwrap().rotate_out(req);
    }

    res
  }
}

impl AroundMiddleware for FlashUtil {
  fn around(self, handler: Box<Handler>) -> Box<Handler> {
    Box::new(FlashRotator {
      handler: handler,
      flash_util: self
    }) as Box<Handler>
  }
}

pub trait WithFlash {
  fn get_flash(&self) -> Option<json::Object>;
  fn set_flash(&self, json::Object);
}

impl<'a, 'b> WithFlash for Request<'a, 'b> {

  fn get_flash(&self) -> Option<json::Object> {
    let flash_util = 
      self.extensions.get::<FlashUtil>().expect("Flash not found");
    flash_util.now.read().ok().and_then(|x| (*x).clone() )
  }

  fn set_flash(&self, val: json::Object) {
    let flash_util = 
      self.extensions.get::<FlashUtil>().expect("Flash not found");
    flash_util.next.write().map(|mut x| *x = Some(val) );
  }

  // fn flash(&self) -> &FlashUtil {
  //   self.extensions.get::<FlashUtil>().expect("Flash not found")
  // }

}

#[test]
fn it_works() {
}
