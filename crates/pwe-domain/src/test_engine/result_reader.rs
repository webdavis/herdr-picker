use super::TestEvent;

pub trait ResultReader {
    fn read_line(&mut self, line: &str) -> Vec<TestEvent>;
    fn finish(&mut self) -> Vec<TestEvent>;
}
