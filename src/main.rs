extern crate console_error_panic_hook;

use log::info;
use reqwasm::http::Request;
use std::panic;
use sycamore::rt::Event;
use wasm_bindgen_futures::spawn_local;

use sycamore::context::{use_context, ContextProvider, ContextProviderProps};
use sycamore::prelude::*;

#[component(MyComponent<G>)]
fn my_component(value: StateHandle<i32>) -> Template<G> {
    template! {
            div() {
                "value: " (value.get())
    }
            form(action="http://localhost:8000/service", method="post") {
                label(for="url") {
                    "Enter URL:"
                    input(type="text", name="url")
                }
                label(for="title") {
                    "Enter Title:"
                    input(type="text", name="title")
                }
                label(for="description") {
                    "Enter description:"
                    input(type="text", name="description")
                }
                input(type="submit", value="submit")
            }
        }
}

#[derive(Clone)]
struct Redact {
    host: Signal<String>,
    port: Signal<String>,
    addr: Signal<String>,
    health: Signal<bool>,
}

impl Redact {
    fn new(host: String, port: String) -> Redact {
        let host = Signal::new(host);
        let port = Signal::new(port);
        let addr = Signal::new(Redact::format_addr(&host.get(), &port.get()));
        let health = Signal::new(false);

        create_effect({
            let addr = addr.clone();
            let health = health.clone();

            move || {
                let health = health.clone();
                let addr = addr.get();

                spawn_local({
                    async move {
                        info!("Sending health check");
                        let health_check_addr = format!("{}/healthz", &addr);
                        let health_response = Request::get(&health_check_addr).send().await;

                        match health_response {
                            Ok(response) => match response.status() {
                                200 => health.set(true),
                                _ => health.set(false),
                            },
                            Err(_) => health.set(false),
                        };
                    }
                });
            }
        });

        Redact {
            host,
            port,
            addr,
            health,
        }
    }

    fn host_signal(&self) -> Signal<String> {
        self.host.clone()
    }

    fn port_signal(&self) -> Signal<String> {
        self.port.clone()
    }

    fn health_signal(&self) -> Signal<bool> {
        self.clone().health
    }

    fn addr_signal(&self) -> Signal<String> {
        self.clone().addr
    }

    fn format_addr(host: &str, port: &str) -> String {
        return format!("http://{}:{}", host, port);
    }

    fn update_addr(&self) {
        info!("updating address...");
        self.addr
            .clone()
            .set(Redact::format_addr(&self.host.get(), &self.port.get()));
    }
}

#[component(RedactView<G>)]
fn redact_view() -> Template<G> {
    let redact = use_context::<Redact>();
    let redact_2 = redact.clone();
    let redact_3 = redact.clone();
    let submit = cloned!((redact) => move |_: Event| {
        redact.update_addr();
    });

    template! {
            label(for="redact-host") {
                "Enter redact hostname:"
                input(bind:value=redact.host_signal(), type="text", name="redact-host")
            }
            label(for="redact-port") {
                "Enter redact port:"
                input(bind:value=redact.port_signal(), type="number", name="redact-port")
            }
            button(on:click=submit) {
                "Submit"
            }
        p {
            div {
                (format!("Redact address: {}", redact_2.addr_signal().get()))
            }
            (if *redact_3.health_signal().get() {
                template! {
                    p {
                        "Valid redact"
                    }

                    div {
                        iframe(src="http://localhost:8080/unsecure/data/.demoapp.name.?edit=true")
                    }
                }

            } else {
                template! {
                    "Invalid redact"
                }
            })
        }
    }
}

fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Debug).unwrap();
    info!("Setting up");
    let redact = Redact::new("localhost".to_string(), "8080".to_string());

    // let increment = cloned!((state) => move |_| state.set(*state.get() + 1));

    sycamore::render(|| {
        template! {
            ContextProvider(ContextProviderProps {
                value: redact,
                children: || template! {
                    RedactView()
                }
            })

        }
    });
}
