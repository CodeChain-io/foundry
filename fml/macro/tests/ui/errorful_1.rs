extern crate fml_macro;

#[fml_macro::service]
pub trait MyHandle: Service {
    fn fn_1(&self, a: i32, b: String) -> String;
    fn fn_2(a: i32, b: Vec<u8>) -> Box<dyn MyHandle>; // no &self
}

fn main() {

}
