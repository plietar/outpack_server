use outpack_parser::query_types::QueryNode;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum Output<'a> {
    Success(QueryNode<'a>),
    Error(String),
}

#[no_mangle]
pub extern "C" fn parse_query(ptr: *const Box<[u8]>) -> *mut Box<[u8]> {
    let input = unsafe { std::str::from_utf8_unchecked(&**ptr) };

    let output = match outpack_parser::parse_query(input) {
        Ok(query) => Output::Success(query),
        Err(err) => Output::Error(err.to_string()),
    };

    let repr = serde_json::to_string(&output).unwrap();
    let result = repr.into_bytes().into_boxed_slice();
    Box::into_raw(Box::new(result))
}

#[no_mangle]
pub extern "C" fn string_alloc(size: usize) -> *mut Box<[u8]> {
    let result = vec![0; size].into_boxed_slice();
    Box::into_raw(Box::new(result))
}

#[no_mangle]
pub extern "C" fn string_free(ptr: *mut Box<[u8]>) {
    unsafe {
        std::mem::drop(Box::from_raw(ptr));
    }
}

#[no_mangle]
pub extern "C" fn string_data(ptr: *mut Box<[u8]>) -> *mut u8 {
    unsafe { (*ptr).as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn string_length(ptr: *mut Box<[u8]>) -> usize {
    unsafe { (*ptr).len() }
}
