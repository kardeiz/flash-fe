extern crate iron;
extern crate session_fe;

use std::fmt::Debug;
use std::any::Any;
use std::marker::PhantomData;


use iron::prelude::*;
use iron::{Handler, AroundMiddleware, typemap};

use session_fe::Util as SessionUtil;

#[derive(Clone, Debug)]
pub struct Util<T: Flashable + Debug + Clone + Any> {
    pub now: Option<T::Object>,
    pub next: Option<T::Object>,
    pub pd_type: PhantomData<T>
}

pub trait Flashable {
    type Object: Debug + Clone + Any;

    fn new() -> Self;
    fn flash(&self) -> Option<&Self::Object>;
    fn set_flash(&mut self, val: Option<Self::Object>);
}

impl<T: Flashable + Debug + Clone + Any> Util<T> {

    pub fn new() -> Self {
        Util { 
            now: None,
            next: None,
            pd_type: PhantomData
        }
    }

    pub fn rotate_in(&mut self, req: &Request) {
        if let Some(obj) = req.extensions.get::<SessionUtil<T>>()
            .and_then(|s| s.get() ) {
            if let Some(flash) = obj.flash() {
                self.now = Some(flash.clone());
            }
        }  
    }

    pub fn rotate_out(&self, req: &Request) {
        if let Some(sess) = req.extensions.get::<SessionUtil<T>>() {
            if let Some(ref next) = self.next {
                if let Some(mut obj) = sess.get() {
                    obj.set_flash(Some(next.clone()));
                    sess.set(obj);
                } else {
                    let mut obj = <T>::new();
                    obj.set_flash(Some(next.clone()));
                    sess.set(obj);
                }
            } else if let Some(mut obj) = sess.get() {
                obj.set_flash(None);
                sess.set(obj);
            }            
        }
    }

    pub fn get(&self) -> Option<&T::Object> {
        self.now.as_ref()
    }

    pub fn set(&mut self, value: Option<T::Object>) {
        self.next = value;
    }

}

impl<T: Flashable + Debug + Clone + Any> typemap::Key for Util<T> { type Value = Self; }

pub struct Builder<T: Flashable + Debug + Clone + Any>(PhantomData<T>);

impl<T: Flashable + Debug + Clone + Any> Builder<T> {
    pub fn new() -> Self {
        Builder(PhantomData)
    }
}

struct Rotator<H: Handler, T: Flashable + Debug + Clone + Any> {
    handler: H,
    pd_type: PhantomData<T>
}

impl<H: Handler, T: Flashable + Debug + Clone + Any + Send + Sync> Handler for Rotator<H, T> {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {

        let mut util = Util::new();

        util.rotate_in(req);

        req.extensions.insert::<Util<T>>(util);

        let res = self.handler.handle(req);

        if res.is_ok() {
            if let Some(util) = req.extensions.get::<Util<T>>() {              
                util.rotate_out(req);
            }
        }

        res
    }
}

impl<T: Flashable + Debug + Clone + Any + Send + Sync> AroundMiddleware for Builder<T> {
    fn around(self, handler: Box<Handler>) -> Box<Handler> {
        let rotator = Rotator {
            handler: handler,
            pd_type: self.0
        };
        Box::new(rotator)
    }
}
