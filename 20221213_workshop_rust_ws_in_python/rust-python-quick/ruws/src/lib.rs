#![deny(unused_must_use)]

use std::format;
use std::thread;
use std::time::SystemTime;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use log::{info, trace, debug, LevelFilter};
use simple_logger::SimpleLogger;
use json::JsonValue;

use futures_util::SinkExt;

use tokio::runtime;
use tokio::sync::mpsc;
use tokio_stream::StreamExt as TokioStreamExt;
use tokio_tungstenite::{tungstenite, WebSocketStream, MaybeTlsStream};

use pyo3::{create_exception, ToPyObject};
use pyo3::prelude::*;
use pyo3::types::PyString;
use pyo3::types::IntoPyDict;


fn get_timestamp() -> f64 {
    let time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    time.as_secs() as f64 + time.subsec_nanos() as f64 / 1e9
}


struct PyAsyncQueue {
    queue: PyObject,
    call_soon_threadsafe: PyObject,
    put_nowait: PyObject,
}


impl PyAsyncQueue {
    pub fn new() -> PyAsyncQueue {
        Python::with_gil(|py| {
            py.run("from asyncio import Queue", None, None).unwrap();
            let res = py.eval("Queue()", None, None).unwrap();
            let queue = res.into_py(py);
            let loop_ = get_running_loop();
            PyAsyncQueue {
                put_nowait: queue.getattr(py, PyString::new(py, "put_nowait")).unwrap(),
                call_soon_threadsafe: loop_.getattr(py, PyString::new(py, "call_soon_threadsafe")).unwrap(),
                queue: queue,
            }
        })
    }

    pub fn push<T: ToPyObject>(&self, value: T) {
        Python::with_gil(|py| {
            self.push_py(py, value.to_object(py));
        });
    }

    pub fn push_py<T: ToPyObject>(&self, py: Python, value: T) {
        self.call_soon_threadsafe.call1(
            py,
            (&self.put_nowait, value.to_object(py))
        ).unwrap();
    }
}


fn get_running_loop() -> PyObject {
    Python::with_gil(|py| {
        py.run("from asyncio import get_running_loop", None, None).unwrap();
        py.eval("get_running_loop()", None, None).unwrap().into_py(py)
    })
}


fn json_value_into_py(py: Python, value: JsonValue) -> PyObject {
    match value {
        JsonValue::Null => { None::<()>.into_py(py) }
        JsonValue::Short(value) => { value.as_str().into_py(py) }
        JsonValue::String(value) => { value.into_py(py) }
        JsonValue::Number(_) => {  value.as_f64().into_py(py) }
        JsonValue::Boolean(value) => { value.into_py(py) }
        JsonValue::Object(data) => { PyCell::new(py, PyJsonObject{ data }).unwrap().to_object(py) }
        JsonValue::Array(data) => { PyCell::new(py, PyJsonArray{ data }).unwrap().to_object(py) }
    }
}


fn json_value_into_py_full(py: Python, value: &JsonValue) -> PyObject {
    match value {
        JsonValue::Null => { None::<()>.into_py(py) }
        JsonValue::Short(value) => { value.as_str().into_py(py) }
        JsonValue::String(value) => { value.into_py(py) }
        JsonValue::Number(_) => {  value.as_f64().into_py(py) }
        JsonValue::Boolean(value) => { value.into_py(py) }
        JsonValue::Object(data) => { data.iter().map(|(k, v)| (k, json_value_into_py_full(py, v))).into_py_dict(py).into_py(py) }
        JsonValue::Array(data) => { None::<()>.into_py(py)  } //data.iter().map(|v| json_value_into_py_full(py, v)) }
    }
}


async fn handle_close(connection: &mut WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>, tx_ack: &mpsc::Sender<()>, queue: &PyAsyncQueue) -> bool {
    trace!("Sending close");
    match connection.close(None).await {
        Ok(_) => {
            trace!("Connection closed");
            queue.push(StreamEnded::new_err("Stream closed"));
            trace!("Queue pushed");
            tx_ack.send(()).await.unwrap();
            true
        }
        Err(err) => {
            let msg = format!("Failed to disconnect because of {:?}", err);
            debug!("Connection cloe failed {}", msg);
            queue.push(StreamError::new_err(msg));
            false
        }
    }
}


async fn handle_send(connection: &mut WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>, msg: String, tx_ack: &mpsc::Sender<()>, queue: &PyAsyncQueue) {
    match connection.send(tungstenite::Message::Text(msg)).await {
        Ok(_) => { tx_ack.send(()).await.unwrap(); },
        Err(err) => {
            let msg = format!("Cannot send because of an error: {:?}", err);
            queue.push(StreamError::new_err(msg));
        }
    };
}


fn handle_data(queue: &PyAsyncQueue, data: Result<tungstenite::Message, tungstenite::Error>) {
    let mut data = match data {
        /*

        There was a HUGE structural penalty while using serde for parsing
        of huge structures

         */
        Ok(tungstenite::Message::Text(text)) => match json::parse(&text) {
            Ok(data) => data,
            Err(err) => {
                let msg = format!("Error parsing message: {:?}", err);
                queue.push(StreamError::new_err(msg));
                return;
            }
        },
        err => {
            let msg = format!("Error when receiving a message: {:?}", err);
            queue.push(StreamError::new_err(msg));
            return;
        }
    };

    data.insert::<f64>("atime", get_timestamp().into()).unwrap();
    Python::with_gil(|py| {
        let data_dict = json_value_into_py_full(py, &data);
        queue.push_py(py, data_dict);
    });
}


