#[derive(Clone, Debug)]
pub struct IndexSelector {
    current: Vec<usize>,
    bounds: Vec<usize>,
    end: bool,
}

impl IndexSelector {
    pub fn new(bounds: Vec<usize>) -> Self {
        Self {
            current: vec![0; bounds.len()],
            end: bounds.iter().product::<usize>() == 0,
            bounds,
        }
    }
}

impl Iterator for IndexSelector {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end {
            return None;
        }

        let old_current = self.current.clone();

        for i in 0..self.current.len() {
            if self.current[i] < self.bounds[i] - 1 {
                self.current[i] += 1;
                return Some(old_current);
            }

            self.current[i] = 0;
        }

        self.end = true;
        Some(old_current)
    }
}

#[cfg(test)]
mod tests {
    use super::IndexSelector;

    #[test]
    fn empty() {
        assert!(IndexSelector::new(vec![2, 4, 1, 0, 4, 5]).next().is_none());
    }

    fn something() {
        assert_eq!(IndexSelector::new(vec![2, 4, 5, 8, 2]).count(), 640);
    }
}
