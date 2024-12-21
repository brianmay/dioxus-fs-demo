use dioxus::prelude::*;

#[cfg(feature = "server")]
use std::any::Any;
#[cfg(feature = "server")]
use std::boxed::Box;
#[cfg(feature = "server")]
use std::sync::Arc;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

macro_rules! my_asset {
    ($base:expr,$name:ident,$extension:expr) => {
        concat!("/", $base, env!(stringify!($name)), $extension)
    };
}

const HEADER_SVG: &str = my_asset!("header-", header_svg_HASH, ".svg");
const MAIN_CSS: &str = my_asset!("main-", main_css_HASH, ".css");
const FAVICON: &str = my_asset!("favicon-", favicon_HASH, ".ico");

#[cfg(feature = "server")]
#[derive(Debug, Clone)]
struct MyContext {
    pub title: String,
}

// The entry point for the server
#[cfg(feature = "server")]
#[tokio::main]
async fn main() {
    let context = MyContext {
        title: "Dioxus Context".to_string(),
    };

    // Get the address the server should run on. If the CLI is running, the CLI proxies fullstack into the main address
    // and we use the generated address the CLI gives us
    let address = dioxus_cli_config::fullstack_address_or_localhost();

    let provider_1 = move || Box::new(context.clone()) as Box<dyn Any>;
    let provider_2 = move || Box::new(42u32) as Box<dyn Any>;

    let cfg = ServeConfigBuilder::default()
        .context_providers(Arc::new(vec![Box::new(provider_1), Box::new(provider_2)]));

    // Set up the axum router
    let router = axum::Router::new()
        // You can add a dioxus application to the router with the `serve_dioxus_application` method
        // This will add a fallback route to the router that will serve your component and server functions
        .serve_dioxus_application(cfg, App);

    // Finally, we can launch the server
    let router = router.into_make_service();
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

// For any other platform, we just launch the app
#[cfg(not(feature = "server"))]
fn main() {
    dioxus::launch(App);
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
                to: Route::Blog { id: 1 },
                "Blog"
            }
        }

        Outlet::<Route> {}
    }
}

/// Echo component that demonstrates fullstack server functions.
#[component]
fn Echo() -> Element {
    let mut response = use_signal(|| String::new());

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
