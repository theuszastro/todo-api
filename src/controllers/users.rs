use super::super::middlewares::users::{valid_user, ValidResponse};
use super::super::utils::{create_error, get_users, parse_body, valid_json};
use super::super::views::tasks::TaskCreated;
use super::super::views::users::{CreatedUser, CreatedUserComplete};

use std::convert::{From, Infallible};
use std::ops::Add;
use std::sync::Arc;

use futures::lock::Mutex;

use chrono::{DateTime, Duration, Local, Utc};
use jsonwebtokens::{encode, Algorithm, AlgorithmID};

use rusqlite::Connection;

use bcrypt::{hash, verify};
use serde_json::json;

use uuid::Uuid;

use hyper::{Body, Request, Response};

#[derive(Serialize)]
struct Users {
   users: Vec<CreatedUserComplete>,
}

#[derive(Serialize, Deserialize, Debug)]
struct RequestBodyUser {
   firstname: String,
   lastname: String,
   email: String,
   password: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct RequestBodyUpdate {
   firstname: Option<String>,
   lastname: Option<String>,
   email: Option<String>,
   password: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct RequestBodyLogin {
   email: String,
   password: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Claims {
   exp: usize,
}

pub async fn list_all_users(conn: Arc<Mutex<Connection>>) -> Result<Response<Body>, Infallible> {
   let conn = conn.lock().await;

   let mut query = conn.prepare("SELECT *, tasks.id as task_id FROM users LEFT OUTER JOIN tasks ON tasks.user_id = users.id").unwrap();
   let data = query
      .query_map([], |row| {
         let id = row.get(5);
         let name = row.get(6);
         let completed = row.get(7);

         let tasks = if id.is_ok() && name.is_ok() && completed.is_ok() {
            let task = TaskCreated {
               id: id.unwrap(),
               name: name.unwrap(),
               completed: completed.unwrap(),
               user: None,
            };

            vec![task.format()]
         } else {
            vec![]
         };

         Ok(CreatedUserComplete {
            id: row.get(0)?,
            firstname: row.get(1)?,
            lastname: row.get(2)?,
            email: row.get(3)?,

            tasks,
         })
      })
      .unwrap();

   let mut users: Vec<CreatedUserComplete> = vec![];
   for user in data {
      let user = user.unwrap();

      let position = users.iter_mut().position(|item| item.id == user.id);

      if position.is_none() {
         users.push(user);
      } else {
         let position = position.unwrap();

         for task in user.tasks {
            users[position].tasks.push(task);
         }
      }
   }

   let json = serde_json::to_string(&Users { users });

   valid_json(json)
}

pub async fn list_by_id(
   req: Request<Body>,
   conn: Arc<Mutex<Connection>>,
   user_id: String,
) -> Result<Response<Body>, Infallible> {
   let result = valid_user(req.headers()).await;

   match result {
      Ok(data) => match data {
         ValidResponse::Id(id) => {
            if id != user_id {
               return create_error("you not have permission for to follow", Option::from(401));
            }

            let conn = conn.lock().await;

            let mut query = conn
               .prepare("SELECT * FROM users LEFT OUTER JOIN tasks ON users.id = tasks.user_id WHERE users.id = ?")
               .unwrap();
            let data = query
               .query_map([user_id.clone()], |row| {
                  let id = row.get(5);
                  let name = row.get(6);
                  let completed = row.get(7);

                  let tasks = if id.is_ok() && name.is_ok() && completed.is_ok() {
                     let task = TaskCreated {
                        id: id.unwrap(),
                        name: name.unwrap(),
                        completed: completed.unwrap(),
                        user: None,
                     };

                     vec![task.format()]
                  } else {
                     vec![]
                  };

                  Ok(CreatedUserComplete {
                     id: row.get(0)?,
                     firstname: row.get(1)?,
                     lastname: row.get(2)?,
                     email: row.get(3)?,

                     tasks,
                  })
               })
               .unwrap();

            let mut users: Vec<CreatedUserComplete> = vec![];
            for user in data {
               let user = user.unwrap();

               let position = users.iter_mut().position(|item| item.id == user.id);

               if position.is_none() {
                  users.push(user);
               } else {
                  let position = position.unwrap();

                  for task in user.tasks {
                     users[position].tasks.push(task);
                  }
               }
            }

            if users.len() < 1 {
               return create_error("this user is not exists", None);
            }

            let json = serde_json::to_string(&users[0]);

            valid_json(json)
         }
         ValidResponse::Respo(data) => return data,
      },
      _ => return create_error("", None),
   }
}

pub async fn create_user(
   req: Request<Body>,
   conn: Arc<Mutex<Connection>>,
) -> Result<Response<Body>, Infallible> {
   let body = parse_body::<RequestBodyUser>(req.into_body()).await;

   match body {
      Ok(RequestBodyUser {
         firstname,
         lastname,
         email,
         password,
      }) => {
         let users = get_users(conn.clone(), email.clone(), String::from("email")).await;
         if users.len() >= 1 {
            return create_error("this email already in use", None);
         }

         let conn = conn.lock().await;

         let user_id = Uuid::new_v4();
         let result = conn.execute(
            "INSERT INTO users VALUES (?, ?, ?, ?, ?)",
            [
               user_id.to_string(),
               firstname,
               lastname,
               email,
               hash(password, 7).unwrap(),
            ],
         );

         match result {
            Ok(_) => Ok(Response::builder()
               .status(201)
               .body(Body::from(""))
               .unwrap()),
            _ => return create_error("", None),
         }
      }
      Err(_) => create_error("data invalid", None),
   }
}

pub async fn login(
   req: Request<Body>,
   conn: Arc<Mutex<Connection>>,
) -> Result<Response<Body>, Infallible> {
   let body = parse_body::<RequestBodyLogin>(req.into_body()).await;

   match body {
      Ok(RequestBodyLogin { email, password }) => {
         let users = get_users(conn.clone(), email.clone(), String::from("email")).await;

         if users.len() < 1 {
            return create_error("this user not exists", None);
         }

         let password_verified = verify(password, &users[0].password);

         match password_verified {
            Ok(result) => {
               if !result {
                  return create_error("password is not valid", None);
               }

               let now = Utc::now();
               let converted: DateTime<Local> = DateTime::from(now);
               let expires = converted.add(Duration::hours(1));

               let data = json!({ "id": &users[0].id, "expires": format!("{}", expires.format("%Y-%m-%d %H:%M:%S")) });
               let header = json!({});
               let alg = Algorithm::new_hmac(AlgorithmID::HS256, "random123").unwrap();

               let token = encode(&header, &data, &alg);

               match token {
                  Ok(jwt) => {
                     let json = json!({ "id": &users[0].id, "token": jwt });

                     Ok(Response::builder()
                        .status(200)
                        .body(Body::from(json.to_string()))
                        .unwrap())
                  }
                  _ => create_error("", None),
               }
            }
            _ => create_error("", None),
         }
      }
      _ => create_error("", None),
   }
}

pub async fn update_user(
   req: Request<Body>,
   conn: Arc<Mutex<Connection>>,
   user_id: String,
) -> Result<Response<Body>, Infallible> {
   let (head, body) = req.into_parts();

   let body = parse_body::<RequestBodyUpdate>(body).await;
   let result = valid_user(&head.headers).await;

   match result {
      Ok(data) => match data {
         ValidResponse::Id(id) => {
            if id != user_id {
               return create_error("you not have permission for to follow", Option::from(401));
            }
         }
         ValidResponse::Respo(data) => return data,
      },
      _ => return create_error("", None),
   }

   match body {
      Ok(RequestBodyUpdate {
         firstname,
         lastname,
         email,
         password,
      }) => {
         let users = get_users(conn.clone(), user_id, String::from("id")).await;
         if users.len() < 1 {
            return create_error("this user not exists", None);
         }

         if firstname == None && lastname == None && email == None && password == None {
            return create_error("data invalid", None);
         }

         let mut user: CreatedUser = users[0].clone();

         if firstname != None {
            user.firstname = firstname.unwrap();
         }

         if lastname != None {
            user.lastname = lastname.unwrap();
         }

         if email != None {
            user.email = email.unwrap();
         }

         if password != None {
            user.password = hash(password.unwrap(), 8).unwrap();
         }

         let conn = conn.lock().await;
         let data = conn.execute(
            "UPDATE users SET firstname = ?, lastname = ?, email = ?, password = ? WHERE id = ?",
            [
               user.firstname,
               user.lastname,
               user.email,
               user.password,
               user.id,
            ],
         );

         match data {
            Ok(_) => Ok(Response::builder()
               .status(200)
               .body(Body::from(""))
               .unwrap()),
            _ => create_error("error on update user", None),
         }
      }
      _ => create_error("data invalid", None),
   }
}

pub async fn delete_user(
   req: Request<Body>,
   conn: Arc<Mutex<Connection>>,
   user_id: String,
) -> Result<Response<Body>, Infallible> {
   let headers = req.headers();
   let response = valid_user(headers).await;

   match response {
      Ok(data) => match data {
         ValidResponse::Id(id) => {
            if id != user_id {
               return create_error("you not have permission for to follow", Option::from(401));
            }
         }
         ValidResponse::Respo(data) => return data,
      },
      _ => return create_error("", None),
   }

   let conn = conn.lock().await;
   let result = conn.execute("DELETE FROM users WHERE id = ?", [user_id]);

   match result {
      Ok(_) => Ok(Response::builder()
         .status(200)
         .body(Body::from(""))
         .unwrap()),
      _ => create_error("error on delete this user", None),
   }
}
