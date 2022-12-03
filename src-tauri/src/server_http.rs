

use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

use crate::START_TIME;
use crate::TIMER_TOTAL_S;
use chrono::{Local}; // TimeZone, NaiveDateTime

async fn process_request(_req: Request<Body>) -> Result<Response<Body>, Infallible> {

	// Get the current value of timer_total_s
	// let current_timer_total_s = TIMER_TOTAL_S.lock().unwrap().clone().to_string();
	// let time_now = Local::now();
	// let time_start = START_TIME.lock().unwrap().clone();
	// println!("time_now  : {:?}", time_now);
	// println!("time_start: {:?}", time_start);
	// let time_since_start_time_in_seconds : String = (time_now - time_start).num_seconds().to_string();
	// let time_since_start_time_in_seconds : String = std::cmp::max(0, (time_start - time_now).num_seconds()).to_string();
    // Ok(Response::new(time_since_start_time_in_seconds.into()))
    // Ok(Response::new("0.87".into()))

    let time_start: chrono::DateTime<Local> = *START_TIME.lock().unwrap();
    let time_since_start_time_in_seconds: i64 = (time_start - Local::now()).num_seconds();
    let a = time_since_start_time_in_seconds as f64 / *TIMER_TOTAL_S.lock().unwrap() as f64;
    let v = f64::max(0.0 as f64, a);
    Ok(Response::new(v.to_string().into()))

}

#[tokio::main]
pub async fn start_httpserver() {

	println!("Starting HTTP server...");

    // We'll bind to 127.0.0.1:3080
    let port : u16        = std::env::var("PORT").unwrap_or("3080".to_string()).parse().unwrap();
    let addr : SocketAddr = SocketAddr::from(([127, 0, 0, 1], port));

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let make_svc = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(process_request))
    });

    let server = Server::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
