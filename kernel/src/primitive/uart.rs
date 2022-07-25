pub trait Uart {
    fn write(&self, char: u8);
    fn read(&self) -> Option<u8>;
}
