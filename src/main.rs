#[cfg(feature = "server")]
mod server;

#[cfg(feature = "server")]
mod schema;

#[cfg(feature = "server")]
use server::database;

#[cfg(feature = "server")]
use server::MyContext;

#[cfg(feature = "server")]
use server::database::list_penguin_encounters;

#[cfg(feature = "server")]
use server_fn::error::NoCustomError;

mod model;
use model::PenguinEncounter;

use dioxus::prelude::*;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::futures::WebSocket;
use gloo_net::websocket::Message;
use tracing::{debug, error};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
    #[route("/websocket")]
    Websocket {},
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
    #[route("/penguin-encounters")]
    PenguinEncounters {},
}

macro_rules! my_asset {
    ($base:expr,$name:ident,$extension:expr) => {
        concat!("/", $base, env!(stringify!($name)), $extension)
    };
}

const HEADER_SVG: &str = my_asset!("header-", header_svg_HASH, ".svg");
const MAIN_CSS: &str = my_asset!("main-", main_css_HASH, ".css");
const FAVICON: &str = my_asset!("favicon-", favicon_HASH, ".ico");

// For any other platform, we just launch the app
#[cfg(not(feature = "server"))]
fn main() {
    dioxus::launch(App);
}

#[cfg(feature = "server")]
#[tokio::main]
async fn main() {
    server::init(App).await;
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}

#[component]
pub fn Hero() -> Element {
    rsx! {
        div {
            id: "hero",
            img { src: HEADER_SVG, id: "header" }
            div { id: "links",
                a { href: "https://dioxuslabs.com/learn/0.6/", "📚 Learn Dioxus" }
                a { href: "https://dioxuslabs.com/awesome", "🚀 Awesome Dioxus" }
                a { href: "https://github.com/dioxus-community/", "📡 Community Libraries" }
                a { href: "https://github.com/DioxusLabs/sdk", "⚙️ Dioxus Development Kit" }
                a { href: "https://marketplace.visualstudio.com/items?itemName=DioxusLabs.dioxus", "💫 VSCode Extension" }
                a { href: "https://discord.gg/XgGxMSkvUM", "👋 Community Discord" }
            }
        }
    }
}

/// Home page
#[component]
fn Home() -> Element {
    rsx! {
        Hero {}
        Echo {}
    }
}

/// Blog page
#[component]
pub fn Blog(id: i32) -> Element {
    rsx! {
        div {
            id: "blog",

            // Content
            h1 { "This is blog #{id}!" }
            p { "In blog #{id}, we show how the Dioxus router works and how URL parameters can be passed as props to our route components." }

            div {
                class: "alert alert-info",
                "This is a blog post!"
            }

            // Navigation links
            Link {
                to: Route::Blog { id: id - 1 },
                "Previous"
            }
            span { " <---> " }
            Link {
                to: Route::Blog { id: id + 1 },
                "Next"
            }
        }
    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        div {
            id: "navbar",
            Link {
                to: Route::Home {},
                "Home"
            }
            Link {
                to: Route::Websocket {},
                "Websocket"
            }
            Link {
                to: Route::Blog { id: 1 },
                "Blog"
            }
            Link {
                to: Route::PenguinEncounters {},
                "Penguin Encounters"
            }
        }

        Outlet::<Route> {}
    }
}

/// Echo component that demonstrates fullstack server functions.
#[component]
fn Echo() -> Element {
    let mut response = use_signal(String::new);

    rsx! {
        div {
            id: "echo",
            h4 { "ServerFn Echo" }
            input {
                placeholder: "Type here to echo...",
                oninput:  move |event| async move {
                    let data = echo_server(event.value()).await.unwrap();
                    response.set(data);
                },
            }

            if !response().is_empty() {
                p {
                    "Server echoed: "
                    i { "{response}" }
                }
            }
        }
    }
}

fn get_websocket_url() -> String {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let protocol = if location.protocol().unwrap() == "https:" {
        "wss"
    } else {
        "ws"
    };
    let host = location.host().unwrap();
    format!("{protocol}://{host}/echo")
}

