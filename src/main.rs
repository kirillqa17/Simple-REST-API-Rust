use actix_web::{web, App, HttpServer, HttpResponse};
use actix_web_actors::ws;
use actix::{Actor, StreamHandler, Message};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize)]
enum Action {
    GetBooks,
    GetBook,
    AddBook,
    UpdateBook,
    DeleteBook,
}

enum WebSocketError {
    InvalidRequest,
    BookNotFound,
    InvalidJson,
}

impl std::fmt::Display for WebSocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebSocketError::InvalidRequest => write!(f, "Invalid request"),
            WebSocketError::BookNotFound => write!(f, "Book not found"),
            WebSocketError::InvalidJson => write!(f, "Invalid JSON request"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct Book {
    id: Option<usize>, // Option - чтобы не происходила десериализация поля id для запроса AddBook
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

impl WebSocketSession {
    fn handle_get_books(&self, books: &HashMap<usize, Book>) -> Result<String, WebSocketError> {
        let books_list: Vec<Book> = books.values().cloned().collect();
        serde_json::to_string(&books_list).map_err(|_| WebSocketError::InvalidJson)
    }

    fn handle_get_book(&self, books: &HashMap<usize, Book>, id: usize) -> Result<String, WebSocketError> {
        books.get(&id)
            .map(|b| serde_json::to_string(b).map_err(|_| WebSocketError::InvalidJson))
            .unwrap_or(Err(WebSocketError::BookNotFound))
    }

    fn handle_add_book(&self, books: &mut HashMap<usize, Book>, mut book: Book) -> Result<String, WebSocketError> {
        let id = books.keys().max().map_or(1, |max_id| max_id + 1);
        book.id = Some(id);
        books.insert(id, book);
        Ok("Book added".to_string())
    }

    fn handle_update_book(&self, books: &mut HashMap<usize, Book>, id: usize, mut book: Book) -> Result<String, WebSocketError> {
        if let std::collections::hash_map::Entry::Occupied(mut entry) = books.entry(id) {
            book.id = Some(id);
            entry.insert(book);
            Ok("Book updated".to_string())
        } else {
            Err(WebSocketError::BookNotFound)
        }
    }

    fn handle_delete_book(&self, books: &mut HashMap<usize, Book>, id: usize) -> Result<String, WebSocketError> {
        if books.remove(&id).is_some() {
            Ok("Book deleted".to_string())
        } else {
            Err(WebSocketError::BookNotFound)
        }
    }
}

impl Actor for WebSocketSession {
    type Context = ws::WebsocketContext<Self>;
}

#[derive(Message, Deserialize)]
#[rtype(result = "()")]
struct WebSocketMessage {
    id: Option<usize>, // Option - для запроса GetBooks
    book: Option<Book>, // Option - для запросов GetBook, GetBooks, DeleteBook
    action: Action,
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(ws::Message::Text(text)) = msg {
            println!("Received message: {}", text);
            let response = match serde_json::from_str::<WebSocketMessage>(&text) {
                Ok(request) => {
                    let mut books = self.state.lock().unwrap();
                    match request.action {
                        Action::GetBooks => self.handle_get_books(&books),
                        Action::GetBook => {
                            if let Some(id) = request.id {
                                self.handle_get_book(&books, id)
                            } else {
                                Err(WebSocketError::InvalidRequest)
                            }
                        },
                        Action::AddBook => {
                            if let Some(book) = request.book {
                                self.handle_add_book(&mut books, book)
                            } else {
                                Err(WebSocketError::InvalidRequest)
                            }
                        },
                        Action::UpdateBook => {
                            if let (Some(id), Some(book)) = (request.id, request.book) {
                                self.handle_update_book(&mut books, id, book)
                            } else {
                                Err(WebSocketError::InvalidRequest)
                            }
                        },
                        Action::DeleteBook => {
                            if let Some(id) = request.id {
                                self.handle_delete_book(&mut books, id)
                            } else {
                                Err(WebSocketError::InvalidRequest)
                            }
                        },
                    }
                },
                Err(_) => Err(WebSocketError::InvalidJson),
            };

            match response {
                Ok(msg) => ctx.text(msg),
                Err(err) => ctx.text(err.to_string()),
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
