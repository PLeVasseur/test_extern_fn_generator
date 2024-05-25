use extern_fn_generator::generate_extern_fn_simple;
use extern_fn_generator::{generate_struct, generate_extern_fn, generate_struct_impl};
use tokio::runtime::Runtime;

#[async_trait::async_trait]
trait UListener: Send + Sync {
    async fn on_msg(&self, param: u32);
}

#[generate_struct("a")]
struct MyListener {}


#[generate_struct_impl("a")]
impl MyListener {
    pub fn new() -> Self {
        MyListener {}
    }
}

#[generate_extern_fn("a")]
#[async_trait::async_trait]
impl UListener for MyListener {
    // how can I make it so that I am able to create an
    // extern "C" fn out of this trait function impl?
    // notes:
    //  I cannot modify anything about the UListener trait
    async fn on_msg(&self, param: u32) {
        println!("the payload: {param}");
    }
}

#[generate_extern_fn_simple]
fn my_callback(param: u32) {
    println!("Called with param: {}", param);
}

fn main() {
    // You can call the generated extern function from Rust (just for demonstration)
    extern_my_callback(42);
}