#![deny(unused_must_use)]

use std::format;
use std::time::SystemTime;
use std::sync::Arc;
use log::{error, info, warn};

use futures_util::SinkExt;

use tokio::runtime;
use tokio::time::Duration;
use tokio_stream::StreamExt as TokioStreamExt;
use tokio_tungstenite::tungstenite::Message;

use pyo3::{create_exception, ToPyObject};
use pyo3::prelude::*;
use pyo3::types::{PyString, IntoPyDict};


// The interface is declared in ruws.pyi.

/*

  https://pyo3.rs/main/doc/pyo3/

*/

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
        py.eval("asyncio.get_running_loop()", None, None).unwrap().into_py(py)
    })
}


fn subscribe(market: &str) -> Message {
    Message::Text(format!("{{\"op\": \"subscribe\", \"channel\": \"ticker\", \"market\": \"{market}\"}}"))
}


async fn connect(stream: Arc<Stream>) {
    let url_res = url::Url::parse(&stream.url).unwrap();
    let queue = &stream.queue;
    let (mut connection, response) = tokio_tungstenite::connect_async(url_res).await.unwrap();
    info!("response: {:?}", response);

    match connection.send(subscribe("BTC-PERP")).await {
        Ok(_) => (),
        Err(err) => {
            error!("Cannot subscribe because of an error: {:?}", err);
            return;
        }
    }

    while let Some(item) = connection.next().await {
        let data = match item {
            Ok(Message::Text(text)) => match json::parse(&text) {
                Ok(data) => data,
                Err(err) => {
                    warn!("{}", err);
                    continue;
                }
            },
            err => {
                error!("Error when receiving a message: {:?}", err);
                break;
            }
        };

        /*
        
        There was a HUGE structural penalty while using serde for parsing
        of huge structures

        */
      
        let ex_time = data["data"]["time"].as_f64().unwrap_or(0.);
        let ru_time = get_timestamp();
        Python::with_gil(|py| {
            let data_dict = [
                ("ex_time", ex_time),
                ("ru_time", ru_time)
            ].into_py_dict(py);
            py.run("from time import time; print(1000 * (time() - ex_time))", Some(data_dict), None).unwrap();
            queue.push_py(py, data_dict);
        });

    };

    match connection.close(None).await {
        Ok(_) => info!("Successfully disconnected"),
        /* Use a Python exception */
        Err(err) => {
            let msg = format!("Failed to disconnect because of {:?}", err);
            info!("{}", msg);
            queue.push(StreamError::new_err(msg));
        }
                
    }
}


async fn stream_loop(stream: Arc<Stream>, duration: u64) {
    info!("running stream");

    let stream_ = stream.clone();

    tokio::spawn(async move { connect(stream).await });
    tokio::time::sleep(Duration::from_secs(duration)).await;

    stream_.queue.push(StreamEnded::new_err(""));

    info!("stream ended");
}


fn run_stream_loop(stream: Arc<Stream>, duration: u64) {
    let runtime = runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
        .unwrap();
    runtime.block_on(async move { stream_loop(stream, duration).await });
}


struct Stream {
    url: String,
    queue: PyAsyncQueue,
}


/*

The object has to be wrapped inside an Arc since it will be shared
across threads with fuzzy lifetimes.  It was opted to wrap everything
so that we don't have to write too much code, passing individual
members.

*/
#[pyclass(extends=PyAny)]
struct PyStream {
    data: Arc<Stream>
}


#[pymethods]
impl PyStream {

    #[new]
    fn new(url: String) -> PyStream {
        info!("creating new PyStream");
        PyStream {
            data: Arc::new(
                Stream {
                    url: url,
                    queue: PyAsyncQueue::new(),
                }
            )
        }
    }

    fn run(&self, duration: u64) {
        info!("spawning ruws thread");
        let data = self.data.clone();
        std::thread::spawn(move || run_stream_loop(data, duration));
    }

    #[getter]
    fn url(&self) -> String {
        self.data.url.clone()
    }

    #[getter]
    fn queue(&self) -> PyObject {
        self.data.queue.queue.clone()
    }

}


/* Create a Python exception */

create_exception!(libruws, StreamEnded, pyo3::exceptions::PyException);
create_exception!(libruws, StreamError, pyo3::exceptions::PyException);


/*
  A Python module implemented in Rust. The name of this function must match
  the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
  import the module.
*/
#[pymodule]
fn libruws(py: Python, m: &PyModule) -> PyResult<()> {
    pyo3_log::init();
    m.add_class::<PyStream>()?;
    m.add("StreamEnded", py.get_type::<StreamEnded>())?;
    m.add("StreamError", py.get_type::<StreamError>())?;
    Ok(())
}
