#![deny(unused_must_use)]

use std::format;
use log::{error, info, warn};

use futures_util::SinkExt;

use serde_json::Value;

use tokio::runtime;
use tokio::time::Duration;
use tokio_stream::StreamExt as TokioStreamExt;
use tokio_tungstenite::tungstenite::Message;

use pyo3::{create_exception, ToPyObject, intern};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple, PyString};


// The interface is declared in rusaber.pyi.



struct PyAsyncQueue {
    loop_: PyObject,
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
                loop_: loop_,
            }
        })
    }

    pub fn push<T: ToPyObject>(&self, value: T) {
        Python::with_gil(|py| {
            let py_value = value.to_object(py);
            self.call_soon_threadsafe.call1(
                py,
                (&self.put_nowait, py_value)
            ).unwrap();
        });
    }
}


fn create_python_queue() -> PyObject {
    Python::with_gil(|py| {
        py.run("from asyncio import Queue", None, None).unwrap();
        let res = py.eval("Queue()", None, None).unwrap();
        res.into_py(py)
    })
}


fn push_python_queue<T: ToPyObject>(loop_: &PyObject, queue: &PyObject, value: T) {
    Python::with_gil(|py| {
        let put_nowait = queue.getattr(py, intern!(py, "put_nowait")).unwrap();
        let py_value = value.to_object(py);
        loop_.call_method1(
            py,
            intern!(py, "call_soon_threadsafe"),
            (put_nowait, py_value)
        ).unwrap();
    });
}


fn get_running_loop() -> PyObject {
    Python::with_gil(|py| {
        py.eval("asyncio.get_running_loop()", None, None).unwrap().into_py(py)
    })
}


/* Working with asyncio futures example */

fn create_python_future() -> PyObject {
    Python::with_gil(|py| {
        py.run("from asyncio import Future", None, None).unwrap();
        let res = py.eval("Future()", None, None).unwrap();
        res.into_py(py)
    })
}


fn set_python_future<T: ToPyObject>(loop_: &PyObject, future: &PyObject, value: T) {
    Python::with_gil(|py| {
        let set_result = future.getattr(py, intern!(py, "set_result")).unwrap();
        let py_value = value.to_object(py);
        loop_.call_method1(
            py,
            intern!(py, "call_soon_threadsafe"),
            (set_result, py_value)
        ).unwrap();
    });
}


fn fail_python_future<T: ToPyObject>(loop_: &PyObject, future: PyObject, value: T) {
    Python::with_gil(|py| {
        let set_exception = future.getattr(py, intern!(py, "set_exception")).unwrap();
        let py_value = value.to_object(py);
        loop_.call_method1(
            py,
            intern!(py, "call_soon_threadsafe"),
            (set_exception, py_value)
        ).unwrap();
    });
}

/* End of futures example */


fn subscribe(market: &str) -> Message {
    Message::Text(format!("{{\"op\": \"subscribe\", \"channel\": \"ticker\", \"market\": \"{market}\"}}"))
}


async fn connect(queue: Arc<PyAsyncQueue>, ws_url: &str) {
    let (mut connection, response) = tokio_tungstenite::connect_async(url::Url::parse(ws_url).unwrap()).await.unwrap();
    info!("response: {:?}", response);

    match connection.send(subscribe("BTC-PERP")).await {
        Ok(_) => (),
        Err(err) => {
            error!("Cannot subscribe because of an error: {:?}", err);
            return;
        }
    }

    while let Some(item) = connection.next().await {
        let data: Value = match item {
            Ok(Message::Text(text)) => match serde_json::from_str(&text) {
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
        let ex_time = data["data"]["time"].as_f64().unwrap_or(0.);

        let data_dict = Python::with_gil(|py| {
            PyDict::from_sequence(
                py,
                PyList::new(
                    py,
                    &[
                        PyTuple::new(
                            py,
                            &[
                                PyString::new(py, "ex_time").to_object(py),
                                ex_time.to_object(py)
                            ]
                        ),
                        PyTuple::new(
                            py,
                            &[
                                PyString::new(py, "bid_size").to_object(py),
                                 data["data"]["bidSize"].as_f64().unwrap_or(0.).to_object(py)
                            ]
                        ),
                        PyTuple::new(
                            py,
                            &[
                                PyString::new(py, "bid").to_object(py),
                                data["data"]["bid"].as_f64().to_object(py)
                            ]
                        ),
                        PyTuple::new(
                            py,
                            &[
                                PyString::new(py, "ask").to_object(py),
                                data["data"]["ask"].as_f64().to_object(py)
                            ]
                        ),
                        PyTuple::new(
                            py,
                            &[
                                PyString::new(py, "ask_size").to_object(py),
                                 data["data"]["askSize"].as_f64().unwrap_or(0.).to_object(py)
                            ]
                        ),
                    ]).to_object(py)
            ).unwrap().to_object(py)
        });

        queue.push(data_dict);
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


async fn stream_loop(queue: Arc<PyAsyncQueue>, duration: u64) {
    info!("running stream");

    let url = "wss://ftx.com/ws/";
    let ptr = Arc::clone(&queue);
    tokio::spawn(async move { connect(ptr, url).await; });
    tokio::time::sleep(Duration::from_secs(duration)).await;

    queue.push(StreamEnded::new_err(""));

    info!("stream ended");
}


fn run_stream(queue: Arc<PyAsyncQueue>, duration: u64) {
    let runtime = runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
        .unwrap();
    runtime.block_on(async move { stream_loop(queue, duration).await });
}


use std::sync::Arc;

#[pyclass(extends=PyAny)]
struct _Stream {
    url: String,
    queue: Arc<PyAsyncQueue>,
    
}


#[pymethods]
impl _Stream {

    #[new]
    fn new(url: String) -> _Stream {
        info!("creating new _Stream");
        _Stream {
            url: url,
            queue: Arc::new(PyAsyncQueue::new()),
        }
    }

    fn run(&self, duration: u64) {
        info!("spawning ruws thread");
        let ptr = Arc::clone(&self.queue);
        std::thread::spawn(move || run_stream(ptr, duration));
    }

    #[getter]
    fn url(&self) -> String {
        self.url.clone()
    }

    #[getter]
    fn queue(&self) -> PyObject {
        self.queue.queue.clone()
    }

}


/* Create a Python exception */

create_exception!(rusaber, StreamEnded, pyo3::exceptions::PyException);
create_exception!(rusaber, StreamError, pyo3::exceptions::PyException);



/*
  A Python module implemented in Rust. The name of this function must match
  the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
  import the module.
*/
#[pymodule]
#[pyo3(name = "libruws")]
fn libpyws(_py: Python, m: &PyModule) -> PyResult<()> {
    pyo3_log::init();
    Python::with_gil(|py| -> PyResult<()> {
        m.add_class::<_Stream>()?;

        m.add("StreamEnded",py.get_type::<StreamEnded>())?;
        m.add("StreamError",py.get_type::<StreamError>())?;

        // #[pyfunction]
        // fn log_something() {
        //     info!("Something!");
        // }

        //    m.add_wrapped(wrap_pyfunction!(log_something))?;

        Ok(())
    })?;

    Ok(())
}
