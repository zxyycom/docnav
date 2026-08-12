use std::convert::Infallible;

pub trait Collector<I> {
    type Output;
    type Error;

    fn accept(&mut self, input: I);
    fn finish(self) -> Result<Self::Output, Self::Error>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VecCollector<I> {
    values: Vec<I>,
}

impl<I> VecCollector<I> {
    pub const fn new() -> Self {
        Self { values: Vec::new() }
    }
}

impl<I> Default for VecCollector<I> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I> Collector<I> for VecCollector<I> {
    type Output = Vec<I>;
    type Error = Infallible;

    fn accept(&mut self, input: I) {
        self.values.push(input);
    }

    fn finish(self) -> Result<Self::Output, Self::Error> {
        Ok(self.values)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StringCollector {
    value: String,
}

impl StringCollector {
    pub const fn new() -> Self {
        Self {
            value: String::new(),
        }
    }
}

impl Collector<String> for StringCollector {
    type Output = String;
    type Error = Infallible;

    fn accept(&mut self, input: String) {
        self.value.push_str(&input);
    }

    fn finish(self) -> Result<Self::Output, Self::Error> {
        Ok(self.value)
    }
}
