use std::sync::Arc;

use futures::future::Future;
use hyper::{Body, Request, Response};

use super::middleware::Middleware;
use super::router::{EndPointHandler, ResponseBuilder};

pub struct Context<'a> {
    request: Request<Body>,
    middleware: &'a [Arc<Middleware>],
    route_endpoint: &'a Arc<dyn EndPointHandler<Output = ResponseBuilder>>,
    // params
}

impl<'a> Context<'a> {
    pub fn new(
        request: Request<Body>,
        route_endpoint: &'a Arc<dyn EndPointHandler<Output = ResponseBuilder>>,
        middleware: &'a [Arc<Middleware>],
    ) -> Self {
        Context {
            request,
            middleware,
            route_endpoint,
        }
    }

    pub fn next(mut self) -> Box<Future<Item = Response<Body>, Error = hyper::Error> + Send> {
        if let Some((current, all_next)) = self.middleware.split_first() {
            self.middleware = all_next;
            current.run(self)
        } else {
            let res = ResponseBuilder::new();
            let route_response = (*self.route_endpoint)(self.request, res);
            route_response.into()
        }
    }
}
