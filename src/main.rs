use actix_web::{web, App, HttpServer, HttpResponse};
use actix_web_actors::ws;
use actix::{Actor, StreamHandler, Message};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Clone)]
struct Book {
    id: usize,
    title: String,
    author: String,
    year: u32,
}

struct AppState {
    books: Arc<Mutex<HashMap<usize, Book>>>,
}

struct WebSocketSession {
    state: Arc<Mutex<HashMap<usize, Book>>>,
}

impl Actor for WebSocketSession {
    type Context = ws::WebsocketContext<Self>;
}

#[derive(Message, Deserialize)]
#[rtype(result = "()")]
struct WebSocketMessage {
    action: String,
    id: Option<usize>,
    book: Option<Book>,
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(ws::Message::Text(text)) = msg {
            if let Ok(request) = serde_json::from_str::<WebSocketMessage>(&text) {
                let mut books = self.state.lock().unwrap();
                let response = match request.action.as_str() {
                    "get_books" => {
                        let books_list: Vec<Book> = books.values().cloned().collect();
                        serde_json::to_string(&books_list).unwrap()
                    },
                    "get_book" => {
                        if let Some(id) = request.id {
                            books.get(&id).map_or("Book not found".to_string(), |b| serde_json::to_string(b).unwrap())
                        } else {
                            "Invalid request".to_string()
                        }
                    },
                    "add_book" => {
                        if let Some(mut book) = request.book {
                            let id = books.len() + 1;
                            book.id = id;
                            books.insert(id, book);
                            "Book added".to_string()
                        } else {
                            "Invalid request".to_string()
                        }
                    },
                    "update_book" => {
                        if let (Some(id), Some(book)) = (request.id, request.book) {
                            if books.contains_key(&id) {
                                books.insert(id, book);
                                "Book updated".to_string()
                            } else {
                                "Book not found".to_string()
                            }
                        } else {
                            "Invalid request".to_string()
                        }
                    },
                    "delete_book" => {
                        if let Some(id) = request.id {
                            if books.remove(&id).is_some() {
                                "Book deleted".to_string()
                            } else {
                                "Book not found".to_string()
                            }
                        } else {
                            "Invalid request".to_string()
                        }
                    },
                    _ => "Unknown action".to_string(),
                };
                ctx.text(response);
            } else {
                ctx.text("Invalid JSON request");
            }
        }
    }
}

async fn websocket_route(req: actix_web::HttpRequest, stream: web::Payload, data: web::Data<AppState>) -> Result<HttpResponse, actix_web::Error> {
    let session = WebSocketSession { state: data.books.clone() };
    ws::start(session, &req, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let books = Arc::new(Mutex::new(HashMap::new()));
    let data = web::Data::new(AppState { books });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/ws", web::get().to(websocket_route))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
