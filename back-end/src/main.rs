
#![deny(warnings)]

use std::convert::Infallible;
use std::error::Error;

use serde_derive::{ Serialize };
use warp::http::StatusCode;
use warp::{ Filter, Rejection, Reply };
use rocksdb::{ DB,  };

#[tokio::main]
async fn main() {

    
    // GET /
    let index = warp::path::end()
    .map( || {

            let path = "database/rocksdb";
            let db = DB::open_default(path).unwrap();
            let mut string = String::new();

            db.put(b"data", b"Welcome Matias Suez!").unwrap();


            match db.get(b"data") {
                Ok(Some(value)) => string = String::from_utf8(value).unwrap(),
                Ok(None) => println!("value not found"),
                Err(e) => println!("operational problem encountered: {}", e),
            }

            warp::reply::json( &ResponseMessage {
                code: StatusCode::OK.as_u16(),
                message: format!("Index page: {}", &string)//"Index page".to_string()
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

#[derive(Serialize)]
struct ResponseMessage {
    code: u16,
    message: String,
}

#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Page not found.";
    } else if let Some(e) = err.find::<warp::filters::body::BodyDeserializeError>() {

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
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED";
    } else {
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