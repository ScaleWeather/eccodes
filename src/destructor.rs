use log::error;
use crate::CodesHandle;
use errno::errno;

impl Drop for CodesHandle {
    ///Executes the desctructor for this type ([read more](https://doc.rust-lang.org/1.54.0/core/ops/drop/trait.Drop.html#tymethod.drop)).
    ///This method calls `codes_handle_delete()` from ecCodes and `fclose()` from libc for graceful cleanup.\
    ///**WARNING:** Currently it is assumed that under normal circumstances this destructor never fails.
    ///However in some edge cases ecCodes or fclose can return non-zero code.
    ///For now user is informed about that through log, because I don't know how to handle it correctly.
    ///If some bugs occurs during drop please enable log output and post issue on Github.
    fn drop(&mut self) {
        let error_code;
        unsafe {
            error_code = eccodes_sys::codes_handle_delete(self.file_handle);
        }

        if error_code != 0 {
            error!(
                "CodesHandle destructor failed with ecCodes error code {:?}",
                error_code
            );
        }

        let error_code;
        unsafe {
            error_code = libc::fclose(self.file_pointer);
        }

        if error_code != 0 {
            let error_val = errno();
            let code = error_val.0;
            error!(
                "CodesHandle destructor failed with libc error {}, code: {}",
                code, error_val
            );
        }

        todo!(
            "Review required! Destructor is assumed to never fail under normal circumstances. 
        However in some edge cases ecCodes or fclose can return non-zero code. 
        For now user is informed about that through log, 
        because I don't know how to handle it correctly."
        );
    }
}