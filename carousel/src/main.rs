use river_carousel_layout::Carousel;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    river_layout_toolkit::run(Carousel::new(Default::default()))?;
    Ok(())
}
