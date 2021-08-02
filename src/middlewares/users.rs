use super::super::utils::create_error;

use std::convert::Infallible;

use chrono::{DateTime, Local, TimeZone, Utc};
use jsonwebtokens::{error::Error as JWTError, raw, raw::TokenSlices};

use hyper::{Body, HeaderMap, Response};

use serde_json::from_value;

#[derive(Deserialize)]
pub struct JWTData {
   pub id: String,
   pub expires: String,
}

pub enum ValidResponse {
   Id(String),
   Respo(Result<Response<Body>, Infallible>),
}

pub async fn valid_user(headers: &HeaderMap) -> Result<ValidResponse, JWTError> {
   let authorization = headers.get("authorization");

   match authorization {
      Some(data) => match data.to_str() {
         Ok(bearer) => {
            if bearer.len() < 2 {
               return Ok(ValidResponse::Respo(create_error(
                  "token is necessary",
                  None,
               )));
            }

            let splited: Vec<_> = bearer.split(" ").collect();
            if splited.len() < 2 {
               return Ok(ValidResponse::Respo(create_error(
                  "token is necessary",
                  None,
               )));
            }

            let TokenSlices { claims, .. } = raw::split_token(splited[1])?;
            let claims = raw::decode_json_token_slice(claims)?;

            let jwtdata = from_value::<JWTData>(claims).unwrap();

            let utc = Utc::now();
            let timezone = utc.timezone();
            let converted: DateTime<Local> = DateTime::from(utc);
            let expires = jwtdata.expires.as_str();

            let diff = converted.signed_duration_since(
               timezone
                  .datetime_from_str(expires, "%Y-%m-%d %H:%M:%S")
                  .unwrap(),
            );

            if diff.num_days() >= 1 {
               return Err(JWTError::TokenExpiredAt(13));
            }

            Ok(ValidResponse::Id(jwtdata.id))
         }
         _ => Ok(ValidResponse::Respo(create_error("", None))),
      },
      _ => {
         return Ok(ValidResponse::Respo(create_error(
            "token is necessary",
            None,
         )))
      }
   }
}
