//! A minimal live line-chart widget, built directly on `GtkDrawingArea` +
//! Cairo rather than pulling in a full charting crate — this project only
//! ever needs "plot the last N temperature samples", so a small dependency-free
//! widget keeps the GUI's dependency surface smaller.

use gtk::cairo;
use gtk::prelude::*;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

const HISTORY_LEN: usize = 120; // 2 minutes at 1 sample/sec

#[derive(Clone)]
pub struct HistoryGraph {
    area: gtk::DrawingArea,
    samples: Rc<RefCell<VecDeque<f32>>>,
    /// Fixed Y-axis ceiling (°C) so the graph doesn't rescale distractingly on
    /// every tick; the line simply clips if a sample somehow exceeds it.
    max_value: f32,
    color: (f64, f64, f64),
}

impl HistoryGraph {
    pub fn new(max_value: f32, color: (f64, f64, f64)) -> Self {
        let area = gtk::DrawingArea::new();
        area.set_content_height(90);
        area.set_hexpand(true);
        let samples = Rc::new(RefCell::new(VecDeque::with_capacity(HISTORY_LEN)));

        let graph = Self { area, samples, max_value, color };

        let samples_for_draw = graph.samples.clone();
        let max_value_for_draw = graph.max_value;
        let color_for_draw = graph.color;
        graph.area.set_draw_func(move |_area, cr, width, height| {
            draw_graph(cr, width, height, &samples_for_draw.borrow(), max_value_for_draw, color_for_draw);
        });

        graph
    }

    pub fn widget(&self) -> &gtk::DrawingArea {
        &self.area
    }

    /// Push a new sample (ignored, i.e. holds the line flat, if the sensor is
    /// temporarily unavailable — a gap would be more visually confusing than
    /// a brief flat segment for a single dropped tick).
    pub fn push(&self, value: Option<f32>) {
        let Some(value) = value else { return };
        let mut samples = self.samples.borrow_mut();
        if samples.len() >= HISTORY_LEN {
            samples.pop_front();
        }
        samples.push_back(value);
        drop(samples);
        self.area.queue_draw();
    }
}

fn draw_graph(cr: &cairo::Context, width: i32, height: i32, samples: &VecDeque<f32>, max_value: f32, color: (f64, f64, f64)) {
    let w = width as f64;
    let h = height as f64;

    // Background + baseline grid (25/50/75%).
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
    let _ = cr.paint();
    cr.set_source_rgba(0.5, 0.5, 0.5, 0.15);
    for frac in [0.25, 0.5, 0.75] {
        cr.move_to(0.0, h * frac);
        cr.line_to(w, h * frac);
    }
    let _ = cr.stroke();

    if samples.len() < 2 {
        return;
    }

    let (r, g, b) = color;
    cr.set_source_rgb(r, g, b);
    cr.set_line_width(2.0);

    let step = w / (HISTORY_LEN as f64 - 1.0);
    // Right-align: most recent sample sits at the right edge.
    let offset = HISTORY_LEN.saturating_sub(samples.len()) as f64 * step;

    for (i, &value) in samples.iter().enumerate() {
        let x = offset + i as f64 * step;
        let y = h - (value.clamp(0.0, max_value) as f64 / max_value as f64) * h;
        if i == 0 {
            cr.move_to(x, y);
        } else {
            cr.line_to(x, y);
        }
    }
    let _ = cr.stroke();
}
