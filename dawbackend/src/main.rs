use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        TypedHeader,
    },
    response::IntoResponse,
    routing::get,
    Router, http::StatusCode,
};
use dawlib::{MidiKey, InstrumentPayloadDto};
use headers::Header;
use tracing::trace;

use std::{borrow::Cow, time::Duration};
use std::ops::ControlFlow;
use std::{net::SocketAddr, path::PathBuf};
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use wavegen::{wf, sine, dc_bias, sawtooth, square, Waveform, WaveformIterator, Precision};

//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;
use axum::extract::ws::CloseFrame;

//allows to split the websocket stream into separate TX and RX branches
use futures::{sink::SinkExt, stream::{StreamExt, SplitSink}};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_websockets=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    // build our application with some routes
    let app = Router::new()
        .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
        .route("/ws", get(ws_handler))
        // logging so we can see whats going on
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}


/// The handler for the HTTP request (this gets called when the HTTP GET lands at the start
/// of websocket negotiation). After this completes, the actual switching from HTTP to
/// websocket protocol will occur.
/// This is the last point where we can extract TCP/IP metadata such as IP address of the client
/// as well as things from HTTP headers such as user-agent of the browser etc.
async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    println!("`{user_agent}` at {addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(socket: WebSocket, who: SocketAddr) {
    let (mut sender, mut receiver) = socket.split();

    loop {
        // receive single message from a client (we can either receive or send with socket).
        // this will likely be the Pong for our Ping or a hello message from client.
        // waiting for message from a client will block this task, but will not block other client's
        // connections.
        if let Some(msg) = receiver.next().await {
            println!("There is a mesage");
            if let Ok(msg) = msg {
                if process_message(msg, who, &mut sender).await.is_break() {
                    return;
                }
            } else {
                println!("client {who} abruptly disconnected");
                return;
            }
        } else {
            println!("Nothing");
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        // By splitting socket we can send and receive at the same time. In this example we will send
        // unsolicited messages to client based on some sort of server's internal event (i.e .timer).


        // Spawn a task that will push several messages to the client (does not matter what client does)
        
        // let chunk_length = 4;
        // let waveform = wf!(f32, 44100., sine!(MidiKey::G1.frequency()), sawtooth!(MidiKey::C4.frequency()), sawtooth!(MidiKey::E2.frequency()));
        // let mut iter = waveform.iter().take(44100 * chunk_length * 20).collect::<Vec<f32>>();
        // let n_msg = 20;
        // for i in 0..n_msg {
        //     // In case of any websocket error, we exit.
        //     let samples = iter.drain(0..44100 * chunk_length)
        //     .collect::<Vec<f32>>();

        //     let mut bytes = samples.into_iter()
        //     .flat_map(|it| it.to_le_bytes())
        //     .collect::<Vec<u8>>();
        //     bytes.append(&mut bytes.clone());

            
        //     if sender
        //         .send(Message::Binary(bytes))
        //         .await
        //         .is_err()
        //     {
        //         return;
        //     }
        // }
    }
}


struct MusicBox;

impl MusicBox {
    fn generate(payload: &InstrumentPayloadDto) -> Vec<Vec<f32>> {
        let mut sawtooth = vec![];
        let mut sine = vec![];
        let mut square = vec![];
        let end = payload.instruments.iter().map(|instrument| {
            instrument.notes.keys().max().copied().unwrap_or(0)
        }).max().unwrap_or(0);

        if let Some(sawtooth_instrument) = payload.instruments.iter().find(|instrument| instrument.name == "sawtooth")  {
            for i in 0..=end {
                if let Some(notes) = sawtooth_instrument.notes.get(&i) {
                    let mut waveforms = notes.iter()
                    .map(|key| wf!(f32, 44100., sawtooth!(key.frequency())))
                    .collect::<Vec<Waveform<f32>>>();

                    let mut waveforms_iterators = waveforms.iter()
                    .map(|waveform| waveform.iter())
                    .collect::<Vec<WaveformIterator<f32, _>>>();

                    let divisor = waveforms.len() as f32;
                    let mut second = (0..44100).map(|_| {
                        waveforms_iterators.iter_mut()
                        .map(|waveform| waveform.next().unwrap() / divisor)
                        .sum()
                    }).collect::<Vec<f32>>();
 

                    second.append(&mut second.clone());
                    sawtooth.push(Some(second));
                } else {
                    sawtooth.push(None);
                }
            }
        } else {
            for _ in 0..=end {
                sawtooth.push(None);
            }
        }

        if let Some(sawtooth_instrument) = payload.instruments.iter().find(|instrument| instrument.name == "sine")  {
            for i in 0..=end {
                if let Some(notes) = sawtooth_instrument.notes.get(&i) {
                    let mut waveforms = notes.iter()
                    .map(|key| wf!(f32, 44100., sine!(key.frequency())))
                    .collect::<Vec<Waveform<f32>>>();

                    let mut waveforms_iterators = waveforms.iter()
                    .map(|waveform| waveform.iter())
                    .collect::<Vec<WaveformIterator<f32, _>>>();

                    let divisor = waveforms.len() as f32;
                    let mut second = (0..44100).map(|_| {
                        waveforms_iterators.iter_mut()
                        .map(|waveform| waveform.next().unwrap() / divisor)
                        .sum()
                    }).collect::<Vec<f32>>();
 

                    second.append(&mut second.clone());
                    sine.push(Some(second));
                } else {
                    sine.push(None);
                }
            } 
        } else {
            for _ in 0..=end {
                sine.push(None);
            }
        }

        if let Some(sawtooth_instrument) = payload.instruments.iter().find(|instrument| instrument.name == "square")  {
            for i in 0..=end {
                if let Some(notes) = sawtooth_instrument.notes.get(&i) {
                    let mut waveforms = notes.iter()
                    .map(|key| wf!(f32, 44100., square!(key.frequency())))
                    .collect::<Vec<Waveform<f32>>>();

                    let mut waveforms_iterators = waveforms.iter()
                    .map(|waveform| waveform.iter())
                    .collect::<Vec<WaveformIterator<f32, _>>>();

                    let divisor = waveforms.len() as f32;
                    let mut second = (0..44100).map(|_| {
                        waveforms_iterators.iter_mut()
                        .map(|waveform| waveform.next().unwrap() / divisor)
                        .sum()
                    }).collect::<Vec<f32>>();
 

                    second.append(&mut second.clone());
                    square.push(Some(second));
                } else {
                    square.push(None);
                }
            }
        } else {
            for _ in 0..=end {
                square.push(None);
            }
        }
        
        let zipped_fragments = sawtooth.into_iter().zip(sine.into_iter()).zip(square.into_iter())
        .map(|((saw, sine), square)| {
            vec![saw, sine, square]
        }).collect::<Vec<_>>();

        zipped_fragments.into_iter().map(|fragment|{
            let mut fragment_chunks = fragment.into_iter()
            .filter_map(|fragment| fragment)
            .collect::<Vec<_>>();
            if fragment_chunks.is_empty() {
                vec![0.0f32; 44100]
            } else {
                let count = fragment_chunks.len() as f32;
                let mut result = fragment_chunks.pop().unwrap();
                for item in result.iter_mut() {
                    *item = *item / count;
                }
                for other_chunk in fragment_chunks {
                    for (index, value) in other_chunk.iter().enumerate() {
                        result[index] += *value / count;
                    }
                }
                result
            }
        }).collect::<Vec<Vec<f32>>>()
    }
}

/// helper to print contents of messages to stdout. Has special treatment for Close.
async fn process_message(msg: Message, who: SocketAddr, sender: &mut SplitSink<WebSocket, Message>) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            println!(">>> {} sent str: {:?}", who, t);
            if let Ok(payload) = serde_json::from_str::<InstrumentPayloadDto>(&t) {
                let chunks = MusicBox::generate(&payload);

                for (index, chunk) in chunks.into_iter().enumerate() {
                    //     // In case of any websocket error, we exit.
                    //     let samples = iter.drain(0..44100 * chunk_length)
                    //     .collect::<Vec<f32>>();
            
                    println!("Sent chunk {index}");
                    let bytes = chunk.into_iter()
                        .flat_map(|it| it.to_le_bytes())
                        .collect::<Vec<u8>>();
            
                        
                    if sender
                        .send(Message::Binary(bytes))
                        .await
                        .is_err()
                    {
                        return ControlFlow::Break(());
                    }
                }
                
            } else {
                println!(">>> {} sent invalid payload: {:?}", who, t);
                return ControlFlow::Break(());
            }
        }
        Message::Binary(d) => {
            println!(">>> {} sent {} bytes: {:?}", who, d.len(), d);
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                println!(
                    ">>> {} sent close with code {} and reason `{}`",
                    who, cf.code, cf.reason
                );
            } else {
                println!(">>> {} somehow sent close message without CloseFrame", who);
            }
            return ControlFlow::Break(());
        }

        Message::Pong(v) => {
            println!(">>> {} sent pong with {:?}", who, v);
        }
        // You should never need to manually handle Message::Ping, as axum's websocket library
        // will do so for you automagically by replying with Pong and copying the v according to
        // spec. But if you need the contents of the pings you can see them here.
        Message::Ping(v) => {
            println!(">>> {} sent ping with {:?}", who, v);
        }
    }
    ControlFlow::Continue(())
}