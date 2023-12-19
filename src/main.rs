mod app;
mod controllers;
mod picking_cycle;

#[tokio::main]
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut cycle = picking_cycle::PickingCycle::new();
    //cycle.start().await;
    cycle.start().await;

    // offer a resteraunt
    // select a random one from the first 20
    // do you like "category"?
    // try again

    // while (not chosen)
    // pick random resteraunt from the list
    // liked?
    // -- exit
    // not liked?
    // add a random category to + or -

    Ok(())
}

fn main() {
    run().unwrap();
    println!("Hello, world!");
}
