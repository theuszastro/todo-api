#[macro_use]
extern crate serde_derive;

mod controllers;
mod database;
mod middlewares;
mod utils;
mod views;

use std::convert::Infallible;
use std::io::Error;
use std::sync::Arc;

use futures::lock::Mutex;

use lazy_static::lazy_static;

use rusqlite::Connection;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server};

use utils::{create_error, get_request_info, RequestInfo};

use controllers::tasks;
use controllers::users;

lazy_static! {
   static ref CONNECTION: Arc<Mutex<Connection>> = {
      let conn = database::create_connection();

      Arc::new(Mutex::new(conn))
   };
}

#[tokio::main]
async fn main() -> Result<(), Error> {
   let addr = ([127, 0, 0, 1], 3333).into();

   let make_svc = make_service_fn(|_| async { Ok::<_, Infallible>(service_fn(routes)) });
   let server = Server::bind(&addr).serve(make_svc);

   if let Err(e) = server.await {
      eprintln!("Server Error: {}", e);
   }

   Ok(())
}

async fn routes(req: Request<Body>) -> Result<Response<Body>, Infallible> {
   let RequestInfo {
      path,
      method,
      path_splited,
   } = get_request_info(&req).await;

   let conn = CONNECTION.clone();

   match (method, path.as_str()) {
      (Method::GET, "/users") => users::list_all_users(conn).await,
      (Method::GET, "/user") if path_splited.len() > 1 => {
         users::list_by_id(req, conn, String::from(path_splited[1].as_str())).await
      }
      (Method::POST, "/register") => users::create_user(req, conn).await,
      (Method::POST, "/login") => users::login(req, conn).await,
      (Method::PUT, "/user") if path_splited.len() > 1 => {
         users::update_user(req, conn, path_splited[1].clone()).await
      }
      (Method::DELETE, "/user") if path_splited.len() > 1 => {
         users::delete_user(req, conn, path_splited[1].clone()).await
      }

      (Method::GET, "/tasks") => tasks::list_tasks(req, conn).await,
      (Method::POST, "/tasks") => tasks::create_task(req, conn).await,
      (Method::PUT, "/task") if path_splited.len() > 1 => {
         tasks::update_task(req, conn, path_splited[1].clone()).await
      }
      (Method::DELETE, "/task") if path_splited.len() > 1 => {
         tasks::delete_task(req, conn, path_splited[1].clone()).await
      }
      _ => create_error("this router is not exists", None),
   }
}
