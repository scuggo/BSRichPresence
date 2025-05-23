mod bs_richpresence;
use crate::bs_richpresence::RichPresence;
mod bs_processing;
use crate::bs_processing::Processing;
use BSDataPuller::BSData;
use BSDataPuller::livedata::schema::BSLivedata;
use BSDataPuller::schema::BSMetadata;
use config;
use discordipc::Client;
use discordipc::activity::*;
use discordipc::packet::*;
use std::process::exit;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use tokio::time::Duration;
use tracing::Level;
use tracing::debug;
use tracing::info;

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    {
        tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .init();
    }
    #[cfg(not(debug_assertions))]
    {
        // This might show red in IDE but it compiles,,
        tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    }
    let config = config::config_init().await.unwrap();
    debug!("{:#?}", config);
    let oneshot_metadata = BSMetadata::get().await.unwrap();
    let bsdata = BSData::from_raw(oneshot_metadata);

    // start threads to update bsdata.
    bsdata.start().await;

    let client = Client::new_simple("1359573855412420741");
    client.connect_and_wait().unwrap();
    let activity = Activity::new().details("Some");
    let activity_packet = Packet::new_activity(Some(&activity), None);
    let start = SystemTime::now();
    let started_at = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let started_at = Arc::new(started_at);

    match client.send_and_wait(activity_packet).unwrap().filter() {
        Ok(_packet) => info!("Activity has been set!"),
        Err(e) => info!("Couldn't set activity: {}", e),
    }

    let bslivedata = BSLivedata::start().await;
    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        let mut activity = bsdata
            .process(&config)
            .await
            .to_activity(bslivedata.clone().lock().await.as_ref().unwrap())
            .await;
        activity.timestamps.replace(Timestamps {
            start: Some(started_at.clone().as_secs() as i64),
            ..Default::default()
        });
        debug!("{:#?}", activity);
        let activity_packet = Packet::new_activity(Some(&activity), None);
        client.send_and_wait(activity_packet).unwrap();
        //info!("awa");
        //print!("{}", aw);
        //print!("{:#?}", awa)
        //print!()
    }
}
