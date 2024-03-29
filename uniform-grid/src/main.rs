use river_uniform_grid_layout::UniformGrid;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    river_layout_toolkit::run(UniformGrid::new(Default::default()))?;
    Ok(())
}
