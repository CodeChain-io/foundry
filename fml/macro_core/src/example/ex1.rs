pub trait MyHandle: Service {
    fn fn_1(&self, a: i32, b: String) -> String;
    fn fn_2(&self, a: i32, b: Vec<u8>) -> Box<dyn MyHandle>;
}