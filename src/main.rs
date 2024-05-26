use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use async_trait::async_trait;
use extern_fn_generator::{generate_extern_fn_simple, generate_extern_fns};
use extern_fn_generator::{generate_struct, generate_extern_fn, generate_struct_impl};
use lazy_static::lazy_static;
use tokio::runtime::Runtime;
use tokio::task;

#[async_trait]
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
#[async_trait]
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

lazy_static! {
    static ref LISTENER_REGISTRY: Mutex<HashMap<usize, Arc<dyn UListener>>> = Mutex::new(HashMap::new());
}

static LISTENER_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

// Generate a fixed number of extern "C" functions
generate_extern_fns!(100);

struct UStatus;

#[async_trait]
pub trait UTransport: Send + Sync {
    async fn register_listener(
        &self,
        listener: Arc<dyn UListener>,
    ) -> Result<(), UStatus>;
}

fn register_message_handler(handler: extern "C" fn(u32)) {
    handler(42);
}

struct MyTransport;

#[async_trait]
impl UTransport for MyTransport {
    async fn register_listener(&self, listener: Arc<dyn UListener>) -> Result<(), UStatus> {
        let listener_id = LISTENER_ID_COUNTER.fetch_add(1, Ordering::SeqCst) as usize;
        LISTENER_REGISTRY.lock().unwrap().insert(listener_id, listener);

        // Register the appropriate extern "C" function with the C++ library
        let extern_fn = match listener_id {
            0 => extern_on_msg_wrapper_0,
            1 => extern_on_msg_wrapper_1,
            2 => extern_on_msg_wrapper_2,
            3 => extern_on_msg_wrapper_3,
            4 => extern_on_msg_wrapper_4,
            // Add more cases as needed up to the number generated by the macro
            _ => panic!("Listener ID out of range"),
        };
        register_message_handler(extern_fn);

        Ok(()) // Ensure the function returns a Result<(), UStatus>
    }
}

struct FooListener {
    foo: u32
}

impl FooListener {
    pub fn new(foo: u32) -> Self {
        FooListener {
            foo
        }
    }
}

#[async_trait]
impl UListener for FooListener {
    async fn on_msg(&self, param: u32) {
        println!("the foo payload: {}", self.foo + param);
    }
}

struct BarListener {
    bar: u32
}

impl BarListener {
    pub fn new(bar: u32) -> Self {
       BarListener {
            bar
        }
    }
}

#[async_trait]
impl UListener for BarListener {
    async fn on_msg(&self, param: u32) {
        println!("the bar payload: {}", self.bar + param);
    }
}

#[tokio::main]
async fn main() {
    let my_transport = MyTransport;

    println!("before 1");
    let foo_listener_1: Arc<dyn UListener> = Arc::new(FooListener::new(100));
    my_transport.register_listener(foo_listener_1).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("after 1");

    println!("before 2");
    let foo_listener_2: Arc<dyn UListener> = Arc::new(FooListener::new(200));
    my_transport.register_listener(foo_listener_2).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("after 2");

    println!("before 3");
    let bar_listener_1: Arc<dyn UListener> = Arc::new(BarListener::new(300));
    my_transport.register_listener(bar_listener_1).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("after 3");
}