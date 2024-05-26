use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use async_trait::async_trait;
use lazy_static::lazy_static;
use tokio::runtime::Runtime;
use tokio::task;

use autocxx::prelude::*;
use const_format::concatcp;
use cxx::{ExternType, type_id};
use extern_fn_generator::generate_extern_fns; // use all the main autocxx functions

const NUMBER_OF_EXTERN_C_FN: u32 = 100;
const NUMBER_OF_EXTERN_C_FN_STR: &str = concatcp!(NUMBER_OF_EXTERN_C_FN, "");

include_cpp! {
    #include "registration.h"
    safety!(unsafe) // see details of unsafety policies described in the 'safety' section of the book
    generate!("register_message_handler")
}

#[cxx::bridge]
mod bridge {
    unsafe extern "C++" {
        include!("registration.h");

        type HandlerFn = crate::CallbackFnPtr;

        // Register the handler function
        pub fn register_message_handler(handler: HandlerFn);
    }
}

// Wrapper around the function pointer type
#[repr(transparent)]
pub struct CallbackFnPtr(
    pub  extern "C" fn(
        param: u32,
    ),
);

unsafe impl ExternType for CallbackFnPtr {
    type Id = type_id!("HandlerFn");
    type Kind = cxx::kind::Trivial;
}

#[async_trait]
trait UListener: Send + Sync {
    async fn on_msg(&self, param: u32);
}

lazy_static! {
    static ref LISTENER_REGISTRY: Mutex<HashMap<usize, Arc<dyn UListener>>> = Mutex::new(HashMap::new());
}

static LISTENER_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

include!(concat!(env!("OUT_DIR"), "/generated_macro_invocation.rs"));

// Generate a fixed number of extern "C" functions
// generate_extern_fns!(100);

struct UStatus;

#[async_trait]
pub trait UTransport: Send + Sync {
    async fn register_listener(
        &self,
        listener: Arc<dyn UListener>,
    ) -> Result<(), UStatus>;
}

struct MyTransport;

#[async_trait]
impl UTransport for MyTransport {
    async fn register_listener(&self, listener: Arc<dyn UListener>) -> Result<(), UStatus> {
        let listener_id = LISTENER_ID_COUNTER.fetch_add(1, Ordering::SeqCst) as usize;
        LISTENER_REGISTRY.lock().unwrap().insert(listener_id, listener);

        // Register the appropriate extern "C" function with the C++ library
        let extern_fn = get_extern_fn(listener_id);
        bridge::register_message_handler(CallbackFnPtr(extern_fn));

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