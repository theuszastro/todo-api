use super::users::{CreatedUser, CreatedUserFormated};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskCreated {
   pub id: String,
   pub name: String,
   pub completed: i32,
   pub user: Option<CreatedUser>,
}

impl TaskCreated {
   pub fn format(self) -> TaskCreatedUserFormated {
      let completed = if self.completed == 0 { false } else { true };

      TaskCreatedUserFormated {
         id: self.id,
         name: self.name,
         completed,
      }
   }

   pub fn format_user(self) -> TaskCreatedFormated {
      let completed = if self.completed == 0 { false } else { true };

      TaskCreatedFormated {
         id: self.id,
         name: self.name,
         completed,
         user: self.user.unwrap().format(),
      }
   }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskCreatedFormated {
   pub id: String,
   pub name: String,
   pub completed: bool,
   pub user: CreatedUserFormated,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskCreatedUserFormated {
   pub id: String,
   pub name: String,
   pub completed: bool,
}
