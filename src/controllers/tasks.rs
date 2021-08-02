use super::super::middlewares::users::{valid_user, ValidResponse};
use super::super::utils::{create_error, get_users, parse_body, valid_json};
use super::super::views::tasks::{TaskCreated, TaskCreatedFormated};
use super::super::views::users::CreatedUser;

use std::convert::{From, Infallible};
use std::sync::Arc;

use futures::lock::Mutex;

use rusqlite::Connection;

use uuid::Uuid;

use hyper::{Body, Request, Response};

#[derive(Serialize, Deserialize, Debug)]
struct RequestBodyCreate {
   name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct RequestBodyUpdate {
   name: Option<String>,
   completed: Option<bool>,
}

pub async fn list_tasks(
   req: Request<Body>,
   conn: Arc<Mutex<Connection>>,
) -> Result<Response<Body>, Infallible> {
   let headers = req.headers();
   let mut _user_id = String::from("");

   let result = valid_user(headers).await;

   match result {
      Ok(data) => match data {
         ValidResponse::Id(id) => {
            let users = get_users(conn.clone(), id.clone(), String::from("id")).await;

            if users.len() < 1 {
               return create_error("this user not exists", None);
            }

            _user_id = id;
         }
         ValidResponse::Respo(data) => return data,
      },
      _ => return create_error("", None),
   }

   let conn = conn.lock().await;

   let mut query = conn.prepare("SELECT *, users.id as userId FROM tasks LEFT OUTER JOIN users ON users.id = tasks.user_id WHERE users.id = ?").unwrap();
   let data = query
      .query_map([_user_id], |row| {
         Ok(TaskCreated {
            id: row.get(0)?,
            name: row.get(1)?,
            completed: row.get(2)?,
            user: Option::from(CreatedUser {
               id: row.get(4)?,
               firstname: row.get(5)?,
               lastname: row.get(6)?,
               email: row.get(7)?,
               password: row.get(8)?,
            }),
         })
      })
      .unwrap();

   let mut tasks: Vec<TaskCreatedFormated> = vec![];

   for task in data {
      tasks.push(task.unwrap().format_user());
   }

   let json = serde_json::to_string(&tasks);

   valid_json(json)
}

pub async fn create_task(
   req: Request<Body>,
   conn: Arc<Mutex<Connection>>,
) -> Result<Response<Body>, Infallible> {
   let (head, body) = req.into_parts();
   let mut _user_id = String::from("");

   let body = parse_body::<RequestBodyCreate>(body).await;
   let result = valid_user(&head.headers).await;

   match result {
      Ok(data) => match data {
         ValidResponse::Id(id) => {
            let users = get_users(conn.clone(), id.clone(), String::from("id")).await;

            if users.len() < 1 {
               return create_error("this user not exists", None);
            }

            _user_id = id;
         }
         ValidResponse::Respo(data) => return data,
      },
      _ => return create_error("", None),
   }

   match body {
      Ok(RequestBodyCreate { name }) => {
         let conn = conn.lock().await;

         let query = conn.execute(
            "INSERT INTO tasks VALUES(?, ?, ?, ?)",
            [
               Uuid::new_v4().to_string(),
               name,
               String::from("0"),
               _user_id,
            ],
         );

         match query {
            Ok(_) => Ok(Response::builder()
               .status(201)
               .body(Body::from(""))
               .unwrap()),
            _ => create_error("not is possible create this task", None),
         }
      }
      _ => create_error("data invalid", None),
   }
}

pub async fn update_task(
   req: Request<Body>,
   conn: Arc<Mutex<Connection>>,
   task_id: String,
) -> Result<Response<Body>, Infallible> {
   let (head, body) = req.into_parts();

   let result = valid_user(&head.headers).await;
   let body = parse_body::<RequestBodyUpdate>(body).await;

   match result {
      Ok(data) => match data {
         ValidResponse::Id(id) => {
            let users = get_users(conn.clone(), id.clone(), String::from("id")).await;

            if users.len() < 1 {
               return create_error("this user not exists", None);
            }
         }
         ValidResponse::Respo(data) => return data,
      },
      _ => return create_error("", None),
   }

   match body {
      Ok(RequestBodyUpdate { name, completed }) => {
         let conn = conn.lock().await;

         let mut query = conn
            .prepare("SELECT * FROM tasks WHERE tasks.id = ?")
            .unwrap();
         let data = query
            .query_map([task_id], |row| {
               Ok(TaskCreated {
                  id: row.get(0)?,
                  name: row.get(1)?,
                  completed: row.get(2)?,
                  user: None,
               })
            })
            .unwrap();

         let mut tasks: Vec<TaskCreated> = vec![];
         for task in data {
            tasks.push(task.unwrap());
         }

         if tasks.len() < 1 {
            return create_error("this task not exists", None);
         }

         let mut task = tasks[0].clone();

         if name == None && completed == None {
            return create_error("data invalid", None);
         }

         if name != None {
            task.name = name.unwrap();
         }

         if completed != None {
            let completed_formated = if completed.unwrap() { 1 } else { 0 };

            task.completed = completed_formated;
         }

         let query = conn.execute(
            "UPDATE tasks SET name = ?, completed = ? WHERE tasks.id = ?",
            [task.name, task.completed.to_string(), task.id],
         );

         match query {
            Ok(_) => Ok(Response::builder()
               .status(200)
               .body(Body::from(""))
               .unwrap()),
            _ => create_error("error on update this task", None),
         }
      }
      _ => create_error("data invalid", None),
   }
}

pub async fn delete_task(
   req: Request<Body>,
   conn: Arc<Mutex<Connection>>,
   task_id: String,
) -> Result<Response<Body>, Infallible> {
   let headers = req.headers();

   let result = valid_user(headers).await;

   match result {
      Ok(data) => match data {
         ValidResponse::Id(id) => {
            let users = get_users(conn.clone(), id.clone(), String::from("id")).await;

            if users.len() < 1 {
               return create_error("this user not exists", None);
            }
         }
         ValidResponse::Respo(data) => return data,
      },
      _ => return create_error("", None),
   }

   let conn = conn.lock().await;

   let mut query = conn
      .prepare("SELECT * FROM tasks WHERE tasks.id = ?")
      .unwrap();
   let data = query
      .query_map([task_id.clone()], |row| {
         Ok(TaskCreated {
            id: row.get(0)?,
            name: row.get(1)?,
            completed: row.get(2)?,
            user: None,
         })
      })
      .unwrap();

   let mut tasks: Vec<TaskCreated> = vec![];
   for task in data {
      tasks.push(task.unwrap());
   }

   if tasks.len() < 1 {
      return create_error("this task not exists", None);
   }

   let query = conn.execute("DELETE FROM tasks WHERE id = ?", [task_id]);

   match query {
      Ok(_) => Ok(Response::builder()
         .status(200)
         .body(Body::from(""))
         .unwrap()),
      _ => create_error("error on delete this task", None),
   }
}
