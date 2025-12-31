use std::thread;
use std::time::{Duration, SystemTime};
use crate::errors::Errors;

pub fn fixed_interval_execution<F>(freq: Duration, mut fn_mut: F) -> Result<(), Errors>
    where F: FnMut() -> Result<(), Errors> {
    loop {
        // time execution
        let start = SystemTime::now();

        // execute main function
        fn_mut()?;

        // optional wait
        if let Ok(elapsed) = start.elapsed() && !elapsed.is_zero() {
            let wait_remainder = freq - elapsed;
            println!("Wait_remainder: {:?}", &wait_remainder);

            if wait_remainder <= freq {
                thread::sleep(wait_remainder);
            }
        }
    }
}