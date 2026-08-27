#[derive(Debug, Clone, PartialEq)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f32>,
}

impl Matrix {
    pub fn zeros(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: vec![0.0; rows * cols],
        }
    }

    pub fn from_row_major(rows: usize, cols: usize, data: Vec<f32>) -> Self {
        assert_eq!(rows * cols, data.len());
        Self { rows, cols, data }
    }

    #[inline]
    pub fn row(&self, row: usize) -> &[f32] {
        let start = row * self.cols;
        &self.data[start..start + self.cols]
    }

    #[inline]
    pub fn row_mut(&mut self, row: usize) -> &mut [f32] {
        let start = row * self.cols;
        &mut self.data[start..start + self.cols]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tensor3 {
    pub a: usize,
    pub b: usize,
    pub c: usize,
    pub data: Vec<f32>,
}

impl Tensor3 {
    pub fn zeros(a: usize, b: usize, c: usize) -> Self {
        Self {
            a,
            b,
            c,
            data: vec![0.0; a * b * c],
        }
    }

    #[inline]
    pub fn get(&self, i: usize, j: usize) -> &[f32] {
        let start = (i * self.b + j) * self.c;
        &self.data[start..start + self.c]
    }

    #[inline]
    pub fn get_mut(&mut self, i: usize, j: usize) -> &mut [f32] {
        let start = (i * self.b + j) * self.c;
        &mut self.data[start..start + self.c]
    }
}
