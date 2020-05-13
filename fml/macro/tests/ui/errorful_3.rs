extern crate fml_macro;

#[fml_macro::service]
pub trait MyHandle: Service {
    fn fn_1(&self, a: i32, b: String) -> String;
    fn fn_2(&self, a: i32, (a, b): (Vec<u8>, f32)) -> Box<dyn MyHandle>; // pattern in arg
}

fn main() {

}