
#![deny(warnings)]

use std::convert::Infallible;
use std::error::Error;

use serde_derive::{ Serialize };
use warp::http::StatusCode;
use warp::{ Filter, Rejection, Reply };

/// Rejections represent cases where a filter should not continue processing
/// the request, but a different filter *could* process it.
#[tokio::main]
async fn main() {

    // let db = DB::open_default("path/for/rocksdb/storage").unwrap();
    // db.put(b"my key", b"my value");
    // match db.get(b"my key") {
    //    Ok(Some(value)) => println!("retrieved value {}", value.to_utf8().unwrap()),
    //    Ok(None) => println!("value not found"),
    //    Err(e) => println!("operational problem encountered: {}", e),
    // }
    // db.delete(b"my key").unwrap();

    // GET /
    let index = warp::path::end()
        .map(|| { 
            warp::reply::json( &ResponseMessage {
                code: StatusCode::OK.as_u16(),
                message: "Index page".to_string()
            })
        });

    // GET /channel
    let channel = warp::path("channel")
        .map(|| { 
            warp::reply::json( &ResponseMessage {
                code: StatusCode::OK.as_u16(),
                message: "Channel page".to_string()
            })
        });

    // GET /example
    let example = warp::path("example")
        .map(|| { 
            warp::reply::json( &ResponseMessage {
                code: StatusCode::OK.as_u16(),
                message: "Example page".to_string()
            })
        });

    let routes = 
        index
        .or(channel)
        .or(example)
        .recover(handle_rejection);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

}

/// An API error serializable to JSON.
#[derive(Serialize)]
struct ResponseMessage {
    code: u16,
    message: String,
}

/// An API error serializable to JSON.
#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

// This function receives a `Rejection` and tries to return a custom
// value, otherwise simply passes the rejection along.
async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Page not found.";
    } else if let Some(e) = err.find::<warp::filters::body::BodyDeserializeError>() {
        // This error happens if the body could not be deserialized correctly
        // We can use the cause to analyze the error and customize the error message
        message = match e.source() {
            Some(cause) => {
                if cause.to_string().contains("denom") {
                    "FIELD_ERROR: denom"
                } else {
                    "BAD_REQUEST"
                }
            }
            None => "BAD_REQUEST",
        };
        code = StatusCode::BAD_REQUEST;
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        // We can handle a specific error, here METHOD_NOT_ALLOWED,
        // and render it however we want
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED";
    } else {
        // We should have expected this... Just log and say its a 500
        eprintln!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION";
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}