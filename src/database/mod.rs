use rusqlite::Connection;

pub fn create_connection() -> Connection {
   let conn = Connection::open("./src/database/database.db").unwrap();

   let users_sql = get_sql("users");
   let tasks_sql = get_sql("tasks");

   let _result = conn.execute(users_sql, []);
   let _result = conn.execute(tasks_sql, []);

   return conn;
}

fn get_sql(table_name: &'static str) -> &'static str {
   let mut _query = "";

   match table_name {
      "users" => {
         _query = "CREATE TABLE IF NOT EXISTS users (
            id VARCHAR PRIMARY KEY,
            firstname VARCHAR NOT NULL,
            lastname VARCHAR NOT NULL,
            email VARCHAR NOT NULL,
            password VARCHAR NOT NULL
         )";
      }
      "tasks" => {
         _query = "CREATE TABLE IF NOT EXISTS tasks (
            id VARCHAR PRIMARY KEY,
            name TEXT NOT NULL,
            completed INT NOT NULL,
            user_id VARCHAR NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id)
         )"
      }
      _ => _query = "",
   }

   _query
}
