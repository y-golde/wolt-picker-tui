mod app;
mod controllers;
mod picking_cycle;

#[tokio::main]
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut cycle = picking_cycle::PickingCycle::new();
    cycle.start().await;

    Ok(())
}

fn main() {
    run().unwrap();
}
