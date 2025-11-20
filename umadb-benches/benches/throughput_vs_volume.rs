use std::fs::OpenOptions;
use std::io::Write;
use std::time::Instant;
use umadb_client::UmaDBClient;
use umadb_dcb::{DCBEvent, DCBEventStoreAsync};
use uuid::Uuid;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Connect to UmaDB instance at 127.0.0.1:50051
    let addr = "http://127.0.0.1:50051";
    let client = UmaDBClient::new(addr.to_string())
        .connect_async()
        .await
        .expect("Failed to connect to UmaDB");

    println!("Connected to UmaDB at {}", addr);

    // Open log file for appending measurements
    let log_path = "../../umadb-throughput.log";
    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;

    println!("Logging measurements to {}", log_path);

    let mut batch_count = 0u64;

    loop {
        // Append 9800 batches of 10 events each
        for _ in 0..99 {
            let tag = format!("tag-{}", Uuid::new_v4());
            let events: Vec<DCBEvent> = (0..1000)
                .map(|_| DCBEvent {
                    event_type: "batch-type".to_string(),
                    data: "batch-data".to_string().into_bytes(),
                    tags: vec![tag.clone()],
                    uuid: None,
                })
                .collect();

            client
                .append(events, None)
                .await
                .expect("Failed to append batch");

            batch_count += 1;
        }

        // After 99 batches, measure the rate of appending 1000 individual events
        let start = Instant::now();
        for _ in 0..1000 {
            let tag = format!("tag-{}", Uuid::new_v4());
            let event = DCBEvent {
                event_type: "batch-type".to_string(),
                data: "batch-data".to_string().into_bytes(),
                tags: vec![tag.clone()],
                uuid: None,
            };

            client
                .append(vec![event], None)
                .await
                .expect("Failed to append individual event");
        }
        let elapsed = start.elapsed();

        // Get the current head position (sequence position)
        let head_position = client
            .head()
            .await
            .expect("Failed to get head position")
            .unwrap_or(0);

        // Calculate rate (events per second)
        let rate = 1000.0 / elapsed.as_secs_f64();

        // Log the measurement: sequence_position rate
        println!("Position: {}, rate: {}", head_position, rate);
        writeln!(log_file, "{} {}", head_position, rate)?;
        log_file.flush()?;

        println!(
            "Batch count: {}, Head position: {}, Rate: {:.2} events/sec",
            batch_count, head_position, rate
        );
    }
}
