mod end_point_type;
mod req_deserializer;
mod resource;
mod response;
mod route;
mod route_trie;

use self::route_trie::{RouteTrie, RouteValueResult};
use crate::context::Context;
use crate::middleware::Middleware;
use crate::Method;
use crate::ObsidianError;

pub use self::end_point_type::EndPointHandler;
pub use self::req_deserializer::{from_cow_map, Error as FormError};
pub use self::resource::Resource;
pub use self::response::ResponseBuilder;
pub use self::route::Route;

pub struct Router {
    routes: RouteTrie,
}

impl Clone for Router {
    fn clone(&self) -> Self {
        Router {
            routes: self.routes.clone(),
        }
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: RouteTrie::new(),
        }
    }

    pub fn get(&mut self, path: &str, handler: impl EndPointHandler) {
        self.insert_route(Method::GET, path, handler);
    }

    pub fn post(&mut self, path: &str, handler: impl EndPointHandler) {
        self.insert_route(Method::POST, path, handler);
    }

    pub fn put(&mut self, path: &str, handler: impl EndPointHandler) {
        self.insert_route(Method::PUT, path, handler);
    }

    pub fn delete(&mut self, path: &str, handler: impl EndPointHandler) {
        self.insert_route(Method::DELETE, path, handler);
    }

    /// Apply middleware in the provided route
    pub fn use_service_to(&mut self, path: &str, middleware: impl Middleware) {
        self.routes.insert_middleware(path, middleware);
    }

    /// Apply middleware in current relative route
    pub fn use_service(&mut self, middleware: impl Middleware) {
        self.routes.insert_default_middleware(middleware);
    }

    /// Serve static files by the virtual path as the route and directory path as the server file path
    pub fn use_static_to(&mut self, virtual_path: &str, dir_path: &str) {
        let mut path = String::from(virtual_path);
        path.push_str("/*");

        self.get(
            &path,
            Self::static_virtual_file_handler(virtual_path, dir_path),
        );
    }

    /// Serve static files by the directory path as the route and server file path
    pub fn use_static(&mut self, dir_path: &str) {
        let mut path = String::from(dir_path);
        path.push_str("/*");

        self.get(&path, Self::static_dir_file_handler());
    }

    /// Apply route handler in current relative route
    pub fn use_router(&mut self, path: &str, other: Router) {
        RouteTrie::insert_sub_route(&mut self.routes, path, other.routes);
    }

    pub fn search_route(&self, path: &str) -> Result<RouteValueResult, ObsidianError> {
        self.routes.search_route(path)
    }

    fn insert_route(&mut self, method: Method, path: &str, handler: impl EndPointHandler) {
        let route = Route::new(method, handler);

        self.routes.insert_route(path, route);
    }

    fn static_virtual_file_handler(
        virtual_path: &str,
        dir_path: &str,
    ) -> impl Fn(Context, ResponseBuilder) -> ResponseBuilder {
        let dir_path = dir_path
            .split('/')
            .filter(|key| !key.is_empty())
            .map(|x| x.to_string())
            .collect::<Vec<String>>();

        let virtual_path_len = virtual_path
            .split('/')
            .filter(|key| !key.is_empty())
            .count();

        move |ctx: Context, res: ResponseBuilder| {
            let mut dir_path = dir_path.clone();
            let mut relative_path = ctx
                .uri()
                .path()
                .split('/')
                .filter(|key| !key.is_empty())
                .skip(virtual_path_len)
                .map(|x| x.to_string())
                .collect::<Vec<String>>();

            dir_path.append(&mut relative_path);

            res.send_file(&dir_path.join("/"))
        }
    }

    fn static_dir_file_handler() -> impl Fn(Context, ResponseBuilder) -> ResponseBuilder {
        move |ctx: Context, res: ResponseBuilder| {
            let relative_path = ctx
                .uri()
                .path()
                .split('/')
                .filter(|key| !key.is_empty())
                .map(|x| x.to_string())
                .collect::<Vec<String>>();

            res.send_file(&relative_path.join("/"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;
    use crate::middleware::Logger;

    fn handler(_ctx: Context, res: ResponseBuilder) -> ResponseBuilder {
        res.body("test")
    }

    #[test]
    fn router_get_test() {
        let mut router = Router::new();

        router.get("router/test", handler);

        let result = router.search_route("router/test");
        let fail_result = router.search_route("failed");

        assert!(result.is_ok());
        assert!(fail_result.is_err());

        match result {
            Ok(route) => {
                let middleware = route.get_middleware();
                let route_value = route.get_route(&Method::GET).unwrap();

                assert_eq!(middleware.len(), 0);
                assert_eq!(route_value.method, Method::GET);
            }
            _ => {
                assert!(false);
            }
        }
    }

    #[test]
    fn router_post_test() {
        let mut router = Router::new();

        router.post("router/test", handler);

        let result = router.search_route("router/test");
        let fail_result = router.search_route("failed");

        assert!(result.is_ok());
        assert!(fail_result.is_err());

        match result {
            Ok(route) => {
                let middleware = route.get_middleware();
                let route_value = route.get_route(&Method::POST).unwrap();

                assert_eq!(middleware.len(), 0);
                assert_eq!(route_value.method, Method::POST);
            }
            _ => {
                assert!(false);
            }
        }
    }

    #[test]
    fn router_put_test() {
        let mut router = Router::new();

        router.put("router/test", handler);

        let result = router.search_route("router/test");
        let fail_result = router.search_route("failed");

        assert!(result.is_ok());
        assert!(fail_result.is_err());

        match result {
            Ok(route) => {
                let middleware = route.get_middleware();
                let route_value = route.get_route(&Method::PUT).unwrap();

                assert_eq!(middleware.len(), 0);
                assert_eq!(route_value.method, Method::PUT);
            }
            _ => {
                assert!(false);
            }
        }
    }

    #[test]
    fn router_delete_test() {
        let mut router = Router::new();

        router.delete("router/test", handler);

        let result = router.search_route("router/test");
        let fail_result = router.search_route("failed");

        assert!(result.is_ok());
        assert!(fail_result.is_err());

        match result {
            Ok(route) => {
                let middleware = route.get_middleware();
                let route_value = route.get_route(&Method::DELETE).unwrap();

                assert_eq!(middleware.len(), 0);
                assert_eq!(route_value.method, Method::DELETE);
            }
            _ => {
                assert!(false);
            }
        }
    }

    #[test]
    fn router_root_middleware_test() {
        let mut router = Router::new();
        let logger = Logger::new();

        router.use_service(logger);

        let result = router.search_route("/");
        let fail_result = router.search_route("failed");

        assert!(result.is_ok());
        assert!(fail_result.is_err());

        match result {
            Ok(route) => {
                let middleware = route.get_middleware();

                assert_eq!(middleware.len(), 1);
            }
            _ => {
                assert!(false);
            }
        }
    }

    #[test]
    fn router_relative_middleware_test() {
        let mut router = Router::new();
        let logger = Logger::new();

        router.use_service_to("middleware/child", logger);

        let result = router.search_route("/middleware/child");
        let fail_result = router.search_route("/");

        assert!(result.is_ok());
        assert!(fail_result.is_err());

        match result {
            Ok(route) => {
                let middleware = route.get_middleware();

                assert_eq!(middleware.len(), 1);
            }
            _ => {
                assert!(false);
            }
        }
    }

    #[test]
    fn router_search_test() {
        let mut router = Router::new();

        router.get("router/test", handler);
        router.post("router/test", handler);
        router.put("router/test", handler);
        router.delete("router/test", handler);

        router.get("route/diff_route", handler);

        let result = router.search_route("router/test");
        let diff_result = router.search_route("route/diff_route");
        let fail_result = router.search_route("failed");

        assert!(result.is_ok());
        assert!(diff_result.is_ok());
        assert!(fail_result.is_err());

        match result {
            Ok(route) => {
                let middleware = route.get_middleware();
                let route_value = route.get_route(&Method::GET).unwrap();

                assert_eq!(middleware.len(), 0);
                assert_eq!(route_value.method, Method::GET);

                let route_value = route.get_route(&Method::POST).unwrap();

                assert_eq!(middleware.len(), 0);
                assert_eq!(route_value.method, Method::POST);

                let route_value = route.get_route(&Method::PUT).unwrap();

                assert_eq!(middleware.len(), 0);
                assert_eq!(route_value.method, Method::PUT);

                let route_value = route.get_route(&Method::DELETE).unwrap();

                assert_eq!(middleware.len(), 0);
                assert_eq!(route_value.method, Method::DELETE);
            }
            _ => {
                assert!(false);
            }
        }

        match diff_result {
            Ok(route) => {
                let middleware = route.get_middleware();
                let route_value = route.get_route(&Method::GET).unwrap();

                assert_eq!(middleware.len(), 0);
                assert_eq!(route_value.method, Method::GET);
            }
            _ => {
                assert!(false);
            }
        }
    }

    #[test]
    fn router_merge_test() {
        let mut main_router = Router::new();
        let mut sub_router = Router::new();

        main_router.get("router/test", handler);
        sub_router.get("router/test", handler);

        let logger = Logger::new();

        sub_router.use_service(logger);

        main_router.use_router("sub_router", sub_router);

        let result = main_router.search_route("router/test");
        let sub_result = main_router.search_route("sub_router/router/test");
        let fail_result = main_router.search_route("failed");

        assert!(result.is_ok());
        assert!(sub_result.is_ok());
        assert!(fail_result.is_err());

        match result {
            Ok(route) => {
                let middleware = route.get_middleware();
                let route_value = route.get_route(&Method::GET).unwrap();

                assert_eq!(middleware.len(), 0);
                assert_eq!(route_value.method, Method::GET);
            }
            _ => {
                assert!(false);
            }
        }

        match sub_result {
            Ok(route) => {
                let middleware = route.get_middleware();
                let route_value = route.get_route(&Method::GET).unwrap();

                assert_eq!(middleware.len(), 1);
                assert_eq!(route_value.method, Method::GET);
            }
            _ => {
                assert!(false);
            }
        }
    }

    #[should_panic]
    #[test]
    fn router_duplicate_path_test() {
        let mut router = Router::new();

        router.get("router/test", handler);
        router.get("router/test", handler);
    }

    #[should_panic]
    #[test]
    fn router_ambiguous_path_test() {
        let mut router = Router::new();

        router.get("router/:test", handler);
        router.get("router/test", handler);
    }

    #[should_panic]
    #[test]
    fn router_duplicate_merge_test() {
        let mut main_router = Router::new();
        let mut sub_router = Router::new();

        main_router.get("sub_router/test", handler);
        sub_router.get("test", handler);

        let logger = Logger::new();

        sub_router.use_service(logger);

        main_router.use_router("sub_router", sub_router);
    }
}
