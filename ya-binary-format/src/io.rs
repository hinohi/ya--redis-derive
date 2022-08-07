pub trait Write {
    fn write(&mut self, b: &[u8]);
}

impl Write for Vec<u8> {
    fn write(&mut self, b: &[u8]) {
        self.extend_from_slice(b);
    }
}

impl<'a> Write for &'a mut Vec<u8> {
    fn write(&mut self, b: &[u8]) {
        self.extend_from_slice(b);
    }
}
