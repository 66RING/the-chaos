use freya::prelude::*;
use freya::events::Code;

// A chatbot state with chat history and a message input field
pub struct AppState {
    pub chat_history: Vec<String>,
    pub message: String,
    pub counter: i32,
}

// NOTE: argument name should be the same as rsx!() input
#[component]
pub fn history_component(mut app_state: Signal<AppState>) -> Element {
    let onclick = move |_| {
        (*app_state.write()).counter += 1
    };
    let his_str = app_state.read().chat_history.join("\n");
    rsx!(
        rect{
            height: "50%",
            width: "100%",
            main_align: "center",
            cross_align: "center",
            background: "rgb(0, 119, 182)",
            color: "white",
            shadow: "0 4 20 5 rgb(0, 0, 0, 80)",
            label {
                onclick,
                "Chat History\n{his_str}"
            }
        }
    )
}

#[component]
pub fn input_box_component(mut app_state: Signal<AppState>) -> Element {
    let onclick = move |_| {
        let state = &mut (*app_state.write());
        state.chat_history.push(state.message.clone());
        state.message = "".to_string();
    };
    let on_enter = move |e: Event<KeyboardData>| {
        if e.code == Code::Enter {
            let state = &mut (*app_state.write());
            state.chat_history.push(state.message.clone());
            state.message = "".to_string();
        }
    };
    let message = &app_state.read().message;
    rsx!(
        rect {
            height: "50%",
            width: "100%",
            main_align: "center",
            cross_align: "center",
            onkeydown: on_enter,
            label {
                "Input: {message}"
            },
            Button {
                onclick: onclick,
                label { "Submit" }
            }
            Input {
                value: app_state.read().message.clone(),
                onchange: move |e| {
                    (*app_state.write()).message = e
                },
                // mode: InputMode::Shown,
            }
        }
    )
}

pub fn app() -> Element {
    let app_state = use_signal(|| AppState {
        chat_history: vec![],
        message: "".to_string(),
        counter: 0,
    });
    rsx!(
        history_component { app_state },
        input_box_component { app_state },
    )
}


fn main() {
    launch(app);
}
