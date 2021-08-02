use super::tasks::TaskCreatedUserFormated;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreatedUser {
   pub id: String,
   pub firstname: String,
   pub lastname: String,
   pub email: String,
   pub password: String,
}

impl CreatedUser {
   pub fn format(self) -> CreatedUserFormated {
      CreatedUserFormated {
         id: self.id,
         firstname: self.firstname,
         lastname: self.lastname,
         email: self.email,
      }
   }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreatedUserFormated {
   pub id: String,
   pub firstname: String,
   pub lastname: String,
   pub email: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreatedUserComplete {
   pub id: String,
   pub firstname: String,
   pub lastname: String,
   pub email: String,
   pub tasks: Vec<TaskCreatedUserFormated>,
}
