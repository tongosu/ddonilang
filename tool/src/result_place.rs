#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResultPlace<T> {
    value: Option<T>,
}

impl<T> ResultPlace<T> {
    pub fn new() -> Self {
        Self { value: None }
    }

    pub fn assign_sequential(&mut self, value: T) {
        self.value = Some(value);
    }

    pub fn value(&self) -> Option<&T> {
        self.value.as_ref()
    }
}

impl<T> Default for ResultPlace<T> {
    fn default() -> Self {
        Self::new()
    }
}
