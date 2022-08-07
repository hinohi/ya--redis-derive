use bytes::Buf;

pub(crate) struct Bytes<'a> {
    data: &'a [u8],
}

impl<'a> Bytes<'a> {
    pub(crate) fn new(data: &'a [u8]) -> Bytes<'a> {
        Bytes { data }
    }
}

impl<'a> Buf for Bytes<'a> {
    fn remaining(&self) -> usize {
        self.data.remaining()
    }

    fn chunk(&self) -> &[u8] {
        self.data
    }

    fn advance(&mut self, cnt: usize) {
        self.data = &self.data[cnt..];
    }
}
