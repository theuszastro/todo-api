use super::views::users::CreatedUser;

use std::convert::Infallible;

use futures::TryStreamExt;

use hyper::{Body, Method, Request, Response};

use futures::lock::Mutex;
use rusqlite::Connection;
use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde_json::{from_slice, Error as SerdeError};

#[derive(Debug, Serialize, Deserialize)]
struct Error {
   error: &'static str,
}

pub struct RequestInfo {
   pub path: String,
   pub method: Method,
   pub path_splited: Vec<String>,
}

pub async fn get_request_info(req: &Request<Body>) -> RequestInfo {
   let path = req.uri().path().to_string();
   let method = req.method();

   let path_splited: Vec<String> = path
      .split("/")
      .filter(|data| data.len() >= 1)
      .map(|item| item.to_string())
      .collect();

   let path_formated = format!(
      "/{}",
      if path_splited.len() < 1 {
         "/"
      } else {
         path_splited[0].as_str()
      }
   );

   RequestInfo {
      path: path_formated,
      method: method.clone(),
      path_splited,
   }
}

pub fn create_error(
   reason: &'static str,
   status: Option<u16>,
) -> Result<Response<Body>, Infallible> {
   let code = if status != None { status.unwrap() } else { 400 };

   let mut label = reason;

   if reason.len() < 1 {
      label = "Internal Server Error";
   }

   let json = serde_json::to_string(&Error { error: label });

   match json {
      Ok(data) => Ok(Response::builder()
         .status(code)
         .body(Body::from(data))
         .unwrap()),
      _ => Ok(Response::builder()
         .status(500)
         .body(Body::from("Internal Server Error"))
         .unwrap()),
   }
}
pub fn valid_json(json: Result<String, SerdeError>) -> Result<Response<Body>, Infallible> {
   match json {
      Ok(string) => {
         let response = Response::builder()
            .status(200)
            .body(Body::from(string))
            .unwrap();

         Ok(response)
      }
      Err(_) => create_error("", None),
   }
}

pub async fn parse_body<'de, T>(body: Body) -> Result<T, SerdeError>
where
   T: DeserializeOwned,
{
   let body_unformated = body
      .try_fold(Vec::new(), |mut data, chunk| async move {
         data.extend_from_slice(&chunk);

         Ok(data)
      })
      .await;

   match body_unformated {
      Ok(slice) => {
         let data: Result<T, SerdeError> = from_slice(&slice);

         match data {
            Ok(new_data) => Ok(new_data),
            Err(e) => Err(e),
         }
      }
      Err(_) => panic!("error"),
   }
}

pub async fn get_users(
   conn: Arc<Mutex<Connection>>,
   value: String,
   query_type: String,
) -> Vec<CreatedUser> {
   let conn = conn.lock().await;

   let sql = match query_type.as_str() {
      "email" => "SELECT * FROM users WHERE email = ?",
      "id" => "SELECT * FROM users WHERE id = ?",
      _ => "",
   };

   let mut query = conn.prepare(sql).unwrap();
   let result = query
      .query_map([value.clone()], |row| {
         Ok(CreatedUser {
            id: row.get(0)?,
            firstname: row.get(1)?,
            lastname: row.get(2)?,
            email: row.get(3)?,
            password: row.get(4)?,
         })
      })
      .unwrap();

   let mut users: Vec<CreatedUser> = vec![];
   for user in result {
      users.push(user.unwrap());
   }

   users
}