/// Echo component that demonstrates fullstack server functions.
#[component]
fn Websocket() -> Element {
    let mut response = use_signal(String::new);

    let tx = use_coroutine(move |mut rx: UnboundedReceiver<String>| async move {
        let url = get_websocket_url();
        debug!("Connecting to websicket at {url}");
        let mut socket = WebSocket::open(&url).unwrap();
        debug!("Connected to websicket.");

        loop {
            match futures::future::select(rx.next(), socket.next()).await {
                futures::future::Either::Left((msg, _)) => {
                    if let Some(msg) = msg {
                        debug!("Sending to socket");
                        socket.send(Message::Text(msg)).await.unwrap();
                    } else {
                        break;
                    }
                }
                futures::future::Either::Right((msg, _)) => match msg {
                    Some(Ok(Message::Text(msg))) => {
                        debug!("Receiving from socket");
                        response.set(msg);
                    }
                    Some(Ok(Message::Bytes(msg))) => {
                        error!("Received binary message: {:?}", msg);
                    }
                    Some(Err(err)) => {
                        error!("Error: {:?}", err);
                        break;
                    }
                    None => {
                        break;
                    }
                },
            }
        }

        debug!("Disconnected from websicket");
    });

    rsx! {
        div {
            id: "echo",
            h4 { "ServerFn Echo" }
            input {
                placeholder: "Type here to echo...",
                oninput:  move |event| async move {
                    tx.send(event.value());
                },
            }

            if !response().is_empty() {
                p {
                    "Server echoed: "
                    i { "{response}" }
                }
            }
        }
    }
}

#[component]
fn NotFound(segments: Vec<String>) -> Element {
    let segments = segments.join(" / ");

    let magic_number = use_resource(magic_number);

    rsx! {
        div {
            id: "not-found",
            h1 { "404 Not Found" }
            p { "The page you are looking for does not exist." }
            p { "You should ask a friendly penguin for help." }
            match &*magic_number.read() {
                Some(Ok(magic_number)) => {
                    rsx! {
                        p { "Magic Number: {magic_number}" }
                    }
                }
                Some(Err(err)) => {
                    rsx! {
                        p { "Error loading your magic number: {err}. Please give up." }
                    }
                }
                None => {
                    rsx! {
                        p { "Loading magic number..." }
                    }
                }
            }
            p { "Segments: {segments}" }
        }
    }
}

#[component]
fn PenguinEncounters() -> Element {
    let mut encounters = use_resource(get_penguin_encounters);
    let mut save_result: Signal<Option<Result<PenguinEncounter, ServerFnError>>> =
        use_signal(|| None);

    rsx! {
        div {
            id: "penguin-encounters",
            h1 { "Penguin Encounters" }
            match &*save_result.read() {
                Some(Ok(encounter)) => {
                    rsx! {
                        div {
                            class: "alert alert-success",
                            "Successfully created penguin encounter: {encounter.name}"
                        }
                    }
                }
                Some(Err(err)) => {
                    rsx! {
                        div {
                            class: "alert alert-danger",
                            "Error creating penguin encounter: {err}"
                        }
                    }
                }
                None => {
                    rsx! {
                        button {
                            onclick: move |_| async move {
                                let result = create_penguin_encounter().await;
                                save_result.set(Some(result));
                                encounters.restart();
                            },
                            "Create Penguin Encounter"
                        }
                    }
                }
            }


            ul {
                for maybe_encounters in &*encounters.read() {
                    match maybe_encounters {
                        Ok(encounters) => {
                            rsx! {
                                for encounter in encounters {
                                    li {
                                        "Name: {encounter.name}, Location: {encounter.location}, Penalty: {encounter.penalty}, Date: {encounter.date_time}"
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            rsx! {
                                li {
                                    "Error loading encounters: {err}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Echo the user input on the server.
#[server(EchoServer)]
async fn echo_server(input: String) -> Result<String, ServerFnError> {
    let FromContext::<MyContext>(context) = extract().await?;
    Ok(context.title.to_string() + ": " + &input.to_uppercase())
}

#[server(MagicNumber)]
async fn magic_number() -> Result<u32, ServerFnError> {
    let FromContext(magic_number) = extract().await?;
    Ok(magic_number)
}

#[server(GetPenguinEncounters)]
async fn get_penguin_encounters() -> Result<Vec<PenguinEncounter>, ServerFnError> {
    let FromContext::<database::DatabasePool>(pool) = extract().await?;

    let mut connection = pool
        .get()
        .await
        .map_err(|err| ServerFnError::<NoCustomError>::ServerError(err.to_string()))?;

    let penguin_encounters = list_penguin_encounters(&mut connection)
        .await
        .map_err(|err| ServerFnError::<NoCustomError>::ServerError(err.to_string()))?;

    Ok(penguin_encounters)
}

#[server(CreatePenguinEncounter)]
async fn create_penguin_encounter() -> Result<PenguinEncounter, ServerFnError> {
    let FromContext::<database::DatabasePool>(pool) = extract().await?;

    let mut connection = pool
        .get()
        .await
        .map_err(|err| ServerFnError::<NoCustomError>::ServerError(err.to_string()))?;

    let penguin_encounter = database::create_penguin_encounter(
        &mut connection,
        "Tux",
        "Antarctica",
        model::PenaltyEnum::PatPenguin,
        chrono::Utc::now().naive_utc(),
    )
    .await
    .map_err(|err| ServerFnError::<NoCustomError>::ServerError(err.to_string()))?;

    Ok(penguin_encounter)
}
