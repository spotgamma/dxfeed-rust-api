use crossbeam_channel::{bounded, select, tick, Receiver};
use dxfeed as dx;
use serde_json;
use signal_hook::{
    consts::{SIGINT, SIGTERM},
    iterator::Signals,
};
use std::ffi::CString;
use std::io::prelude::*;
use std::io::{self, BufWriter, Stdout};
use std::os::raw::c_void;
use std::time::Duration;
use std::time::SystemTime;
use widestring::U32CString;

#[no_mangle]
pub extern "C" fn evt_listener(
    event_type: std::os::raw::c_int,
    sym: dx::dxf_const_string_t,
    data: *const dx::dxf_event_data_t,
    _data_count: i32, // always 1, and deprecated
    writer_ptr: *mut std::ffi::c_void,
) {
    let writer: &mut BufWriter<Stdout> = unsafe { &mut *(writer_ptr as *mut BufWriter<Stdout>) };
    match dx::Event::try_from_c(event_type, sym, data) {
        Ok(evt) => {
            write!(writer, "{}\n", serde_json::to_string(&evt).unwrap()).unwrap();
        }
        Err(e) => eprintln!("{:?}", e),
    }
}

#[no_mangle]
pub extern "C" fn termination_listener(
    _connection: dx::dxf_connection_t,
    _user_data: *mut ::std::os::raw::c_void,
) {
    eprintln!("!!! conn terminated !!!");
}

#[no_mangle]
pub extern "C" fn sub_listener(
    _connection: dx::dxf_connection_t,
    old_status: dx::dxf_connection_status_t,
    new_status: dx::dxf_connection_status_t,
    _user_data: *mut ::std::os::raw::c_void,
) {
    eprintln!("!!! sub !!! {} => {}", old_status, new_status);
}

pub fn sig_channel() -> Result<Receiver<i32>, std::io::Error> {
    let (sender, receiver) = bounded(100);
    // NOTE: SIGKILL, SIGSTOP, SIGILL, SIGFPE, SIGSEGV are forbidden
    // https://docs.rs/signal-hook-registry/latest/signal_hook_registry/fn.register.html#panics
    let mut signals = Signals::new(&[SIGINT, SIGTERM])?;
    std::thread::spawn(move || {
        for sig in signals.forever() {
            let _ = sender.send(sig);
        }
    });
    Ok(receiver)
}

// Listen to options expiring within 60 days for deriving spot prices
const SUCCESS: i32 = dx::DXF_SUCCESS as i32;
fn main() -> anyhow::Result<()> {
    let mut conn: dx::dxf_connection_t = std::ptr::null_mut();
    let mut sub: dx::dxf_subscription_t = std::ptr::null_mut();

    // This is a contrived example of using the `user_data` parameter.  This could easily be a
    // println! in the listener, but provides an illustrative example of how to pass unsafe objects
    // over to a given listener
    let mut writer = BufWriter::with_capacity(4096, io::stdout());
    let writer_ptr: *mut c_void = &mut writer as *mut _ as *mut c_void;

    // scope to drop/free large vectors once consumed for setting up subscriptions.
    let mut symbols: Vec<U32CString> = vec![];
    let sym_str = U32CString::from_str("AAPL")?;
    symbols.push(sym_str);

    let mut rsyms: Vec<*const i32> = symbols
        .iter()
        .map(|u32_sym| u32_sym.as_ptr() as *const i32)
        .collect();
    let c_syms: *mut dx::dxf_const_string_t =
        rsyms.as_mut_slice().as_ptr() as *mut dx::dxf_const_string_t;
    let c_host = CString::new("demo.dxfeed.com:7300")?;
    assert_eq!(SUCCESS, unsafe {
        dx::dxf_create_connection(
            c_host.as_ptr(),            // const char* address,
            Some(termination_listener), // dxf_conn_termination_notifier_t notifier,
            Some(sub_listener),         // dxf_conn_status_notifier_t conn_status_notifier,
            None,                       // dxf_socket_thread_creation_notifier_t stcn,
            None,                       // dxf_socket_thread_destruction_notifier_t stdn,
            std::ptr::null_mut(),       // void* user_data,
            &mut conn,                  // OUT dxf_connection_t* connection);
        )
    });
    eprintln!("connected");

    // Listen to quote events. Other events:
    // dx::DXF_ET_TIME_AND_SALE | dx::DXF_ET_GREEKS | dx::DXF_ET_TRADE | dx::DXF_ET_TRADE_ETH;
    assert_eq!(SUCCESS, unsafe {
        dx::dxf_create_subscription(conn, dx::DXF_ET_QUOTE, &mut sub)
    });
    assert_eq!(SUCCESS, unsafe {
        dx::dxf_attach_event_listener(sub, Some(evt_listener), writer_ptr)
    });
    assert_eq!(SUCCESS, unsafe {
        dx::dxf_add_symbols(sub, c_syms, symbols.len() as i32)
    });

    eprintln!("Ctrl-c to stop");
    let ticks = tick(Duration::from_secs(60));
    let sig_events = sig_channel()?;
    loop {
        select! {
            recv(ticks) -> _ => {
                eprintln!("{:?}: Running...", SystemTime::now());
            }
            recv(sig_events) -> sig => {
                match sig {
                    result => {
                        eprintln!("Received {:?}. Quitting...", result);
                        break;
                    },
                }
            }
        }
    }

    assert_eq!(SUCCESS, unsafe { dx::dxf_close_subscription(sub) });
    let close_close_conn_result = unsafe { dx::dxf_close_connection(conn) };
    eprintln!("close_close_conn_result: {}", close_close_conn_result);
    eprintln!("flushing stdout");
    writer.flush()?;
    Ok(())
}
