use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct BaseHttpResponse<T> {
    result: T,
    success: bool,
    result_code: usize,
}

pub fn generate_base_response<T>(
    result: T,
    success: bool,
    result_code: usize,
) -> BaseHttpResponse<T> {
    BaseHttpResponse {
        result,
        success,
        result_code,
    }
}
