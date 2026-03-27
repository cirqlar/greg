use std::fmt::{Debug, Display};

use actix_web::{
    HttpRequest, HttpResponse, Responder, ResponseError, body::BoxBody, cookie::Cookie,
    http::StatusCode,
};
use serde::Serialize;

pub type ApiResponse<T = MessageBody, U = MessageBody> = Result<Success<T>, Failure<U>>;

#[derive(Serialize, Debug)]
pub struct MessageBody {
    message: String,
}

impl Display for MessageBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.message, f)
    }
}

#[derive(Debug)]
pub struct Success<T: Serialize = MessageBody> {
    body: T,
    status: StatusCode,
    cookie: Option<Cookie<'static>>,
}

impl<T: Serialize> Responder for Success<T> {
    type Body = BoxBody;

    fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
        let mut res = HttpResponse::build(self.status);
        if let Some(c) = self.cookie {
            res.cookie(c);
        }

        res.json(self.body)
    }
}

impl<T: Serialize> Success<T> {
    pub fn ok(body: T) -> Self {
        Success {
            body,
            status: StatusCode::OK,
            cookie: None,
        }
    }

    pub fn with_cookie(mut self, cookie: Cookie<'static>) -> Success<T> {
        self.cookie = Some(cookie);

        self
    }
}

impl Success<MessageBody> {
    pub fn ok_message(message: String) -> Self {
        Success {
            body: MessageBody { message },
            status: StatusCode::OK,
            cookie: None,
        }
    }
}

#[derive(Debug)]
pub struct Failure<T: Serialize + Display + Debug = MessageBody> {
    body: T,
    status: StatusCode,
    // Clippy warns that Failure is big and most of that comes from cookie
    // Following suggestion to box it
    cookie: Option<Box<Cookie<'static>>>,
}

impl<T: Serialize + Display + Debug> Display for Failure<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.status.as_u16(), self.body)
    }
}

impl<T: Serialize + Display + Debug> ResponseError for Failure<T> {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let mut res = HttpResponse::build(self.status);
        if let Some(ref c) = self.cookie {
            res.cookie(*c.clone());
        }

        res.json(&self.body)
    }
}

impl<T: Serialize + Display + Debug> Failure<T> {
    pub fn with_cookie(mut self, cookie: Cookie<'static>) -> Failure<T> {
        self.cookie = Some(Box::new(cookie));

        self
    }
}

impl Failure<MessageBody> {
    pub fn server_error_message(message: String) -> Self {
        Failure {
            body: MessageBody { message },
            status: StatusCode::INTERNAL_SERVER_ERROR,
            cookie: None,
        }
    }

    pub fn bad_request_message(message: String) -> Self {
        Failure {
            body: MessageBody { message },
            status: StatusCode::INTERNAL_SERVER_ERROR,
            cookie: None,
        }
    }

    pub fn unauthorized_message(message: String) -> Self {
        Failure {
            body: MessageBody { message },
            status: StatusCode::UNAUTHORIZED,
            cookie: None,
        }
    }

    pub fn server_error(error: impl std::error::Error) -> Self {
        Failure {
            body: MessageBody {
                message: format!("{error}"),
            },
            status: StatusCode::INTERNAL_SERVER_ERROR,
            cookie: None,
        }
    }

    pub fn bad_request(error: impl std::error::Error) -> Self {
        Failure {
            body: MessageBody {
                message: format!("{error}"),
            },
            status: StatusCode::INTERNAL_SERVER_ERROR,
            cookie: None,
        }
    }

    pub fn unauthorized(error: impl std::error::Error) -> Self {
        Failure {
            body: MessageBody {
                message: format!("{error}"),
            },
            status: StatusCode::UNAUTHORIZED,
            cookie: None,
        }
    }
}
