use poller::{EventContext, Events, Poller};
use std::io::stdin;
use std::sync::Arc;

type Callback = fn() -> bool;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the Poller.
    let mut poller = Poller::new()?;

    // Callback for handle raised event.
    let cb: Arc<Callback> = Arc::new(|| -> bool {
        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Ok(n) => {
                let trimmed = input.trim_end();
                println!("{} bytes readed: \"{}\"", n, trimmed);
                // Return false if input 'q'.
                trimmed != "q"
            }
            Err(e) => {
                println!("error: {}", e);
                false
            }
        }
    });

    // Add stdin to the watching list of the Poller.
    poller.add(
        0,
        Events::new().read(),
        Some(Arc::clone(&cb) as EventContext),
    )?;

    println!("Press ctrl+c or 'q' to exit ...");

    'outer: loop {
        // Pull all events with 1 seconds timeout.
        let events = poller.pull_events(1000)?;
        for (_fd, _events, _ctx) in events.iter() {
            // Use EventContext to processing the event.
            if let Some(x) = _ctx {
                if let Some(cb) = x.downcast_ref::<Callback>() {
                    if cb() == false {
                        break 'outer;
                    }
                }
            }
        }
    }

    Ok(())
}
