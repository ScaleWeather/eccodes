use eccodes_sys::codes_handle;
use crate::codes_handle::CodesHandle;

impl Iterator for CodesHandle {
    type Item = *mut codes_handle;

    fn next(&mut self) -> Option<Self::Item> {
        self.file_handle =
            CodesHandle::codes_handle_new_from_file(self.file_pointer, self.product_kind).unwrap();

        if self.file_handle.is_null() {
            None
        } else {
            Some(self.file_handle)
        }
    }
}