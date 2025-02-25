### Task 2: Create a Simple REST API

Description: Develop a small WebSocket API in Rust using a lightweight web framework like Actix-web or Rocket. The API should provide basic CRUD (Create, Read, Update, Delete) operations for managing a list of books, where each book has a title, author, and year of publication.

Requirements:
	- Implement a server that handles WebSocket connections and accepts requests in JSON format.
	- Supported commands:
    	{"action": "get_books"} – Returns a list of all books.
    	{"action": "get_book", "id": "<book_id>"} – Returns a specific book by its ID.
    	{"action": "add_book", "book": {"title": "<title>", "author": "<author>", "year": "<year>"}} – Adds a new book.
    	{"action": "update_book", "id": "<book_id>", "book": {"title": "<title>", "author": "<author>", "year": "<year>"}} – Updates an existing book by ID.
    	{"action": "delete_book", "id": "<book_id>"} – Deletes a book by ID.
	- Store the books in an in-memory data structure (like a Vec or HashMap).
	- Handle common errors, such as non-existent book IDs or invalid request formats.


Learning Objectives:
- Get hands-on experience with Rust web frameworks.
- Learn about concurrency and asynchronous programming in Rust.
- Understand how to use Rust's data structures and how ownership works in the context of web development.


Подсматривал сюда:
https://github.com/wpcodevo/simple-api-actix-web


### Тестирование

Лично я тестировал с помощью wscat.
Установите wscat через npm:
``` bash
npm install -g wscat
```
Тестирование:
Запустите сервер:
``` bash
cargo run
```
Подключитесь к WebSocket серверу с помощью команды:
```bash
wscat -c ws://127.0.0.1:8080/ws
```
После подключения отправьте JSON-сообщение:
```json
{"action": "AddBook", "book": {"title": "1984", "author": "George Orwell", "year": 1949}}
```
```json
{"action": "GetBooks"}
```
