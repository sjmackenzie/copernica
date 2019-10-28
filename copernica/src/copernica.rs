extern crate bincode;
extern crate faces;
extern crate router;
extern crate futures;
extern crate content_store;
extern crate log;
extern crate env_logger;

use {
    log::{trace},
    faces::{Udp},
    router::{Router},
    packets::{response},
    futures::executor::{ThreadPool},
    std::thread,
};

fn main() {
    env_logger::init();
    trace!("copernica started");
    let node2 = "127.0.0.1:8072";
    let node3 = "127.0.0.1:8073";

    thread::spawn( move || {
        let mut executor = ThreadPool::new().expect("Failed to create threadpool");
        let mut r = Router::new(executor.clone());
        let f = Udp::new(node3.clone().into(), node2.clone().into());
        r.insert_into_cs(response("hello1".to_string(), "hello".to_string().as_bytes().to_vec()));
        r.add_face(f);
        executor.run(r.run())
    });

    thread::park();

}