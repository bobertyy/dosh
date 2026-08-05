pub mod model;
pub mod port;
pub mod use_case;

pub struct Greeter;

impl Greeter {
    pub fn say_hello() -> String {
        "Aup, duck!".to_string()
    }
}
