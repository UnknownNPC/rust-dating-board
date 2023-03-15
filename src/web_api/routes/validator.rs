use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use super::add_profile_page::{AddOrEditProfileFormRequest, AddOrEditProfileFormRequestRaw};

type Key = String;
type Code = String;

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct ValidationErrorContext {
    #[serde_as(as = "Vec<(_, _)>")]
    pub data: HashMap<Key, Vec<Code>>,
}

impl ValidationErrorContext {
    pub fn new() -> Self {
        ValidationErrorContext {
            data: HashMap::<Key, Vec<Code>>::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.data.len() == 0
    }

    pub fn if_true_add_error(&mut self, predicate: bool, key: &str, code: &str) -> () {
        if predicate {
            self.add_error(key, code)
        }
    }

    fn add_error(&mut self, key: &str, code: &str) {
        let value = self.data.entry(key.to_string()).or_insert(Vec::new());
        value.push(code.to_string());
    }
}

pub trait Validator<R: Sized> {
    fn validate(&self) -> Result<R, ValidationErrorContext>;

    fn is_empty(&self, f: fn(&Self) -> &String) -> bool {
        f(self).is_empty()
    }

    fn is_not_in_range(&self, f: fn(&Self) -> &String, from: i64, to: i64) -> bool {
        let value = f(self).parse::<i64>();
        if let Ok(data) = value {
            data < from || data > to
        } else {
            true
        }
    }

    fn has_not_length(&self, f: fn(&Self) -> &String, from: i64, to: i64) -> bool {
        let value = f(self);
        value.len() < from.try_into().unwrap() || value.len() > to.try_into().unwrap()
    }
}

impl Validator<AddOrEditProfileFormRequest> for AddOrEditProfileFormRequestRaw {
    fn validate(&self) -> Result<AddOrEditProfileFormRequest, ValidationErrorContext> {
        let mut err_context = ValidationErrorContext::new();

        // name
        err_context.if_true_add_error(self.is_empty(|f| &f.name), "name", "is_empty");
        err_context.if_true_add_error(self.has_not_length(|f| &f.name, 3, 10), "name", "length");

        //height
        err_context.if_true_add_error(self.is_empty(|f| &f.height), "height", "is_empty");
        err_context.if_true_add_error(
            self.is_not_in_range(|f| &f.height, 100, 250),
            "height",
            "range",
        );

        // city
        err_context.if_true_add_error(self.is_empty(|f| &f.city), "city", "is_empty");

        //phone
        err_context.if_true_add_error(
            self.is_empty(|f| &f.phone_number),
            "phone_number",
            "is_empty",
        );
        err_context.if_true_add_error(
            self.has_not_length(|f| &f.phone_number, 9, 9),
            "phone_number",
            "length",
        );

        //description
        err_context.if_true_add_error(self.is_empty(|f| &f.description), "description", "is_empty");
        err_context.if_true_add_error(
            self.has_not_length(|f| &f.description, 10, 600),
            "description",
            "length",
        );

        // captcha
        err_context.if_true_add_error(
            self.is_empty(|f| &f.captcha_token),
            "captcha_token",
            "is_empty",
        );

        if err_context.is_empty() {
            Ok(AddOrEditProfileFormRequest::from_raw(self))
        } else {
            Err(err_context)
        }
    }
}
