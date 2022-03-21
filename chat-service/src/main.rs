// Importing using macro_use so that all the rocket macros are available globally
#[macro_use] extern crate rocket;

use rocket::fs::{relative, FileServer};
use rocket::tokio::sync::broadcast::{channel, Sender, error::RecvError};
use rocket::serde::{Serialize, Deserialize};
use rocket::form::Form;
use rocket::{State, Shutdown};
use rocket::response::stream::{EventStream, Event};
use rocket::tokio::select;

// Message struct is deriving few traits.
// Debug: so we can print out in debug format
// Clone: so we can duplicate messages
// FromForm: so we can take the input into message from a form
// Serialize: so we can serialize the message elements
// Deserialize: so we can deserialize the message elements
#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
// Serialization and deserialization will have via rocket::serde crate
#[serde(crate = "rocket::serde")]
struct Message {
    #[field(validate = len(..30))]
    pub room: String,
    #[field(validate = len(..20))]
    pub username: String,
    pub message: String,
}

#[post("/message", data = "<form>")]
fn post(form: Form<Message>, queue: &State<Sender<Message>>) {
    let _res = queue.send(form.into_inner()); 
}

// get request to the events path and Return type is an infinite stream of server sent events
// Server sent events allow client to open a long live connection to server and then the server
// can send the data to the clients whenever it wants. It is similar to websocket except that
// this only works in a single direction.
// Server side events are produced asynchronously.
// The function takes two argument: queue which is Server state and Shutdown (when instance is shutdown)
#[get("/events")]
async fn events(queue: &State<Sender<Message>>, mut end: Shutdown) -> EventStream![] {
    
    let mut rx = queue.subscribe(); // New receiver created

    // The below is a generator syntax to yield an infinite series of server sent event.
    // Select macro Waits on multiple concurrent branches, returning when the first branch completes, 
    // cancelling the remaining branches.

    EventStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break, // when server shutdown is triggered
            };

            yield Event::json(&msg);
        }
    }  
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(channel::<Message>(1024).0)
        .mount("/", rocket::routes![post, events])
        .mount("/", FileServer::from(relative!("static")))
}