async fn stream_loop(stream: Arc<Mutex<Stream>>) {
    let mut stream = stream.lock().unwrap();
    let url_res = url::Url::parse(&stream.url).unwrap();
    let (mut connection, response) = tokio_tungstenite::connect_async(url_res).await.unwrap();

    stream.tx_ack.send(()).await.unwrap();
    debug!("Stream connected: {:?}", response);

    let rx = &mut stream.rx_send.take().expect("Communications closed");
    let rx_close = &mut stream.rx_close.take().expect("Communications closed");
    let tx_ack = &stream.tx_ack;
    let queue = &stream.queue;

    loop {
        trace!("Run select loop");
        tokio::select! {
            Some(item) = connection.next() => { handle_data(queue, item); }
            Some(item) = rx.recv() => { handle_send(&mut connection, item, tx_ack, queue).await }
            Some(_) = rx_close.recv() => { if handle_close(&mut connection, tx_ack, queue).await { break; } }
        };
    }

    rx.close();
    rx_close.close();

}


fn run_stream_loop(stream: Arc<Mutex<Stream>>) {
    let runtime = runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    debug!("Spawning stream loop...");
    runtime.block_on(async move { stream_loop(stream).await });
}


#[pyclass(extends=PyAny)]
struct PyJsonObject {
    data: json::object::Object
}


#[pymethods]
impl PyJsonObject {
    fn pop(&mut self, py: Python, key: String) -> PyObject {
        match self.data.remove(&key) {
            Some(item) => { json_value_into_py(py, item) }
            None => { None::<()>.into_py(py) }
        }
    }
}


#[pyclass(extends=PyAny)]
struct PyJsonArray {
    data: Vec<JsonValue>
}

#[pymethods]
impl PyJsonArray {
    fn pop(&mut self, py: Python, key: usize) -> PyObject {
        json_value_into_py(py, self.data.remove(key))
    }
}


struct Stream {
    url: String,
    queue: PyAsyncQueue,
    rx_send: Option<mpsc::Receiver<String>>,
    rx_close: Option<mpsc::Receiver<()>>,
    tx_ack: mpsc::Sender<()>,
}


/*

The object has to be wrapped inside an Arc since it will be shared
across threads with fuzzy lifetimes.  It was opted to wrap everything
so that we don't have to write too much code, passing individual
members.

*/
#[pyclass(extends=PyAny)]
struct PyStream {
    tx_send: mpsc::Sender<String>,
    tx_close: mpsc::Sender<()>,
    rx_ack: mpsc::Receiver<()>,
    thread: Option<thread::JoinHandle<()>>,
    queue: PyObject,
}


#[pymethods]
impl PyStream {

    #[new]
    fn new(url: String) -> PyStream {
        let (tx_send, rx_send) = mpsc::channel::<String>(1);
        let (tx_close, rx_close) = mpsc::channel::<()>(1);
        let (tx_ack, mut rx_ack) = mpsc::channel::<()>(1);

        let queue = PyAsyncQueue::new();
        let py_queue = queue.queue.clone();

        let stream_ptr = Arc::new(Mutex::new(
            Stream {
                url: url,
                queue: queue,
                rx_send: Some(rx_send),
                rx_close: Some(rx_close),
                tx_ack: tx_ack,
            }
        ));

        debug!("Spawning new ruws thread");
        let ptr = stream_ptr.clone();
        let thread = Some(thread::spawn(move || { run_stream_loop(ptr) }));

        rx_ack.blocking_recv().unwrap();

        let py_stream = PyStream {
            tx_send: tx_send,
            tx_close: tx_close,
            rx_ack: rx_ack,
            thread: thread,
            queue: py_queue,
        };

        py_stream
    }

    fn close(&mut self, py: Python) -> PyResult<()> {
        // We need to push into python during the close => we need GIL
        match self.thread {
            Some(_) => {
                py.allow_threads(|| {
                    debug!("Stopping thread");
                    self.tx_close.blocking_send(()).unwrap();
                    self.rx_ack.blocking_recv().unwrap();
                    match self.thread.take() {
                        Some(thread) => { thread.join().expect("Panic in thread"); Ok(()) }
                        None => { Ok(()) }
                    }
                })
            }
            None => { Err(StreamError::new_err("Stream already closed")) }
        }
    }

    fn send(&mut self, msg: String) -> PyResult<()> {
        match self.tx_send.blocking_send(msg) {
            Ok(()) => { self.rx_ack.blocking_recv().unwrap(); Ok(()) }
            Err(err) => { Err(StreamError::new_err(err.to_string())) }
        }
    }

    #[getter]
    fn queue(&self) -> PyResult<PyObject> {
        match self.thread {
            Some(_) => { Ok(self.queue.clone()) }
            None => { Err(StreamEnded::new_err("Stream already closed")) }
        }
    }

}


/* Create a Python exception */

create_exception!(libruws, StreamEnded, pyo3::exceptions::PyStopAsyncIteration);
create_exception!(libruws, StreamError, pyo3::exceptions::PyException);


/*
  A Python module implemented in Rust. The name of this function must match
  the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
  import the module.
*/
#[pymodule]
fn libruws(py: Python, m: &PyModule) -> PyResult<()> {
    SimpleLogger::new().with_level(LevelFilter::Error).init().unwrap();
    m.add_class::<PyStream>()?;
    m.add("StreamEnded", py.get_type::<StreamEnded>())?;
    m.add("StreamError", py.get_type::<StreamError>())?;
    Ok(())
}
