use glam::{IVec2, Vec2};
use log::error;
use river_layout_toolkit::{GeneratedLayout, Layout, Rectangle};

#[non_exhaustive]
pub struct Config {
    /// The aspect ratio to approximate with every grid extension.
    target_aspect: f32,

    /// Padding around the edge of the layout area, in pixels.
    pub outer_padding: i32,

    /// Padding between views, in pixels.
    pub view_padding: i32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            target_aspect: 16.0 / 9.0,
            outer_padding: 6,
            view_padding: 6,
        }
    }
}

#[derive(Clone, Copy)]
struct Grid {
    size: IVec2,
}

impl Grid {
    fn total_cells(&self) -> i32 {
        self.size.x * self.size.y
    }

    fn layout(&self, config: &Config, output_size: IVec2) -> GridLayout {
        let offset = IVec2::splat(config.outer_padding).as_vec2();
        let padded_size = output_size.as_vec2() - 2.0 * offset;

        let view_padding = IVec2::splat(config.view_padding);
        let stride = (padded_size + view_padding.as_vec2()) / self.size.as_vec2();
        let view_size = stride.as_ivec2() - IVec2::splat(config.view_padding);
        GridLayout {
            offset,
            stride,
            view_size,
        }
    }
}

struct GridLayout {
    offset: Vec2,
    stride: Vec2,
    view_size: IVec2,
}

impl GridLayout {
    fn aspect_ratio(&self) -> f32 {
        self.view_size.x as f32 / self.view_size.y as f32
    }

    /// Fraction of the view area that the target aspect ratio would fill.
    fn efficiency(&self, target_aspect: f32) -> f32 {
        let arr = self.aspect_ratio() / target_aspect;
        if arr > 1.0 {
            arr
        } else {
            1.0 / arr
        }
    }

    fn at(&self, grid_position: IVec2) -> Rectangle {
        let position = (self.offset + self.stride * grid_position.as_vec2()).as_ivec2();
        Rectangle {
            x: position.x,
            y: position.y,
            width: self.view_size.x.try_into().unwrap(),
            height: self.view_size.y.try_into().unwrap(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("unknown command: {0:?}")]
    UnknownCommand(String),

    #[error("missing argument: {0:?}")]
    MissingArgument(&'static str),

    #[error("invalid value for argument {0:?}")]
    InvalidArgument(&'static str),
}

pub struct UniformGrid {
    config: Config,
}

impl UniformGrid {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    fn user_cmd_inner(
        &mut self,
        cmd: String,
        tags: Option<u32>,
        output: &str,
    ) -> Result<(), Error> {
        let _ = (tags, output);

        let mut parts = cmd.split_whitespace();

        match parts.next().unwrap_or("") {
            other => Err(Error::UnknownCommand(other.into())),
        }
    }
}

impl Layout for UniformGrid {
    type Error = Error;

    const NAMESPACE: &'static str = "uniform-grid";

    fn user_cmd(
        &mut self,
        cmd: String,
        tags: Option<u32>,
        output: &str,
    ) -> Result<(), Self::Error> {
        let result = self.user_cmd_inner(cmd, tags, output);
        if let Err(e) = &result {
            error!("{e}");
        }

        result
    }

    fn generate_layout(
        &mut self,
        view_count: u32,
        usable_width: u32,
        usable_height: u32,
        tags: u32,
        output: &str,
    ) -> Result<GeneratedLayout, Self::Error> {
        let _ = (tags, output);

        let output_size = IVec2::new(usable_width as i32, usable_height as i32);

        let mut grid = Grid { size: IVec2::ONE };

        while (grid.total_cells() as u32) < view_count {
            let options = [
                Grid {
                    size: grid.size + IVec2::X,
                },
                Grid {
                    size: grid.size + IVec2::Y,
                },
            ];
            grid = options
                .into_iter()
                .min_by_key(|grid| {
                    let eff = grid
                        .layout(&self.config, output_size)
                        .efficiency(self.config.target_aspect);
                    (eff * 1000000.0) as i32
                })
                .unwrap();
        }

        // Generate cell views in a snaking layout
        let layout = grid.layout(&self.config, output_size);
        let views = (0..view_count as i32).map(|i_view| {
            let column_base = i_view % grid.size.x;
            let row = i_view / grid.size.x;
            let column = if row % 2 == 0 {
                column_base
            } else {
                grid.size.x - 1 - column_base
            };
            layout.at(IVec2::new(column, row))
        });

        Ok(GeneratedLayout {
            layout_name: format!("{}: {}x{}", Self::NAMESPACE, grid.size.y, grid.size.x),
            views: views.collect(),
        })
    }
}
