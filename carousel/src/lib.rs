use log::error;
use river_layout_toolkit::{GeneratedLayout, Layout, Rectangle};

pub enum Edge {
    Left,
    Right,
    Bottom,
    Top,
}

#[non_exhaustive]
pub struct Config {
    /// The main area will extend out from this edge.
    ///
    /// This also implicitly defines the location, scroll axis, and size axis of
    /// the secondary area:
    ///
    /// - The output will be split _horizontally_ between main and secondary
    ///   areas for `Left` or `Right` main locations, and split _vertically_ for
    ///   `Bottom` or `Top`.
    ///
    /// - The scroll direction will be "perpendicular" to the split - _vertical_
    ///   for `Left` or `Right` main locations, and _horizontal_ for `Bottom` or
    ///   `Top`.
    ///
    /// - The `secondary_window_size` will apply to the _vertical_ size of
    ///   secondary windows for `Left` or `Right` main locations, and the
    ///   _horizontal_ size for `Bottom` or `Top`.
    ///
    pub main_location: Edge,

    /// Ratio of main area to total layout area.
    ///
    /// This defines the split location between main and secondary areas.
    ///
    /// Whether this applies to width or height depends on `main_location`; see
    /// `main_location` for a detailed explanation.
    pub main_ratio: f32,

    /// Ratio of secondary window size to total secondary area.
    ///
    /// Whether this applies to width or height depends on `main_location`; see
    /// `main_location` for a detailed explanation.
    ///
    /// The inverse of this number is how many windows will fit in the secondary
    /// area at one time. (Padding is internally accounted for, so a value of
    /// `0.5` will fit exactly two windows with perfect padding.)
    pub secondary_window_size: f32,

    /// Padding around the edge of the layout area, in pixels.
    pub outer_padding: i32,

    /// Padding between views, in pixels.
    pub view_padding: i32,

    /// Offset of the secondary window, in "number of windows".
    pub scroll_offset: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            main_location: Edge::Left,
            main_ratio: 0.6,
            secondary_window_size: 0.5,
            outer_padding: 6,
            view_padding: 6,
            scroll_offset: 0.0,
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

pub struct Carousel {
    config: Config,
}

impl Carousel {
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
            "scroll" => {
                let amount: f32 = parts
                    .next()
                    .ok_or(Error::MissingArgument("amount"))?
                    .parse()
                    .map_err(|_| Error::InvalidArgument("amount"))?;

                self.config.scroll_offset += amount;
            }
            other => return Err(Error::UnknownCommand(other.into())),
        }
        Ok(())
    }
}

impl Layout for Carousel {
    type Error = Error;

    const NAMESPACE: &'static str = "carousel";

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

        let padded_width = usable_width as i32 - 2 * self.config.outer_padding;
        let padded_height = usable_height as i32 - 2 * self.config.outer_padding;

        let main_split_widthwise =
            ((padded_width - self.config.view_padding) as f32 * self.config.main_ratio) as i32;
        let main_split_heightwise =
            ((padded_height - self.config.view_padding) as f32 * self.config.main_ratio) as i32;

        let secondary_split_widthwise =
            padded_width - self.config.view_padding - main_split_widthwise;
        let secondary_split_heightwise =
            padded_height - self.config.view_padding - main_split_heightwise;

        let main_area = match self.config.main_location {
            Edge::Left => Rectangle {
                x: self.config.outer_padding,
                y: self.config.outer_padding,
                width: main_split_widthwise.try_into().unwrap(),
                height: padded_height.try_into().unwrap(),
            },
            Edge::Top => Rectangle {
                x: self.config.outer_padding,
                y: self.config.outer_padding,
                width: padded_width.try_into().unwrap(),
                height: main_split_heightwise.try_into().unwrap(),
            },
            Edge::Right => Rectangle {
                x: usable_width as i32 - self.config.outer_padding - main_split_widthwise,
                y: self.config.outer_padding,
                width: main_split_widthwise.try_into().unwrap(),
                height: padded_height.try_into().unwrap(),
            },
            Edge::Bottom => Rectangle {
                x: self.config.outer_padding,
                y: usable_width as i32 - self.config.outer_padding - main_split_heightwise,
                width: padded_width.try_into().unwrap(),
                height: main_split_heightwise.try_into().unwrap(),
            },
        };

        let secondary_size_widthwise = ((padded_width + self.config.view_padding) as f32
            * self.config.secondary_window_size) as i32
            - self.config.view_padding;
        let secondary_size_heightwise = ((padded_height + self.config.view_padding) as f32
            * self.config.secondary_window_size) as i32
            - self.config.view_padding;

        let secondary_base = match self.config.main_location {
            Edge::Left => Rectangle {
                x: self.config.outer_padding + main_split_widthwise + self.config.view_padding,
                y: self.config.outer_padding,
                width: secondary_split_widthwise.try_into().unwrap(),
                height: secondary_size_heightwise.try_into().unwrap(),
            },
            Edge::Top => Rectangle {
                x: self.config.outer_padding,
                y: self.config.outer_padding + main_split_heightwise + self.config.view_padding,
                width: secondary_size_widthwise.try_into().unwrap(),
                height: secondary_split_heightwise.try_into().unwrap(),
            },
            Edge::Right => Rectangle {
                x: self.config.outer_padding,
                y: self.config.outer_padding,
                width: secondary_split_widthwise.try_into().unwrap(),
                height: secondary_size_heightwise.try_into().unwrap(),
            },
            Edge::Bottom => Rectangle {
                x: self.config.outer_padding,
                y: self.config.outer_padding,
                width: secondary_size_widthwise.try_into().unwrap(),
                height: secondary_split_heightwise.try_into().unwrap(),
            },
        };

        let secondary_stride_x = match self.config.main_location {
            Edge::Left | Edge::Right => 0,
            Edge::Top | Edge::Bottom => secondary_size_widthwise + self.config.view_padding,
        };
        let secondary_stride_y = match self.config.main_location {
            Edge::Left | Edge::Right => secondary_size_heightwise + self.config.view_padding,
            Edge::Top | Edge::Bottom => 0,
        };

        let scroll_x = (secondary_stride_x as f32 * self.config.scroll_offset) as i32;
        let scroll_y = (secondary_stride_y as f32 * self.config.scroll_offset) as i32;

        Ok(GeneratedLayout {
            layout_name: Self::NAMESPACE.into(),
            views: [main_area]
                .into_iter()
                .chain((0i32..).map(|i| {
                    Rectangle {
                        x: secondary_base
                            .x
                            .saturating_add(secondary_stride_x.saturating_mul(i))
                            .saturating_sub(scroll_x),
                        y: secondary_base
                            .y
                            .saturating_add(secondary_stride_y.saturating_mul(i))
                            .saturating_sub(scroll_y),
                        width: secondary_base.width,
                        height: secondary_base.height,
                    }
                }))
                .inspect(|r| println!("{:?}", r))
                .take(view_count as usize)
                .collect(),
        })
    }
}
