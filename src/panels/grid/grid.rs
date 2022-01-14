use super::{
    layout::{CellSize, Layout, LineDirection},
    sizing::Sizing,
    Padding, Size,
};
use eframe::egui::Ui;

enum GridDirection {
    Horizontal,
    Vertical,
}

pub struct GridBuilder<'a> {
    ui: &'a mut Ui,
    sizing: Sizing,
    padding: Padding,
}

impl<'a> GridBuilder<'a> {
    /// Create new grid builder
    /// After adding size hints with [Self::column]/[Self::columns] the grid can be build with [Self::horizontal]/[Self::vertical]
    pub fn new(ui: &'a mut Ui, padding: Padding) -> Self {
        let sizing = Sizing::new();

        Self {
            ui,
            sizing,
            padding,
        }
    }

    /// Add size hint for column/row
    pub fn size(mut self, size: Size) -> Self {
        self.sizing.add_size(size);
        self
    }

    /// Add size hint for columns/rows [count] times
    pub fn sizes(mut self, size: Size, count: usize) -> Self {
        for _ in 0..count {
            self.sizing.add_size(size.clone());
        }
        self
    }

    /// Build horizontal grid
    pub fn horizontal<F>(self, grid: F)
    where
        F: for<'b> FnOnce(Grid<'a, 'b>),
    {
        let widths = self.sizing.into_lengths(
            self.ui.available_rect_before_wrap().width() - 2.0 * self.padding.outer,
            self.padding.inner,
        );
        let mut layout = Layout::new(self.ui, self.padding.clone(), LineDirection::TopToBottom);
        grid(Grid {
            layout: &mut layout,
            direction: GridDirection::Horizontal,
            padding: self.padding.clone(),
            widths,
        });
    }

    /// Build vertical grid
    pub fn vertical<F>(self, grid: F)
    where
        F: for<'b> FnOnce(Grid<'a, 'b>),
    {
        let widths = self.sizing.into_lengths(
            self.ui.available_rect_before_wrap().height() - 2.0 * self.padding.outer,
            self.padding.inner,
        );
        let mut layout = Layout::new(self.ui, self.padding.clone(), LineDirection::LeftToRight);
        grid(Grid {
            layout: &mut layout,
            direction: GridDirection::Vertical,
            padding: self.padding.clone(),
            widths,
        });
    }
}

pub struct Grid<'a, 'b> {
    layout: &'b mut Layout<'a>,
    direction: GridDirection,
    padding: Padding,
    widths: Vec<f32>,
}

impl<'a, 'b> Grid<'a, 'b> {
    fn size(&mut self) -> (CellSize, CellSize) {
        match self.direction {
            GridDirection::Horizontal => (
                CellSize::Absolute(self.widths.remove(0)),
                CellSize::Remainder,
            ),
            GridDirection::Vertical => (
                CellSize::Remainder,
                CellSize::Absolute(self.widths.remove(0)),
            ),
        }
    }

    /// Add empty cell
    pub fn empty(&mut self) {
        assert!(
            !self.widths.is_empty(),
            "Tried using more grid cells then available."
        );

        let (width, height) = self.size();
        self.layout.empty(width, height);
    }

    pub fn _cell(&mut self, clip: bool, add_contents: impl FnOnce(&mut Ui)) {
        assert!(
            !self.widths.is_empty(),
            "Tried using more grid cells then available."
        );

        let (width, height) = self.size();
        self.layout.add(width, height, clip, add_contents);
    }

    /// Add cell, content is clipped
    pub fn cell(&mut self, add_contents: impl FnOnce(&mut Ui)) {
        self._cell(true, add_contents);
    }

    /// Add cell, content is not clipped
    pub fn cell_noclip(&mut self, add_contents: impl FnOnce(&mut Ui)) {
        self._cell(false, add_contents);
    }

    pub fn _grid(&mut self, clip: bool, grid_builder: impl FnOnce(GridBuilder)) {
        let padding = self.padding.clone();
        self._cell(clip, |ui| {
            grid_builder(GridBuilder::new(ui, padding));
        });
    }
    /// Add grid as cell, content is clipped
    pub fn grid(&mut self, grid_builder: impl FnOnce(GridBuilder)) {
        self._grid(true, grid_builder)
    }

    /// Add grid as cell, content is not clipped
    pub fn grid_noclip(&mut self, grid_builder: impl FnOnce(GridBuilder)) {
        self._grid(false, grid_builder)
    }
}

impl<'a, 'b> Drop for Grid<'a, 'b> {
    fn drop(&mut self) {
        while !self.widths.is_empty() {
            self.empty();
        }
    }
}
