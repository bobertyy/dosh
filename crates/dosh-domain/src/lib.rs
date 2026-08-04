pub mod account;
pub mod account_code;

pub struct Greeter;

impl Greeter {
    pub fn say_hello() -> String {
        "Aup, duck!".to_string()
    }
}
